//! Shared types for MakeReel fuel intent status and Buzz message formatting.

use std::collections::HashSet;

use serde::Deserialize;

/// Core-owned intent status values we understand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentPhase {
    AwaitingClaim,
    AwaitingPayment,
    Paid,
    Running,
    Delivered,
    Failed,
    Refunded,
    Expired,
    Unknown(String),
}

impl IntentPhase {
    pub fn parse(s: &str) -> Self {
        match s {
            "awaiting_claim" => Self::AwaitingClaim,
            "awaiting_payment" => Self::AwaitingPayment,
            "paid" => Self::Paid,
            "running" => Self::Running,
            "delivered" => Self::Delivered,
            "failed" => Self::Failed,
            "refunded" => Self::Refunded,
            "expired" => Self::Expired,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Transitions we publish to Buzz at most once.
    pub fn transition_key(&self) -> Option<&'static str> {
        match self {
            Self::AwaitingPayment | Self::AwaitingClaim => None, // payment link already sent
            Self::Paid => Some("fueled"),
            Self::Running => Some("running"), // optional; may be skipped
            Self::Delivered => Some("delivered"),
            Self::Failed | Self::Refunded => Some("failed"),
            Self::Expired => Some("failed"),
            Self::Unknown(_) => None,
        }
    }

    /// True once the intent is terminal for the Buzz status poller.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Delivered | Self::Failed | Self::Refunded | Self::Expired
        )
    }
}

/// Ordered transition keys to publish for one observed core status.
///
/// When the first poll already returns `delivered`/`failed`, still emit `fueled`
/// first so C1 payment correlation is visible, then the terminal key.
pub fn keys_for_phase(phase: &IntentPhase, publish_running: bool) -> Vec<&'static str> {
    match phase.transition_key() {
        Some("delivered") => vec!["fueled", "delivered"],
        Some("failed") => vec!["fueled", "failed"],
        Some("running") if !publish_running => vec![],
        Some(other) => vec![other],
        None => vec![],
    }
}

/// Pure poller planner: given successive core status strings, return the
/// sequence of transition keys that would be published (at most once each).
///
/// Stops after the first poll that yields a terminal publication, matching
/// production poller control flow.
pub fn plan_publications(poll_statuses: &[&str], publish_running: bool) -> Vec<&'static str> {
    let mut published: HashSet<&'static str> = HashSet::new();
    let mut out: Vec<&'static str> = Vec::new();

    for status in poll_statuses {
        let phase = IntentPhase::parse(status);
        let keys = keys_for_phase(&phase, publish_running);
        if keys.is_empty() {
            continue;
        }

        let mut reached_terminal = false;
        for key in keys {
            if published.insert(key) {
                out.push(key);
            }
            if matches!(key, "delivered" | "failed") || phase.is_terminal() {
                reached_terminal = true;
            }
        }
        if reached_terminal {
            break;
        }
    }
    out
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // decoded for logging / future C1-dry fields
pub struct CreateIntentResponse {
    pub intent_id: String,
    pub start_token: String,
    pub telegram_url: String,
    pub status: String,
    /// Resolved generation prompt (server default or per-`/fuel` override).
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // full public status surface; not all fields used every transition
pub struct IntentStatus {
    pub intent_id: String,
    pub status: String,
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub share_url: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub buzz_request_event_id: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
}

/// Max chars of user-supplied prompt after `/fuel` (matches core `MAX_PROMPT_LEN`).
pub const MAX_FUEL_PROMPT_LEN: usize = 500;

/// Parse `/fuel` or `/fuel <prompt>`.
///
/// - `None` — not a fuel command (ignore)
/// - `Some(None)` — bare `/fuel` → server default prompt
/// - `Some(Some(text))` — custom prompt for this generation
///
/// Case-sensitive command token; requires whitespace before a custom prompt
/// (`/fueling` is not accepted).
pub fn parse_fuel_command(content: &str) -> Option<Option<String>> {
    let t = content.trim();
    if t == "/fuel" {
        return Some(None);
    }
    let rest = t.strip_prefix("/fuel")?;
    if rest.is_empty() {
        return Some(None);
    }
    // Require whitespace after the command token so `/fueling` is ignored.
    let mut chars = rest.chars();
    let first = chars.next()?;
    if !first.is_whitespace() {
        return None;
    }
    let prompt = rest.trim();
    if prompt.is_empty() {
        return Some(None);
    }
    let capped: String = prompt.chars().take(MAX_FUEL_PROMPT_LEN).collect();
    Some(Some(capped))
}

/// True when content is a fuel command (`/fuel` or `/fuel <prompt>`).
pub fn is_exact_fuel(content: &str) -> bool {
    parse_fuel_command(content).is_some()
}

/// Footer with correlation IDs only (no TG identity / charge IDs / payment headers).
fn ref_footer(intent_id: &str, buzz_event_id: &str, job_id: Option<&str>) -> String {
    let mut lines = vec![
        "—".to_string(),
        format!("intent: {intent_id}"),
        format!("buzz: {buzz_event_id}"),
    ];
    if let Some(j) = job_id {
        lines.push(format!("job: {j}"));
    }
    lines.join("\n")
}

/// Demo-visible prompt block (always show when known so takes differ on camera).
fn prompt_block(prompt: Option<&str>) -> String {
    match prompt.map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) => format!("Prompt:\n\"{p}\"\n\n"),
        None => String::new(),
    }
}

