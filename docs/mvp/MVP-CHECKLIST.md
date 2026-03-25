# MVP Master Checklist

> **AI Security RelayNode v0.1.0**
> Comprehensive workload tracker — every task from every spec document.
> Structure: **Stage → Phase → Step → Task**

---

## Stage 1: Infrastructure & Foundation

### Phase 1.1: Dependency Cleanup

#### Step 1.1.1 — Trim Cargo.toml
_Ref: [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.1_

- [ ] 1.1.1.1 — Remove `libp2p` dependency (never imported, 200+ transitive deps)
- [ ] 1.1.1.2 — Remove `num-bigint` dependency
- [ ] 1.1.1.3 — Remove `num-traits` dependency
- [ ] 1.1.1.4 — Remove `crossbeam-channel` dependency (not used in live code)
- [ ] 1.1.1.5 — Add `toml = "0.8"` (config file parsing)
- [ ] 1.1.1.6 — Add `dirs = "5"` (platform-specific data directories)
- [ ] 1.1.1.7 — Add `notify = "6"` (config file hot-reload, security monitor)
- [ ] 1.1.1.8 — Verify `cargo check` passes
- [ ] 1.1.1.9 — Commit: `chore: trim unused deps, add toml + dirs`

#### Step 1.1.2 — Add Release Build Profile
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [ ] 1.1.2.1 — Add `[profile.release]` to Cargo.toml: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`
- [ ] 1.1.2.2 — Verify `cargo build --release` produces binary < 30MB

---

### Phase 1.2: Configuration System

#### Step 1.2.1 — Implement Config Loading
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.2_

- [ ] 1.2.1.1 — Implement `Config::defaults()` returning hardcoded relay_port=8080, api_port=8081, max_connections=200
- [ ] 1.2.1.2 — Implement `Config::config_path()` using `dirs::config_dir()` with `./data/` fallback
- [ ] 1.2.1.3 — Define `ConfigFile` struct with `#[derive(Deserialize)]` matching TOML structure
- [ ] 1.2.1.4 — Implement `Config::load()`: read `data/config.toml` → parse → merge over defaults
- [ ] 1.2.1.5 — Add env var overrides: `RELAY_PORT`, `API_PORT`, `JWT_SECRET`, `DATABASE_URL`, `DATA_DIR`
- [ ] 1.2.1.6 — Auto-generate `JWT_SECRET` (64-byte random hex) if not set
- [ ] 1.2.1.7 — Remove dead fields: `SubnetMode`, `GatewayConfig`, `TeamSubnetConfig`

#### Step 1.2.2 — First-Run Bootstrapping
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [ ] 1.2.2.1 — Create `data/` directory on first run if missing
- [ ] 1.2.2.2 — Generate `data/config.toml` with defaults written out
- [ ] 1.2.2.3 — Generate `data/security_config.toml` with default rules
- [ ] 1.2.2.4 — Create `data/cas/` directory for content store

#### Step 1.2.3 — Config Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §6_

- [ ] 1.2.3.1 — Write `test_config_defaults` — `Config::defaults()` returns correct values
- [ ] 1.2.3.2 — Write `test_config_from_file` — TOML file overrides defaults
- [ ] 1.2.3.3 — Write `test_config_env_override` — env var overrides all
- [ ] 1.2.3.4 — Write `test_config_env_overrides_file` — env wins over file
- [ ] 1.2.3.5 — Write `test_config_missing_file_uses_defaults` — no file → defaults apply
- [ ] 1.2.3.6 — Write `test_jwt_secret_auto_generated` — empty secret → auto-generate
- [ ] 1.2.3.7 — Commit: `feat: config loading from file + env vars`

---

### Phase 1.3: Database & Migration System

#### Step 1.3.1 — Schema Version Tracking
_Ref: [05-data-model.md](mvp/05-data-model.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.5_

- [ ] 1.3.1.1 — Create `schema_version` table (version INTEGER PK, applied_at INTEGER)
- [ ] 1.3.1.2 — Implement migration runner: check version → run pending → update version
- [ ] 1.3.1.3 — Run migrations on startup in `DatabaseManager::new()`

#### Step 1.3.2 — MVP Schema Migration
_Ref: [05-data-model.md](mvp/05-data-model.md)_

- [ ] 1.3.2.1 — Write `src/migrations/001_mvp_schema.sql`
- [ ] 1.3.2.2 — Create `events` table (id, pubkey, created_at, kind, tags, content, sig, stored_at)
- [ ] 1.3.2.3 — Create events indexes: pubkey, created_at DESC, kind, (pubkey, kind)
- [ ] 1.3.2.4 — Create `content_index` table (cid, size_bytes, content_type, stored_at, accessed_at)
- [ ] 1.3.2.5 — Create content_index indexes: stored_at DESC, size_bytes
- [ ] 1.3.2.6 — Create `security_alerts` table (id, timestamp, alert_type, severity, source_ip, source_pubkey, connection_id, details, event_id)
- [ ] 1.3.2.7 — Create security_alerts indexes: timestamp DESC, alert_type, severity
- [ ] 1.3.2.8 — Create `relay_stats` table (timestamp, connections, events_stored, events_minute, content_items, content_bytes, alerts_total, memory_bytes)
- [ ] 1.3.2.9 — Remove old investigation tables from startup migration path

#### Step 1.3.3 — Database Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §5_

- [ ] 1.3.3.1 — Write `test_database_migrations_run` — all tables exist after migration on empty DB
- [ ] 1.3.3.2 — Write `test_stats_snapshot_insert` — relay_stats row insert/retrieve
- [ ] 1.3.3.3 — Commit: `feat: versioned database migrations`

---

## Stage 2: Working Nostr Relay

### Phase 2.1: Event Store Fixes

#### Step 2.1.1 — Fix Query Parameter Binding
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.3_

- [ ] 2.1.1.1 — Fix `build_filter_where_clause()` to return `(String, Vec<sqlx::Value>)` tuple
- [ ] 2.1.1.2 — Use returned params in `sqlx::query()` bindings (fix discarded params bug)
- [ ] 2.1.1.3 — Add prefix matching for IDs using `LIKE 'prefix%'`
- [ ] 2.1.1.4 — Add prefix matching for authors using `LIKE 'prefix%'`
- [ ] 2.1.1.5 — Add tag-based querying using SQLite `json_each()` + `json_extract()`
- [ ] 2.1.1.6 — Remove Earth Alliance clearance columns (Phase 2)
- [ ] 2.1.1.7 — Simplify to MVP schema (events table only, no investigation)

#### Step 2.1.2 — Event Store Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §5_

- [ ] 2.1.2.1 — Write `test_event_insert_and_retrieve` — store → SELECT by ID → match
- [ ] 2.1.2.2 — Write `test_event_duplicate_ignored` — duplicate ID → no error, still 1 row
- [ ] 2.1.2.3 — Write `test_event_filter_by_kind` — kinds=[1] → returns only kind-1 events
- [ ] 2.1.2.4 — Write `test_event_filter_by_author_prefix` — authors=["aa"] → prefix match
- [ ] 2.1.2.5 — Write `test_event_filter_since_until` — time range filtering
- [ ] 2.1.2.6 — Write `test_event_filter_limit` — LIMIT clause honored
- [ ] 2.1.2.7 — Commit: `fix: event store query parameter binding`

---

### Phase 2.2: Nostr Protocol Fixes

#### Step 2.2.1 — Protocol Handler Updates
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.2.1.1 — Fix signature verification: switch from ECDSA to Schnorr (BIP-340) per NIP-01
- [ ] 2.2.1.2 — Add prefix matching for filter IDs/authors (currently exact match only)
- [ ] 2.2.1.3 — Add tag filter matching (`#e`, `#p` tag queries)
- [ ] 2.2.1.4 — Remove Earth Alliance metadata extraction (Phase 2)

#### Step 2.2.2 — Protocol Unit Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §1_

- [ ] 2.2.2.1 — Write `test_nostr_event_id_computation` — SHA-256 matches NIP-01 known vectors
- [ ] 2.2.2.2 — Write `test_nostr_signature_validation_valid` — valid Schnorr sig accepted
- [ ] 2.2.2.3 — Write `test_nostr_signature_validation_invalid` — tampered sig rejected
- [ ] 2.2.2.4 — Write `test_nostr_message_parse_event` — JSON → ClientMessage::Event
- [ ] 2.2.2.5 — Write `test_nostr_message_parse_req` — JSON → ClientMessage::Req with filters
- [ ] 2.2.2.6 — Write `test_nostr_message_parse_close` — JSON → ClientMessage::Close
- [ ] 2.2.2.7 — Write `test_nostr_message_parse_invalid` — malformed JSON → error (not panic)

---

### Phase 2.3: WebSocket Message Dispatch

#### Step 2.3.1 — Wire Read Loop
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.4_

- [ ] 2.3.1.1 — Replace `handle_websocket_messages()` sleep loop with `while let Some(msg) = ws_reader.next().await`
- [ ] 2.3.1.2 — Split WebSocket stream using `ws_stream.split()` → `(ws_writer, ws_reader)`
- [ ] 2.3.1.3 — On `Message::Text` → parse via `NostrProtocolHandler` → dispatch → send response
- [ ] 2.3.1.4 — On `Message::Ping` → reply with Pong
- [ ] 2.3.1.5 — On `Message::Close` / `None` → cleanup connection, break loop
- [ ] 2.3.1.6 — On error → log, break

#### Step 2.3.2 — Event Processing Pipeline
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.3.2.1 — EVENT handler: validate event ID (SHA-256 check)
- [ ] 2.3.2.2 — EVENT handler: validate signature (Schnorr BIP-340)
- [ ] 2.3.2.3 — EVENT handler: store in EventStore via `INSERT OR IGNORE`
- [ ] 2.3.2.4 — EVENT handler: broadcast to matching subscriptions
- [ ] 2.3.2.5 — EVENT handler: send `["OK", event_id, true/false, message]` response

#### Step 2.3.3 — Subscription Processing
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.3.3.1 — REQ handler: register subscription with SubscriptionManager
- [ ] 2.3.3.2 — REQ handler: query EventStore for historical matching events
- [ ] 2.3.3.3 — REQ handler: send matching events as `["EVENT", sub_id, event]`
- [ ] 2.3.3.4 — REQ handler: send `["EOSE", sub_id]` after historical events
- [ ] 2.3.3.5 — REQ handler: replace existing subscription if same sub_id reused
- [ ] 2.3.3.6 — CLOSE handler: remove subscription, send `["CLOSED", sub_id, ""]`
- [ ] 2.3.3.7 — COUNT handler: query count, send `["COUNT", sub_id, {"count": N}]`

#### Step 2.3.4 — Subscription Manager Fixes
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.3.4.1 — Remove clearance/team access control (not MVP)
- [ ] 2.3.4.2 — Fix duplicate subscription handling: REQ with existing sub_id replaces old
- [ ] 2.3.4.3 — Verify EOSE sent after historical events before streaming

#### Step 2.3.5 — Connection Lifecycle
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.3.5.1 — Assign UUID `connection_id` on WebSocket accept
- [ ] 2.3.5.2 — Register connection in SubscriptionManager
- [ ] 2.3.5.3 — On disconnect: remove all subscriptions for connection
- [ ] 2.3.5.4 — Implement 300s idle timeout
- [ ] 2.3.5.5 — Commit: `feat: wire WebSocket read loop to message processing`

---

### Phase 2.4: Relay Integration Tests

#### Step 2.4.1 — Automated Integration Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §1, [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.6_

- [ ] 2.4.1.1 — Write `test_relay_accepts_websocket_connection` — handshake succeeds
- [ ] 2.4.1.2 — Write `test_relay_stores_valid_event` — EVENT → OK(true) + in DB
- [ ] 2.4.1.3 — Write `test_relay_rejects_invalid_signature` — bad sig → OK(false)
- [ ] 2.4.1.4 — Write `test_relay_subscription_receives_stored_events` — REQ → events + EOSE
- [ ] 2.4.1.5 — Write `test_relay_subscription_receives_new_events` — subscribe → publish → receive
- [ ] 2.4.1.6 — Write `test_relay_close_subscription` — CLOSE removes subscription
- [ ] 2.4.1.7 — Write `test_relay_count_events` — COUNT returns correct count

#### Step 2.4.2 — Manual Relay Verification
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §1_

- [ ] 2.4.2.1 — Test with `nak` CLI: publish and retrieve events via `ws://localhost:8080`
- [ ] 2.4.2.2 — Test 100 concurrent WebSocket connections (all events stored, no crash, <200MB RAM)
- [ ] 2.4.2.3 — Commit: `test: relay round-trip integration test`

#### Step 2.4.3 — Performance Verification
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.4.3.1 — Verify 100+ concurrent connections supported
- [ ] 2.4.3.2 — Verify 500+ events/second sustained throughput
- [ ] 2.4.3.3 — Verify event validation latency < 5ms
- [ ] 2.4.3.4 — Verify historical query (1000 events) < 50ms
- [ ] 2.4.3.5 — Verify memory per connection < 500KB

> **STAGE 2 MILESTONE:** `cargo test` passes. WebSocket client can connect, publish events, subscribe, and receive events. NIP-01 compliant, SQLite-backed relay.

---

## Stage 3: Content-Addressed Storage

### Phase 3.1: Storage Module

#### Step 3.1.1 — Replace Fake IPFS
_Ref: [02-content-store.md](mvp/02-content-store.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 2.1_

- [ ] 3.1.1.1 — Rename `src/ipfs_node.rs` → `src/content_store.rs`
- [ ] 3.1.1.2 — Rename `IPFSNode` struct → `ContentStore`
- [ ] 3.1.1.3 — Update `src/lib.rs` module declaration (`pub mod content_store`)
- [ ] 3.1.1.4 — Update all imports referencing `IPFSNode` / `ipfs_node`
- [ ] 3.1.1.5 — Remove in-memory `HashMap<String, Vec<u8>>` storage
- [ ] 3.1.1.6 — Remove XOR "encryption" logic
- [ ] 3.1.1.7 — Remove fake hash generation (`Qm{uuid}`)

#### Step 3.1.2 — Content-Addressed Storage Engine
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [ ] 3.1.2.1 — Implement SHA-256 → CID computation (`store(bytes) → cid`)
- [ ] 3.1.2.2 — Implement 2-level directory layout: `data/cas/ab/cd/abcd...`
- [ ] 3.1.2.3 — Implement `store(&[u8]) → Result<String>`: hash → write file → insert metadata
- [ ] 3.1.2.4 — Implement `retrieve(cid) → Result<Vec<u8>>`: lookup → read file → return bytes
- [ ] 3.1.2.5 — Implement `exists(cid) → Result<bool>`
- [ ] 3.1.2.6 — Implement `delete(cid) → Result<bool>`: remove file + metadata
- [ ] 3.1.2.7 — Implement `stats() → Result<ContentStoreStats>`: item count + total bytes
- [ ] 3.1.2.8 — Implement `list(offset, limit) → Result<Vec<ContentMetadata>>`: paginated listing

#### Step 3.1.3 — Size Limit Enforcement
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [ ] 3.1.3.1 — Enforce `max_content_size` per item (default 10MB) → 413 error
- [ ] 3.1.3.2 — Enforce `max_storage_bytes` total (default 5GB) → 507 error
- [ ] 3.1.3.3 — Deduplication: same content → same CID, no new file written

#### Step 3.1.4 — Content Store Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §2_

- [ ] 3.1.4.1 — Write `test_content_store_computes_correct_cid` — bytes → CID matches SHA-256
- [ ] 3.1.4.2 — Write `test_content_store_deduplicates` — same content twice = one file
- [ ] 3.1.4.3 — Write `test_content_store_retrieve` — retrieve by CID returns original bytes
- [ ] 3.1.4.4 — Write `test_content_store_retrieve_missing` — missing CID returns error
- [ ] 3.1.4.5 — Write `test_content_store_rejects_oversized` — >max_size returns error
- [ ] 3.1.4.6 — Write `test_content_store_list_items` — list returns paginated metadata
- [ ] 3.1.4.7 — Write `test_content_store_stats` — stats reflects actual items/bytes
- [ ] 3.1.4.8 — Write `test_content_store_persists_across_restart` — store → drop → recreate → retrieve
- [ ] 3.1.4.9 — Commit: `feat: content-addressed storage replacing fake IPFS`

---

### Phase 3.2: Content Store API

#### Step 3.2.1 — HTTP Routes
_Ref: [02-content-store.md](mvp/02-content-store.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 2.2_

- [ ] 3.2.1.1 — Implement `POST /api/v1/content` → store raw bytes, return `{ "cid": "..." }`
- [ ] 3.2.1.2 — Implement `GET /api/v1/content/:cid` → return raw bytes with Content-Type
- [ ] 3.2.1.3 — Implement `HEAD /api/v1/content/:cid` → 200 if exists, 404 if not
- [ ] 3.2.1.4 — Implement `GET /api/v1/content` → list items (paginated, JSON)
- [ ] 3.2.1.5 — Implement `GET /api/v1/content/stats` → storage statistics
- [ ] 3.2.1.6 — Remove hardcoded `/ipfs/status` route from api_gateway.rs
- [ ] 3.2.1.7 — Add content-type detection for stored content

#### Step 3.2.2 — Content API Tests
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [ ] 3.2.2.1 — Write `test_http_store_retrieve` — POST content → GET by CID → match
- [ ] 3.2.2.2 — Write `test_http_head_exists` — HEAD returns 200/404 correctly
- [ ] 3.2.2.3 — Commit: `feat: content store API routes`

#### Step 3.2.3 — Performance Verification
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [ ] 3.2.3.1 — Verify store 1MB file < 100ms
- [ ] 3.2.3.2 — Verify retrieve 1MB file < 50ms
- [ ] 3.2.3.3 — Verify CID computation (1MB) < 10ms
- [ ] 3.2.3.4 — Verify 50+ concurrent reads supported

> **STAGE 3 MILESTONE:** Content can be stored and retrieved by hash via HTTP API. Durable, deduplicated, size-limited.

---

## Stage 4: Security Monitor

### Phase 4.1: Rules Engine

#### Step 4.1.1 — Security Monitor Module
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 3.1_

- [ ] 4.1.1.1 — Rewrite `src/security_layer.rs` → define `SecurityMonitor` struct
- [ ] 4.1.1.2 — Remove XOR encryption logic entirely
- [ ] 4.1.1.3 — Remove `EarthAllianceProfile` struct (Phase 2)
- [ ] 4.1.1.4 — Remove `TeamConfiguration` struct (Phase 2)
- [ ] 4.1.1.5 — Remove `verify_team_membership()` (returns true unconditionally)
- [ ] 4.1.1.6 — Remove `validate_earth_alliance_event()` (Phase 2)
- [ ] 4.1.1.7 — Remove hardcoded encryption key `b"simple_key_for_phase1_testing_32"`

#### Step 4.1.2 — Rate Limiter
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.2.1 — Implement `RateLimiter` struct with sliding window counter
- [ ] 4.1.2.2 — Track events per pubkey (default 60/min)
- [ ] 4.1.2.3 — Track events per connection (default 100/min)
- [ ] 4.1.2.4 — Track subscriptions per connection (default max 20)
- [ ] 4.1.2.5 — Track filters per REQ (default max 10)
- [ ] 4.1.2.6 — Window reset after configured seconds (default 60)

#### Step 4.1.3 — Size Validator
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.3.1 — Implement `SizeValidator` with configurable limits
- [ ] 4.1.3.2 — Check max event bytes (default 64KB)
- [ ] 4.1.3.3 — Check max content length (default 32KB)
- [ ] 4.1.3.4 — Check max tags count (default 2000)
- [ ] 4.1.3.5 — Check max tag value length (default 1024)

#### Step 4.1.4 — Content Policy
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.4.1 — Implement `ContentPolicy` with configurable rules
- [ ] 4.1.4.2 — Blocked event kinds list (configurable)
- [ ] 4.1.4.3 — Max event age check (default 24h)
- [ ] 4.1.4.4 — Max future drift check (default 15min)

#### Step 4.1.5 — Connection Limits
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.5.1 — Implement connection tracking per IP
- [ ] 4.1.5.2 — Max total connections (default 200)
- [ ] 4.1.5.3 — Max connections per IP (default 10)
- [ ] 4.1.5.4 — Idle timeout enforcement (default 300s)

#### Step 4.1.6 — Event Validator
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.6.1 — Verify event ID matches SHA-256 of serialized content
- [ ] 4.1.6.2 — Verify event signature (Schnorr BIP-340)

#### Step 4.1.7 — Alert Logging
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.7.1 — Implement `AlertLog`: write rejections to `security_alerts` table
- [ ] 4.1.7.2 — `SecurityVerdict::Block` → insert alert with type, severity, details
- [ ] 4.1.7.3 — Implement `broadcast::Sender<SecurityAlert>` for real-time alert channel
- [ ] 4.1.7.4 — Implement 7-day alert retention with hourly cleanup

#### Step 4.1.8 — TOML Config Loading
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.8.1 — Define TOML schema: `[rate_limits]`, `[size_limits]`, `[content_policy]`, `[connections]`, `[alerts]`
- [ ] 4.1.8.2 — Load `data/security_config.toml` on startup
- [ ] 4.1.8.3 — Implement hot-reload via `notify` filesystem watcher (optional MVP)

#### Step 4.1.9 — Security Monitor Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §3_

- [ ] 4.1.9.1 — Write `test_rate_limiter_allows_within_window` — under-limit events pass
- [ ] 4.1.9.2 — Write `test_rate_limiter_blocks_over_window` — over-limit events blocked
- [ ] 4.1.9.3 — Write `test_rate_limiter_window_resets` — counter resets after window
- [ ] 4.1.9.4 — Write `test_content_size_rule_allows_normal` — normal-size events pass
- [ ] 4.1.9.5 — Write `test_content_size_rule_rejects_oversized` — oversized events rejected
- [ ] 4.1.9.6 — Write `test_blocklist_rejects_blocked_pubkey` — blocked pubkey returns Block
- [ ] 4.1.9.7 — Write `test_allowlist_mode_rejects_unknown` — unknown pubkey in allowlist mode rejected
- [ ] 4.1.9.8 — Write `test_event_validation_rejects_bad_id` — bad event ID rejected
- [ ] 4.1.9.9 — Write `test_security_alert_logged` — rejection creates security_alerts row
- [ ] 4.1.9.10 — Commit: `feat: rules-based security monitor`

---

### Phase 4.2: Relay Pipeline Integration

#### Step 4.2.1 — Insert Monitor into Event Path
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 3.2_

- [ ] 4.2.1.1 — After parsing EVENT message, call `security_monitor.check_event()`
- [ ] 4.2.1.2 — If `SecurityVerdict::Block` → send `["OK", id, false, "blocked: {reason}"]`, skip storage
- [ ] 4.2.1.3 — If `SecurityVerdict::Allow` → proceed with storage and broadcast
- [ ] 4.2.1.4 — Rate limiting checked per-connection using `connection_id`
- [ ] 4.2.1.5 — Connection limit checked on WebSocket accept

#### Step 4.2.2 — Pipeline Integration Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §3_

- [ ] 4.2.2.1 — Write `test_monitor_in_relay_pipeline` — 3 events at rate_limit=2 → first 2 OK(true), third OK(false)
- [ ] 4.2.2.2 — Write `test_relay_rejects_spam` — client exceeding rate limit gets OK(false)
- [ ] 4.2.2.3 — Write `test_alerts_appear_in_api` — blocked event → alert visible in API
- [ ] 4.2.2.4 — Write `test_config_applied_on_startup` — custom TOML limits respected immediately
- [ ] 4.2.2.5 — Commit: `feat: security monitor in relay event pipeline`

---

### Phase 4.3: Security Alert API

#### Step 4.3.1 — Alert HTTP Endpoints
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 3.3_

- [ ] 4.3.1.1 — Implement `GET /api/v1/security/alerts` → recent alerts (paginated, JSON)
- [ ] 4.3.1.2 — Implement `GET /api/v1/security/stats` → alert counts by type
- [ ] 4.3.1.3 — Implement `SecurityMonitor::recent_alerts(limit)` query method
- [ ] 4.3.1.4 — Implement `SecurityMonitor::stats()` returning total checked/blocked/allowed
- [ ] 4.3.1.5 — Commit: `feat: security alert API endpoints`

> **STAGE 4 MILESTONE:** Relay rejects spam/malicious events before storage. All violations logged. Alert feed available via API.

---

## Stage 5: Tauri Desktop GUI

### Phase 5.1: Command Bridge

#### Step 5.1.1 — AppState & Service Management
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.1_

- [ ] 5.1.1.1 — Define `AppState` struct with `Arc<NostrRelay>`, `Arc<ContentStore>`, `Arc<SecurityMonitor>`
- [ ] 5.1.1.2 — Add `started_at: Arc<RwLock<Option<Instant>>>` for uptime tracking
- [ ] 5.1.1.3 — Add `service_handles: Arc<RwLock<Vec<JoinHandle<()>>>>` for lifecycle
- [ ] 5.1.1.4 — Register `AppState` via `tauri::Builder::manage()`

#### Step 5.1.2 — Tauri Commands
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md)_

- [ ] 5.1.2.1 — Implement `#[tauri::command] start_services()` — start relay + content store + monitor
- [ ] 5.1.2.2 — Implement `#[tauri::command] stop_services()` — graceful shutdown of all services
- [ ] 5.1.2.3 — Implement `#[tauri::command] get_service_status()` → `ServiceStatus` JSON
- [ ] 5.1.2.4 — Implement `#[tauri::command] get_security_alerts(limit)` → alert array JSON
- [ ] 5.1.2.5 — Implement `#[tauri::command] clear_security_alerts()` — truncate alerts
- [ ] 5.1.2.6 — Implement `#[tauri::command] get_network_info()` → relay URL, API URL, local IP

#### Step 5.1.3 — Register Commands in Builder
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md)_

- [ ] 5.1.3.1 — Add `tauri::generate_handler![...]` with all 6 commands
- [ ] 5.1.3.2 — Replace pure-tokio startup in `main()` with Tauri builder
- [ ] 5.1.3.3 — Implement graceful shutdown on window close (shutdown signal to all tasks)

#### Step 5.1.4 — Command Bridge Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §4_

- [ ] 5.1.4.1 — Write `test_tauri_get_status_returns_json` — correct JSON structure
- [ ] 5.1.4.2 — Write `test_tauri_get_security_alerts_returns_array` — limit respected
- [ ] 5.1.4.3 — Write `test_tauri_get_relay_config_returns_current` — config values accurate
- [ ] 5.1.4.4 — Write `test_tauri_update_relay_config_validates` — invalid input rejected
- [ ] 5.1.4.5 — Commit: `feat: register Tauri command bridge`

---

### Phase 5.2: Frontend Implementation

#### Step 5.2.1 — Dashboard Layout
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.2_

- [ ] 5.2.1.1 — Replace placeholder HTML with dashboard grid: Service Controls + 3 status cards
- [ ] 5.2.1.2 — Nostr Relay card: status indicator, address, connections, events stored, events/min
- [ ] 5.2.1.3 — Content Store card: status indicator, item count, storage used / max
- [ ] 5.2.1.4 — Security Monitor card: events checked, events blocked, active rules count
- [ ] 5.2.1.5 — Security Alert Feed section: scrollable list of recent violations
- [ ] 5.2.1.6 — Connection Info section: relay URL + API URL with copy buttons
- [ ] 5.2.1.7 — Remove team configuration form (Phase 2)

#### Step 5.2.2 — Service Control Wiring
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md)_

- [ ] 5.2.2.1 — Wire Start button → `invoke('start_services')`
- [ ] 5.2.2.2 — Wire Stop button → `invoke('stop_services')`
- [ ] 5.2.2.3 — Update button state (disabled while starting/stopping)
- [ ] 5.2.2.4 — Show running/stopped indicator

#### Step 5.2.3 — Status Polling
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md)_

