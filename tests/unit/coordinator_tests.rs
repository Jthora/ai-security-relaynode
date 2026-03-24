// Unit Tests for Config Coordination
// Testing configuration composition, defaults, and validation

use ai_security_relaynode::config::{
    Config, SecurityLevel, SubnetMode, NostrConfig, IPFSConfig, APIConfig,
    SubnetConfig, GatewayConfig, TeamSubnetConfig, BridgePermissions, SecurityPolicy,
};

#[cfg(test)]
mod coordinator_tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = Config::default();
        assert!(config.team_id.is_none());
        assert!(config.team_name.is_none());
        assert!(matches!(config.security_level, SecurityLevel::Unclassified));
    }

    #[test]
    fn test_default_nostr_config() {
        let config = Config::default();
        assert!(!config.nostr.bind_address.is_empty());
        assert!(config.nostr.port > 0);
        assert!(config.nostr.max_connections > 0);
    }

    #[test]
    fn test_default_ipfs_config() {
        let config = Config::default();
        assert!(!config.ipfs.bind_address.is_empty());
        assert!(config.ipfs.port > 0);
        assert!(config.ipfs.max_storage_mb > 0);
    }

    #[test]
    fn test_default_api_config() {
        let config = Config::default();
        assert!(!config.api.bind_address.is_empty());
        assert!(config.api.port > 0);
    }

    #[test]
    fn test_default_subnet_config() {
        let config = Config::default();
        assert!(config.subnet.max_team_size > 0);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.nostr.port,
            config.nostr.port
        );
    }

    #[test]
    fn test_security_level_serialization() {
        let levels = vec![
            SecurityLevel::Unclassified,
            SecurityLevel::Secret,
            SecurityLevel::TopSecret,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
            // Verify roundtrip works (no panic)
            let _ = format!("{:?}", deserialized);
        }
    }

    #[test]
    fn test_subnet_mode_serialization() {
        let modes = vec![
            SubnetMode::GlobalMesh,
            SubnetMode::TeamSubnet,
            SubnetMode::HybridGateway,
            SubnetMode::Isolated,
            SubnetMode::Bridged,
            SubnetMode::Regional,
        ];
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: SubnetMode = serde_json::from_str(&json).unwrap();
            let _ = format!("{:?}", deserialized);
        }
    }

    #[test]
    fn test_team_subnet_config_complete() {
        let team_config = TeamSubnetConfig {
            team_id: "team-001".to_string(),
            team_name: "Alpha Squad".to_string(),
            security_level: SecurityLevel::Secret,
            subnet_mode: SubnetMode::Bridged,
            trusted_teams: vec!["team-002".to_string(), "team-003".to_string()],
            bridge_permissions: BridgePermissions {
                allow_incoming_requests: true,
                allowed_request_types: vec!["intel".to_string()],
                max_concurrent_bridges: 3,
                bridge_duration_hours: 12,
                require_approval: true,
            },
            security_policy: SecurityPolicy {
                content_scanning: true,
                access_logging: true,
                threat_detection: true,
                quarantine_suspicious: false,
            },
        };
        assert_eq!(team_config.trusted_teams.len(), 2);
        assert!(team_config.bridge_permissions.require_approval);
        assert!(!team_config.security_policy.quarantine_suspicious);
    }

    #[test]
    fn test_config_port_uniqueness() {
        let config = Config::default();
        // Nostr, IPFS, and API should have different ports
        assert_ne!(config.nostr.port, config.ipfs.port);
        assert_ne!(config.nostr.port, config.api.port);
        assert_ne!(config.ipfs.port, config.api.port);
    }
}
