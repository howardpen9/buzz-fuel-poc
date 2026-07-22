//! MakeReel Fuel Bot — Buzz channel adapter for the Buzz Fuel P0 POC.
//!
//! Derived from Buzz `examples/countdown-bot` (Apache-2.0). See NOTICE.
//! Accepts exact `/fuel` only; creates a MakeReel fuel intent; mirrors core
//! status transitions as signed kind 9 messages.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use nostr::{
    Alphabet, Event, EventBuilder, Filter, JsonUtil, Keys, Kind, RelayUrl, SingleLetterTag, Tag,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url as WsUrl;

mod makereel;
mod model;

use makereel::MakeReelClient;
use model::{
    format_payment_link, format_status_message, is_exact_fuel, keys_for_phase, CreateIntentResponse,
    IntentPhase,
};

const DEFAULT_RELAY_URL: &str = "ws://localhost:3000";
const SUBSCRIPTION_ID: &str = "buzz-fuel-bot";
const BOT_NAME: &str = "makereel-fuel-bot";
const BOT_DISPLAY_NAME: &str = "MakeReel Fuel Bot";
const BOT_ABOUT: &str =
    "Pays for one fixed Seedance Fast launch reel via Telegram Stars and reports status.";
const BOT_ICON_DATA_URL: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 128 128'%3E%3Crect width='128' height='128' rx='28' fill='%230a0a12'/%3E%3Ctext x='64' y='78' text-anchor='middle' font-size='48' fill='%23facc15'%3E%E2%9A%A1%3C/text%3E%3C/svg%3E";

const POLL_INTERVAL_SECS: u64 = 5;
const POLL_TIMEOUT_SECS: u64 = 15 * 60;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let makereel = MakeReelClient::new(
        config.makereel_api_url.clone(),
        config.internal_api_key.clone(),
    );

    eprintln!(
        "buzz-fuel-bot pubkey: {}",
        config.bot_keys.public_key().to_hex()
    );
    eprintln!("connecting to {}", config.relay_url);

    let mut ws = connect_and_authenticate(&config).await?;
    publish_profile(&mut ws, &config).await?;
    announce_channel_membership(&mut ws, &config).await?;
    subscribe_to_channel(&mut ws, &config.channel_id).await?;

    let started_at = nostr::Timestamp::now();
    let state = Arc::new(Mutex::new(BotState::default()));

    eprintln!("listening in channel {} for exact /fuel", config.channel_id);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("shutting down");
                return Ok(());
            }
            next = ws.next() => {
                let Some(message) = next else { bail!("relay closed the WebSocket"); };
                match message? {
                    Message::Text(text) => {
                        handle_relay_text(
                            &mut ws,
                            &config,
                            &makereel,
                            &state,
                            started_at,
                            &text,
                        )
                        .await?;
                    }
                    Message::Ping(bytes) => ws.send(Message::Pong(bytes)).await?,
                    Message::Close(frame) => bail!("relay closed connection: {frame:?}"),
                    _ => {}
                }
            }
        }
    }
}

struct Config {
    relay_url: String,
    channel_id: String,
    bot_keys: Keys,
    owner_auth_tag: Option<Tag>,
    makereel_api_url: String,
    internal_api_key: String,
    poll_interval: Duration,
    poll_timeout: Duration,
    publish_running: bool,
}