- [ ] 5.2.3.1 — Implement `startStatusPolling()` with 3-second `setInterval`
- [ ] 5.2.3.2 — Call `invoke('get_service_status')` → `updateDashboard(status)`
- [ ] 5.2.3.3 — Call `invoke('get_security_alerts', { limit: 50 })` → `updateAlertFeed(alerts)`
- [ ] 5.2.3.4 — Implement `stopStatusPolling()` on service stop
- [ ] 5.2.3.5 — Implement `resetUI()` clearing all values on stop

#### Step 5.2.4 — Alert Feed Rendering
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md)_

- [ ] 5.2.4.1 — Render alerts as list items with timestamp, type, source pubkey, details
- [ ] 5.2.4.2 — Truncate pubkeys for display (first 8 chars + "...")
- [ ] 5.2.4.3 — Format timestamps as relative time ("2m ago")
- [ ] 5.2.4.4 — Auto-scroll to newest alert
- [ ] 5.2.4.5 — Commit: `feat: wire frontend to Tauri command bridge`

---

### Phase 5.3: UI Polish

#### Step 5.3.1 — Visual Enhancements
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.3_

- [ ] 5.3.1.1 — Connection count badge on Nostr Relay card
- [ ] 5.3.1.2 — Alert severity color coding: info=blue, warning=yellow, critical=red
- [ ] 5.3.1.3 — Content store usage progress bar (used / max)
- [ ] 5.3.1.4 — Responsive layout for 800x600 minimum window
- [ ] 5.3.1.5 — Error state messaging (show message if backend unreachable)
- [ ] 5.3.1.6 — Uptime display (formatted: Xd Xh Xm)
- [ ] 5.3.1.7 — Dark theme refinement (CSS custom properties)

