//! Shared types for MakeReel fuel intent status and Buzz message formatting.

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
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // decoded for logging / future C1-dry fields
pub struct CreateIntentResponse {
    pub intent_id: String,
    pub start_token: String,
    pub telegram_url: String,
    pub status: String,
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
}

/// Exact `/fuel` only — no NLP, no aliases.
pub fn is_exact_fuel(content: &str) -> bool {
    content.trim() == "/fuel"
}

/// Format a signed kind-9 status body. Safe IDs only — no TG identity or charge IDs.
pub fn format_status_message(
    phase_key: &str,
    buzz_event_id: &str,
    intent_id: &str,
    job_id: Option<&str>,
    share_url: Option<&str>,
) -> String {
    match phase_key {
        "fueled" => format!(
            "Fueled ✅\nintent: {intent_id}\nbuzz: {buzz_event_id}{}",
            job_id.map(|j| format!("\njob: {j}")).unwrap_or_default()
        ),
        "running" => format!(
            "Running…\nintent: {intent_id}\nbuzz: {buzz_event_id}{}",
            job_id.map(|j| format!("\njob: {j}")).unwrap_or_default()
        ),
        "delivered" => format!(
            "Delivered 🎬\nintent: {intent_id}\nbuzz: {buzz_event_id}{}\n{}",
            job_id.map(|j| format!("\njob: {j}")).unwrap_or_default(),
            share_url.unwrap_or("(share url pending)")
        ),
        "failed" => format!(
            "Failed/Refunded\nintent: {intent_id}\nbuzz: {buzz_event_id}{}",
            job_id.map(|j| format!("\njob: {j}")).unwrap_or_default()
        ),
        other => format!("{other}\nintent: {intent_id}\nbuzz: {buzz_event_id}"),
    }
}

pub fn format_payment_link(telegram_url: &str, intent_id: &str, buzz_event_id: &str) -> String {
    format!(
        "Fuel this launch reel with Telegram Stars:\n{telegram_url}\n\nintent: {intent_id}\nbuzz: {buzz_event_id}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c01_exact_fuel() {
        assert!(is_exact_fuel("/fuel"));
        assert!(is_exact_fuel("  /fuel  "));
    }

    #[test]
    fn c02_unknown_text_ignored() {
        assert!(!is_exact_fuel("fuel"));
        assert!(!is_exact_fuel("/fuel please"));
        assert!(!is_exact_fuel("/Fuel"));
        assert!(!is_exact_fuel("!fuel"));
        assert!(!is_exact_fuel("can you fuel this"));
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
    fn status_messages_omit_secrets() {
        let msg = format_status_message(
            "delivered",
            "evt123",
            "fi_abc",
            Some("job-1"),
            Some("https://example.test/s/tok"),
        );
        assert!(msg.contains("Delivered"));
        assert!(msg.contains("https://example.test/s/tok"));
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
        );
        assert!(msg.contains("t.me/MakeReel_xyz_bot?start=fuel_abc"));
        assert!(msg.contains("Stars"));
        assert!(!msg.contains("USDC"));
        assert!(!msg.contains("x402"));
    }
}
