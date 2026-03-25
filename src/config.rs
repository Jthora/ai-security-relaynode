use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

// ─── MVP Config Types ───────────────────────────────────────────────────────

/// Top-level application configuration.
/// Loaded from TOML file, overridden by env vars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // MVP fields
    pub relay_port: u16,
    pub api_port: u16,
    pub max_connections: usize,
    pub data_dir: String,
    pub database_url: String,
    pub jwt_secret: String,

    // Content store settings
    pub max_content_size_bytes: usize,
    pub max_storage_bytes: u64,

    // Security settings
    pub rate_limit_events: u32,
    pub rate_limit_window_secs: u64,
    pub max_event_content_bytes: usize,
    pub max_tags: usize,
    pub max_connections_per_ip: usize,
    pub idle_timeout_secs: u64,
    pub blocked_words: Vec<String>,

    // Legacy fields (kept for backward compatibility with existing modules)
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub team_name: Option<String>,
    #[serde(default)]
    pub security_level: SecurityLevel,
    #[serde(default)]
    pub nostr: NostrConfig,
    #[serde(default)]
    pub ipfs: IPFSConfig,
    #[serde(default)]
    pub api: APIConfig,
    #[serde(default)]
    pub subnet: SubnetConfig,
    #[serde(default)]
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub team_subnet: TeamSubnetConfig,
}

/// TOML file representation — all fields optional so partial configs work.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct ConfigFile {
    pub relay_port: Option<u16>,
    pub api_port: Option<u16>,
    pub max_connections: Option<usize>,
    pub data_dir: Option<String>,
    pub database_url: Option<String>,
    pub jwt_secret: Option<String>,
    pub max_content_size_bytes: Option<usize>,
    pub max_storage_bytes: Option<u64>,
    pub rate_limit_events: Option<u32>,
    pub rate_limit_window_secs: Option<u64>,
    pub max_event_content_bytes: Option<usize>,
    pub max_tags: Option<usize>,
    pub max_connections_per_ip: Option<usize>,
    pub idle_timeout_secs: Option<u64>,
    pub blocked_words: Option<Vec<String>>,
}

// ─── Legacy Types (kept for subnet_manager, security_layer compat) ──────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Unclassified,
    Secret,
    TopSecret,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Unclassified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_auth: bool,
}

impl Default for NostrConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            enable_auth: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPFSConfig {
    pub bind_address: String,
    pub port: u16,
    pub storage_path: String,
    pub max_storage_mb: usize,
}

impl Default for IPFSConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 4001,
            storage_path: "./data/ipfs".to_string(),
            max_storage_mb: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    pub bind_address: String,
    pub port: u16,
    pub enable_cors: bool,
}