#### Step 5.3.2 — Manual GUI Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §4_

- [ ] 5.3.2.1 — Test: launch app → dashboard values update within 5 seconds
- [ ] 5.3.2.2 — Test: trigger rate-limit violation → alert appears < 3 seconds
- [ ] 5.3.2.3 — Test: click Stop → services go offline → click Start → services resume
- [ ] 5.3.2.4 — Test: connect Nostr client → connection count increments in GUI
- [ ] 5.3.2.5 — Test: close window → process exits cleanly (no orphan processes)
- [ ] 5.3.2.6 — Commit: `feat: dashboard UI polish`

> **STAGE 5 MILESTONE:** Desktop app launches, shows live data, displays security alerts, allows service control. **This is the shippable MVP.**

---

## Stage 6: Release & Verification

### Phase 6.1: Cross-Platform Packaging

#### Step 6.1.1 — Build Configuration
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [ ] 6.1.1.1 — Update `tauri.conf.json`: productName, version 0.1.0, bundle targets "all"
- [ ] 6.1.1.2 — Add icon files: 32x32.png, 128x128.png, icon.icns, icon.ico
- [ ] 6.1.1.3 — Verify `cargo tauri build` completes successfully

#### Step 6.1.2 — Platform Builds
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [ ] 6.1.2.1 — Linux: verify `.deb` package installs and runs
- [ ] 6.1.2.2 — Linux: verify `.AppImage` launches correctly
- [ ] 6.1.2.3 — macOS: verify `.dmg` package (if available)
- [ ] 6.1.2.4 — Windows: verify `.msi` installer (if available)

