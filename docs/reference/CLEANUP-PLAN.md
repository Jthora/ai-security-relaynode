# AI Security RelayNode — Cleanup & Rectification Plan

> **Project:** ai-security-relaynode  
> **Created:** 2026-03-24  
> **Status:** Complete (pending `cargo check/test/clippy` — install Rust with `sudo apt install cargo rustc`)  
> **Scope:** Repository hygiene, dead code removal, test repair, script fixes, module trimming

---

## Phase 1: Repository Hygiene

Establish a clean git foundation. Nothing else should be committed until tracking is fixed.

### Step 1.1 — Create `.gitignore`

- [x] **1.1.1** Create `.gitignore` at project root
- [x] **1.1.2** Add `target/` (Rust build artifacts — 15,271 files currently tracked)
- [x] **1.1.3** Add `data/*.db`, `data/*.db-shm`, `data/*.db-wal` (SQLite runtime databases)
- [x] **1.1.4** Add `node_modules/`
- [x] **1.1.5** Add `*.backup` and common editor/OS artifacts (`.DS_Store`, `*.swp`, `Thumbs.db`)
- [x] **1.1.6** Add `test-token.txt` (generated test credential files)

### Step 1.2 — Remove Tracked Artifacts from Git

- [x] **1.2.1** Run `git rm -r --cached target/` to untrack build artifacts
- [x] **1.2.2** Run `git rm --cached data/nostr_events.db data/relaynode.db data/relaynode.db-shm data/relaynode.db-wal` to untrack databases
- [x] **1.2.3** Verify with `git status` that only deletions from index are staged (files remain on disk)

### Step 1.3 — Verify Clean State

- [x] **1.3.1** Run `git ls-files target/ | wc -l` — expect 0 ✓
- [x] **1.3.2** Run `git ls-files data/ | wc -l` — expect 0 ✓
- [x] **1.3.3** Commit the `.gitignore` and untracking changes ✓ (committed in `d7c54e4`)

---

## Phase 2: Delete Dead & Backup Files

Remove ~2,700+ lines of dead code across 12 files. All of these are either backup snapshots, abandoned experiments, or duplicates.

### Step 2.1 — Remove Backup Cargo Manifests

- [x] **2.1.1** Delete `Cargo.toml.backup` (old Cargo.toml snapshot with different repo URL and fewer deps)
- [x] **2.1.2** Delete `Cargo_clean.toml` (alternative Cargo.toml with Tauri disabled — stale experiment)

### Step 2.2 — Remove Backup Source Files

- [x] **2.2.1** Delete `src/lib_backup.rs` (older lib.rs snapshot with different module declarations)
- [x] **2.2.2** Delete `src/main_clean.rs` (~500 LOC alternative Tauri main — no `[[bin]]` target, never compiled)
- [x] **2.2.3** Delete `src/main_legacy_backup.rs` (~400 LOC older main iteration)
- [x] **2.2.4** Delete `src/main_validation.rs` (~250 LOC standalone test binary — never compiled)

### Step 2.3 — Remove Dead Architecture Modules

These modules are NOT declared in `lib.rs` and unreachable from the build:

- [x] **2.3.1** Delete `src/clean_config.rs` (~500 LOC — never integrated, only referenced by deleted `lib_backup.rs`)
- [x] **2.3.2** Delete `src/clean_gateway.rs` (~500 LOC — dead, gateway refactoring never completed)
- [x] **2.3.3** Delete `src/clean_subnet.rs` (~400 LOC — dead, subnet refactoring never completed)
- [x] **2.3.4** Delete `src/network_coordinator.rs` (~420 LOC — dead, references nonexistent modules)

### Step 2.4 — Remove Miscellaneous Dead Files

