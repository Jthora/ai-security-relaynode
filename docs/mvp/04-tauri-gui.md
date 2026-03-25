# MVP Spec 04: Tauri Desktop GUI

> **Files:** `src/main.rs`, `index.html`, `main.js`, `tauri.conf.json`
> **Framework:** Tauri 1.5
> **Personas:** River (one-click start), Sage (connection monitoring), Cipher (security feed)

---

## Purpose

Provide a graphical interface that lets non-technical users start and manage the relay node without touching a terminal. The GUI is the product differentiator — Nostr relays exist as CLI tools already. This makes it accessible to River in a van and Sage at a gathering.

---

## Screen Layout

```
┌──────────────────────────────────────────────────────────┐
│  🛡️  AI Security RelayNode                    [_][□][X] │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─── Service Controls ────────────────────────────────┐ │
│  │  [ ▶ Start Services ]  [ ■ Stop ]  [ ↻ Refresh ]   │ │
│  │  Status: ● Running (uptime: 2h 14m)                 │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  ┌─── Nostr Relay ──────────┐  ┌─── Content Store ────┐ │
│  │  Status: ● Online        │  │  Status: ● Online    │ │
│  │  Address: ws://:8080     │  │  Items: 142          │ │
│  │  Connections: 12         │  │  Storage: 48.2 MB    │ │
│  │  Events stored: 4,281    │  │  Max: 5.0 GB         │ │
│  │  Events/min: 23          │  │                      │ │
│  └──────────────────────────┘  └──────────────────────┘ │
│                                                          │
│  ┌─── Security Monitor ───────────────────────────────┐ │
│  │  Events checked: 4,281  |  Blocked: 7  |  Rules: 4 │ │
│  │                                                     │ │
│  │  ⚠ 14:23:05  rate_limit_exceeded  npub1abc...      │ │
│  │    60 events in 60s from 192.168.1.42               │ │
│  │  ⚠ 14:21:12  event_too_large      npub1def...      │ │
│  │    Event 78KB exceeded 64KB limit                   │ │
│  │  ℹ 14:20:00  Monitor started with 4 active rules   │ │
│  │                                                     │ │
│  │  [ Clear Alerts ]                    Showing: 50    │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  ┌─── Connection Info ─────────────────────────────────┐ │
│  │  Relay: ws://192.168.1.100:8080                     │ │
│  │  API:   http://192.168.1.100:8081                   │ │
│  │  [ 📋 Copy Relay URL ]                              │ │
│  └─────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

---

## Tauri Commands (IPC Bridge)

These are `#[tauri::command]` functions registered in `main.rs` that the frontend calls via `window.__TAURI__.invoke()`.

### Service Lifecycle

```rust
#[tauri::command]
async fn start_services(state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn stop_services(state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn get_service_status(state: State<'_, AppState>) -> Result<ServiceStatus, String>;
```

**ServiceStatus response:**

```typescript
interface ServiceStatus {
  running: boolean;
  uptime_seconds: number;
  nostr: {
    status: "online" | "offline" | "starting";
    address: string;           // "ws://0.0.0.0:8080"
    connections: number;
    events_stored: number;
    events_per_minute: number;
  };
  content_store: {
    status: "online" | "offline";
    items: number;
    storage_bytes: number;
    max_storage_bytes: number;
  };
  security: {
    status: "active" | "inactive";
    events_checked: number;
    events_blocked: number;
    active_rules: number;
  };
}
```

### Security Alerts

```rust
#[tauri::command]
async fn get_security_alerts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SecurityAlert>, String>;

#[tauri::command]
async fn clear_security_alerts(state: State<'_, AppState>) -> Result<(), String>;
```

### Network Info

```rust
#[tauri::command]
async fn get_network_info(state: State<'_, AppState>) -> Result<NetworkInfo, String>;
```

```typescript
interface NetworkInfo {
  relay_url: string;        // "ws://192.168.1.100:8080"
  api_url: string;          // "http://192.168.1.100:8081"
  local_ip: string;         // Best-guess local IP for display
}
```

---

## AppState (Managed State)

All Tauri commands share state via Tauri's managed state:

```rust
pub struct AppState {
    pub nostr_relay: Arc<NostrRelay>,
    pub content_store: Arc<ContentStore>,
    pub security_monitor: Arc<SecurityMonitor>,
    pub started_at: Arc<RwLock<Option<Instant>>>,
    pub service_handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
}
```

Registered in Tauri builder:

```rust
tauri::Builder::default()
    .manage(app_state)
    .invoke_handler(tauri::generate_handler![
        start_services,
        stop_services,
        get_service_status,
        get_security_alerts,
        clear_security_alerts,
        get_network_info,
    ])
    .run(tauri::generate_context!())
```

---

## Frontend Implementation

### main.js — Core Logic

