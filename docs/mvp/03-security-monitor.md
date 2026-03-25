# MVP Spec 03: Security Monitor

> **Module:** `src/security_monitor.rs` (new — replaces `src/security_layer.rs`)
> **Dependencies:** Nostr relay event pipeline, SQLite
> **Personas:** Cipher (rule management), Sage (spam protection), Flint (audit logs)

---

## Purpose

Inspect every event entering the Nostr relay and every content upload to the store. Apply configurable rules to detect and block unwanted activity. Log all violations. Surface alerts to the GUI dashboard.

This is a **rules engine**, not AI. It's a series of configurable `if` checks that run on every inbound event. AI/ML analysis is a Phase 3+ enhancement.

---

## Architecture

```
Incoming Event/Content
  │
  ├─ SecurityMonitor.check_event(event, connection_id)
  │   │
  │   ├─ RateLimiter.check(pubkey, connection_id)
  │   ├─ SizeValidator.check(event)
  │   ├─ ContentPolicy.check(event)
  │   └─ EventValidator.check(event)  (structural, not crypto — crypto is in relay)
  │
  ├─ Result: Allow | Block(reason)
  │
  ├─ If Blocked:
  │   ├─ Log to security_alerts table
  │   ├─ Push to GUI alert feed (via channel)
  │   └─ Return rejection to caller
  │
  └─ If Allowed: continue pipeline
```

---

## Security Rules

### 1. Rate Limiting

Prevent any single client or pubkey from flooding the relay.

```rust
pub struct RateLimiterConfig {
    /// Max events per pubkey per window
    pub events_per_pubkey: u32,        // Default: 60
    /// Max events per connection per window
    pub events_per_connection: u32,    // Default: 100
    /// Window duration in seconds
    pub window_seconds: u64,           // Default: 60
    /// Max REQ subscriptions per connection
    pub max_subscriptions: u32,        // Default: 20
    /// Max filters per REQ
    pub max_filters_per_req: u32,      // Default: 10
}
```

**Implementation:** Sliding window counter using `HashMap<String, Vec<Instant>>`. Prune expired entries on each check.

**Alert type:** `rate_limit_exceeded`

### 2. Size Validation

Prevent oversized events from consuming storage or bandwidth.

```rust
pub struct SizeConfig {
    /// Maximum event JSON size in bytes
    pub max_event_bytes: usize,        // Default: 65_536 (64KB)
    /// Maximum content field length
    pub max_content_length: usize,     // Default: 32_768 (32KB)
    /// Maximum number of tags
    pub max_tags: usize,               // Default: 2000
    /// Maximum tag value length
    pub max_tag_value_length: usize,   // Default: 1024
}
```

**Alert type:** `event_too_large`

### 3. Content Policy

Block events matching configurable content patterns.

```rust
pub struct ContentPolicyConfig {
    /// Blocked event kinds (e.g., block kind 30023 long-form if relay doesn't want them)
    pub blocked_kinds: Vec<u64>,
    /// Maximum event age (reject events with created_at too far in the past)
    pub max_event_age_seconds: u64,    // Default: 86400 (24h)
    /// Maximum event future drift (reject events claiming to be from the future)
    pub max_future_drift_seconds: u64, // Default: 900 (15min)
}
```

**Alert type:** `blocked_kind`, `event_too_old`, `event_from_future`

### 4. Connection Limits

Prevent resource exhaustion at the connection level.

```rust
pub struct ConnectionConfig {
    /// Maximum concurrent WebSocket connections
    pub max_connections: u32,          // Default: 200
    /// Maximum connections per IP address
    pub max_connections_per_ip: u32,   // Default: 10
    /// Idle timeout before disconnection (seconds)
    pub idle_timeout_seconds: u64,     // Default: 300
    /// Challenge (NIP-42 AUTH) required before publishing
    pub require_auth_to_publish: bool, // Default: false
}
```

**Alert type:** `connection_limit`, `ip_limit_exceeded`

---

## Configuration

### TOML Config File

Security rules are loaded from `data/security_config.toml`:

```toml
[rate_limits]
events_per_pubkey = 60
events_per_connection = 100
window_seconds = 60
max_subscriptions = 20
max_filters_per_req = 10

[size_limits]
max_event_bytes = 65536
max_content_length = 32768
max_tags = 2000
max_tag_value_length = 1024

[content_policy]
blocked_kinds = []
max_event_age_seconds = 86400
max_future_drift_seconds = 900

[connections]
max_connections = 200
max_connections_per_ip = 10
idle_timeout_seconds = 300
require_auth_to_publish = false
```

### Defaults

If no config file exists, use built-in defaults. The app generates a default config file on first run so users can edit it.

### Hot Reload

Config file changes are detected via `notify` crate (filesystem watcher) and applied without restart. Changed rules take effect on the next event check.

---

## Rust API

```rust
pub struct SecurityMonitor {
    rate_limiter: RateLimiter,
    size_validator: SizeValidator,
    content_policy: ContentPolicy,
    connection_limits: ConnectionLimits,
    alert_log: AlertLog,
    alert_channel: broadcast::Sender<SecurityAlert>,
    config_path: PathBuf,
}

/// Result of checking an event
pub enum SecurityVerdict {
    Allow,
    Block {
        rule: String,           // e.g., "rate_limit_exceeded"
        reason: String,         // Human-readable explanation
    },
}

/// A logged security alert
pub struct SecurityAlert {
    pub id: String,             // UUID
    pub timestamp: DateTime<Utc>,
    pub alert_type: String,     // Rule that triggered
    pub severity: AlertSeverity,
    pub source_ip: Option<String>,
    pub source_pubkey: Option<String>,
    pub connection_id: String,
    pub details: String,        // Human-readable
    pub event_id: Option<String>,
}

pub enum AlertSeverity {
    Info,       // Informational (e.g., rate limit warning at 80%)
    Warning,    // Policy violation (blocked event)
    Critical,   // Sustained attack pattern
}

impl SecurityMonitor {
    /// Check an incoming Nostr event against all rules
    pub async fn check_event(
        &self,
        event: &NostrEvent,
        connection_id: &str,
        source_ip: &str,
    ) -> SecurityVerdict;

    /// Check a new connection attempt
    pub async fn check_connection(
        &self,
        source_ip: &str,
    ) -> SecurityVerdict;

    /// Check a content store upload
    pub async fn check_content(
        &self,
        content: &[u8],
        connection_id: &str,
    ) -> SecurityVerdict;

    /// Get recent alerts (for dashboard)
    pub async fn recent_alerts(&self, limit: usize) -> Vec<SecurityAlert>;

    /// Get alert statistics
    pub async fn stats(&self) -> SecurityStats;

    /// Subscribe to new alerts (for GUI real-time feed)
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<SecurityAlert>;

    /// Reload config from file
    pub async fn reload_config(&self) -> Result<()>;
}

pub struct SecurityStats {
    pub total_events_checked: u64,
    pub total_blocked: u64,
    pub total_allowed: u64,
    pub blocks_by_rule: HashMap<String, u64>,
    pub active_connections: u32,
    pub unique_pubkeys_seen: u32,
}
```

---

## Alert Storage (SQLite)

```sql
CREATE TABLE IF NOT EXISTS security_alerts (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    alert_type TEXT NOT NULL,
    severity TEXT NOT NULL,       -- 'info', 'warning', 'critical'
    source_ip TEXT,
    source_pubkey TEXT,
    connection_id TEXT NOT NULL,
    details TEXT NOT NULL,
    event_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX idx_alerts_timestamp ON security_alerts(timestamp DESC);
CREATE INDEX idx_alerts_type ON security_alerts(alert_type);
CREATE INDEX idx_alerts_severity ON security_alerts(severity);
CREATE INDEX idx_alerts_pubkey ON security_alerts(source_pubkey);
```

### Retention

- Default: 7 days of alerts retained
- Cleanup task runs hourly: `DELETE FROM security_alerts WHERE timestamp < ?`
- Configurable in TOML: `[alerts] retention_days = 7`

---

## Integration Points

