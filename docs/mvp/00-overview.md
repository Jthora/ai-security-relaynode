# AI Security RelayNode — MVP Technical Overview

> **Version:** 0.2.0-mvp
> **Target:** Cross-platform desktop app that turns any computer into a Nostr Relay + IPFS Node with security monitoring
> **Stack:** Rust + Tauri + SQLite

---

## MVP Definition

The MVP ships three capabilities:

1. **Relay** — A NIP-01 compliant Nostr relay that accepts WebSocket connections, validates events, stores them in SQLite, and serves subscriptions.
2. **Node** — Content-addressed storage that persists data to disk, accessible via API and integrated with the relay.
3. **Monitor** — A configurable security rules engine that inspects relay traffic in real-time, enforces rate limits, and logs violations.

Wrapped in a Tauri desktop app with Start/Stop controls, a status dashboard, and a security alert feed. Installable via native packages on Linux, Windows, and macOS.

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                  Tauri Desktop Shell                  │
│  ┌────────────────────────────────────────────────┐  │
│  │              Frontend (WebView)                 │  │
│  │  index.html + main.js                          │  │
│  │  • Start/Stop toggle                           │  │
│  │  • Status dashboard (connections, events, etc)  │  │
│  │  • Security alert feed                         │  │
│  └──────────────────┬─────────────────────────────┘  │
│                     │ #[tauri::command] IPC           │
│  ┌──────────────────┴─────────────────────────────┐  │
│  │              Rust Backend (tokio)               │  │
│  │                                                 │  │
│  │  ┌──────────────┐     ┌──────────────┐         │  │
│  │  │ NostrRelay   │     │ ContentStore │         │  │
│  │  │ :8080 (WS)   │     │ (iroh/local) │         │  │
│  │  └──────┬───────┘     └──────┬───────┘         │  │
│  │         │                     │                 │  │
│  │  ┌──────┴─────────────────────┴──────┐         │  │
│  │  │       SecurityMonitor (Bot)       │         │  │
│  │  │  • RateLimiter                    │         │  │
│  │  │  • EventValidator                 │         │  │
│  │  │  • ContentPolicy                  │         │  │
│  │  │  • AlertLogger                    │         │  │
│  │  └──────────────────────────────────-┘         │  │
│  │                                                 │  │
│  │  ┌──────────────────────────────────────┐      │  │
│  │  │  SQLite (data/relaynode.db)          │      │  │
│  │  │  • events table (NIP-01)             │      │  │
│  │  │  • security_alerts table             │      │  │
│  │  │  • config table                      │      │  │
│  │  └──────────────────────────────────────┘      │  │
│  └─────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

---

## Component Specifications

| Component | Spec Document | Status |
|-----------|---------------|--------|
| Nostr Relay | [01-nostr-relay.md](01-nostr-relay.md) | Core — must work |
| Content Store (IPFS) | [02-content-store.md](02-content-store.md) | Core — must work |
| Security Monitor | [03-security-monitor.md](03-security-monitor.md) | Core — must work |
| Tauri GUI | [04-tauri-gui.md](04-tauri-gui.md) | Shell — wraps core |
| Data Model | [05-data-model.md](05-data-model.md) | Foundation |
| Implementation Roadmap | [06-implementation-roadmap.md](06-implementation-roadmap.md) | Execution plan |

---

## What's In vs. Out

### IN (MVP)

- NIP-01 relay (EVENT, REQ, CLOSE, EOSE, OK, NOTICE)
- Event signature validation (secp256k1)
- Event ID validation (SHA-256)
- SQLite event persistence
- Content-addressed local storage (store/retrieve by hash)
- Per-pubkey rate limiting
- Max event size enforcement
- Security alert logging + dashboard feed
- Configurable security rules (TOML)
- Tauri desktop app with status dashboard
- Native installers (deb/AppImage, msi, dmg)

### OUT (Phase 2+)

- Investigation CRUD system
- Earth Alliance profiles / clearance levels
- Bridge/subnet inter-node coordination
- Pubkey allowlist/blocklist management
- AI/ML content analysis
- Full IPFS DHT participation
- Event sync between relays
- Headless (no-GUI) mode
- Auto-start on boot

---

## Current Codebase Gap Analysis

| MVP Feature | Current Code | Gap | Effort |
|-------------|-------------|-----|--------|
| NIP-01 relay | WebSocket accepts, protocol parser exists, subscription manager exists | Message dispatch loop missing — `handle_websocket_messages()` is a sleep stub | Medium |
| Event ID validation | `validate_event_id()` implemented correctly (SHA-256) | Working but never called (relay doesn't dispatch) | Wiring only |
| Signature validation | `validate_signature()` implemented (secp256k1) | Working but never called | Wiring only |
| SQLite event storage | `SqliteEventStore` with schema, `store_event()`, `query_events()` | `query_events()` has broken parameter binding | Small fix |
| Content store | In-memory HashMap with fake hashes | Complete rewrite needed — either iroh or local CAS | Large |
| Rate limiting | Not implemented | New module | Medium |
| Security alerts | Not implemented | New table + new module | Medium |
| Configurable rules | Not implemented | New config format + parser | Small |
| Tauri commands | Zero `#[tauri::command]` registered | Bridge between frontend and backend doesn't exist | Medium |
| Status dashboard | UI elements exist in HTML | Backend status queries need real data (currently hardcoded) | Small |
| Native installers | Tauri conf exists | CI/CD pipeline needed | Infrastructure |

---

## Resource Budget

Target: River's Raspberry Pi (4GB RAM, ARM64)

| Component | RAM Target | CPU Target |
|-----------|-----------|-----------|
| Nostr Relay | <50MB | Idle when no connections |
| Content Store | <30MB + disk | Idle when no operations |
| Security Monitor | <10MB | Per-event processing |
| SQLite | <20MB | Connection pool (5 conns) |
| Tauri WebView | <80MB | Only when GUI visible |
| **Total** | **<190MB** | |

---

## Success Criteria

The MVP is complete when:

1. A user downloads a single installer, runs it, and clicks "Start"
2. A Nostr client (e.g., Damus, Amethyst, Snort) can connect and exchange messages through the relay
3. Content can be stored and retrieved by hash via the API
4. The dashboard shows live connection count, event throughput, and storage usage
5. A spam bot hitting the relay gets rate-limited and the violation appears in the security feed
6. The app runs for 24 hours on a Raspberry Pi 4 without crashing or exceeding 200MB RAM
