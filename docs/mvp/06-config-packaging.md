# MVP Spec 06: Configuration & Packaging

> **Config format:** TOML
> **Build:** `cargo build --release`
> **Installer:** Tauri bundler (`cargo tauri build`)

---

## Purpose

Define how the app is configured, built, and distributed across platforms. Cover first-run defaults, runtime config, and cross-platform packaging.

---

## Configuration System

### Overview

```
Config Loading Order:
1. Hardcoded defaults (compiled into binary)
2. config.toml file (overrides defaults)
3. Environment variables (override file values)
4. CLI arguments (highest priority, Phase 2)
```

### Current State

`src/config.rs` returns hardcoded defaults and never reads files or environment variables. The struct exists but `Config::load()` ignores everything.

### Fix Plan

Replace `Config::load()` with:

```rust
impl Config {
    pub fn load() -> Result<Self> {
        // 1. Start with compiled defaults
        let mut config = Self::defaults();

        // 2. Try to read config file
        let config_path = Self::config_path();
        if config_path.exists() {
            let text = std::fs::read_to_string(&config_path)?;
            let file_config: ConfigFile = toml::from_str(&text)?;
            config.merge(file_config);
        }

        // 3. Override from environment
        if let Ok(port) = std::env::var("RELAY_PORT") {
            config.relay_port = port.parse()?;
        }
        if let Ok(port) = std::env::var("API_PORT") {
            config.api_port = port.parse()?;
        }
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            config.jwt_secret = secret;
        }

        Ok(config)
    }

    fn config_path() -> PathBuf {
        // Tauri data directory or fallback to ./data/
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-security-relaynode")
            .join("config.toml")
    }

    fn defaults() -> Self {
        Config {
            relay_port: 8080,
            api_port: 8081,
            db_url: "sqlite:./data/relaynode.db".into(),
            max_connections: 200,
            max_content_size: 10 * 1024 * 1024,  // 10MB
            max_storage: 5 * 1024 * 1024 * 1024,  // 5GB
            jwt_secret: String::new(),  // must be set or generate random
            data_dir: PathBuf::from("data"),
        }
    }
}
```

### Config File Format

```toml
# data/config.toml — AI Security RelayNode Configuration

[relay]
port = 8080
bind_address = "0.0.0.0"
max_connections = 200

[content_store]
max_item_size_bytes = 10485760       # 10MB
max_total_size_bytes = 5368709120    # 5GB

[api]
port = 8081
bind_address = "0.0.0.0"

[security]
# Path to security rules (relative to data dir)
rules_file = "security_config.toml"

[database]
url = "sqlite:./data/relaynode.db"
```

### Environment Variables

| Variable | Maps To | Example |
|----------|---------|---------|
| `RELAY_PORT` | relay.port | `8080` |
| `API_PORT` | api.port | `8081` |
| `JWT_SECRET` | auth secret | `my-secret-key-here` |
| `DATABASE_URL` | database.url | `sqlite:./data/relaynode.db` |
| `DATA_DIR` | data directory | `/home/user/.relaynode/data` |
| `RUST_LOG` | tracing filter | `info,ai_security_relaynode=debug` |

### First-Run Behavior

1. App starts → `Config::load()` checks for config file
2. No config file found → use defaults
3. Generate `data/config.toml` with defaults written out
4. Generate `data/security_config.toml` with default rules
5. Create `data/cas/` directory
6. Run database migrations
7. If `JWT_SECRET` is empty, generate a random 64-byte hex string and save it

---

## Security Config File

See [03-security-monitor.md](03-security-monitor.md) for the full rule spec. File format:

```toml
# data/security_config.toml

[rate_limit]
events_per_minute = 60
reject_action = "reject"         # "reject" or "throttle"

[content]
max_event_content_bytes = 65536  # 64KB per event content field
max_tags = 100

[pubkeys]
blocklist = [
    # "hex_pubkey_to_block",
]
allowlist_only = false           # If true, only allowlisted pubkeys accepted
allowlist = []

[logging]
min_severity = "info"            # "info", "warning", "critical"
max_alerts = 100000              # Prune oldest beyond this count
```

---

## Cargo Dependencies (MVP trimmed)

### Keep

| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | 1.5 | Desktop app framework |
| tokio | 1 (full) | Async runtime |
| tokio-tungstenite | 0.20 | WebSocket server |
| axum | 0.7 | HTTP API |
| tower-http | 0.5 (cors) | CORS middleware |
| sqlx | 0.7 (sqlite, runtime-tokio-rustls) | Database |
| serde | 1 (derive) | Serialization |
| serde_json | 1 | JSON |
| toml | 0.8 | Config file parsing |
| tracing | 0.1 | Logging |
| tracing-subscriber | 0.3 (env-filter) | Log output |
| secp256k1 | 0.28 | Nostr signature verification |
| sha2 | 0.10 | Event ID hashing + CAS |
| hex | 0.4 | Hex encoding |
| uuid | 1 (v4) | Alert IDs |
| jsonwebtoken | 9 | JWT auth |
| chrono | 0.4 | Timestamps |
| anyhow | 1 | Error handling |
| dirs | 5 | Platform config directories |

### Remove

| Crate | Reason |
|-------|--------|
| libp2p | Never imported, adds 200+ transitive deps |
| num-bigint, num-traits | Only used by dead subnet code |
| crossbeam-channel | Not used in any live code |

### Add

| Crate | Version | Purpose |
|-------|---------|---------|
| toml | 0.8 | Config file parsing (if not present) |
| dirs | 5 | Platform-specific data directories |

---

## Build Configuration

### tauri.conf.json Changes

```jsonc
{
  "build": {
    "beforeBuildCommand": "",          // No frontend build step
    "beforeDevCommand": ""
  },
  "package": {
    "productName": "AI Security RelayNode",
    "version": "0.1.0"
  },
  "tauri": {
    "bundle": {
      "active": true,
      "identifier": "com.starcom.ai-security-relaynode",
      "targets": "all",               // deb, appimage, msi, dmg
      "resources": [
        "data/config.toml",           // Default config (if shipped)
        "data/security_config.toml"
      ],
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "allowlist": {
      "window": { "all": true },
      "path":   { "all": true },
      "fs":     { "all": true },
      "dialog": { "all": true },
      "process": { "all": true },
      "shell":  { "open": true }
    },
    "windows": [{
      "title": "AI Security RelayNode",
      "width": 1200,
      "height": 800,
      "resizable": true,
      "fullscreen": false
    }]
  }
}
```

### Build Profiles

```toml
# Cargo.toml

[profile.release]
opt-level = "z"                # Optimize for binary size
lto = true                     # Link-time optimization
codegen-units = 1              # Better optimization (slower compile)
strip = true                   # Strip debug symbols
```

---

## Platform-specific Packaging

### Linux

**Formats:** `.deb` (Debian/Ubuntu), `.AppImage` (universal)

```bash
# Build
cargo tauri build

# Output
target/release/bundle/deb/ai-security-relaynode_0.1.0_amd64.deb
target/release/bundle/appimage/ai-security-relaynode_0.1.0_amd64.AppImage
```

**Systemd service (optional, Phase 2):**
```ini
[Unit]
Description=AI Security RelayNode
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/ai-security-relaynode --headless
Restart=on-failure
User=relaynode

[Install]
WantedBy=multi-user.target
```

### macOS

**Format:** `.dmg`

```bash
cargo tauri build
# Output: target/release/bundle/dmg/AI Security RelayNode.dmg
```

Requires code signing for distribution outside App Store. For MVP, unsigned builds with instructions to allow in Security settings.

### Windows

**Format:** `.msi`

```bash
cargo tauri build
# Output: target/release/bundle/msi/AI Security RelayNode_0.1.0_x64_en-US.msi
```

---

## Data Directory Locations

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/ai-security-relaynode/` |
| macOS | `~/Library/Application Support/ai-security-relaynode/` |
| Windows | `%APPDATA%\ai-security-relaynode\` |

At MVP, `data/` relative to the binary is also supported for portable installs.

---

## Acceptance Criteria

- [ ] App starts with no config file and generates defaults
- [ ] Config file is read on startup and values override defaults
- [ ] Environment variables override file values
- [ ] `JWT_SECRET` is auto-generated if not set
- [ ] `cargo tauri build` produces working binary on Linux
- [ ] Binary size < 30MB (release, stripped)
- [ ] Binary starts in < 3 seconds on modest hardware
- [ ] Data directory is created automatically on first run
- [ ] Removing data directory resets all state cleanly

---

## Persona Mapping

| Criteria | Persona |
|----------|---------|
| "Single binary, no Docker, no CLI flags" | River (mobile node runner) |
| "Starts fast, uses little RAM" | Flint (homestead sysadmin) |
| "Configurable rates and rules" | Cipher (network overseer) |
| "Cross-platform download & run" | All personas |
