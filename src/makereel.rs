//! Internal MakeReel Buzz Fuel HTTP client (Bearer).

use anyhow::{anyhow, Context, Result};
use reqwest::Client;

use crate::model::{CreateIntentResponse, IntentStatus};

#[derive(Clone)]
pub struct MakeReelClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl MakeReelClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    pub async fn create_intent(
        &self,
        buzz_channel_id: &str,
        buzz_request_event_id: &str,
    ) -> Result<CreateIntentResponse> {
        let url = format!("{}/internal/buzz-fuel/intents", self.base_url);
        let res = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "buzz_channel_id": buzz_channel_id,
                "buzz_request_event_id": buzz_request_event_id,
            }))
            .send()
            .await
            .context("create_intent request failed")?;
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("create_intent HTTP {status}: {body}"));
        }
        serde_json::from_str(&body).context("create_intent decode")
    }

    pub async fn get_intent(&self, intent_id: &str) -> Result<IntentStatus> {
        let url = format!("{}/internal/buzz-fuel/intents/{intent_id}", self.base_url);
        let res = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("get_intent request failed")?;
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("get_intent HTTP {status}: {body}"));
        }
        serde_json::from_str(&body).context("get_intent decode")
    }
}
