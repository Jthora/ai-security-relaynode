# MVP Spec 08: Implementation Order

> **Goal:** Ship a working desktop app (Nostr relay + content store + security monitor) in the minimum number of ordered steps.

---

## Purpose

This document defines the exact sequence of code changes. Each step is a single commit (or small PR) that leaves the codebase in a compilable, testable state. No step depends on a later step.

---

## Prerequisites

- Rust toolchain installed (`rustup`, stable channel)
- `cargo check` passes on current codebase (after cleanup)
- Git clean working tree

---

## Phase 1: Foundation (Working Relay)

### Step 1.1 — Trim Cargo.toml

**What:** Remove unused dependencies, add missing ones.

```diff
- libp2p = { version = "0.53", features = [...] }
- num-bigint = "0.4"
- num-traits = "0.2"
- crossbeam-channel = "0.5"
+ toml = "0.8"
+ dirs = "5"
```

**Verify:** `cargo check` passes with fewer dependencies and faster compile.

**Commit:** `chore: trim unused deps, add toml + dirs`

---

### Step 1.2 — Fix Config::load()

**What:** Replace hardcoded config with file → env → default loading.

**Files:** `src/config.rs`

**Changes:**
1. Implement `Config::defaults()` returning current hardcoded values
2. Implement `Config::load()` reading `data/config.toml` via `toml` crate
3. Add env var overrides (`RELAY_PORT`, `API_PORT`, `JWT_SECRET`, `DATABASE_URL`)
4. Auto-generate config file on first run
5. Remove dead fields: `SubnetMode`, `GatewayConfig`, `TeamSubnetConfig`

**Tests:** `test_config_defaults`, `test_config_from_file`, `test_config_env_override`

**Commit:** `feat: config loading from file + env vars`

---

### Step 1.3 — Fix Event Store Query Building

**What:** Fix the broken parameter binding in `build_filter_where_clause()`.

**Files:** `src/event_store.rs`

**Changes:**
1. Make `build_filter_where_clause()` return `(String, Vec<sqlx::Value>)` tuple
2. Use the returned params in `sqlx::query()` bindings
3. Remove Earth Alliance clearance columns (Phase 2)
4. Simplify to MVP schema (events table only)

**Tests:** `test_event_insert_and_retrieve`, `test_event_filter_by_kind`, `test_event_filter_since_until`

**Commit:** `fix: event store query parameter binding`

---

### Step 1.4 — Wire WebSocket Message Dispatch

**What:** Fix the critical bug: `handle_websocket_messages()` currently sleeps instead of reading messages.

**Files:** `src/nostr_relay.rs`

**Changes:**
1. Replace sleep loop with `while let Some(msg) = ws_stream.next().await`
2. On Text message → `process_message()` → send response back
3. On Close → cleanup connection
4. On Ping → Pong
5. Store event in EventStore on valid EVENT
6. Query EventStore on REQ, send matching events + EOSE
7. Remove subscription on CLOSE
8. Broadcast new events to matching subscriptions

**Tests:** `test_relay_accepts_websocket_connection`, `test_relay_stores_valid_event`, `test_relay_rejects_invalid_signature`, `test_relay_subscription_receives_stored_events`

**Commit:** `feat: wire WebSocket read loop to message processing`

---

### Step 1.5 — Database Migration System

**What:** Replace ad-hoc table creation with versioned migrations.

**Files:** `src/database.rs`, `src/migrations/001_mvp_schema.sql`

**Changes:**
1. Create `schema_version` table
2. Write `001_mvp_schema.sql` with events table (MVP columns only)
3. Run migrations on startup in `DatabaseManager::new()`
4. Remove investigation tables from MVP migration (Phase 2)

**Tests:** `test_database_migrations_run`

**Commit:** `feat: versioned database migrations`

---

### Step 1.6 — Integration Test: Full Relay Round-Trip

**What:** End-to-end test that starts relay, publishes event, subscribes, receives it.

