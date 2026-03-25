# MVP Master Checklist

> **AI Security RelayNode v0.1.0**
> Comprehensive workload tracker — every task from every spec document.
> Structure: **Stage → Phase → Step → Task → Subtasks**

---

## Stage 1: Infrastructure & Foundation

### Phase 1.1: Dependency Cleanup

#### Step 1.1.1 — Trim Cargo.toml
_Ref: [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.1_

- [x] 1.1.1.1 — Remove `libp2p` dependency (never imported, 200+ transitive deps)
  - [x] Verify no source file imports libp2p types
  - [x] Run `cargo check` to confirm no compile errors after removal
- [x] 1.1.1.2 — Remove `num-bigint` dependency
  - [x] Grep codebase for `num_bigint` / `num-bigint` usage
- [x] 1.1.1.3 — Remove `num-traits` dependency
  - [x] Grep codebase for `num_traits` / `num-traits` usage
- [x] 1.1.1.4 — Remove `crossbeam-channel` dependency (not used in live code)
  - [x] Confirm no live module imports crossbeam
- [x] 1.1.1.5 — Add `toml = "0.8"` (config file parsing)
  - [x] Verify version compatibility with MSRV / other deps
- [x] 1.1.1.6 — Add `dirs = "5"` (platform-specific data directories)
  - [x] Verify cross-platform support (Linux, macOS, Windows)
- [x] 1.1.1.7 — Add `notify = "6"` (config file hot-reload, security monitor)
  - [x] Verify cross-platform filesystem watcher support
- [x] 1.1.1.8 — Verify `cargo check` passes
  - [x] Fix any compile errors from dependency removal
  - [x] Ensure error messages are clear if dep removal breaks something
- [x] 1.1.1.9 — Commit: `chore: trim unused deps, add toml + dirs`

#### Step 1.1.2 — Add Release Build Profile
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [x] 1.1.2.1 — Add `[profile.release]` to Cargo.toml: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`
  - [x] Test that debug builds still work normally
  - [x] Verify release profile doesn't strip error context from `anyhow` backtraces
- [x] 1.1.2.2 — Verify `cargo build --release` produces binary < 30MB
  - [x] Document binary size in build output for regression tracking

---

### Phase 1.2: Configuration System

#### Step 1.2.1 — Implement Config Loading
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.2_

- [x] 1.2.1.1 — Implement `Config::defaults()` returning hardcoded relay_port=8080, api_port=8081, max_connections=200
  - [x] Ensure all defaults are valid and documented in code comments
  - [x] Return `Config` struct, not `Result` — defaults never fail
- [x] 1.2.1.2 — Implement `Config::config_path()` using `dirs::config_dir()` with `./data/` fallback
  - [x] Handle `dirs::config_dir()` returning `None` gracefully (fallback, not panic)
  - [x] Log which config path is being used at startup (tracing::info)
- [x] 1.2.1.3 — Define `ConfigFile` struct with `#[derive(Deserialize)]` matching TOML structure
  - [x] Use `Option<T>` for every field so partial config files work
  - [x] Add `#[serde(default)]` to tolerate missing sections
- [x] 1.2.1.4 — Implement `Config::load()`: read `data/config.toml` → parse → merge over defaults
  - [x] On TOML parse error: log warning with line/column and fall back to defaults
  - [x] On file read error (permissions, etc.): log warning and fall back to defaults
  - [x] Never panic on bad config — always degrade to defaults with logged warning
- [x] 1.2.1.5 — Add env var overrides: `RELAY_PORT`, `API_PORT`, `JWT_SECRET`, `DATABASE_URL`, `DATA_DIR`
  - [x] On invalid env var value (e.g. non-numeric port): log error with var name and value, use default
  - [x] Validate port range (1–65535) with clear error message
- [x] 1.2.1.6 — Auto-generate `JWT_SECRET` (64-byte random hex) if not set
  - [x] Use `getrandom` or OS CSPRNG — never use weak RNG for secrets
  - [x] Log info "JWT_SECRET not configured, generating random secret"
- [x] 1.2.1.7 — Remove dead fields: `SubnetMode`, `GatewayConfig`, `TeamSubnetConfig`
  - [x] Verify no live code references removed fields

#### Step 1.2.2 — First-Run Bootstrapping
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [x] 1.2.2.1 — Create `data/` directory on first run if missing
  - [x] Handle permission denied: log actionable error "Cannot create data directory at {path}: {err}"
  - [x] Handle disk full: report clearly, exit with non-zero code
- [x] 1.2.2.2 — Generate `data/config.toml` with defaults written out
  - [x] Skip if file already exists (never overwrite user config)
  - [x] On write failure: log warning, continue with in-memory defaults
- [x] 1.2.2.3 — Generate `data/security_config.toml` with default rules
  - [x] Skip if file already exists
  - [x] Include comments in generated TOML explaining each setting
- [x] 1.2.2.4 — Create `data/cas/` directory for content store
  - [x] Handle permission denied with actionable error message

#### Step 1.2.3 — Config Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §6_

- [x] 1.2.3.1 — Write `test_config_defaults` — `Config::defaults()` returns correct values
  - [x] Assert every field has expected default
- [x] 1.2.3.2 — Write `test_config_from_file` — TOML file overrides defaults
  - [x] Test partial TOML (only some fields set)
- [x] 1.2.3.3 — Write `test_config_env_override` — env var overrides all
  - [x] Test with valid and invalid env var values
- [x] 1.2.3.4 — Write `test_config_env_overrides_file` — env wins over file
  - [x] Verify precedence: env > file > defaults
- [x] 1.2.3.5 — Write `test_config_missing_file_uses_defaults` — no file → defaults apply
  - [x] Ensure no error/panic on missing file
- [x] 1.2.3.6 — Write `test_jwt_secret_auto_generated` — empty secret → auto-generate
  - [x] Verify generated secret has sufficient entropy (at least 64 hex chars)
- [x] 1.2.3.7 — Commit: `feat: config loading from file + env vars`

---

### Phase 1.3: Database & Migration System

#### Step 1.3.1 — Schema Version Tracking
_Ref: [05-data-model.md](mvp/05-data-model.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.5_

- [x] 1.3.1.1 — Create `schema_version` table (version INTEGER PK, applied_at INTEGER)
  - [x] Use `CREATE TABLE IF NOT EXISTS` for idempotency
- [x] 1.3.1.2 — Implement migration runner: check version → run pending → update version
  - [x] On migration SQL error: log full error with migration file name, abort startup
  - [x] Report which migration failed and at what step
  - [x] Wrap each migration in a transaction for atomicity
- [x] 1.3.1.3 — Run migrations on startup in `DatabaseManager::new()`
  - [x] On DB connection failure: log "Cannot connect to database at {url}: {err}"
  - [x] On locked database: retry with backoff, then report error to user
  - [x] Log "Database at version N, applying M pending migrations"

#### Step 1.3.2 — MVP Schema Migration
_Ref: [05-data-model.md](mvp/05-data-model.md)_

- [x] 1.3.2.1 — Write `src/migrations/001_mvp_schema.sql`
  - [x] Use `IF NOT EXISTS` on all CREATE statements for safe re-runs
- [x] 1.3.2.2 — Create `events` table (id, pubkey, created_at, kind, tags, content, sig, stored_at)
  - [x] Validate column types match NIP-01 spec (TEXT for hex, INTEGER for timestamps)
- [x] 1.3.2.3 — Create events indexes: pubkey, created_at DESC, kind, (pubkey, kind)
  - [x] Use `IF NOT EXISTS` on indexes
- [x] 1.3.2.4 — Create `content_index` table (cid, size_bytes, content_type, stored_at, accessed_at)
  - [x] Validate CID column is TEXT PRIMARY KEY
- [x] 1.3.2.5 — Create content_index indexes: stored_at DESC, size_bytes
  - [x] Use `IF NOT EXISTS` on indexes
- [x] 1.3.2.6 — Create `security_alerts` table (id, timestamp, alert_type, severity, source_ip, source_pubkey, connection_id, details, event_id)
  - [x] Validate all columns have appropriate NOT NULL constraints
- [x] 1.3.2.7 — Create security_alerts indexes: timestamp DESC, alert_type, severity
  - [x] Use `IF NOT EXISTS` on indexes
- [x] 1.3.2.8 — Create `relay_stats` table (timestamp, connections, events_stored, events_minute, content_items, content_bytes, alerts_total, memory_bytes)
  - [x] Validate timestamp is INTEGER PRIMARY KEY for efficient time-series queries
- [x] 1.3.2.9 — Remove old investigation tables from startup migration path
  - [x] Ensure existing DBs with investigation tables still work (don't DROP)

#### Step 1.3.3 — Database Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §5_

- [x] 1.3.3.1 — Write `test_database_migrations_run` — all tables exist after migration on empty DB
  - [x] Test idempotency: run migrations twice, no errors
  - [x] Verify error reporting if migration SQL is malformed
- [x] 1.3.3.2 — Write `test_stats_snapshot_insert` — relay_stats row insert/retrieve
  - [x] Test with boundary values (0, MAX_INT)
- [x] 1.3.3.3 — Commit: `feat: versioned database migrations`

---

## Stage 2: Working Nostr Relay

### Phase 2.1: Event Store Fixes

#### Step 2.1.1 — Fix Query Parameter Binding
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.3_

- [x] 2.1.1.1 — Fix `build_filter_where_clause()` to return `(String, Vec<sqlx::Value>)` tuple
  - [x] Ensure SQL injection is impossible — all user input goes through bind params
  - [x] Add debug-level logging of generated WHERE clause for troubleshooting
- [x] 2.1.1.2 — Use returned params in `sqlx::query()` bindings (fix discarded params bug)
  - [x] Write regression test proving params are actually bound
  - [x] Log error if param count mismatches placeholder count
- [x] 2.1.1.3 — Add prefix matching for IDs using `LIKE 'prefix%'`
  - [x] Escape LIKE special chars (%, _) in user-supplied prefixes
- [x] 2.1.1.4 — Add prefix matching for authors using `LIKE 'prefix%'`
  - [x] Validate author prefix is valid hex before querying
- [x] 2.1.1.5 — Add tag-based querying using SQLite `json_each()` + `json_extract()`
  - [x] Handle malformed JSON in tags column gracefully (skip row, don't crash)
  - [x] Log warning on corrupted tags data
- [x] 2.1.1.6 — Remove Earth Alliance clearance columns (Phase 2)
  - [x] Verify removal doesn't break existing queries
- [x] 2.1.1.7 — Simplify to MVP schema (events table only, no investigation)
  - [x] Keep investigation code compiling but unreferenced

#### Step 2.1.2 — Event Store Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §5_

- [x] 2.1.2.1 — Write `test_event_insert_and_retrieve` — store → SELECT by ID → match
  - [x] Test with events containing Unicode content
  - [x] Test with events containing empty content
- [x] 2.1.2.2 — Write `test_event_duplicate_ignored` — duplicate ID → no error, still 1 row
  - [x] Verify INSERT OR IGNORE semantics (no error, no update)
- [x] 2.1.2.3 — Write `test_event_filter_by_kind` — kinds=[1] → returns only kind-1 events
  - [x] Test with multiple kinds filter
- [x] 2.1.2.4 — Write `test_event_filter_by_author_prefix` — authors=["aa"] → prefix match
  - [x] Test with full-length author (64 hex chars) as prefix
- [x] 2.1.2.5 — Write `test_event_filter_since_until` — time range filtering
  - [x] Test boundary: event exactly at `since` timestamp (inclusive)
  - [x] Test boundary: event exactly at `until` timestamp (inclusive)
- [x] 2.1.2.6 — Write `test_event_filter_limit` — LIMIT clause honored
  - [x] Test with limit=0 (should return empty)
  - [x] Test with limit larger than event count
- [x] 2.1.2.7 — Commit: `fix: event store query parameter binding`

---

### Phase 2.2: Nostr Protocol Fixes

#### Step 2.2.1 — Protocol Handler Updates
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [x] 2.2.1.1 — Fix signature verification: switch from ECDSA to Schnorr (BIP-340) per NIP-01
  - [x] Enable `global-context` feature flag on secp256k1 crate
  - [x] Handle invalid pubkey format gracefully (return error, not panic)
- [x] 2.2.1.2 — Add prefix matching for filter IDs/authors (currently exact match only)
  - [x] Validate prefix is valid hex before matching
- [ ] 2.2.1.3 — Add tag filter matching (`#e`, `#p` tag queries)
  - [ ] Handle events with no tags gracefully
  - [ ] Handle tags with unexpected structure (missing value element)
- [ ] 2.2.1.4 — Remove Earth Alliance metadata extraction (Phase 2)
  - [ ] Verify removal doesn't break event parsing

#### Step 2.2.2 — Protocol Unit Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §1_

- [ ] 2.2.2.1 — Write `test_nostr_event_id_computation` — SHA-256 matches NIP-01 known vectors
  - [ ] Use official NIP-01 test vector if available
- [ ] 2.2.2.2 — Write `test_nostr_signature_validation_valid` — valid Schnorr sig accepted
  - [ ] Generate real keypair for test (not hardcoded invalid data)
- [ ] 2.2.2.3 — Write `test_nostr_signature_validation_invalid` — tampered sig rejected
  - [ ] Test: valid sig with wrong pubkey
  - [ ] Test: valid sig with modified content
- [ ] 2.2.2.4 — Write `test_nostr_message_parse_event` — JSON → ClientMessage::Event
  - [ ] Test error path: missing required fields
- [ ] 2.2.2.5 — Write `test_nostr_message_parse_req` — JSON → ClientMessage::Req with filters
  - [ ] Test empty filter (matches all)
  - [ ] Test multiple filters in single REQ
- [ ] 2.2.2.6 — Write `test_nostr_message_parse_close` — JSON → ClientMessage::Close
  - [ ] Test error path: missing subscription ID
- [ ] 2.2.2.7 — Write `test_nostr_message_parse_invalid` — malformed JSON → error (not panic)
  - [ ] Test empty string, null, number, truncated JSON
  - [ ] Verify error message is descriptive for debugging

---

### Phase 2.3: WebSocket Message Dispatch

#### Step 2.3.1 — Wire Read Loop
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.4_

- [x] 2.3.1.1 — Replace `handle_websocket_messages()` sleep loop with `while let Some(msg) = ws_reader.next().await`
  - [x] Ensure loop exits cleanly on client disconnect (no infinite retries)
- [x] 2.3.1.2 — Split WebSocket stream using `ws_stream.split()` → `(ws_writer, ws_reader)`
  - [x] Verify writer task doesn't block reader task
- [x] 2.3.1.3 — On `Message::Text` → parse via `NostrProtocolHandler` → dispatch → send response
  - [x] On parse error: send `["NOTICE", "error: {description}"]` to client
  - [x] Never expose internal stack traces in NOTICE messages
  - [x] Log full parse error at debug level for server-side troubleshooting
- [x] 2.3.1.4 — On `Message::Ping` → reply with Pong
  - [x] Handle Pong send failure gracefully (client may have disconnected)
- [x] 2.3.1.5 — On `Message::Close` / `None` → cleanup connection, break loop
  - [x] Log connection close reason at info level
  - [x] Ensure all resources (subscriptions, channels) are cleaned up
- [x] 2.3.1.6 — On error → log, break
  - [x] Distinguish between expected errors (client disconnect) and unexpected errors
  - [x] Log unexpected errors at warn level with connection_id context

#### Step 2.3.2 — Event Processing Pipeline
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [x] 2.3.2.1 — EVENT handler: validate event ID (SHA-256 check)
  - [x] On invalid ID: respond `["OK", event_id, false, "invalid: event id does not match"]`
  - [x] Log at debug: "Event ID mismatch: expected={computed} got={claimed}"
- [x] 2.3.2.2 — EVENT handler: validate signature (Schnorr BIP-340)
  - [x] On invalid sig: respond `["OK", event_id, false, "invalid: bad signature"]`
  - [x] On malformed pubkey/sig hex: respond with descriptive error
- [x] 2.3.2.3 — EVENT handler: store in EventStore via `INSERT OR IGNORE`
  - [x] On DB write error: respond `["OK", event_id, false, "error: could not store event"]`
  - [x] Log DB errors at error level with full context
  - [x] Never expose DB error details to client (security)
- [x] 2.3.2.4 — EVENT handler: broadcast to matching subscriptions
  - [x] Handle broadcast channel full: log warning, skip (don't block event processing)
  - [x] Handle individual subscriber send failure: remove dead subscription
- [x] 2.3.2.5 — EVENT handler: send `["OK", event_id, true/false, message]` response
  - [x] Handle response send failure (client disconnected between receive and reply)

#### Step 2.3.3 — Subscription Processing
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [x] 2.3.3.1 — REQ handler: register subscription with SubscriptionManager
  - [x] Reject if connection has too many subs (send CLOSED with reason)
  - [x] Log subscription creation at debug level
- [x] 2.3.3.2 — REQ handler: query EventStore for historical matching events
  - [x] On DB query error: send NOTICE with generic message, log full error
  - [x] Limit historical results to prevent memory exhaustion (max 5000 per REQ)
- [x] 2.3.3.3 — REQ handler: send matching events as `["EVENT", sub_id, event]`
  - [x] Handle send failure mid-stream: cleanup subscription, break
- [x] 2.3.3.4 — REQ handler: send `["EOSE", sub_id]` after historical events
  - [x] Always send EOSE even if 0 historical events matched
- [x] 2.3.3.5 — REQ handler: replace existing subscription if same sub_id reused
  - [x] Clean up old subscription filters before registering new ones
- [x] 2.3.3.6 — CLOSE handler: remove subscription, send `["CLOSED", sub_id, ""]`
  - [x] Handle CLOSE for non-existent subscription: send CLOSED anyway (idempotent)
- [x] 2.3.3.7 — COUNT handler: query count, send `["COUNT", sub_id, {"count": N}]`
  - [x] On DB error: send COUNT with count=0 and log error

#### Step 2.3.4 — Subscription Manager Fixes
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [x] 2.3.4.1 — Remove clearance/team access control (not MVP)
  - [x] Verify all subscription paths work without clearance checks
- [x] 2.3.4.2 — Fix duplicate subscription handling: REQ with existing sub_id replaces old
  - [x] Drop old subscription's channel to free resources
- [x] 2.3.4.3 — Verify EOSE sent after historical events before streaming
  - [x] Add integration test asserting EOSE ordering

#### Step 2.3.5 — Connection Lifecycle
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [x] 2.3.5.1 — Assign UUID `connection_id` on WebSocket accept
  - [x] Include connection_id in all log messages for this connection (tracing span)
- [x] 2.3.5.2 — Register connection in SubscriptionManager
  - [x] Handle registration failure: close WebSocket with error frame
- [x] 2.3.5.3 — On disconnect: remove all subscriptions for connection
  - [x] Ensure no resource leaks (channels, hashmap entries)
  - [x] Log "Connection {id} disconnected after {duration}s, {n} subs cleaned up"
- [x] 2.3.5.4 — Implement 300s idle timeout
  - [x] Reset timeout on any client message (not just events)
  - [x] Send `["NOTICE", "idle timeout"]` before closing
- [x] 2.3.5.5 — Commit: `feat: wire WebSocket read loop to message processing`

---

### Phase 2.4: Relay Integration Tests

#### Step 2.4.1 — Automated Integration Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §1, [08-implementation-order.md](mvp/08-implementation-order.md) Step 1.6_

- [ ] 2.4.1.1 — Write `test_relay_accepts_websocket_connection` — handshake succeeds
  - [ ] Verify clean disconnect after test (no leaked tasks)
- [ ] 2.4.1.2 — Write `test_relay_stores_valid_event` — EVENT → OK(true) + in DB
  - [ ] Verify event retrievable via subsequent REQ
- [ ] 2.4.1.3 — Write `test_relay_rejects_invalid_signature` — bad sig → OK(false)
  - [ ] Verify rejection message contains "invalid" or "bad signature"
  - [ ] Verify event is NOT stored in DB after rejection
- [ ] 2.4.1.4 — Write `test_relay_subscription_receives_stored_events` — REQ → events + EOSE
  - [ ] Verify EOSE comes after all matching events
- [ ] 2.4.1.5 — Write `test_relay_subscription_receives_new_events` — subscribe → publish → receive
  - [ ] Verify events arrive within 100ms of publish
- [ ] 2.4.1.6 — Write `test_relay_close_subscription` — CLOSE removes subscription
  - [ ] Verify no more events after CLOSE
- [ ] 2.4.1.7 — Write `test_relay_count_events` — COUNT returns correct count
  - [ ] Test COUNT with empty filter (all events)

#### Step 2.4.2 — Manual Relay Verification
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §1_

- [ ] 2.4.2.1 — Test with `nak` CLI: publish and retrieve events via `ws://localhost:8080`
  - [ ] Document exact nak commands used for reproducibility
- [ ] 2.4.2.2 — Test 100 concurrent WebSocket connections (all events stored, no crash, <200MB RAM)
  - [ ] Monitor for connection errors or dropped events
  - [ ] Check for error log entries during load test
- [ ] 2.4.2.3 — Commit: `test: relay round-trip integration test`

#### Step 2.4.3 — Performance Verification
_Ref: [01-nostr-relay.md](mvp/01-nostr-relay.md)_

- [ ] 2.4.3.1 — Verify 100+ concurrent connections supported
  - [ ] Check for file descriptor exhaustion errors
- [ ] 2.4.3.2 — Verify 500+ events/second sustained throughput
  - [ ] Monitor error rate under load (should be 0% for valid events)
- [ ] 2.4.3.3 — Verify event validation latency < 5ms
  - [ ] Benchmark p50, p95, p99 latency
- [ ] 2.4.3.4 — Verify historical query (1000 events) < 50ms
  - [ ] Test with complex filters (multiple kinds + tags)
- [ ] 2.4.3.5 — Verify memory per connection < 500KB
  - [ ] Check for memory leaks over 1000 connect/disconnect cycles

> **STAGE 2 MILESTONE:** `cargo test` passes. WebSocket client can connect, publish events, subscribe, and receive events. NIP-01 compliant, SQLite-backed relay.

---

## Stage 3: Content-Addressed Storage

### Phase 3.1: Storage Module

#### Step 3.1.1 — Replace Fake IPFS
_Ref: [02-content-store.md](mvp/02-content-store.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 2.1_

- [ ] 3.1.1.1 — Rename `src/ipfs_node.rs` → `src/content_store.rs`
  - [ ] Use `git mv` to preserve file history
- [ ] 3.1.1.2 — Rename `IPFSNode` struct → `ContentStore`
  - [ ] Update struct documentation comments
- [ ] 3.1.1.3 — Update `src/lib.rs` module declaration (`pub mod content_store`)
  - [ ] Update re-exports in lib.rs
- [ ] 3.1.1.4 — Update all imports referencing `IPFSNode` / `ipfs_node`
  - [ ] Search all .rs files for old names
  - [ ] Verify `cargo check` after all renames
- [x] 3.1.1.5 — Remove in-memory `HashMap<String, Vec<u8>>` storage
  - [x] Verify no other code path depends on in-memory store
- [x] 3.1.1.6 — Remove XOR "encryption" logic
  - [x] Remove hardcoded XOR key
  - [x] Verify no data corruption path remains
- [x] 3.1.1.7 — Remove fake hash generation (`Qm{uuid}`)
  - [x] Replace with real SHA-256 hashing

#### Step 3.1.2 — Content-Addressed Storage Engine
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [x] 3.1.2.1 — Implement SHA-256 → CID computation (`store(bytes) → cid`)
  - [x] Verify CID is deterministic (same bytes → same CID always)
- [x] 3.1.2.2 — Implement 2-level directory layout: `data/cas/ab/cd/abcd...`
  - [x] Handle directory creation errors (permissions, disk full)
  - [x] Log errors with full path for troubleshooting
- [x] 3.1.2.3 — Implement `store(&[u8]) → Result<String>`: hash → write file → insert metadata
  - [x] On disk write failure: return error with "failed to store content: {err}"
  - [x] On DB insert failure after file written: clean up orphaned file
  - [x] Ensure atomicity: use temp file + rename to prevent partial writes
- [x] 3.1.2.4 — Implement `retrieve(cid) → Result<Vec<u8>>`: lookup → read file → return bytes
  - [x] On missing file (DB says exists but file gone): return clear error, log data inconsistency warning
  - [x] On read error (permissions): return error with context
  - [x] Validate CID format before filesystem access (prevent path traversal)
- [x] 3.1.2.5 — Implement `exists(cid) → Result<bool>`
  - [x] Check both DB metadata and file on disk
  - [x] Validate CID format to prevent path traversal attacks
- [x] 3.1.2.6 — Implement `delete(cid) → Result<bool>`: remove file + metadata
  - [x] Delete file first, then DB row (if file delete fails, content is still tracked)
  - [x] Return false (not error) for already-deleted content
- [ ] 3.1.2.7 — Implement `stats() → Result<ContentStoreStats>`: item count + total bytes
  - [ ] On DB query error: return error, don't return stale/wrong numbers
- [ ] 3.1.2.8 — Implement `list(offset, limit) → Result<Vec<ContentMetadata>>`: paginated listing
  - [ ] Clamp limit to reasonable max (e.g., 1000) to prevent OOM
  - [ ] Validate offset is non-negative

#### Step 3.1.3 — Size Limit Enforcement
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [x] 3.1.3.1 — Enforce `max_content_size` per item (default 10MB) → 413 error
  - [x] Check size BEFORE writing to disk (not after)
  - [x] Return clear error: "Content size {n} exceeds maximum {max}"
- [ ] 3.1.3.2 — Enforce `max_storage_bytes` total (default 5GB) → 507 error
  - [ ] Check total BEFORE storing (not after)
  - [ ] Return clear error: "Storage quota exceeded ({used}/{max})"
- [x] 3.1.3.3 — Deduplication: same content → same CID, no new file written
  - [x] Return existing CID without error (200, not 409)

#### Step 3.1.4 — Content Store Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §2_

- [x] 3.1.4.1 — Write `test_content_store_computes_correct_cid` — bytes → CID matches SHA-256
  - [x] Test with known SHA-256 test vectors
- [x] 3.1.4.2 — Write `test_content_store_deduplicates` — same content twice = one file
  - [x] Verify only one file on disk after double store
- [x] 3.1.4.3 — Write `test_content_store_retrieve` — retrieve by CID returns original bytes
  - [x] Test with binary content (non-UTF8)
- [x] 3.1.4.4 — Write `test_content_store_retrieve_missing` — missing CID returns error
  - [x] Verify error message is user-friendly (not internal panic)
- [x] 3.1.4.5 — Write `test_content_store_rejects_oversized` — >max_size returns error
  - [x] Verify no partial file left on disk after rejection
- [ ] 3.1.4.6 — Write `test_content_store_list_items` — list returns paginated metadata
  - [ ] Test empty store returns empty list (not error)
- [ ] 3.1.4.7 — Write `test_content_store_stats` — stats reflects actual items/bytes
  - [ ] Test stats after store + delete cycle
- [ ] 3.1.4.8 — Write `test_content_store_persists_across_restart` — store → drop → recreate → retrieve
  - [ ] Ensures filesystem + DB are in sync after restart
- [ ] 3.1.4.9 — Commit: `feat: content-addressed storage replacing fake IPFS`

---

### Phase 3.2: Content Store API

#### Step 3.2.1 — HTTP Routes
_Ref: [02-content-store.md](mvp/02-content-store.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 2.2_

- [ ] 3.2.1.1 — Implement `POST /api/v1/content` → store raw bytes, return `{ "cid": "..." }`
  - [ ] On oversized body: return 413 with JSON `{"error": "Content too large", "max_bytes": N}`
  - [ ] On quota exceeded: return 507 with JSON `{"error": "Storage quota exceeded"}`
  - [ ] On internal error: return 500 with JSON `{"error": "Internal server error"}` (no details leaked)
- [ ] 3.2.1.2 — Implement `GET /api/v1/content/:cid` → return raw bytes with Content-Type
  - [ ] On missing CID: return 404 with JSON `{"error": "Content not found"}`
  - [ ] On invalid CID format: return 400 with JSON `{"error": "Invalid CID format"}`
  - [ ] Validate CID to prevent path traversal (reject if contains `..` or `/`)
- [ ] 3.2.1.3 — Implement `HEAD /api/v1/content/:cid` → 200 if exists, 404 if not
  - [ ] Return `Content-Length` header on 200
- [ ] 3.2.1.4 — Implement `GET /api/v1/content` → list items (paginated, JSON)
  - [ ] Validate `limit` and `offset` query params (reject negative, cap maximum)
  - [ ] Return empty array (not error) for offset past end
- [ ] 3.2.1.5 — Implement `GET /api/v1/content/stats` → storage statistics
  - [ ] On DB error: return 503 with `{"error": "Service temporarily unavailable"}`
- [ ] 3.2.1.6 — Remove hardcoded `/ipfs/status` route from api_gateway.rs
  - [ ] Verify no frontend code depends on old route
- [ ] 3.2.1.7 — Add content-type detection for stored content
  - [ ] Default to `application/octet-stream` if detection fails

#### Step 3.2.2 — Content API Tests
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [ ] 3.2.2.1 — Write `test_http_store_retrieve` — POST content → GET by CID → match
  - [ ] Test error response format (JSON with "error" field)
- [ ] 3.2.2.2 — Write `test_http_head_exists` — HEAD returns 200/404 correctly
  - [ ] Test with invalid CID format (should be 400)
- [ ] 3.2.2.3 — Commit: `feat: content store API routes`

#### Step 3.2.3 — Performance Verification
_Ref: [02-content-store.md](mvp/02-content-store.md)_

- [ ] 3.2.3.1 — Verify store 1MB file < 100ms
  - [ ] Monitor for I/O errors during performance test
- [ ] 3.2.3.2 — Verify retrieve 1MB file < 50ms
  - [ ] Test cold read (not in OS page cache)
- [ ] 3.2.3.3 — Verify CID computation (1MB) < 10ms
  - [ ] Benchmark SHA-256 throughput
- [ ] 3.2.3.4 — Verify 50+ concurrent reads supported
  - [ ] Check for file handle exhaustion errors under load

> **STAGE 3 MILESTONE:** Content can be stored and retrieved by hash via HTTP API. Durable, deduplicated, size-limited.

---

## Stage 4: Security Monitor

### Phase 4.1: Rules Engine

#### Step 4.1.1 — Security Architecture
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 3.1_

- [ ] 4.1.1.1 — Rename `src/security_layer.rs` → `src/security_monitor.rs`
  - [ ] Use `git mv` to preserve file history
- [ ] 4.1.1.2 — Rename struct → `SecurityMonitor`
  - [ ] Update all references in other modules
- [ ] 4.1.1.3 — Update `src/lib.rs` module declaration
  - [ ] Verify `cargo check` after rename
- [ ] 4.1.1.4 — Define `SecurityRule` trait: `fn evaluate(&self, event: &NostrEvent) → RuleResult`
  - [ ] `RuleResult` enum: `Allow`, `Deny(reason)`, `Flag(severity, reason)`
  - [ ] Document trait contract for implementors
- [ ] 4.1.1.5 — Define `SecurityAlert` struct: `timestamp`, `rule_name`, `severity`, `details`
  - [ ] Severity levels: Info, Warning, Critical
  - [ ] Include source pubkey and event ID in alert details
- [ ] 4.1.1.6 — Implement `SecurityMonitor` with `Vec<Box<dyn SecurityRule>>` pipeline
  - [ ] Evaluate rules in order, short-circuit on first `Deny`
  - [ ] On rule panic: catch_unwind, log error, treat as Deny (fail-closed)
  - [ ] Never let a faulty rule crash the server

#### Step 4.1.2 — Rate Limiter Rule
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [x] 4.1.2.1 — Implement per-pubkey sliding window rate limiter
  - [x] Default: 10 events per 60 seconds per pubkey
  - [x] On rate exceeded: return `Deny` with "Rate limit exceeded for pubkey {first8chars}"
- [x] 4.1.2.2 — Use `HashMap<String, VecDeque<Instant>>` for tracking
  - [x] Periodically evict stale entries to prevent memory growth
  - [x] Log at debug level when evicting stale entries
- [ ] 4.1.2.3 — Make window size and max count configurable from TOML
  - [ ] On invalid config values (≤0): log warning, use defaults
- [ ] 4.1.2.4 — Log rate-limit events at `warn` level with pubkey prefix
  - [ ] Include current count and window size in log message

#### Step 4.1.3 — Size Validator Rule
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.3.1 — Reject events with content > `max_event_content_bytes` (default 64KB)
  - [ ] Check size BEFORE processing content (not after parsing)
  - [ ] Return `Deny` with "Event content too large: {size} bytes (max {max})"
- [ ] 4.1.3.2 — Reject events with > `max_tags` tags (default 2000)
  - [ ] Return `Deny` with "Too many tags: {count} (max {max})"
- [ ] 4.1.3.3 — Make thresholds configurable
  - [ ] On invalid config values: log warning, use defaults

#### Step 4.1.4 — Content Policy Rule
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [x] 4.1.4.1 — Implement configurable blocked-words filter (substring match)
  - [x] Default blocked list: empty (opt-in, not opt-out)
  - [x] Case-insensitive matching
- [ ] 4.1.4.2 — Reject matching events with `Deny("Content policy violation")`
  - [ ] Do NOT include the matched word in the client-facing reason (privacy)
  - [ ] Log matched word at debug level for operator review only
- [ ] 4.1.4.3 — Allow operator to load word list from `config.toml` `[security.blocked_words]`
  - [ ] On empty list: rule is effectively a no-op (passes everything)
  - [ ] On malformed config: log error, skip rule, alert operator

#### Step 4.1.5 — Connection Limits Rule
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.1.5.1 — Implement global connection limit (default 100)
  - [ ] On limit reached: reject new connections with NOTICE "Server at capacity"
  - [ ] Log at warn level: "Connection limit reached ({count}/{max})"
- [ ] 4.1.5.2 — Implement per-IP connection limit (default 5)
  - [ ] On per-IP limit: reject with NOTICE "Too many connections from your address"
  - [ ] Log at info level: "Per-IP limit reached for {ip_masked}"
- [ ] 4.1.5.3 — Track connections in `HashMap<IpAddr, usize>`
  - [ ] Decrement count on disconnect (never go negative — use saturating_sub)
  - [ ] Periodically audit connection count vs actual connections

#### Step 4.1.6 — Rules Engine Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §3_

- [x] 4.1.6.1 — Write `test_rate_limiter_allows_under_limit`
  - [x] Verify exactly N events allowed within window
- [x] 4.1.6.2 — Write `test_rate_limiter_denies_over_limit`
  - [x] Verify denial reason contains "Rate limit"
- [x] 4.1.6.3 — Write `test_rate_limiter_resets_after_window`
  - [ ] Use mockable time/clock for deterministic tests
- [ ] 4.1.6.4 — Write `test_size_validator_accepts_normal`
  - [ ] Test at exactly max size (edge case)
- [ ] 4.1.6.5 — Write `test_size_validator_rejects_oversized`
  - [ ] Test at max+1 byte (boundary)
- [ ] 4.1.6.6 — Write `test_content_policy_blocks_word`
  - [ ] Test case-insensitive matching
- [ ] 4.1.6.7 — Write `test_content_policy_allows_clean`
  - [ ] Verify no false positives with partial word matches
- [ ] 4.1.6.8 — Write `test_connection_limit_enforced`
  - [ ] Verify connections at limit-1 succeed, at limit fail
- [ ] 4.1.6.9 — Write `test_pipeline_runs_all_rules` — multi-rule pipeline evaluation
  - [ ] Verify rules run in configured order
  - [ ] Verify short-circuit on first Deny
- [ ] 4.1.6.10 — Commit: `feat: security monitor with 4 rules`

---

### Phase 4.2: Alert System

#### Step 4.2.1 — Alert Storage
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md), [05-data-model.md](mvp/05-data-model.md)_

- [ ] 4.2.1.1 — Create `security_alerts` table (via existing migration)
  - [ ] Use `CREATE TABLE IF NOT EXISTS` for idempotent migration
  - [ ] Index on `timestamp DESC` for recency queries
- [ ] 4.2.1.2 — Implement `insert_alert(alert: &SecurityAlert) → Result<i64>`
  - [ ] On DB error: log at error level, don't crash the relay
  - [ ] Alert storage failure must never block event processing
- [ ] 4.2.1.3 — Implement `list_alerts(limit, offset) → Result<Vec<SecurityAlert>>`
  - [ ] Clamp limit to reasonable max (e.g., 500)
  - [ ] Return empty Vec for no results (not error)
- [ ] 4.2.1.4 — Implement `alert_count() → Result<u64>`: total alert count
  - [ ] On DB error: log and return error (don't return 0)
- [ ] 4.2.1.5 — Generate alert when any rule returns `Deny` or `Flag`
  - [ ] Include rule name, severity, and triggering event details

#### Step 4.2.2 — Alert API Routes
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 4.2.2.1 — Implement `GET /api/v1/security/alerts` → paginated alert list
  - [ ] Validate pagination params (reject negative, cap maximum)
  - [ ] On DB error: return 503 `{"error": "Service temporarily unavailable"}`
- [ ] 4.2.2.2 — Implement `GET /api/v1/security/alerts/count` → alert count
  - [ ] Return `{"count": N}` format
- [ ] 4.2.2.3 — Implement `GET /api/v1/security/status` → monitor summary (rules loaded, active connections, alerts/hour)
  - [ ] Return degraded status if any subsystem is unhealthy
  - [ ] On partial failures: include available data, note unavailable fields

#### Step 4.2.3 — Alert Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §3_

- [ ] 4.2.3.1 — Write `test_alert_stored_on_violation` — rule deny → alert in DB
  - [ ] Verify alert contains correct rule name and severity
- [ ] 4.2.3.2 — Write `test_alert_api_returns_alerts` — GET returns stored alerts
  - [ ] Test pagination (offset, limit)
- [ ] 4.2.3.3 — Write `test_alert_count` — count endpoint accurate after inserts
  - [ ] Test count after multiple alerts of different severities
- [ ] 4.2.3.4 — Commit: `feat: security alert storage and API`

---

### Phase 4.3: Security Integration

#### Step 4.3.1 — Wire Into Relay Pipeline
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 3.2_

- [ ] 4.3.1.1 — Insert security monitor evaluation before `event_store.insert()`
  - [ ] On Deny: send NOTICE to client with reason, do NOT store event
  - [ ] On Flag: store event but also create alert
  - [ ] On Allow: proceed normally
- [ ] 4.3.1.2 — Apply connection limits in WebSocket accept handler
  - [ ] Check limit BEFORE upgrading to WebSocket (save resources)
  - [ ] On rejection: close with appropriate WebSocket close code
- [ ] 4.3.1.3 — Ensure security check latency < 1ms for single event
  - [ ] If check is slow, log at warn level with timing
- [ ] 4.3.1.4 — Write integration test: blocked event → not stored + alert created + NOTICE sent
  - [ ] Verify event is truly absent from DB (not just filtered from query)
- [ ] 4.3.1.5 — Commit: `feat: wire security monitor into relay pipeline`

> **STAGE 4 MILESTONE:** Every event passes through a configurable rules pipeline. Violations are denied, logged, and queryable. Relay is protected from abuse.

---

## Stage 5: Tauri Desktop GUI

### Phase 5.1: Tauri Command Bridge

#### Step 5.1.1 — Command Infrastructure
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.1_

- [ ] 5.1.1.1 — Create `src/commands.rs` with `#[tauri::command]` functions
  - [ ] Module exists purely as a bridge — no business logic here
- [ ] 5.1.1.2 — Add `mod commands` to `src/main.rs`
  - [ ] Verify module compiles before wiring commands
- [ ] 5.1.1.3 — Register commands via `tauri::Builder::default().invoke_handler(tauri::generate_handler![...])`
  - [ ] Register ALL commands in a single generate_handler! call
- [ ] 5.1.1.4 — Implement `get_relay_status` → `Result<RelayStatus, String>`
  - [ ] Return connected count, events stored, uptime
  - [ ] On backend error: return Err with user-facing message, not internal details
  - [ ] Error UX: frontend shows "Unable to fetch relay status" with retry button
- [ ] 5.1.1.5 — Implement `get_content_stats` → `Result<ContentStoreStats, String>`
  - [ ] Return item count, total bytes, max per item
  - [ ] On backend error: return Err("Unable to fetch content statistics")
- [ ] 5.1.1.6 — Implement `get_security_summary` → `Result<SecuritySummary, String>`
  - [ ] Return active rules, recent alert count, connection count
  - [ ] On backend error: return Err("Unable to fetch security status")
- [ ] 5.1.1.7 — Implement `get_recent_alerts(limit)` → `Result<Vec<SecurityAlert>, String>`
  - [ ] Clamp limit to 100 max on backend (don't trust frontend input)
  - [ ] On no alerts: return empty Vec (not error)
- [ ] 5.1.1.8 — Implement `get_config` → `Result<AppConfig, String>`
  - [ ] Redact any sensitive fields (keys, passwords) before returning
  - [ ] On config read error: return Err("Unable to load configuration")
- [ ] 5.1.1.9 — Implement `store_content(bytes)` → `Result<String, String>` (returns CID)
  - [ ] On oversized content: return Err("Content too large (max {max}MB)")
  - [ ] On quota exceeded: return Err("Storage quota exceeded")
  - [ ] Error UX: frontend shows specific message with actionable guidance

#### Step 5.1.2 — Command Tests
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §4_

- [ ] 5.1.2.1 — Write `test_get_relay_status_returns_valid_json`
  - [ ] Verify all expected fields present in response
- [ ] 5.1.2.2 — Write `test_get_content_stats_returns_valid_json`
  - [ ] Verify counts are non-negative
- [ ] 5.1.2.3 — Write `test_get_security_summary_returns_valid_json`
  - [ ] Verify rule count matches configured rules
- [ ] 5.1.2.4 — Write `test_store_content_via_command` — store and verify CID returned
  - [ ] Test error case: oversized content returns descriptive error
- [ ] 5.1.2.5 — Commit: `feat: Tauri command bridge with 6 commands`

---

### Phase 5.2: Frontend UI

#### Step 5.2.1 — Dashboard Layout
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.2_

- [ ] 5.2.1.1 — Create `index.html` with three-panel layout (relay, content, security)
  - [ ] Semantic HTML: use `<section>`, `<header>`, `<main>` for accessibility
  - [ ] Include `<noscript>` fallback message
- [ ] 5.2.1.2 — Implement relay status panel: connection count, events stored, uptime
  - [ ] Show loading spinner while fetching
  - [ ] On error: show "Relay status unavailable" with ⚠️ icon and retry link
  - [ ] Error UX: grey out stale data, show "last updated" timestamp
- [ ] 5.2.1.3 — Implement content store panel: items count, total size, CID list
  - [ ] Show loading spinner while fetching
  - [ ] On error: show "Content stats unavailable" with retry option
  - [ ] Error UX: display last known data with "stale" indicator
- [ ] 5.2.1.4 — Implement security panel: rule status indicators, alert count, recent alerts
  - [ ] Show loading spinner while fetching
  - [ ] Color-code alerts by severity (Info=blue, Warning=amber, Critical=red)
  - [ ] On error: show "Security status unavailable" with retry option
- [ ] 5.2.1.5 — Style with CSS: dark theme, monospace for data, responsive at 800px+
  - [ ] Sufficient color contrast for accessibility (WCAG AA minimum)
  - [ ] Error states visually distinct from success states
- [ ] 5.2.1.6 — Add status indicator dots: green (healthy), yellow (degraded), red (error)
  - [ ] Update dots based on actual subsystem health, not just connectivity

#### Step 5.2.2 — Data Binding
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md)_

- [ ] 5.2.2.1 — Implement `main.js` with `window.__TAURI__.invoke()` calls
  - [ ] Check that `window.__TAURI__` exists before invoking (graceful fallback for dev mode)
  - [ ] On missing Tauri bridge: show "Running outside Tauri desktop app" message
- [ ] 5.2.2.2 — Implement 5-second polling loop for status updates
  - [ ] Use `setInterval` with error handling (don't let one failed poll stop all future polls)
  - [ ] On consecutive errors (3+): increase interval to 15s, show "Connection issues" banner
  - [ ] On recovery after errors: reset interval to 5s, clear error banner
- [ ] 5.2.2.3 — Implement `formatBytes()`, `formatDuration()`, `formatTimestamp()` helpers
  - [ ] Handle edge cases: 0 bytes, 0 duration, null/undefined timestamps
  - [ ] Never throw on bad input — return "N/A" or "unknown"
- [ ] 5.2.2.4 — Implement alert list rendering with severity badges
  - [ ] Sanitize alert text before inserting into DOM (prevent XSS from event content)
  - [ ] Use `textContent` not `innerHTML` for user-derived data
- [ ] 5.2.2.5 — Handle invoke errors gracefully: show inline error message per panel
  - [ ] Each panel independently recoverable (one failure doesn't break others)
  - [ ] Error messages are user-friendly: "Could not load X" not "TypeError: undefined"

#### Step 5.2.3 — Frontend Tests (Manual)
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §4_

- [ ] 5.2.3.1 — Manual: launch app → verify all three panels render
  - [ ] Check no JavaScript errors in console
- [ ] 5.2.3.2 — Manual: verify status updates every 5 seconds
  - [ ] Verify polling continues after network interruption
- [ ] 5.2.3.3 — Manual: verify error states display correctly when backend is down
  - [ ] Each panel should show error independently
- [ ] 5.2.3.4 — Manual: publish Nostr event → verify event count increments in UI
  - [ ] Verify security violation shows in alert panel
- [ ] 5.2.3.5 — Manual: verify window title, size (1200×800), and app icon
  - [ ] Verify minimum window size is usable
- [ ] 5.2.3.6 — Commit: `feat: Tauri dashboard UI with polling`

---

### Phase 5.3: Tauri Integration

#### Step 5.3.1 — Startup Orchestration
_Ref: [04-tauri-gui.md](mvp/04-tauri-gui.md), [06-config-packaging.md](mvp/06-config-packaging.md)_

- [x] 5.3.1.1 — Wire `main.rs` to start all subsystems in order: config → DB → relay → content store → security monitor → API
  - [x] Each subsystem startup: log success or failure with subsystem name
  - [x] On critical failure (DB, relay): log error, show user dialog, exit cleanly
  - [ ] On non-critical failure (content store): log warning, continue in degraded mode
- [ ] 5.3.1.2 — Pass shared state (DB pool, ContentStore, SecurityMonitor) via Tauri managed state
  - [ ] Use `Arc<Mutex<_>>` or `Arc<RwLock<_>>` for shared mutable state
  - [ ] Document which locks are held during which operations
- [ ] 5.3.1.3 — Ensure graceful shutdown: Ctrl+C → close WebSocket → flush DB → exit
  - [ ] Register tokio signal handler for SIGINT/SIGTERM
  - [ ] Timeout shutdown after 5 seconds (force exit if subsystems hang)
  - [ ] Log shutdown progress: "Closing WebSocket server... Flushing database... Done."
- [ ] 5.3.1.4 — Add startup splash log: app version, config path, listen addresses
  - [ ] Include subsystem health summary at startup
- [ ] 5.3.1.5 — Verify app launches fresh (no prior data) without errors
  - [ ] DB creates tables automatically
  - [ ] Data directories created automatically

#### Step 5.3.2 — Packaging & Distribution
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.3_

- [ ] 5.3.2.1 — Verify `cargo tauri build` succeeds on Linux
  - [ ] Fix any build errors (missing system libs, linker errors)
  - [ ] Binary is under 50MB (per resource budget)
- [ ] 5.3.2.2 — Verify `cargo tauri build` succeeds on macOS (cross-compile or CI)
  - [ ] Handle platform-specific file paths (/ vs \)
- [ ] 5.3.2.3 — Verify `cargo tauri build` succeeds on Windows (cross-compile or CI)
  - [ ] Handle Windows-specific firewall prompts for WebSocket server
- [ ] 5.3.2.4 — Add default `config.toml` to bundled resources
  - [ ] Config clearly commented with all options
  - [ ] Defaults are safe and conservative
- [ ] 5.3.2.5 — Verify installed app creates data directory on first run
  - [ ] Handle permission errors on data directory creation
  - [ ] Error UI: show dialog "Cannot create data directory at {path}: {error}"
- [ ] 5.3.2.6 — Commit: `feat: Tauri packaging and distribution`

> **STAGE 5 MILESTONE:** Desktop app launches, displays real-time relay/content/security status, and can be distributed as a single installable binary per platform.

---

## Stage 6: Release & Verification

### Phase 6.1: Acceptance Testing

#### Step 6.1.1 — Full Test Suite
_Ref: [07-acceptance-tests.md](mvp/07-acceptance-tests.md), [08-implementation-order.md](mvp/08-implementation-order.md) Step 4.4_

- [ ] 6.1.1.1 — Run full `cargo test` — all unit tests pass
  - [ ] Fix any test failures before proceeding (zero tolerance)
  - [ ] Log test report to file for auditability
- [ ] 6.1.1.2 — Run integration tests — WebSocket + API end-to-end
  - [ ] Test with real network connections (not just mocks)
  - [ ] On intermittent failures: retry once, then investigate
- [ ] 6.1.1.3 — Execute manual test checklist (from [07-acceptance-tests.md](mvp/07-acceptance-tests.md) §5)
  - [ ] Screenshot evidence for each manual test
  - [ ] Document any deviations or known issues
- [ ] 6.1.1.4 — Verify resource budget compliance:
  - [ ] Binary < 50MB
  - [ ] Idle RAM < 100MB
  - [ ] Idle CPU < 2%
  - [ ] SQLite DB < 1GB after 10k events
  - [ ] Cold start < 3 seconds
- [ ] 6.1.1.5 — Commit test results/screenshots: `test: acceptance test pass`

---

### Phase 6.2: Documentation Review

#### Step 6.2.1 — Documentation Audit
_Ref: [00-overview.md](mvp/00-overview.md)_

- [ ] 6.2.1.1 — Verify README.md has build instructions, run instructions, screenshot
  - [ ] Build instructions tested on clean checkout
  - [ ] Include prerequisite system dependencies
- [ ] 6.2.1.2 — Verify ARCHITECTURE.md matches final code structure
  - [ ] Update any module names that changed during implementation
  - [ ] Remove references to deleted/renamed modules
- [ ] 6.2.1.3 — Verify all docs/mvp/ specs match implementation
  - [ ] Flag any spec‒code divergences for follow-up
- [ ] 6.2.1.4 — Verify FAQ.md and TROUBLESHOOTING.md are current
  - [ ] Add any issues encountered during development
  - [ ] Include common build errors and fixes
- [ ] 6.2.1.5 — Commit: `docs: final documentation review`

---

### Phase 6.3: Security Audit

#### Step 6.3.1 — Security Review
_Ref: [03-security-monitor.md](mvp/03-security-monitor.md)_

- [ ] 6.3.1.1 — Audit: no panics on malformed input (fuzz key message paths)
  - [ ] Test with empty strings, max-length strings, null bytes, unicode edge cases
  - [ ] Any panic found → fix and add regression test
- [ ] 6.3.1.2 — Audit: no SQL injection vectors (all queries via SQLx bind params)
  - [ ] Grep for raw string interpolation in SQL queries
  - [ ] Verify no user input reaches raw SQL
- [ ] 6.3.1.3 — Audit: no path traversal in content store (CID validated before use)
  - [ ] Test with CIDs containing `..`, `/`, `\`, null bytes
  - [ ] Verify all file paths are within data directory
- [ ] 6.3.1.4 — Audit: no XSS in frontend (all user content uses `textContent`, not `innerHTML`)
  - [ ] Grep for `innerHTML` usage — replace with `textContent` if found
  - [ ] Test with event content containing `<script>` tags
- [ ] 6.3.1.5 — Audit: WebSocket connections timeout after inactivity (default 5min)
  - [ ] Verify zombie connections are cleaned up
  - [ ] Test with connection that sends no data
- [ ] 6.3.1.6 — Commit: `security: audit pass for MVP`

---

### Phase 6.4: Release Tagging

#### Step 6.4.1 — Release
_Ref: [06-config-packaging.md](mvp/06-config-packaging.md)_

- [ ] 6.4.1.1 — Update `Cargo.toml` version to `0.1.0`
  - [ ] Verify `tauri.conf.json` version matches
- [ ] 6.4.1.2 — Update CHANGELOG or release notes with feature summary
  - [ ] List all major features: Nostr relay, content store, security monitor, desktop UI
  - [ ] List known limitations and planned follow-ups
- [ ] 6.4.1.3 — Create Git tag `v0.1.0-mvp`
  - [ ] Verify tag is on correct commit (all tests pass)
- [ ] 6.4.1.4 — Build release binaries for all target platforms
  - [ ] Verify each binary launches and passes smoke test
  - [ ] Record binary sizes for resource budget verification
- [ ] 6.4.1.5 — Archive: copy tag SHA + build artifacts link to MVP-PLAN.md
  - [ ] Update MVP-PLAN.md status to "Complete"

> **STAGE 6 MILESTONE:** MVP is tested, documented, audited, versioned, and released. All acceptance criteria from [00-overview.md] are verified.

---

## Summary

| Stage | Name | Phases | Steps | Tasks | Subtasks (approx) |
|-------|------|--------|-------|-------|--------------------|
| 1 | Infrastructure & Foundation | 3 | 9 | 33 | ~90 |
| 2 | Working Nostr Relay | 4 | 12 | 53 | ~140 |
| 3 | Content-Addressed Storage | 2 | 7 | 38 | ~100 |
| 4 | Security Monitor | 3 | 9 | 42 | ~115 |
| 5 | Tauri Desktop GUI | 3 | 8 | 39 | ~105 |
| 6 | Release & Verification | 4 | 4 | 18 | ~50 |
| **Total** | | **19** | **49** | **223** | **~600** |

> **How to use this checklist:** Work through stages in order. Within each stage, complete phases top-to-bottom. Check off each task and its subtasks as completed. Every step ends with a Git commit. The subtasks ensure that error handling, error reporting, error UI/UX, and code robustness are addressed at every level — not as an afterthought, but as part of the implementation itself.
