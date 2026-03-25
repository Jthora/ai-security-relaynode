# MVP Spec 07: Acceptance Tests

> **Test framework:** Rust's built-in `#[test]` + `#[tokio::test]`
> **Location:** `tests/unit/`, `tests/integration/`
> **No external test runners required**

---

## Purpose

Consolidated acceptance criteria for every MVP component. Each test maps to a user story from [MVP-PLAN.md](../../MVP-PLAN.md). A test passes iff the feature works end-to-end from the user's perspective.

---

## Test Categories

| Category | Location | Runs Against |
|----------|----------|--------------|
| Unit | `tests/unit/` | Individual structs/functions |
| Integration | `tests/integration/` | Multiple modules wired together |
| Manual | Documented below | Running binary + WebSocket client |

---

## 1. Nostr Relay Tests

Source spec: [01-nostr-relay.md](01-nostr-relay.md)

### Unit Tests

```
test_nostr_event_id_computation
    Given a Nostr event with known fields
    When SHA-256 is computed per NIP-01
    Then event.id matches expected hex string

test_nostr_signature_validation_valid
    Given an event with a valid Schnorr signature
    When validate_signature() is called
    Then returns true

test_nostr_signature_validation_invalid
    Given an event with a tampered signature
    When validate_signature() is called
    Then returns false

test_nostr_message_parse_event
    Given a JSON array: ["EVENT", {event_object}]
    When parse_client_message() is called
    Then returns ClientMessage::Event with correct fields

test_nostr_message_parse_req
    Given a JSON array: ["REQ", "sub1", {filter}]
    When parse_client_message() is called
    Then returns ClientMessage::Req with subscription_id and filters

test_nostr_message_parse_close
    Given a JSON array: ["CLOSE", "sub1"]
    When parse_client_message() is called
    Then returns ClientMessage::Close with subscription_id

test_nostr_message_parse_invalid
    Given malformed JSON
    When parse_client_message() is called
    Then returns error (not panic)
```

### Integration Tests

```
test_relay_accepts_websocket_connection
    Given a running NostrRelay on port 8080
    When a client opens a WebSocket connection
    Then connection is accepted and client receives no immediate error

test_relay_stores_valid_event
    Given a connected client
    When client sends ["EVENT", {valid_event}]
    Then relay responds ["OK", event_id, true, ""]
    And event exists in database

test_relay_rejects_invalid_signature
    Given a connected client
    When client sends ["EVENT", {event_with_bad_sig}]
    Then relay responds ["OK", event_id, false, "invalid: signature verification failed"]

test_relay_subscription_receives_stored_events
    Given a database with 3 events of kind 1
    When client sends ["REQ", "sub1", {"kinds": [1]}]
    Then client receives 3 ["EVENT", "sub1", {event}] messages
    Followed by ["EOSE", "sub1"]

test_relay_subscription_receives_new_events
    Given client A subscribed with ["REQ", "sub1", {"kinds": [1]}]
    When client B sends a valid kind-1 event
    Then client A receives ["EVENT", "sub1", {new_event}]

test_relay_close_subscription
    Given a client with active subscription "sub1"
    When client sends ["CLOSE", "sub1"]
    Then no more events delivered for "sub1"
    And new events matching the old filter are NOT forwarded

test_relay_count_events
    Given 5 events of kind 1 in the database
    When client sends ["COUNT", "count1", {"kinds": [1]}]
    Then client receives ["COUNT", "count1", {"count": 5}]
```

### Manual Tests

```
manual_relay_nak_compatibility
    Using: nak (https://github.com/fiatjaf/nak)
    Steps:
      1. Start relay
      2. nak event -k 1 -c "hello" ws://localhost:8080
      3. nak req -k 1 ws://localhost:8080
    Expected: Event published and retrieved

manual_relay_handles_100_concurrent_connections
    Using: any WebSocket load tool
    Steps:
      1. Start relay
      2. Open 100 simultaneous WebSocket connections
      3. Each sends 1 event
    Expected: All 100 events stored, no crashes, memory < 200MB
```

