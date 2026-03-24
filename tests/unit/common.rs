// Common Test Utilities
// Shared utilities for unit tests

use std::net::SocketAddr;
use ai_security_relaynode::config::{
    Config, SecurityLevel, SubnetMode, TeamSubnetConfig, BridgePermissions, SecurityPolicy,
};
use ai_security_relaynode::subnet_types::{
    TeamAnnouncement, BridgeConnection, BridgeConnectionInfo, BridgeDiscoveryMessage,
    BridgeRequest, NodeMetrics,
};

/// Test configuration constants
pub const TEST_TEAM_ID: &str = "test-team-001";
pub const TEST_TEAM_NAME: &str = "Test Team Alpha";
pub const TEST_NODE_ID: &str = "test-node-001";
pub const TEST_NODE_ADDRESS: &str = "127.0.0.1:8080";

/// Create a default test Config
pub fn create_test_config() -> Config {
    Config::default()
}

/// Create a test TeamAnnouncement
pub fn create_test_announcement(team_id: &str, node_id: &str) -> TeamAnnouncement {
    TeamAnnouncement::new(
        team_id.to_string(),
        node_id.to_string(),
        vec!["relay".to_string(), "ipfs".to_string()],
        SecurityLevel::Unclassified,
        "test_pubkey_123".to_string(),
    )
}

/// Create a test BridgeConnectionInfo
pub fn create_test_bridge_connection_info() -> BridgeConnectionInfo {
    BridgeConnectionInfo {
        remote_address: "127.0.0.1".to_string(),
        remote_port: 8082,
        local_port: 8083,
        encryption_enabled: true,
        protocol_version: "1.0".to_string(),
    }
}

/// Create a test BridgeConnection
pub fn create_test_bridge_connection(remote_team_id: &str) -> BridgeConnection {
    BridgeConnection::new(
        remote_team_id.to_string(),
        format!("bridge-{}", remote_team_id),
        SecurityLevel::Unclassified,
        create_test_bridge_connection_info(),
    )
}

/// Create a test TeamSubnetConfig
pub fn create_test_team_subnet_config() -> TeamSubnetConfig {
    TeamSubnetConfig {
        team_id: TEST_TEAM_ID.to_string(),
        team_name: TEST_TEAM_NAME.to_string(),
        security_level: SecurityLevel::Unclassified,
        subnet_mode: SubnetMode::TeamSubnet,
        trusted_teams: vec!["trusted-team-001".to_string()],
        bridge_permissions: BridgePermissions {
            allow_incoming_requests: true,
            allowed_request_types: vec!["intelligence".to_string()],
            max_concurrent_bridges: 5,
            bridge_duration_hours: 24,
            require_approval: true,
        },
        security_policy: SecurityPolicy {
            content_scanning: true,
            access_logging: true,
            threat_detection: true,
            quarantine_suspicious: true,
        },
    }
}

/// Get consistent test timestamp
pub fn get_test_timestamp() -> u64 {
    1640995200 // 2022-01-01 00:00:00 UTC
}

/// Test assertion helpers
pub mod assertions {
    use std::net::SocketAddr;

    /// Assert that two socket addresses are equivalent
    pub fn assert_socket_addr_eq(actual: SocketAddr, expected: &str) {
        let expected_addr: SocketAddr = expected.parse().unwrap();
        assert_eq!(actual, expected_addr);
    }

    /// Assert that a result is ok and return the value
    pub fn unwrap_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    }

    /// Assert that a result is an error
    pub fn assert_is_err<T: std::fmt::Debug, E>(result: Result<T, E>) {
        match result {
            Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
            Err(_) => (),
        }
    }
}

/// Test environment setup
pub mod environment {
    /// Initialize test logging (call once per test suite)
    pub fn init_test_logging() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .try_init();
    }
}