**Files:** `tests/integration/relay_test.rs`

**Tests:** `test_relay_subscription_receives_new_events`, `test_relay_count_events`

**Commit:** `test: relay round-trip integration test`

---

**Phase 1 Milestone:** `cargo test` passes. A WebSocket client can connect, publish events, subscribe, and receive events. The relay is NIP-01 compliant and durable (SQLite-backed).

---

## Phase 2: Content Store

### Step 2.1 — Content-Addressed Storage Module

**What:** Replace fake IPFS with filesystem-backed SHA-256 content store.

**Files:** `src/ipfs_node.rs` (rename to `src/content_store.rs`)

**Changes:**
1. Rename `IPFSNode` → `ContentStore`
2. `store(bytes)` → compute SHA-256 → write to `data/cas/ab/cd/abcd...` → return CID
3. `retrieve(cid)` → read from disk → return bytes
4. `delete(cid)` → remove file
5. Size limit enforcement
6. SQLite metadata table (`content_index`)
7. Update `src/lib.rs` module declaration

**Tests:** All content store unit tests from [07-acceptance-tests.md](07-acceptance-tests.md)

**Commit:** `feat: content-addressed storage replacing fake IPFS`

---

### Step 2.2 — Content Store API Routes

**What:** Wire content store to HTTP API.

**Files:** `src/api_gateway.rs`

**Changes:**
1. `POST /content` → store content, return CID
2. `GET /content/:cid` → retrieve content
3. `GET /content` → list stored items (paginated)
4. `GET /content/stats` → storage statistics
5. Remove hardcoded `/ipfs/status` route
6. Add content-type detection

**Tests:** API-level tests via integration test hitting HTTP endpoints

**Commit:** `feat: content store API routes`

---

**Phase 2 Milestone:** Content can be stored and retrieved via HTTP API. Storage is durable, deduplicated, and size-limited.

---

## Phase 3: Security Monitor

### Step 3.1 — Security Monitor Module

**What:** Replace XOR stub with rules-based event filter.

**Files:** `src/security_layer.rs` (rewrite)

**Changes:**
1. Define `SecurityMonitor` struct with rule pipeline
2. Implement `RateLimiter` (per-connection event counting)
3. Implement `ContentSizeRule` (max bytes check)
4. Implement `PubkeyFilter` (blocklist + optional allowlist)
5. Implement `EventValidator` (id + signature verification)
6. Load rules from `data/security_config.toml`
7. `check_event()` runs all rules, returns Allow/Reject
8. Log rejections to `security_alerts` table

**Tests:** All security monitor unit tests from [07-acceptance-tests.md](07-acceptance-tests.md)

**Commit:** `feat: rules-based security monitor`

---

### Step 3.2 — Integrate Monitor into Relay Pipeline

**What:** Insert security checks between WebSocket receive and event processing.

**Files:** `src/nostr_relay.rs`

**Changes:**
1. After parsing EVENT message, call `security_monitor.check_event()`
2. If Reject → send `["OK", id, false, reason]`, skip storage
3. If Allow → proceed with storage and broadcast
4. Rate limiting checked per-connection (using connection_id)

**Tests:** `test_monitor_in_relay_pipeline`

**Commit:** `feat: security monitor in relay event pipeline`

---

### Step 3.3 — Security Alert API

**What:** HTTP endpoints for querying security alerts.

**Files:** `src/api_gateway.rs`

**Changes:**
1. `GET /alerts` → recent alerts (paginated)
2. `GET /alerts/stats` → alert counts by type
3. Used by GUI for security feed

**Commit:** `feat: security alert API endpoints`

---

**Phase 3 Milestone:** Relay rejects malicious/spam events before storage. All violations logged. Alert feed available via API.

---

## Phase 4: Tauri Desktop GUI

### Step 4.1 — Register Tauri Commands

**What:** Bridge Rust backend to frontend via `#[tauri::command]`.

**Files:** `src/main.rs`