impl Config {
    fn from_env() -> Result<Self> {
        let relay_url =
            std::env::var("BUZZ_RELAY_URL").unwrap_or_else(|_| DEFAULT_RELAY_URL.to_string());
        let channel_id = required_env("BUZZ_CHANNEL_ID")?;
        let bot_keys = Keys::parse(&required_env("BUZZ_BOT_PRIVATE_KEY")?)
            .context("BUZZ_BOT_PRIVATE_KEY must be an nsec or hex private key")?;

        let auth_mode =
            std::env::var("BUZZ_BOT_AUTH_MODE").unwrap_or_else(|_| "standalone".to_string());
        let owner_auth_tag = match auth_mode.as_str() {
            "standalone" => None,
            "owner-attested" => {
                let tag_json = match std::env::var("BUZZ_AUTH_TAG") {
                    Ok(value) if !value.trim().is_empty() => value,
                    _ => {
                        let owner_keys = Keys::parse(&required_env("BUZZ_OWNER_PRIVATE_KEY")?)
                            .context("BUZZ_OWNER_PRIVATE_KEY must be an nsec or hex private key")?;
                        buzz_sdk::nip_oa::compute_auth_tag(&owner_keys, &bot_keys.public_key(), "")?
                    }
                };
                let owner = buzz_sdk::nip_oa::verify_auth_tag(&tag_json, &bot_keys.public_key())
                    .context("BUZZ_AUTH_TAG is not valid for BUZZ_BOT_PRIVATE_KEY")?;
                eprintln!("owner-attested auth tag verified; owner={}", owner.to_hex());
                Some(buzz_sdk::nip_oa::parse_auth_tag(&tag_json)?)
            }
            other => {
                bail!("BUZZ_BOT_AUTH_MODE must be 'standalone' or 'owner-attested', got {other:?}")
            }
        };

        let poll_interval = Duration::from_secs(
            std::env::var("BUZZ_FUEL_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(POLL_INTERVAL_SECS),
        );
        let poll_timeout = Duration::from_secs(
            std::env::var("BUZZ_FUEL_POLL_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(POLL_TIMEOUT_SECS),
        );
        let publish_running = std::env::var("BUZZ_FUEL_PUBLISH_RUNNING")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Ok(Self {
            relay_url,
            channel_id,
            bot_keys,
            owner_auth_tag,
            makereel_api_url: required_env("MAKEREEL_API_URL")?,
            internal_api_key: required_env("INTERNAL_API_KEY")?,
            poll_interval,
            poll_timeout,
            publish_running,
        })
    }
}

#[derive(Default)]
struct BotState {
    /// Buzz request event IDs already handled (dedupe).
    seen_events: HashSet<String>,
    /// intent_id → set of transition keys already published.
    published: HashMap<String, HashSet<String>>,
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect_and_authenticate(config: &Config) -> Result<Ws> {
    let parsed = WsUrl::parse(&config.relay_url)?;
    let (mut ws, _) = connect_async(parsed.as_str()).await?;

    let challenge = wait_for_auth_challenge(&mut ws).await?;
    let auth_event = build_auth_event(config, &challenge)?;
    let auth_event_id = auth_event.id.to_hex();

    send_json(&mut ws, json!(["AUTH", auth_event])).await?;
    wait_for_ok(&mut ws, &auth_event_id).await?;
    Ok(ws)
}

fn build_auth_event(config: &Config, challenge: &str) -> Result<Event> {
    let relay_url: RelayUrl = RelayUrl::parse(&config.relay_url)?;
    if let Some(auth_tag) = &config.owner_auth_tag {
        let tags = vec![
            Tag::parse(["relay", config.relay_url.as_str()])?,
            Tag::parse(["challenge", challenge])?,
            auth_tag.clone(),
        ];
        Ok(EventBuilder::new(Kind::Authentication, "")
            .tags(tags)
            .sign_with_keys(&config.bot_keys)?)
    } else {
        Ok(EventBuilder::auth(challenge, relay_url).sign_with_keys(&config.bot_keys)?)
    }
}

async fn publish_profile(ws: &mut Ws, config: &Config) -> Result<()> {
    let builder = buzz_sdk::builders::build_profile(
        Some(BOT_DISPLAY_NAME),
        Some(BOT_NAME),
        Some(BOT_ICON_DATA_URL),
        Some(BOT_ABOUT),
        None,
    )?;
    let profile_event = builder.sign_with_keys(&config.bot_keys)?;
    let profile_event_id = profile_event.id.to_hex();

    send_json(ws, json!(["EVENT", profile_event])).await?;
    wait_for_ok(ws, &profile_event_id).await?;
    eprintln!("published kind:0 profile for {BOT_DISPLAY_NAME}");
    Ok(())
}

async fn announce_channel_membership(ws: &mut Ws, config: &Config) -> Result<()> {
    let builder = EventBuilder::new(Kind::Custom(9000), "").tags([
        Tag::parse(["h", config.channel_id.as_str()])?,
        Tag::parse(["p", &config.bot_keys.public_key().to_hex()])?,
        Tag::parse(["role", "bot"])?,
    ]);
    let event = builder.sign_with_keys(&config.bot_keys)?;
    let event_id = event.id.to_hex();

    send_json(ws, json!(["EVENT", event])).await?;
    match wait_for_ok(ws, &event_id).await {
        Ok(()) => eprintln!("announced {BOT_DISPLAY_NAME} as a channel bot member"),
        Err(err) => eprintln!(
            "could not self-add {BOT_DISPLAY_NAME} as a channel bot member: {err}. For private channels, add the bot pubkey as a channel member."
        ),
    }
    Ok(())
}

async fn subscribe_to_channel(ws: &mut Ws, channel_id: &str) -> Result<()> {
    let filter = Filter::new().kind(Kind::Custom(9)).custom_tag(
        SingleLetterTag::lowercase(Alphabet::H),
        channel_id.to_string(),
    );
    send_json(ws, json!(["REQ", SUBSCRIPTION_ID, filter])).await
}

async fn handle_relay_text(
    ws: &mut Ws,
    config: &Config,
    makereel: &MakeReelClient,
    state: &Arc<Mutex<BotState>>,
    started_at: nostr::Timestamp,
    text: &str,
) -> Result<()> {
    let value: Value = serde_json::from_str(text)?;
    match value.get(0).and_then(Value::as_str) {
        Some("EVENT") => {
            let event_value = value
                .get(2)
                .ok_or_else(|| anyhow!("EVENT message missing event payload"))?;
            let event = Event::from_json(event_value.to_string())?;
            maybe_handle_fuel(ws, config, makereel, state, started_at, &event).await?;
        }
        Some("EOSE") => {}
        Some("NOTICE") | Some("CLOSED") => eprintln!("relay: {value}"),
        Some(other) => eprintln!("ignored relay message type: {other}"),
        None => eprintln!("ignored malformed relay message: {text}"),
    }
    Ok(())
}

async fn maybe_handle_fuel(
    ws: &mut Ws,
    config: &Config,
    makereel: &MakeReelClient,
    state: &Arc<Mutex<BotState>>,
    started_at: nostr::Timestamp,
    event: &Event,
) -> Result<()> {
    // C03/C04: ignore historical and own events.
    if event.pubkey == config.bot_keys.public_key() || event.created_at < started_at {
        return Ok(());
    }
    if !is_exact_fuel(&event.content) {
        return Ok(());
    }

    let event_id = event.id.to_hex();
    {
        let mut st = state.lock().await;
        if !st.seen_events.insert(event_id.clone()) {
            // C05: duplicate Buzz event — no second intent create from this process.
            return Ok(());
        }
    }

    match makereel.create_intent(&config.channel_id, &event_id).await {
        Ok(intent) => {
            let body = format_payment_link(&intent.telegram_url, &intent.intent_id, &event_id);
            publish_reply(ws, config, event, &body).await?;
            spawn_status_poller(config, makereel.clone(), state.clone(), intent, event_id);
        }
        Err(err) => {
            eprintln!("create_intent failed for {event_id}: {err}");
            let body = format!(
                "Could not open a fuel link right now (maintenance or config). Try again later.\nbuzz: {event_id}"
            );
            // Best-effort failure message — still require relay OK.
            if let Err(e) = publish_reply(ws, config, event, &body).await {
                eprintln!("failed to publish error reply: {e}");
            }
        }
    }
    Ok(())
}

fn spawn_status_poller(
    config: &Config,
    makereel: MakeReelClient,
    state: Arc<Mutex<BotState>>,
    intent: CreateIntentResponse,
    buzz_event_id: String,
) {
    // Status publishing from the poller requires its own relay connection so the
    // main read loop is not blocked. Poller publishes via a short-lived session.
    let relay_url = config.relay_url.clone();
    let channel_id = config.channel_id.clone();
    let bot_keys = config.bot_keys.clone();
    let owner_auth_tag = config.owner_auth_tag.clone();
    let poll_interval = config.poll_interval;
    let poll_timeout = config.poll_timeout;
    let publish_running = config.publish_running;
    let intent_id = intent.intent_id.clone();

    tokio::spawn(async move {
        if let Err(err) = poll_and_publish_status(
            relay_url,
            channel_id,
            bot_keys,
            owner_auth_tag,
            makereel,
            state,
            intent_id,
            buzz_event_id,
            poll_interval,
            poll_timeout,
            publish_running,
        )
        .await
        {
            eprintln!("status poller ended with error: {err}");
        }
    });
}

#[allow(clippy::too_many_arguments)]
async fn poll_and_publish_status(
    relay_url: String,
    channel_id: String,
    bot_keys: Keys,
    owner_auth_tag: Option<Tag>,
    makereel: MakeReelClient,
    state: Arc<Mutex<BotState>>,
    intent_id: String,
    buzz_event_id: String,
    poll_interval: Duration,
    poll_timeout: Duration,
    publish_running: bool,
) -> Result<()> {
    let mini = Config {
        relay_url: relay_url.clone(),
        channel_id: channel_id.clone(),
        bot_keys: bot_keys.clone(),
        owner_auth_tag,
        makereel_api_url: String::new(),
        internal_api_key: String::new(),
        poll_interval,
        poll_timeout,
        publish_running,
    };

    let deadline = tokio::time::Instant::now() + poll_timeout;
    loop {
        if tokio::time::Instant::now() >= deadline {
            eprintln!("poll timeout for intent {intent_id}");
            break;
        }
        tokio::time::sleep(poll_interval).await;

        let status = match makereel.get_intent(&intent_id).await {
            Ok(s) => s,
            Err(err) => {
                eprintln!("get_intent {intent_id}: {err}");
                continue;
            }
        };
        let phase = IntentPhase::parse(&status.status);
        // If job races ahead to delivered/failed before we observed paid, still
        // emit Fueled (C1) so payment correlation is visible on the channel.
        let keys = keys_for_phase(&phase, publish_running);
        if keys.is_empty() {
            continue;
        }

        let mut reached_terminal = false;
        for key in keys {
            {
                let mut st = state.lock().await;
                let set = st.published.entry(intent_id.clone()).or_default();
                if !set.insert(key.to_string()) {
                    // C06: already published this transition.
                    if matches!(key, "delivered" | "failed") {
                        reached_terminal = true;
                    }
                    continue;
                }
            }

            let body = format_status_message(
                key,
                &buzz_event_id,
                &intent_id,
                status.job_id.as_deref(),
                status.share_url.as_deref(),
            );

            match publish_channel_message(&mini, &body).await {
                Ok(id) => eprintln!("published {key} for {intent_id} as {id}"),
                Err(err) => {
                    // C07: surface relay rejection clearly; do not claim success.
                    eprintln!(
                        "ERROR: relay rejected or failed status publish ({key} intent={intent_id}): {err}"
                    );
                    // Allow retry of this transition.
                    let mut st = state.lock().await;
                    if let Some(set) = st.published.get_mut(&intent_id) {
                        set.remove(key);
                    }
                }
            }

            if matches!(key, "delivered" | "failed") {
                reached_terminal = true;
            }
        }
        if reached_terminal {
            break;
        }
    }
    Ok(())
}

async fn publish_channel_message(config: &Config, body: &str) -> Result<String> {
    let mut ws = connect_and_authenticate(config).await?;
    let builder =
        buzz_sdk::builders::build_message(config.channel_id.parse()?, body, None, &[], false, &[])?;
    let event = builder.sign_with_keys(&config.bot_keys)?;
    let event_id = event.id.to_hex();
    send_json(&mut ws, json!(["EVENT", event])).await?;
    wait_for_ok(&mut ws, &event_id).await?;
    let _ = ws.close(None).await;
    Ok(event_id)
}

async fn publish_reply(ws: &mut Ws, config: &Config, event: &Event, body: &str) -> Result<()> {
    let builder = buzz_sdk::builders::build_message(
        config.channel_id.parse()?,
        body,
        None,
        &[&event.pubkey.to_hex()],
        false,
        &[],
    )?;
    let reply_event = builder.sign_with_keys(&config.bot_keys)?;
    let reply_event_id = reply_event.id.to_hex();

    send_json(ws, json!(["EVENT", reply_event])).await?;
    // C07: explicit failure if relay rejects.
    wait_for_ok(ws, &reply_event_id)
        .await
        .with_context(|| format!("relay rejected kind 9 reply {reply_event_id}"))?;
    eprintln!("replied to {} with {}", event.id.to_hex(), reply_event_id);
    Ok(())
}

async fn wait_for_ok(ws: &mut Ws, event_id: &str) -> Result<()> {
    loop {
        let text = next_text(ws, Duration::from_secs(5)).await?;
        let value: Value = serde_json::from_str(&text)?;
        if value.get(0).and_then(Value::as_str) != Some("OK") {
            continue;
        }
        if value.get(1).and_then(Value::as_str) != Some(event_id) {
            continue;
        }
        if value.get(2).and_then(Value::as_bool) == Some(true) {
            return Ok(());
        }
        let reason = value
            .get(3)
            .and_then(Value::as_str)
            .unwrap_or("unknown reason");
        bail!("relay rejected event {event_id}: {reason}");
    }
}

async fn wait_for_auth_challenge(ws: &mut Ws) -> Result<String> {
    loop {
        let text = next_text(ws, Duration::from_secs(5)).await?;
        let value: Value = serde_json::from_str(&text)?;
        if value.get(0).and_then(Value::as_str) == Some("AUTH") {
            return value
                .get(1)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| anyhow!("AUTH message missing challenge"));
        }
    }
}

async fn next_text(ws: &mut Ws, timeout: Duration) -> Result<String> {
    loop {
        let message = tokio::time::timeout(timeout, ws.next())
            .await
            .context("timed out waiting for relay message")?
            .ok_or_else(|| anyhow!("relay closed the WebSocket"))??;
        match message {
            Message::Text(text) => return Ok(text.to_string()),
            Message::Ping(bytes) => ws.send(Message::Pong(bytes)).await?,
            Message::Close(frame) => bail!("relay closed connection: {frame:?}"),
            _ => {}
        }
    }
}

async fn send_json(ws: &mut Ws, value: Value) -> Result<()> {
    ws.send(Message::Text(value.to_string().into())).await?;
    Ok(())
}

fn required_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("{name} is required"))
}