---

### Phase 6.2: End-to-End Acceptance

#### Step 6.2.1 — MVP Success Criteria
_Ref: [00-overview.md](mvp/00-overview.md)_

- [ ] 6.2.1.1 — User downloads single installer, runs, clicks "Start" → relay online
- [ ] 6.2.1.2 — Nostr client (Damus, Amethyst, Snort) connects and exchanges messages
- [ ] 6.2.1.3 — Content stored and retrieved by hash via API
- [ ] 6.2.1.4 — Dashboard shows live connection count, event throughput, storage usage
- [ ] 6.2.1.5 — Spam bot gets rate-limited; violation appears in security feed
- [ ] 6.2.1.6 — App runs 24h on Raspberry Pi 4 without crash, < 200MB RAM

#### Step 6.2.2 — Resource Budget Verification
_Ref: [00-overview.md](mvp/00-overview.md)_

- [ ] 6.2.2.1 — Nostr Relay < 50MB RAM
- [ ] 6.2.2.2 — Content Store < 30MB RAM (+ disk)
- [ ] 6.2.2.3 — Security Monitor < 10MB RAM
- [ ] 6.2.2.4 — SQLite < 20MB RAM
- [ ] 6.2.2.5 — Tauri WebView < 80MB RAM
- [ ] 6.2.2.6 — Total process < 190MB RAM
- [ ] 6.2.2.7 — Binary size < 30MB (release, stripped)
- [ ] 6.2.2.8 — Startup time < 3 seconds