**Changes:**
1. Define `AppState` struct holding all service handles
2. Implement 9 Tauri commands (see [04-tauri-gui.md](04-tauri-gui.md))
3. Register commands in `tauri::Builder`
4. Pass `AppState` via `tauri::Builder::manage()`

**Commit:** `feat: register Tauri command bridge`

---

### Step 4.2 — Update Frontend

**What:** Connect UI to real backend data via Tauri invoke.

**Files:** `index.html`, `main.js`

**Changes:**
1. Replace placeholder data with `invoke('get_service_status')` calls
2. Implement status polling (every 3 seconds)
3. Implement security alert feed with `invoke('get_security_alerts')`
4. Wire start/stop buttons to `invoke('start_relay')` / `invoke('stop_relay')`
5. Config form: load → edit → save via commands

**Commit:** `feat: wire frontend to Tauri command bridge`

---

### Step 4.3 — UI Polish

**What:** Visual improvements for MVP release.

**Files:** `index.html`, `main.js`

**Changes:**
1. Connection count badge
2. Alert severity color coding (info=blue, warning=yellow, critical=red)
3. Content store usage bar
4. Responsive layout for different window sizes
5. Error state handling (show message if backend unreachable)

**Commit:** `feat: dashboard UI polish`

---

**Phase 4 Milestone:** Desktop app launches, shows live status, displays security alerts, allows service control. This is the shippable MVP.

---

## Post-MVP Cleanup

### Step 5.1 — Remove Dead Modules

After MVP is working, remove modules no longer needed:

- `src/investigation_service.rs` (defer to Phase 2)
- `src/investigation_api.rs` (defer to Phase 2)
- `src/subnet_manager.rs` (defer to Phase 3)
- `src/subnet_types.rs` (defer to Phase 3)
- `src/auth.rs` JWT auth (not needed for local-only MVP)

**Or** keep them compiled but unreachable if planning Phase 2 soon.

---

## Commit Graph

```
main
  │
  ├─ chore: trim unused deps, add toml + dirs
  ├─ feat: config loading from file + env vars
  ├─ fix: event store query parameter binding
  ├─ feat: wire WebSocket read loop to message processing
  ├─ feat: versioned database migrations
  ├─ test: relay round-trip integration test
  │  ── Phase 1 Complete: Working Relay ──
  ├─ feat: content-addressed storage replacing fake IPFS
  ├─ feat: content store API routes
  │  ── Phase 2 Complete: Content Store ──
  ├─ feat: rules-based security monitor
  ├─ feat: security monitor in relay event pipeline
  ├─ feat: security alert API endpoints
  │  ── Phase 3 Complete: Security Monitor ──
  ├─ feat: register Tauri command bridge
  ├─ feat: wire frontend to Tauri command bridge
  ├─ feat: dashboard UI polish
  │  ── Phase 4 Complete: MVP Release ──
  └─ tag: v0.1.0
```

---

## Time Investment Estimate

| Phase | Steps | Complexity |
|-------|-------|------------|
| Phase 1: Working Relay | 6 steps | Highest — fixing core bugs |
| Phase 2: Content Store | 2 steps | Medium — new module, simple API |
| Phase 3: Security Monitor | 3 steps | Medium — new module, pipeline integration |
| Phase 4: Tauri GUI | 3 steps | Low-Medium — wiring + UI |
| **Total** | **14 steps** | **14 atomic commits** |

Each step is independent and testable. If any step fails, the previous steps still work.

---

## Decision Log

| Decision | Rationale |
|----------|-----------|
| Fix relay before content store | Relay is the core product; content store is secondary |
| Config before relay fix | Config needed for port binding and database URL |
| Event store fix before WebSocket fix | WebSocket handler needs working event store |
| Security after relay + content | Security wraps existing functionality |
| GUI last | GUI needs all backend services operational |
| Keep investigation code for now | It's the most complete code; removing it risks regressions |
| 14 small commits vs 4 big ones | Each commit compiles + tests pass = easy bisect |