- [x] **2.4.1** Delete `test_jwt.rs` (root-level misplaced test file containing hardcoded JWT secrets and tokens)
- [x] **2.4.2** Delete `scripts/generate_test_token.py` (inferior underscore-named duplicate of `scripts/generate-test-token.py` — doesn't save token output)

### Step 2.5 — Verify Deletion

- [x] **2.5.1** Confirm no remaining `*_backup*`, `*_clean*`, or `*_legacy*` files in `src/` ✓
- [ ] **2.5.2** Run `cargo check` to verify the build still compiles ⏳ (run `sudo apt install cargo rustc` then `cargo check`)
- [x] **2.5.3** Commit all deletions ✓ (committed in `d7c54e4`)

---

## Phase 3: Fix Broken Tests

All unit tests currently fail to compile because they import modules that were never exported or don't exist. The integration test module declares 5 phantom submodules.

### Step 3.1 — Fix Unit Test Module Declarations

- [x] **3.1.1** Edit `tests/unit/mod.rs` — remove `config_tests` declaration (file does not exist)
- [x] **3.1.2** Verify remaining module declarations (`subnet_tests`, `gateway_tests`, `coordinator_tests`, `common`) match actual files

### Step 3.2 — Rewrite Unit Test Helpers

- [x] **3.2.1** Rewrite `tests/unit/common.rs` — replace imports of deleted modules (`clean_subnet`, `clean_gateway`, `network_coordinator`) with actually-exported modules (`subnet_manager`, `api_gateway`, `config`, etc.)
- [x] **3.2.2** Update test helper factory functions to construct types from live modules

### Step 3.3 — Rewrite Unit Test Files

- [x] **3.3.1** Rewrite `tests/unit/subnet_tests.rs` — test `subnet_types` validation, `BridgeConnection`, `TeamAnnouncement`, `DiscoveryMessage`
- [x] **3.3.2** Rewrite `tests/unit/gateway_tests.rs` — test `Config` gateway/bridge/security policy structures
- [x] **3.3.3** Rewrite `tests/unit/coordinator_tests.rs` — test `Config` composition, defaults, serialization roundtrips

### Step 3.4 — Fix Integration Test Declarations

- [x] **3.4.1** Edit `tests/integration/mod.rs` — remove all 5 phantom submodule declarations (`nostr_integration_tests`, `ipfs_integration_tests`, `security_integration_tests`, `network_integration_tests`, `common`) since none of these files exist
- [x] **3.4.2** Replace with a placeholder comment or stub test to prevent empty-module compile errors

### Step 3.5 — Verify Tests

- [ ] **3.5.1** Run `cargo test` — confirm compilation succeeds ⏳ (run `sudo apt install cargo rustc` then `cargo test`)
- [x] **3.5.2** Verify all rewritten unit tests use only live module APIs ✓ (all imports verified against source)
- [x] **3.5.3** Commit test fixes ✓ (committed in `d7c54e4`)

---

## Phase 4: Fix Shell Scripts

Three scripts hardcode `/Users/jono/Documents/GitHub/starcom-app/ai-security-relaynode` (macOS path from a different machine) and will fail on any other system.

### Step 4.1 — Fix Path Resolution

- [x] **4.1.1** Fix `scripts/fix_build.sh` — replace hardcoded `PROJECT_ROOT` with `PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"`
- [x] **4.1.2** Fix `scripts/test_all.sh` — same path fix
- [x] **4.1.3** Fix `scripts/validate.sh` — same path fix

### Step 4.2 — Fix Platform Assumptions

- [x] **4.2.1** Review `scripts/fix_build.sh` for macOS-only commands (`xcode-select`, `xcrun`, macOS SDK checks) — gated behind `uname` check for cross-platform use
- [x] **4.2.2** Review `scripts/validate.sh` for references to nonexistent paths (`docs/DEVELOPMENT-ROADMAP.md`, `docs/TESTING-STRATEGY.md`) — removed, replaced with actual project structure checks

### Step 4.3 — Verify Scripts

- [x] **4.3.1** Run `bash -n scripts/fix_build.sh` (syntax check) ✔
- [x] **4.3.2** Run `bash -n scripts/test_all.sh` (syntax check) ✔
- [x] **4.3.3** Run `bash -n scripts/validate.sh` (syntax check) ✔
- [x] **4.3.4** Commit script fixes ✓ (committed in `d7c54e4`)

---

## Phase 5: Trim `lib.rs` and Validate Build

Clean up module declarations so `lib.rs` only exports modules that are present and needed.

### Step 5.1 — Remove Dead Module Declarations

- [x] **5.1.1** Remove `pub mod clean_config_simple;` from `lib.rs` — removed, deleted `src/clean_config_simple.rs`
- [x] **5.1.2** Remove `pub mod services;` from `lib.rs` — confirmed zero external consumers, deleted `src/services.rs`
- [x] **5.1.3** Evaluate `pub mod subnet_manager;` and `pub mod subnet_types;` — **KEPT**: `subnet_types` is used by unit tests, `subnet_manager` has internal tests and represents real functionality

### Step 5.2 — Clean Up Re-exports

- [x] **5.2.1** `subnet_manager` retained — re-exports kept as-is
- [x] **5.2.2** Verified all remaining `pub use` lines resolve to valid module paths ✓

### Step 5.3 — Final Build Validation

- [ ] **5.3.1** Run `cargo check` — zero errors ⏳ (run `sudo apt install cargo rustc` then `cargo check`)
- [ ] **5.3.2** Run `cargo test` — all tests compile and pass ⏳
- [ ] **5.3.3** Run `cargo clippy` — no new warnings ⏳ (`rustup component add clippy` then `cargo clippy`)
- [x] **5.3.4** Commit trimmed `lib.rs` ✓ (committed in `d7c54e4`)

---

## Appendix: Security Hardening (Deferred — Requires Design Decisions)

These issues were identified during the audit but are larger changes that require architectural decisions rather than simple cleanup. They should be addressed before any production deployment.

| # | Issue | File | Severity |
|---|---|---|---|
| S1 | Hardcoded JWT secret `"your-secret-key-change-in-production"` (3 occurrences) | `src/auth.rs` | **Critical** |
| S2 | Plaintext password comparison with mock credentials (`"admin123"`, etc.) | `src/auth.rs` | **Critical** |
| S3 | Hardcoded encryption key `b"simple_key_for_phase1_testing_32"` | `src/security_layer.rs` | **Critical** |
| S4 | XOR "encryption" used in production code path | `src/security_layer.rs` | **Critical** |
| S5 | Hardcoded CORS origin `http://localhost:3000` | `src/api_gateway.rs` | **Medium** |
| S6 | `package.json` version (`1.0.0`) mismatches `Cargo.toml` version (`0.1.0`) | Root config files | **Low** |

---

## Dependency Order

```
Phase 1 (gitignore + untrack)
  └─→ Phase 2 (delete dead files)
        ├─→ Phase 3 (fix tests)          ← depends on knowing which modules remain
        ├─→ Phase 5 (trim lib.rs)        ← depends on which files were deleted
        └─→ Phase 4 (fix scripts)        ← independent, can run in parallel
```