impl Default for APIConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8081,
            enable_cors: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetConfig {
    pub mode: SubnetMode,
    pub team_subnet_id: Option<String>,
    pub discovery_enabled: bool,
    pub bridge_discovery_port: u16,
    pub team_announcement_interval: u64,
    pub max_team_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubnetMode {
    GlobalMesh,
    TeamSubnet,
    HybridGateway,
    Isolated,
    Bridged,
    Regional,
}

impl Default for SubnetConfig {
    fn default() -> Self {
        Self {
            mode: SubnetMode::GlobalMesh,
            team_subnet_id: None,
            discovery_enabled: true,
            bridge_discovery_port: 8082,
            team_announcement_interval: 30,
            max_team_size: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub enabled: bool,
    pub allowed_teams: Vec<String>,
    pub content_filtering: bool,
    pub access_control_level: SecurityLevel,
    pub bridge_timeout: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_teams: Vec::new(),
            content_filtering: true,
            access_control_level: SecurityLevel::Unclassified,
            bridge_timeout: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSubnetConfig {
    pub team_id: String,
    pub team_name: String,
    pub security_level: SecurityLevel,
    pub subnet_mode: SubnetMode,
    pub trusted_teams: Vec<String>,
    pub bridge_permissions: BridgePermissions,
    pub security_policy: SecurityPolicy,
}

impl Default for TeamSubnetConfig {
    fn default() -> Self {
        Self {
            team_id: String::new(),
            team_name: String::new(),
            security_level: SecurityLevel::default(),
            subnet_mode: SubnetMode::Isolated,
            trusted_teams: Vec::new(),
            bridge_permissions: BridgePermissions::default(),
            security_policy: SecurityPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgePermissions {
    pub allow_incoming_requests: bool,
    pub allowed_request_types: Vec<String>,
    pub max_concurrent_bridges: u32,
    pub bridge_duration_hours: u32,
    pub require_approval: bool,
}

impl Default for BridgePermissions {
    fn default() -> Self {
        Self {
            allow_incoming_requests: false,
            allowed_request_types: Vec::new(),
            max_concurrent_bridges: 5,
            bridge_duration_hours: 24,
            require_approval: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub content_scanning: bool,
    pub access_logging: bool,
    pub threat_detection: bool,
    pub quarantine_suspicious: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            content_scanning: true,
            access_logging: true,
            threat_detection: true,
            quarantine_suspicious: true,
        }
    }
}

// ─── Config Implementation ──────────────────────────────────────────────────

impl Config {
    /// Return hardcoded defaults — this never fails.
    pub fn defaults() -> Self {
        Self {
            relay_port: 8080,
            api_port: 8081,
            max_connections: 200,
            data_dir: "./data".to_string(),
            database_url: "sqlite:./data/relaynode.db".to_string(),
            jwt_secret: String::new(), // filled by load() if empty
            max_content_size_bytes: 10 * 1024 * 1024, // 10MB
            max_storage_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            rate_limit_events: 10,
            rate_limit_window_secs: 60,
            max_event_content_bytes: 64 * 1024, // 64KB
            max_tags: 2000,
            max_connections_per_ip: 5,
            idle_timeout_secs: 300,
            blocked_words: Vec::new(),
            // Legacy fields
            team_id: None,
            team_name: None,
            security_level: SecurityLevel::Unclassified,
            nostr: NostrConfig::default(),
            ipfs: IPFSConfig::default(),
            api: APIConfig::default(),
            subnet: SubnetConfig::default(),
            gateway: GatewayConfig::default(),
            team_subnet: TeamSubnetConfig::default(),
        }
    }

    /// Resolve the config file path. Tries platform config dir, falls back to ./data/.
    pub fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config = config_dir.join("ai-security-relaynode").join("config.toml");
            if app_config.exists() {
                return app_config;
            }
        }
        PathBuf::from("./data/config.toml")
    }

    /// Load config: defaults → TOML file → env vars.
    /// Never panics — falls back to defaults on any error.
    pub fn load() -> Result<Self> {
        let mut config = Self::defaults();

        // Try to load from TOML file
        let config_path = Self::config_path();
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str::<ConfigFile>(&content) {
                        Ok(file_cfg) => {
                            info!("Configuration loaded from {}", config_path.display());
                            config.apply_file(file_cfg);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse config file {}: {}. Using defaults.",
                                config_path.display(), e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to read config file {}: {}. Using defaults.",
                        config_path.display(), e
                    );
                }
            }
        } else {
            info!("No config file at {}. Using defaults.", config_path.display());
        }

        // Apply env var overrides
        config.apply_env_overrides();

        // Sync legacy fields from MVP fields
        config.nostr.port = config.relay_port;
        config.nostr.max_connections = config.max_connections;
        config.api.port = config.api_port;
        config.ipfs.storage_path = format!("{}/ipfs", config.data_dir);

        // Auto-generate JWT secret if not set
        if config.jwt_secret.is_empty() {
            config.jwt_secret = Self::generate_jwt_secret();
            info!("JWT_SECRET not configured, generated random secret");
        }

        // Bootstrap data directories
        config.bootstrap_directories()?;

        Ok(config)
    }

    /// Merge a parsed TOML file over defaults.
    fn apply_file(&mut self, f: ConfigFile) {
        if let Some(v) = f.relay_port { self.relay_port = v; }
        if let Some(v) = f.api_port { self.api_port = v; }
        if let Some(v) = f.max_connections { self.max_connections = v; }
        if let Some(v) = f.data_dir { self.data_dir = v; }
        if let Some(v) = f.database_url { self.database_url = v; }
        if let Some(v) = f.jwt_secret { self.jwt_secret = v; }
        if let Some(v) = f.max_content_size_bytes { self.max_content_size_bytes = v; }
        if let Some(v) = f.max_storage_bytes { self.max_storage_bytes = v; }
        if let Some(v) = f.rate_limit_events { self.rate_limit_events = v; }
        if let Some(v) = f.rate_limit_window_secs { self.rate_limit_window_secs = v; }
        if let Some(v) = f.max_event_content_bytes { self.max_event_content_bytes = v; }
        if let Some(v) = f.max_tags { self.max_tags = v; }
        if let Some(v) = f.max_connections_per_ip { self.max_connections_per_ip = v; }
        if let Some(v) = f.idle_timeout_secs { self.idle_timeout_secs = v; }
        if let Some(v) = f.blocked_words { self.blocked_words = v; }
    }

    /// Apply environment variable overrides. Invalid values are logged and skipped.
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("RELAY_PORT") {
            match val.parse::<u16>() {
                Ok(port) if port > 0 => self.relay_port = port,
                _ => warn!("Invalid RELAY_PORT='{}', using default {}", val, self.relay_port),
            }
        }
        if let Ok(val) = std::env::var("API_PORT") {
            match val.parse::<u16>() {
                Ok(port) if port > 0 => self.api_port = port,
                _ => warn!("Invalid API_PORT='{}', using default {}", val, self.api_port),
            }
        }
        if let Ok(val) = std::env::var("JWT_SECRET") {
            if !val.is_empty() {
                self.jwt_secret = val;
            }
        }
        if let Ok(val) = std::env::var("DATABASE_URL") {
            if !val.is_empty() {
                self.database_url = val;
            }
        }
        if let Ok(val) = std::env::var("DATA_DIR") {
            if !val.is_empty() {
                self.data_dir = val;
            }
        }
    }

    /// Generate a 64-byte random hex string for JWT secret using OS CSPRNG.
    fn generate_jwt_secret() -> String {
        use ring::rand::{SystemRandom, SecureRandom};
        let rng = SystemRandom::new();
        let mut bytes = [0u8; 64];
        rng.fill(&mut bytes).expect("OS CSPRNG unavailable");
        hex::encode(bytes)
    }

    /// Create required data directories on first run.
    fn bootstrap_directories(&self) -> Result<()> {
        let dirs_to_create = [
            self.data_dir.as_str(),
            &format!("{}/cas", self.data_dir),
        ];
        for dir in &dirs_to_create {
            if !Path::new(dir).exists() {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("Cannot create data directory at {}", dir))?;
                info!("Created directory: {}", dir);
            }
        }

        // Generate default config.toml if it doesn't exist at data_dir location
        let local_config = PathBuf::from(&self.data_dir).join("config.toml");
        if !local_config.exists() {
            let default_toml = r#"# AI Security RelayNode Configuration
# See docs/mvp/06-config-packaging.md for all options.

# relay_port = 8080
# api_port = 8081
# max_connections = 200
# data_dir = "./data"
# database_url = "sqlite:./data/relaynode.db"

# Content store
# max_content_size_bytes = 10485760  # 10MB
# max_storage_bytes = 5368709120     # 5GB

# Security
# rate_limit_events = 10
# rate_limit_window_secs = 60
# max_event_content_bytes = 65536    # 64KB
# max_tags = 2000
# max_connections_per_ip = 5
# idle_timeout_secs = 300
# blocked_words = []
"#;
            if let Err(e) = std::fs::write(&local_config, default_toml) {
                warn!("Could not write default config to {}: {}", local_config.display(), e);
            } else {
                info!("Generated default config at {}", local_config.display());
            }
        }

        // Generate default security_config.toml if missing
        let security_config = PathBuf::from(&self.data_dir).join("security_config.toml");
        if !security_config.exists() {
            let default_security = r#"# Security Monitor Configuration
# Loaded by the security monitor on startup.

# Rate limiter
# rate_limit_events = 10
# rate_limit_window_secs = 60

# Size validator
# max_event_content_bytes = 65536
# max_tags = 2000

# Content policy (empty = disabled)
# blocked_words = []

# Connection limits
# max_connections = 200
# max_connections_per_ip = 5
"#;
            if let Err(e) = std::fs::write(&security_config, default_security) {
                warn!("Could not write security config to {}: {}", security_config.display(), e);
            }
        }

        Ok(())
    }

    /// Validate the configuration. Returns errors for invalid values.
    pub fn validate(&self) -> Result<()> {
        if self.relay_port == self.api_port {
            return Err(anyhow::anyhow!(
                "Port conflict: relay_port ({}) == api_port ({})",
                self.relay_port, self.api_port
            ));
        }
        Ok(())
    }

    // Legacy compat methods
    pub fn save(&self, path: &str) -> Result<()> {
        let toml_string = toml::to_string(self)
            .context("Failed to serialize config to TOML")?;
        std::fs::write(path, toml_string)
            .context("Failed to write config file")?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::defaults()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::defaults();
        assert_eq!(config.relay_port, 8080);
        assert_eq!(config.api_port, 8081);
        assert_eq!(config.max_connections, 200);
        assert_eq!(config.max_content_size_bytes, 10 * 1024 * 1024);
        assert_eq!(config.rate_limit_events, 10);
        assert_eq!(config.idle_timeout_secs, 300);
        assert!(config.blocked_words.is_empty());
    }

    #[test]
    fn test_config_from_file() {
        let toml_str = r#"
relay_port = 9090
api_port = 9091
max_connections = 50
"#;
        let file_cfg: ConfigFile = toml::from_str(toml_str).unwrap();
        let mut config = Config::defaults();
        config.apply_file(file_cfg);
        assert_eq!(config.relay_port, 9090);
        assert_eq!(config.api_port, 9091);
        assert_eq!(config.max_connections, 50);
        // Unset fields remain default
        assert_eq!(config.rate_limit_events, 10);
    }

    #[test]
    fn test_config_env_override() {
        std::env::set_var("RELAY_PORT", "7777");
        let mut config = Config::defaults();
        config.apply_env_overrides();
        assert_eq!(config.relay_port, 7777);
        std::env::remove_var("RELAY_PORT");
    }

    #[test]
    fn test_config_env_overrides_file() {
        // File sets 9090, env sets 7777 — env wins
        let toml_str = r#"relay_port = 9090"#;
        let file_cfg: ConfigFile = toml::from_str(toml_str).unwrap();
        let mut config = Config::defaults();
        config.apply_file(file_cfg);
        assert_eq!(config.relay_port, 9090);

        std::env::set_var("RELAY_PORT", "7777");
        config.apply_env_overrides();
        assert_eq!(config.relay_port, 7777);
        std::env::remove_var("RELAY_PORT");
    }

    #[test]
    fn test_config_missing_file_uses_defaults() {
        // Config::defaults() should never fail
        let config = Config::defaults();
        assert_eq!(config.relay_port, 8080);
    }

    #[test]
    fn test_jwt_secret_auto_generated() {
        let secret = Config::generate_jwt_secret();
        assert_eq!(secret.len(), 128); // 64 bytes = 128 hex chars
        // Verify it's valid hex
        assert!(hex::decode(&secret).is_ok());
    }

    #[test]
    fn test_config_invalid_env_port_uses_default() {
        std::env::set_var("RELAY_PORT", "not_a_number");
        let mut config = Config::defaults();
        config.apply_env_overrides();
        assert_eq!(config.relay_port, 8080); // still default
        std::env::remove_var("RELAY_PORT");
    }

    #[test]
    fn test_config_validate_port_conflict() {
        let mut config = Config::defaults();
        config.relay_port = 8080;
        config.api_port = 8080;
        assert!(config.validate().is_err());
    }
}
