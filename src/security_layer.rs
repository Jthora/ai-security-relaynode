use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use crate::nostr_relay::NostrEvent;

/// Security monitor for the relay.
///
/// Provides rate limiting per pubkey, content-size enforcement, blocked-word
/// filtering, and basic statistics.  Replaces the previous Earth Alliance
/// validation stub so that *all* valid Nostr events are accepted by default
/// while still protecting against abuse.
#[derive(Clone)]
pub struct SecurityLayer {
    /// Rate-limit state: pubkey → list of timestamps (sliding window).
    rate_limits: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    /// Config knobs.
    max_events_per_window: usize,
    window_secs: u64,
    max_content_bytes: usize,
    blocked_words: Vec<String>,
    /// Counters for stats.
    events_accepted: Arc<RwLock<u64>>,
    events_rejected: Arc<RwLock<u64>>,
}

impl SecurityLayer {
    /// Create with sensible defaults.
    pub async fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            max_events_per_window: 30,
            window_secs: 60,
            max_content_bytes: 64 * 1024, // 64 KB
            blocked_words: Vec::new(),
            events_accepted: Arc::new(RwLock::new(0)),
            events_rejected: Arc::new(RwLock::new(0)),
        }))
    }

    /// Validate a Nostr event.
    ///
    /// Called from `nostr_protocol.rs`.  Returns `Ok(true)` when the event
    /// passes all checks, `Ok(false)` when it should be rejected without
    /// being an internal error.
    pub async fn validate_earth_alliance_event(&self, event: &NostrEvent) -> Result<bool> {
        // 1. Content size check
        if event.content.len() > self.max_content_bytes {
            debug!("Rejected event {}: content too large ({} bytes)", event.id, event.content.len());
            *self.events_rejected.write().await += 1;
            return Ok(false);
        }

        // 2. Blocked-word filter
        if !self.blocked_words.is_empty() {
            let lower = event.content.to_lowercase();
            for word in &self.blocked_words {
                if lower.contains(word) {
                    debug!("Rejected event {}: blocked word", event.id);
                    *self.events_rejected.write().await += 1;
                    return Ok(false);
                }
            }
        }

        // 3. Rate limiting per pubkey
        if !self.check_rate_limit(&event.pubkey).await {
            debug!("Rejected event {}: rate limited (pubkey {})", event.id, &event.pubkey[..8.min(event.pubkey.len())]);
            *self.events_rejected.write().await += 1;
            return Ok(false);
        }

        *self.events_accepted.write().await += 1;
        Ok(true)
    }

    /// Sliding-window rate limiter.  Returns `true` if the request is allowed.
    async fn check_rate_limit(&self, pubkey: &str) -> bool {
        let now = Instant::now();
        let cutoff = now - std::time::Duration::from_secs(self.window_secs);

        let mut limits = self.rate_limits.write().await;
        let timestamps = limits.entry(pubkey.to_string()).or_default();

        // Evict expired entries
        timestamps.retain(|t| *t > cutoff);

        if timestamps.len() >= self.max_events_per_window {
            return false;
        }

        timestamps.push(now);
        true
    }

    // ---- Compatibility stubs (used by subscription_manager) ----

    pub async fn get_user_team(&self, _pubkey: &str) -> Result<Option<String>> {
        Ok(None)
    }

    pub async fn get_user_clearance(&self, _pubkey: &str) -> Result<ClearanceLevel> {
        Ok(ClearanceLevel::Unclassified)
    }

    pub async fn get_security_stats(&self) -> Result<serde_json::Value> {
        let accepted = *self.events_accepted.read().await;
        let rejected = *self.events_rejected.read().await;
        let active_keys = self.rate_limits.read().await.len();

        Ok(serde_json::json!({
            "events_accepted": accepted,
            "events_rejected": rejected,
            "active_rate_limit_keys": active_keys,
        }))
    }

    pub async fn log_security_event(&self, event_type: &str, details: &str) {
        info!("🔐 Security event [{}]: {}", event_type, details);
    }

    // ---- Legacy method retained for any callers ----
    pub async fn verify_team_membership(&self, _user_id: &str) -> Result<bool> {
        Ok(true) // MVP: accept all
    }
}

/// Clearance levels (kept for backward compat; re-exported from event_store)
use crate::event_store::ClearanceLevel;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_accepts_normal_event() {
        let layer = SecurityLayer::new().await.unwrap();
        let event = make_event("abc123", "test content");
        assert!(layer.validate_earth_alliance_event(&event).await.unwrap());
    }

    #[tokio::test]
    async fn test_rejects_oversized_content() {
        let layer = SecurityLayer::new().await.unwrap();
        let big = "x".repeat(100_000);
        let event = make_event("abc123", &big);
        assert!(!layer.validate_earth_alliance_event(&event).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let layer = Arc::new(SecurityLayer {
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            max_events_per_window: 3,
            window_secs: 60,
            max_content_bytes: 64 * 1024,
            blocked_words: Vec::new(),
            events_accepted: Arc::new(RwLock::new(0)),
            events_rejected: Arc::new(RwLock::new(0)),
        });

        let event = make_event("pubkey1", "hi");
        assert!(layer.validate_earth_alliance_event(&event).await.unwrap());
        assert!(layer.validate_earth_alliance_event(&event).await.unwrap());
        assert!(layer.validate_earth_alliance_event(&event).await.unwrap());
        // 4th should be rate-limited
        assert!(!layer.validate_earth_alliance_event(&event).await.unwrap());
    }

    #[tokio::test]
    async fn test_stats() {
        let layer = SecurityLayer::new().await.unwrap();
        let event = make_event("abc", "hello");
        layer.validate_earth_alliance_event(&event).await.unwrap();

        let stats = layer.get_security_stats().await.unwrap();
        assert_eq!(stats["events_accepted"], 1);
    }

    fn make_event(pubkey: &str, content: &str) -> NostrEvent {
        NostrEvent {
            id: "0".repeat(64),
            pubkey: pubkey.to_string(),
            created_at: 1700000000,
            kind: 1,
            tags: vec![],
            content: content.to_string(),
            sig: "0".repeat(128),
        }
    }
}