/// Format a signed kind-9 status body. Demo-first copy; safe IDs only at the bottom.
pub fn format_status_message(
    phase_key: &str,
    buzz_event_id: &str,
    intent_id: &str,
    job_id: Option<&str>,
    share_url: Option<&str>,
    prompt: Option<&str>,
) -> String {
    let footer = ref_footer(intent_id, buzz_event_id, job_id);
    let pb = prompt_block(prompt);
    match phase_key {
        "fueled" => format!(
            "✅ Fueled — Stars received\n\n\
{pb}\
Telegram payment is in. One generation is starting now.\n\
I'll post again when the reel is ready.\n\n\
{footer}"
        ),
        "running" => format!(
            "⏳ Generating…\n\n\
{pb}\
Rendering your launch reel (Seedance Fast · 5s · 720p).\n\
Hang tight — the share link comes next.\n\n\
{footer}"
        ),
        "delivered" => {
            let link = share_url.unwrap_or("(share link pending — check Telegram)");
            format!(
                "🎬 Delivered — reel ready\n\n\
{pb}\
Watch / share:\n\
{link}\n\n\
Same result is available in Telegram from @MakeReel_xyz_bot.\n\n\
{footer}"
            )
        }
        "failed" => format!(
            "❌ Failed\n\n\
{pb}\
This run did not finish. If Stars were charged, they should be refunded.\n\
You can try again with `/fuel` or `/fuel <prompt>`.\n\n\
{footer}"
        ),
        other => format!("{other}\n\n{footer}"),
    }
}

/// First reply after /fuel: human steps first, correlation IDs last.
/// Stars-only wording (no USDC / x402 in channel copy — claims safety).
pub fn format_payment_link(
    telegram_url: &str,
    intent_id: &str,
    buzz_event_id: &str,
    prompt: Option<&str>,
) -> String {
    let pb = prompt_block(prompt);
    format!(
        "🚀 Pay with Telegram Stars\n\n\
{pb}\
Fund one launch reel (Seedance Fast · 5s · 720p):\n\
{telegram_url}\n\n\
What to do:\n\
1. Open the link in Telegram\n\
2. Confirm the Stars invoice\n\
3. Come back here — status updates post as signed messages\n\n\
Tip: next time use `/fuel <your prompt>` so each reel looks different.\n\n\
{}",
        ref_footer(intent_id, buzz_event_id, None)
    )
}

/// Optional Blossom-hosted QR metadata for the payment reply.
#[derive(Debug, Clone)]
pub struct PaymentImage {
    pub url: String,
    pub imeta_tag: Vec<String>,
}