---

## 2. Content Store Tests

Source spec: [02-content-store.md](02-content-store.md)

### Unit Tests

```
test_content_store_computes_correct_cid
    Given content bytes b"hello world"
    When store() is called
    Then returned CID matches sha256("hello world") hex

test_content_store_deduplicates
    Given content "hello" already stored
    When store("hello") is called again
    Then no new file created, same CID returned

test_content_store_retrieve
    Given content stored with CID "abc123..."
    When retrieve("abc123...") is called
    Then original bytes returned

test_content_store_retrieve_missing
    Given CID "nonexistent"
    When retrieve("nonexistent") is called
    Then returns None/error

test_content_store_rejects_oversized
    Given max_content_size = 1024
    When store(2048_bytes) is called
    Then returns error "content exceeds maximum size"

test_content_store_list_items
    Given 3 items stored
    When list(limit=10, offset=0) is called
    Then returns 3 items with CID, size, stored_at

test_content_store_stats
    Given 3 items stored totaling 1500 bytes
    When stats() is called
    Then returns {items: 3, total_bytes: 1500}
```

### Integration Tests

```
test_content_store_persists_across_restart
    Given content stored
    When ContentStore is dropped and recreated with same data_dir
    Then content still retrievable by CID
```

---

## 3. Security Monitor Tests

Source spec: [03-security-monitor.md](03-security-monitor.md)

### Unit Tests

```
test_rate_limiter_allows_under_limit
    Given rate_limit = 60/min, counter at 0
    When 30 events checked in 1 second
    Then all 30 return RuleResult::Allow

test_rate_limiter_blocks_over_limit
    Given rate_limit = 60/min, counter at 60
    When another event is checked
    Then returns RuleResult::Reject("rate_limit_exceeded")

test_rate_limiter_window_resets
    Given rate_limit = 60/min, counter at 60
    When 61 seconds pass and new event checked
    Then returns RuleResult::Allow

test_content_size_rule_allows_normal
    Given max_content_bytes = 65536
    When event with 1000-byte content checked
    Then returns RuleResult::Allow

test_content_size_rule_rejects_oversized
    Given max_content_bytes = 65536
    When event with 100000-byte content checked
    Then returns RuleResult::Reject("content_too_large")

test_blocklist_rejects_blocked_pubkey
    Given pubkey "abc..." on blocklist
    When event from "abc..." checked
    Then returns RuleResult::Reject("pubkey_blocked")

test_allowlist_mode_rejects_unknown
    Given allowlist_only = true, allowlist = ["abc..."]
    When event from "def..." checked
    Then returns RuleResult::Reject("pubkey_not_allowlisted")

test_event_validation_rejects_bad_id
    Given event where id != sha256(serialized)
    When check_event() is called
    Then returns RuleResult::Reject("invalid_event_id")

test_security_alert_logged
    Given a rejection for rate_limit_exceeded
    When alert is logged
    Then security_alerts table has new row with correct type and severity
```

### Integration Tests

```
test_monitor_in_relay_pipeline
    Given relay running with rate_limit = 2/min
    When client sends 3 events in 10 seconds
    Then first 2 get ["OK", id, true, ""]
    And third gets ["OK", id, false, "rate_limit_exceeded"]
    And security_alerts has 1 new row
```

---

## 4. Tauri GUI Tests

Source spec: [04-tauri-gui.md](04-tauri-gui.md)

### Unit Tests (Rust side)

```
test_tauri_get_status_returns_json
    Given NostrRelay running, ContentStore initialized
    When get_service_status() invoked
    Then returns JSON with relay.running=true, content_store.items >= 0

test_tauri_get_security_alerts_returns_array
    Given 5 alerts in database
    When get_security_alerts(limit=3) invoked
    Then returns JSON array of length 3

test_tauri_get_relay_config_returns_current
    When get_relay_config() invoked
    Then returns JSON matching current Config values

test_tauri_update_relay_config_validates
    Given invalid port = 99999
    When update_relay_config({port: 99999}) invoked
    Then returns error (not crash)
```