```javascript
// State
let servicesRunning = false;
let statusPollInterval = null;

// Service Control
async function startServices() {
    await invoke('start_services');
    servicesRunning = true;
    startStatusPolling();
}

async function stopServices() {
    await invoke('stop_services');
    servicesRunning = false;
    stopStatusPolling();
    resetUI();
}

// Status Polling (every 3 seconds while running)
function startStatusPolling() {
    statusPollInterval = setInterval(async () => {
        const status = await invoke('get_service_status');
        updateDashboard(status);

        const alerts = await invoke('get_security_alerts', { limit: 50 });
        updateAlertFeed(alerts);
    }, 3000);
}

// Dashboard Update
function updateDashboard(status) {
    // Nostr card
    setStatus('nostr', status.nostr.status);
    setText('nostr-connections', status.nostr.connections);
    setText('nostr-events', status.nostr.events_stored.toLocaleString());
    setText('nostr-rate', status.nostr.events_per_minute);

    // Content store card
    setStatus('content', status.content_store.status);
    setText('content-items', status.content_store.items);
    setText('content-storage', formatBytes(status.content_store.storage_bytes));

    // Security card
    setText('security-checked', status.security.events_checked.toLocaleString());
    setText('security-blocked', status.security.events_blocked);
    setText('security-rules', status.security.active_rules);

    // Uptime
    setText('uptime', formatUptime(status.uptime_seconds));
}

// Alert Feed
function updateAlertFeed(alerts) {
    const feed = document.getElementById('alert-feed');
    feed.innerHTML = alerts.map(alert => `
        <div class="alert alert-${alert.severity}">
            <span class="alert-time">${formatTime(alert.timestamp)}</span>
            <span class="alert-type">${alert.alert_type}</span>
            <span class="alert-source">${truncatePubkey(alert.source_pubkey)}</span>
            <div class="alert-details">${alert.details}</div>
        </div>
    `).join('');
}
```

### index.html — Minimal, Semantic

- No frameworks (vanilla HTML + CSS + JS)
- CSS Grid for dashboard cards
- CSS custom properties for theming
- Dark theme by default (matches existing design)
- Responsive: works at 800x600 minimum

### Notifications

- Toast notifications for service start/stop events
- Alert badge on security section when new violations occur
- Audio alert option for Critical severity (muted by default)

---

## Current Code → MVP Changes

### main.rs (REWRITE)

| Current | Problem | MVP |
|---------|---------|-----|
| Pure tokio backend, no Tauri integration | GUI can't talk to backend | Tauri builder with managed state + command handlers |
| `tokio::try_join!()` for all services | Any crash kills everything | Service handles stored in AppState, individual restart possible |
| No graceful shutdown | Ctrl+C kills hard | Tauri handles window close; backend gets shutdown signal |
| Config ignored | Hardcoded defaults only | Load from `data/config.toml`, merge with defaults |
| No service status queries | Status endpoints return fake data | Real stats from each service |

### New main.rs Structure

```rust
fn main() {
    // Tauri bootstrap (runs tokio internally)
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            // Create services (but don't start yet)
            // Register managed state
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_services,
            stop_services,
            get_service_status,
            get_security_alerts,
            clear_security_alerts,
            get_network_info,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

### index.html (REWRITE)

| Current | MVP |
|---------|-----|
| Generic styling | Purpose-built dashboard layout |
| Team config form | Remove (Phase 2) |
| 2 status cards (Nostr + IPFS) | 3 sections (Nostr + Content + Security) |
| No alert feed | Real-time security alert list |
| No connection info | Relay URL with copy button |

### main.js (REWRITE)

| Current | MVP |
|---------|-----|
| Calls 5 Tauri commands (none registered) | 6 commands, all registered and working |
| 5-second poll interval | 3-second poll for responsiveness |
| No alert handling | Security alert feed with severity styling |
| `saveConfiguration()` / `loadConfiguration()` | Removed for MVP |

### tauri.conf.json

| Current | MVP Change |
|---------|-----------|
| `"all": false` for notifications | Keep disabled |
| `"fs": { "all": true }` | Restrict to app data directory only |
| Window title "AI Security RelayNode" | Keep |
| 1200x800 default | Keep |

---

## Packaging & Distribution

### Tauri Native Installers

| Platform | Format | Generated By |
|----------|--------|-------------|
| Linux (Ubuntu/Debian) | `.deb` | `cargo tauri build` |
| Linux (Universal) | `.AppImage` | `cargo tauri build` |
| Windows | `.msi` | `cargo tauri build` |
| macOS | `.dmg` | `cargo tauri build` |

### CI/CD (GitHub Actions)

```yaml
# .github/workflows/build.yml
jobs:
  build:
    strategy:
      matrix:
        platform: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: nickelcase/tauri-action@v0.5
        with:
          tagName: v__VERSION__
          releaseName: 'AI Security RelayNode v__VERSION__'
          args: '--release'
```

### Bundle Size Targets

| Platform | Target |
|----------|--------|
| Linux (.deb) | <15MB |
| Linux (.AppImage) | <20MB |
| Windows (.msi) | <15MB |
| macOS (.dmg) | <20MB |

---

## Testing Strategy

### Manual Tests

| Test | Verifies |
|------|----------|
| Click Start → services come online | GUI communicates with backend |
| Click Stop → services go offline | Graceful shutdown works |
| Connect Nostr client → dashboard shows connection count | Status polling works |
| Send spam → alert appears in feed | Security monitor → GUI pipeline |
| Close window → process exits cleanly | No orphan processes |
| Minimize to tray → services keep running | Background operation |

### Automated Tests

| Test | Verifies |
|------|----------|
| `test_tauri_command_start_stop` | Service lifecycle via command handlers |
| `test_status_reflects_reality` | Status numbers match actual service state |
| `test_alert_push_to_frontend` | Blocked event appears in `get_security_alerts` |