### With Nostr Relay

```rust
// In nostr_relay.rs message handler:
async fn handle_event(&self, connection_id: &str, event: NostrEvent) -> NostrResponse {
    // 1. Crypto validation (ID + signature) — relay's job
    // 2. Security check — monitor's job
    let verdict = self.security_monitor
        .check_event(&event, connection_id, &source_ip)
        .await;

    match verdict {
        SecurityVerdict::Allow => {
            self.event_store.store_event(&event).await?;
            self.subscription_manager.broadcast_event(&event).await;
            NostrResponse::Ok { event_id: event.id, accepted: true, message: "".into() }
        }
        SecurityVerdict::Block { rule, reason } => {
            NostrResponse::Ok {
                event_id: event.id,
                accepted: false,
                message: format!("blocked: {}", reason),
            }
        }
    }
}
```

### With GUI (via Tauri)

```rust
// SecurityMonitor provides a broadcast channel
// Tauri command subscribes and forwards to frontend
#[tauri::command]
async fn get_security_alerts(
    monitor: State<'_, Arc<SecurityMonitor>>
) -> Result<Vec<SecurityAlert>, String> {
    Ok(monitor.recent_alerts(50).await)
}
```

---

## Current Code → MVP Changes

### security_layer.rs → security_monitor.rs (REWRITE)

| Current | Problem | MVP Replacement |
|---------|---------|----------------|
| XOR encryption | Not real encryption | Remove entirely — not the monitor's job |
| `EarthAllianceProfile` | Phase 2+ feature | Remove from MVP |
| `TeamConfiguration` | Phase 2+ feature | Remove from MVP |
| `verify_team_membership()` returns true | Stub | Remove |
| `validate_earth_alliance_event()` | Phase 2+ feature | Remove |
| No rate limiting | Core MVP feature missing | `RateLimiter` module |
| No size validation | Core MVP feature missing | `SizeValidator` module |
| No alert logging | Core MVP feature missing | `AlertLog` + SQLite table |
| Hardcoded encryption key | Security liability | Remove |

**Note:** `security_layer.rs` can be kept as a separate module for Phase 2 Earth Alliance features. The MVP security monitor is a distinct concern.

---

## Testing Strategy

### Unit Tests

| Test | Verifies |
|------|----------|
| `test_rate_limit_allows_within_window` | Events under limit pass |
| `test_rate_limit_blocks_over_window` | Events exceeding limit blocked |
| `test_rate_limit_resets_after_window` | Counter resets after window expires |
| `test_size_rejects_oversized_event` | Event > max_event_bytes blocked |
| `test_size_rejects_long_content` | Content > max_content_length blocked |
| `test_size_rejects_too_many_tags` | Tags > max_tags blocked |
| `test_content_policy_blocks_kind` | Blocked kind returns Block |
| `test_content_policy_rejects_future_event` | Event > max_future_drift blocked |
| `test_content_policy_rejects_old_event` | Event > max_event_age blocked |
| `test_connection_limit_per_ip` | Exceeding per-IP limit returns Block |
| `test_config_hot_reload` | Changed TOML file applies new limits |
| `test_alert_logged_on_block` | Block verdict creates security_alerts row |
| `test_alert_broadcast_on_block` | Block verdict sends on broadcast channel |

### Integration Tests

| Test | Verifies |
|------|----------|
| `test_relay_rejects_spam` | Client sending > rate limit gets OK(false) responses |
| `test_alerts_appear_in_api` | Blocked event → GET /api/v1/security/alerts returns it |
| `test_config_applied_on_startup` | Custom TOML limits respected immediately |

---

## Dependencies

### New

| Crate | Version | Use |
|-------|---------|-----|
| `notify` | 6.x | Filesystem watcher for config hot-reload |
| `toml` | 0.8 | Parse security config file |

### Already Present

| Crate | Use |
|-------|-----|
| `tokio` | Async runtime, broadcast channels |
| `sqlx` | Alert storage |
| `chrono` | Timestamps |
| `uuid` | Alert IDs |
| `serde` | Config deserialization |