---

### Phase 6.3: Post-MVP Cleanup (Optional)

#### Step 6.3.1 — Remove Deferred Modules
_Ref: [08-implementation-order.md](mvp/08-implementation-order.md) Step 5.1_

- [ ] 6.3.1.1 — Evaluate: remove or keep `src/investigation_service.rs` (Phase 2)
- [ ] 6.3.1.2 — Evaluate: remove or keep `src/investigation_api.rs` (Phase 2)
- [ ] 6.3.1.3 — Evaluate: remove or keep `src/subnet_manager.rs` (Phase 3)
- [ ] 6.3.1.4 — Evaluate: remove or keep `src/subnet_types.rs` (Phase 3)
- [ ] 6.3.1.5 — Evaluate: remove or keep `src/auth.rs` (not needed for local-only MVP)

---

### Phase 6.4: Release

#### Step 6.4.1 — Tag & Ship
- [ ] 6.4.1.1 — All tests pass: `cargo test --all`
- [ ] 6.4.1.2 — No clippy warnings: `cargo clippy -- -D warnings`
- [ ] 6.4.1.3 — Git tag: `v0.1.0`
- [ ] 6.4.1.4 — GitHub release with platform binaries attached
- [ ] 6.4.1.5 — Update README.md with install instructions

> **STAGE 6 MILESTONE:** v0.1.0 released. AI Security RelayNode is a downloadable, installable desktop app that runs a Nostr relay, content store, and security monitor.

---

## Summary

| Stage | Phases | Steps | Tasks | Focus |
|-------|--------|-------|-------|-------|
| 1. Infrastructure | 3 | 8 | 35 | Deps, config, database |
| 2. Working Relay | 4 | 12 | 46 | Event store, protocol, WebSocket, tests |
| 3. Content Store | 2 | 6 | 23 | CAS module, API routes |
| 4. Security Monitor | 3 | 11 | 38 | Rules engine, pipeline, alerts |
| 5. Desktop GUI | 3 | 8 | 33 | Commands, frontend, polish |
| 6. Release | 4 | 5 | 18 | Packaging, acceptance, tag |
| **Total** | **19** | **50** | **193** | |

---

_Generated from docs/mvp/ spec documents 00–08 + MVP-PLAN.md_