### Manual Tests

```
manual_gui_shows_status_on_launch
    Steps:
      1. Start app
      2. Observe dashboard
    Expected: Status cards show relay port, connection count, content items, alert count
    Expected: All values update within 5 seconds of startup

manual_gui_security_feed_updates
    Steps:
      1. Start app
      2. Trigger a rate-limit violation via WebSocket tool
      3. Observe security alerts panel
    Expected: New alert appears < 3 seconds after violation

manual_gui_start_stop_services
    Steps:
      1. Start app
      2. Click "Stop Relay"
      3. Verify WebSocket connections refused
      4. Click "Start Relay"
      5. Verify WebSocket connections accepted
    Expected: Service toggling works without app restart
```

---

## 5. Data Model Tests

Source spec: [05-data-model.md](05-data-model.md)

### Unit Tests

```
test_database_migrations_run
    Given empty SQLite database
    When migrations applied
    Then tables events, content_index, security_alerts, relay_stats, schema_version exist

test_event_insert_and_retrieve
    Given empty events table
    When valid event inserted
    Then SELECT by id returns same event

test_event_duplicate_ignored
    Given event with id "abc" already stored
    When same event inserted again
    Then no error, still 1 row

test_event_filter_by_kind
    Given events of kind 1, 1, 3
    When queried with kinds=[1]
    Then returns 2 events

test_event_filter_by_author_prefix
    Given events from pubkeys "aabb..." and "aacc..."
    When queried with authors=["aa"]
    Then returns 2 events (prefix match)

test_event_filter_since_until
    Given events at timestamps 100, 200, 300
    When queried with since=150, until=250
    Then returns 1 event (timestamp 200)

test_event_filter_limit
    Given 10 events
    When queried with limit=3
    Then returns 3 events (most recent)

test_stats_snapshot_insert
    When relay_stats row inserted
    Then retrievable by timestamp range
```

---

## 6. Configuration Tests

Source spec: [06-config-packaging.md](06-config-packaging.md)

### Unit Tests

```
test_config_defaults
    When Config::defaults() called
    Then relay_port=8080, api_port=8081, max_connections=200

test_config_from_file
    Given config.toml with relay.port=9090
    When Config::load() called
    Then relay_port=9090, other fields at defaults

test_config_env_override
    Given env RELAY_PORT=7777
    When Config::load() called
    Then relay_port=7777

test_config_env_overrides_file
    Given config.toml with relay.port=9090 AND env RELAY_PORT=7777
    When Config::load() called
    Then relay_port=7777 (env wins)

test_config_missing_file_uses_defaults
    Given no config.toml exists
    When Config::load() called
    Then no error, all defaults applied

test_jwt_secret_auto_generated
    Given no JWT_SECRET env var and no file secret
    When Config::load() called
    Then jwt_secret is non-empty string of sufficient length
```

---

## Test Matrix Summary

| Component | Unit Tests | Integration Tests | Manual Tests |
|-----------|-----------|-------------------|-------------|
| Nostr Relay | 7 | 7 | 2 |
| Content Store | 7 | 1 | 0 |
| Security Monitor | 9 | 1 | 0 |
| Tauri GUI | 4 | 0 | 3 |
| Data Model | 8 | 0 | 0 |
| Configuration | 6 | 0 | 0 |
| **Total** | **41** | **9** | **5** |

---

## Running Tests

```bash
# All unit tests
cargo test --lib

# All integration tests
cargo test --test '*'

# Specific component
cargo test nostr
cargo test content_store
cargo test security

# With output
cargo test -- --nocapture

# Single test
cargo test test_relay_stores_valid_event
```

---

## CI Readiness (Phase 2)

```yaml
# .github/workflows/test.yml (future)
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all
      - run: cargo clippy -- -D warnings
```

Not needed for MVP, but tests should be written to be CI-compatible from day one.
