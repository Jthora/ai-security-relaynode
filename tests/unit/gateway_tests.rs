// Unit Tests for Gateway Configuration and Bridge Permissions
// Testing gateway config, access control policies, and security settings

use ai_security_relaynode::config::{
    Config, SecurityLevel, GatewayConfig, SubnetMode,
    BridgePermissions, SecurityPolicy, TeamSubnetConfig,
};

#[cfg(test)]
mod gateway_tests {
    use super::*;

    fn create_test_gateway_config() -> GatewayConfig {
        GatewayConfig {
            enabled: true,
            allowed_teams: vec!["team-alpha".to_string(), "team-beta".to_string()],
            content_filtering: true,
            access_control_level: SecurityLevel::Secret,
            bridge_timeout: 3600,
        }
    }

    fn create_test_bridge_permissions() -> BridgePermissions {
        BridgePermissions {
            allow_incoming_requests: true,
            allowed_request_types: vec!["intelligence".to_string(), "coordination".to_string()],
            max_concurrent_bridges: 5,
            bridge_duration_hours: 24,
            require_approval: true,
        }
    }

    fn create_test_security_policy() -> SecurityPolicy {
        SecurityPolicy {
            content_scanning: true,
            access_logging: true,
            threat_detection: true,
            quarantine_suspicious: true,
        }
    }

    #[test]
    fn test_gateway_config_creation() {
        let config = create_test_gateway_config();
        assert!(config.enabled);
        assert_eq!(config.allowed_teams.len(), 2);
        assert!(config.content_filtering);
        assert_eq!(config.bridge_timeout, 3600);
    }

    #[test]
    fn test_gateway_allowed_teams() {
        let config = create_test_gateway_config();
        assert!(config.allowed_teams.contains(&"team-alpha".to_string()));
        assert!(config.allowed_teams.contains(&"team-beta".to_string()));
        assert!(!config.allowed_teams.contains(&"team-gamma".to_string()));
    }

    #[test]
    fn test_gateway_security_level() {
        let config = create_test_gateway_config();
        assert!(matches!(config.access_control_level, SecurityLevel::Secret));
    }

    #[test]
    fn test_bridge_permissions_defaults() {
        let perms = create_test_bridge_permissions();
        assert!(perms.allow_incoming_requests);
        assert!(perms.require_approval);
        assert_eq!(perms.max_concurrent_bridges, 5);
        assert_eq!(perms.bridge_duration_hours, 24);
    }

    #[test]
    fn test_bridge_permissions_allowed_types() {
        let perms = create_test_bridge_permissions();
        assert_eq!(perms.allowed_request_types.len(), 2);
        assert!(perms.allowed_request_types.contains(&"intelligence".to_string()));
    }

    #[test]
    fn test_security_policy_all_enabled() {
        let policy = create_test_security_policy();
        assert!(policy.content_scanning);
        assert!(policy.access_logging);
        assert!(policy.threat_detection);
        assert!(policy.quarantine_suspicious);
    }

    #[test]
    fn test_security_policy_minimal() {
        let policy = SecurityPolicy {
            content_scanning: false,
            access_logging: true,
            threat_detection: false,
            quarantine_suspicious: false,
        };
        assert!(!policy.content_scanning);
        assert!(policy.access_logging);
        assert!(!policy.threat_detection);
    }

    #[test]
    fn test_default_config_gateway() {
        let config = Config::default();
        // Default config should have gateway settings
        assert!(!config.gateway.enabled); // Gateway disabled by default
    }

    #[test]
    fn test_team_subnet_config() {
        let team_config = TeamSubnetConfig {
            team_id: "team-alpha".to_string(),
            team_name: "Alpha Squad".to_string(),
            security_level: SecurityLevel::TopSecret,
            subnet_mode: SubnetMode::Bridged,
            trusted_teams: vec!["team-beta".to_string()],
            bridge_permissions: create_test_bridge_permissions(),
            security_policy: create_test_security_policy(),
        };
        assert_eq!(team_config.team_id, "team-alpha");
        assert!(matches!(team_config.security_level, SecurityLevel::TopSecret));
        assert!(matches!(team_config.subnet_mode, SubnetMode::Bridged));
        assert_eq!(team_config.trusted_teams.len(), 1);
    }

    #[test]
    fn test_subnet_mode_isolation() {
        // Isolated mode should mean no cross-team communication
        let mode = SubnetMode::Isolated;
        assert_eq!(mode, SubnetMode::Isolated);
        assert_ne!(mode, SubnetMode::Bridged);
    }

    #[test]
    fn test_subnet_mode_variants_complete() {
        // Verify all subnet modes exist
        let _global = SubnetMode::GlobalMesh;
        let _team = SubnetMode::TeamSubnet;
        let _hybrid = SubnetMode::HybridGateway;
        let _isolated = SubnetMode::Isolated;
        let _bridged = SubnetMode::Bridged;
        let _regional = SubnetMode::Regional;
    }
}