/// Build payment kind-9 body + media tags.
///
/// - With image: append `![image](url)` (Desktop render contract) and imeta tags.
/// - Without image (upload failure / skip): text-only, empty media tags.
pub fn build_payment_reply(
    telegram_url: &str,
    intent_id: &str,
    buzz_event_id: &str,
    prompt: Option<&str>,
    image: Option<&PaymentImage>,
) -> (String, Vec<Vec<String>>) {
    let text = format_payment_link(telegram_url, intent_id, buzz_event_id, prompt);
    match image {
        Some(img) => {
            let body = format!("{text}\n\n![image]({})", img.url);
            (body, vec![img.imeta_tag.clone()])
        }
        None => (text, Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c01_exact_fuel() {
        assert!(is_exact_fuel("/fuel"));
        assert!(is_exact_fuel("  /fuel  "));
        assert_eq!(parse_fuel_command("/fuel"), Some(None));
        assert_eq!(
            parse_fuel_command("/fuel neon city BUZZ logo"),
            Some(Some("neon city BUZZ logo".into()))
        );
        assert!(is_exact_fuel("/fuel neon city BUZZ logo"));
    }

    #[test]
    fn c02_unknown_text_ignored() {
        assert!(!is_exact_fuel("fuel"));
        assert!(!is_exact_fuel("/fueling"));
        assert!(!is_exact_fuel("/Fuel"));
        assert!(!is_exact_fuel("!fuel"));
        assert!(!is_exact_fuel("can you fuel this"));
        // With space after /fuel, remainder is a custom prompt (demo path).
        assert_eq!(
            parse_fuel_command("/fuel please"),
            Some(Some("please".into()))
        );
    }

    #[test]
    fn transition_keys_are_stable() {
        assert_eq!(IntentPhase::Paid.transition_key(), Some("fueled"));
        assert_eq!(IntentPhase::Delivered.transition_key(), Some("delivered"));
        assert_eq!(IntentPhase::Failed.transition_key(), Some("failed"));
        assert_eq!(IntentPhase::Refunded.transition_key(), Some("failed"));
        assert_eq!(IntentPhase::AwaitingClaim.transition_key(), None);
    }

    #[test]
    fn first_poll_delivered_publishes_fueled_then_delivered_once() {
        // Regression: direct terminal state must still emit Fueled first.
        assert_eq!(
            keys_for_phase(&IntentPhase::Delivered, false),
            vec!["fueled", "delivered"]
        );
        assert_eq!(
            plan_publications(&["delivered"], false),
            vec!["fueled", "delivered"]
        );
        // Repeated polls of delivered (or any later status) must not re-publish.
        assert_eq!(
            plan_publications(&["delivered", "delivered", "delivered"], false),
            vec!["fueled", "delivered"]
        );
        // Paid then delivered still yields each transition once, in order.
        assert_eq!(
            plan_publications(&["paid", "running", "delivered"], false),
            vec!["fueled", "delivered"]
        );
    }

    #[test]
    fn status_messages_omit_secrets() {
        let msg = format_status_message(
            "delivered",
            "evt123",
            "fi_abc",
            Some("job-1"),
            Some("https://example.test/s/tok"),
            Some("neon BUZZ rooftop"),
        );
        assert!(msg.contains("Delivered"));
        assert!(msg.contains("https://example.test/s/tok"));
        assert!(msg.contains("Prompt:"));
        assert!(msg.contains("neon BUZZ rooftop"));
        assert!(msg.contains("intent: fi_abc"));
        assert!(msg.contains("buzz: evt123"));
        assert!(msg.contains("job: job-1"));
        // Human story above the ref footer
        assert!(msg.find("Watch").unwrap() < msg.find("intent:").unwrap());
        assert!(!msg.contains("tg_user"));
        assert!(!msg.contains("charge"));
        assert!(!msg.contains("USDC"));
    }

    #[test]
    fn payment_link_message() {
        let msg = format_payment_link(
            "https://t.me/MakeReel_xyz_bot?start=fuel_abc",
            "fi_1",
            "evt1",
            Some("dark neon workspace BUZZ"),
        );
        assert!(msg.contains("t.me/MakeReel_xyz_bot?start=fuel_abc"));
        assert!(msg.contains("Stars"));
        assert!(msg.contains("What to do:"));
        assert!(msg.contains("Prompt:"));
        assert!(msg.contains("dark neon workspace BUZZ"));
        assert!(msg.contains("intent: fi_1"));
        // Demo steps before operator refs
        assert!(msg.find("Open the link").unwrap() < msg.find("intent:").unwrap());
        assert!(!msg.contains("USDC"));
        assert!(!msg.contains("x402"));
        assert!(!msg.contains("tg_user"));
        assert!(!msg.contains("charge"));
    }

    #[test]
    fn payment_reply_with_media_has_markdown_and_imeta() {
        let media_url = "http://localhost:3000/media/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png";
        let image = PaymentImage {
            url: media_url.to_string(),
            imeta_tag: vec![
                "imeta".into(),
                format!("url {media_url}"),
                "m image/png".into(),
                "x aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                "size 999".into(),
                "dim 320x320".into(),
            ],
        };
        let (body, tags) = build_payment_reply(
            "https://t.me/MakeReel_xyz_bot?start=fuel_abc",
            "fi_1",
            "evt1",
            Some("demo prompt"),
            Some(&image),
        );
        assert!(body.contains("![image]("));
        assert!(body.contains(media_url));
        assert!(body.contains("What to do:"));
        assert!(body.contains("Prompt:"));
        assert!(body.contains("demo prompt"));
        assert!(body.contains("intent: fi_1"));
        assert!(body.contains("t.me/MakeReel_xyz_bot?start=fuel_abc"));
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0][0], "imeta");
        // Steps + footer still present; secrets still out
        assert!(!body.contains("USDC"));
        assert!(!body.contains("x402"));
        assert!(!body.contains("tg_user"));
        assert!(!body.contains("charge"));
    }

    #[test]
    fn payment_reply_upload_failure_is_text_only() {
        let (body, tags) = build_payment_reply(
            "https://t.me/MakeReel_xyz_bot?start=fuel_abc",
            "fi_1",
            "evt1",
            None,
            None,
        );
        assert!(!body.contains("![image]("));
        assert!(body.contains("t.me/MakeReel_xyz_bot?start=fuel_abc"));
        assert!(body.contains("intent: fi_1"));
        assert!(tags.is_empty());
        assert!(!body.contains("USDC"));
        assert!(!body.contains("x402"));
    }

    #[test]
    fn fueled_message_is_demo_readable() {
        let msg = format_status_message(
            "fueled",
            "evt9",
            "fi_9",
            Some("job-9"),
            None,
            Some("kinetic typography BUZZ"),
        );
        assert!(msg.contains("Fueled"));
        assert!(msg.contains("Stars received") || msg.contains("Stars payment"));
        assert!(msg.contains("Prompt:"));
        assert!(msg.contains("kinetic typography BUZZ"));
        assert!(msg.contains("intent: fi_9"));
        assert!(!msg.contains("USDC"));
    }
}
