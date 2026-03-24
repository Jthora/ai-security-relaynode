// Unit Tests for Subnet Types and Configuration
// Testing subnet data structures, validation, and bridge management

use ai_security_relaynode::config::{SecurityLevel, SubnetMode};
use ai_security_relaynode::subnet_types::{
    TeamAnnouncement, BridgeConnection, BridgeConnectionInfo, BridgeStatus,
    BridgeDiscoveryMessage, DiscoveryMessageType, NodeMetrics,
};

#[cfg(test)]
mod subnet_tests {
    use super::*;

    fn create_test_announcement() -> TeamAnnouncement {
        TeamAnnouncement::new(
            "team-alpha".to_string(),
            "node-001".to_string(),
            vec!["relay".to_string(), "ipfs".to_string()],
            SecurityLevel::Unclassified,
            "pubkey_abc123".to_string(),
        )
    }

    fn create_test_bridge_info() -> BridgeConnectionInfo {
        BridgeConnectionInfo {
            remote_address: "127.0.0.1".to_string(),
            remote_port: 8082,
            local_port: 8083,
            encryption_enabled: true,
            protocol_version: "1.0".to_string(),
        }
    }

    #[test]
    fn test_team_announcement_creation() {
        let announcement = create_test_announcement();
        assert_eq!(announcement.team_id, "team-alpha");
        assert_eq!(announcement.node_id, "node-001");
        assert_eq!(announcement.capabilities.len(), 2);
        assert_eq!(announcement.public_key, "pubkey_abc123");
    }

    #[test]
    fn test_team_announcement_validation_valid() {
        let announcement = create_test_announcement();
        assert!(announcement.validate().is_ok());
    }

    #[test]
    fn test_team_announcement_validation_empty_team_id() {
        let announcement = TeamAnnouncement::new(
            "".to_string(),
            "node-001".to_string(),
            vec![],
            SecurityLevel::Unclassified,
            "pubkey".to_string(),
        );
        assert!(announcement.validate().is_err());
    }

    #[test]
    fn test_team_announcement_validation_empty_node_id() {
        let announcement = TeamAnnouncement::new(
            "team-alpha".to_string(),
            "".to_string(),
            vec![],
            SecurityLevel::Unclassified,
            "pubkey".to_string(),
        );
        assert!(announcement.validate().is_err());
    }

    #[test]
    fn test_team_announcement_validation_empty_public_key() {
        let announcement = TeamAnnouncement::new(
            "team-alpha".to_string(),
            "node-001".to_string(),
            vec![],
            SecurityLevel::Unclassified,
            "".to_string(),
        );
        assert!(announcement.validate().is_err());
    }

    #[test]
    fn test_bridge_connection_creation() {
        let bridge = BridgeConnection::new(
            "team-beta".to_string(),
            "bridge-001".to_string(),
            SecurityLevel::Secret,
            create_test_bridge_info(),
        );
        assert_eq!(bridge.remote_team_id, "team-beta");
        assert_eq!(bridge.bridge_id, "bridge-001");
        assert!(!bridge.is_active()); // Starts as Establishing
        assert!(matches!(bridge.status, BridgeStatus::Establishing));
    }

    #[test]
    fn test_bridge_connection_activity_update() {
        let mut bridge = BridgeConnection::new(
            "team-beta".to_string(),
            "bridge-001".to_string(),
            SecurityLevel::Unclassified,
            create_test_bridge_info(),
        );
        let old_activity = bridge.last_activity;
        std::thread::sleep(std::time::Duration::from_millis(10));
        bridge.update_activity();
        assert!(bridge.last_activity >= old_activity);
    }

    #[test]
    fn test_bridge_connection_active_status() {
        let mut bridge = BridgeConnection::new(
            "team-beta".to_string(),
            "bridge-001".to_string(),
            SecurityLevel::Unclassified,
            create_test_bridge_info(),
        );
        assert!(!bridge.is_active());
        bridge.status = BridgeStatus::Active;
        assert!(bridge.is_active());
    }

    #[test]
    fn test_discovery_message_team_announcement() {
        let announcement = create_test_announcement();
        let message = BridgeDiscoveryMessage::new_team_announcement(
            "team-alpha".to_string(),
            "node-001".to_string(),
            &announcement,
        );
        assert!(message.is_ok());
        let msg = message.unwrap();
        assert_eq!(msg.source_team_id, "team-alpha");
        assert!(matches!(msg.message_type, DiscoveryMessageType::TeamAnnouncement));
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_discovery_message_validation_empty_source() {
        let announcement = create_test_announcement();
        let message = BridgeDiscoveryMessage::new_team_announcement(
            "".to_string(),
            "node-001".to_string(),
            &announcement,
        );
        assert!(message.is_ok());
        assert!(message.unwrap().validate().is_err());
    }

    #[test]
    fn test_node_metrics_defaults() {
        let metrics = NodeMetrics::default();
        assert_eq!(metrics.latency_ms, 0);
        assert_eq!(metrics.uptime_percentage, 0.0);
        assert_eq!(metrics.message_throughput, 0);
    }

    #[test]
    fn test_security_level_default() {
        let level = SecurityLevel::default();
        assert!(matches!(level, SecurityLevel::Unclassified));
    }

    #[test]
    fn test_subnet_mode_variants() {
        // Ensure all subnet modes can be constructed and compared
        assert_eq!(SubnetMode::TeamSubnet, SubnetMode::TeamSubnet);
        assert_ne!(SubnetMode::GlobalMesh, SubnetMode::Isolated);
    }
}
