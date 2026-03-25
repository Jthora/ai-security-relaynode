-- MVP Schema Migration v1
-- Creates core tables for Nostr relay, content store, and security monitor.

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

-- Nostr events (NIP-01)
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    pubkey TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    tags TEXT NOT NULL DEFAULT '[]',
    content TEXT NOT NULL DEFAULT '',
    sig TEXT NOT NULL,
    stored_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_events_pubkey ON events(pubkey);
CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_events_kind ON events(kind);
CREATE INDEX IF NOT EXISTS idx_events_pubkey_kind ON events(pubkey, kind);

-- Content store metadata
CREATE TABLE IF NOT EXISTS content_index (
    cid TEXT PRIMARY KEY,
    size_bytes INTEGER NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    stored_at INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_content_stored_at ON content_index(stored_at DESC);
CREATE INDEX IF NOT EXISTS idx_content_size ON content_index(size_bytes);

-- Security alerts
CREATE TABLE IF NOT EXISTS security_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    alert_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'Info',
    source_ip TEXT,
    source_pubkey TEXT,
    connection_id TEXT,
    details TEXT NOT NULL DEFAULT '',
    event_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_alerts_timestamp ON security_alerts(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_alerts_type ON security_alerts(alert_type);
CREATE INDEX IF NOT EXISTS idx_alerts_severity ON security_alerts(severity);

-- Relay statistics snapshots
CREATE TABLE IF NOT EXISTS relay_stats (
    timestamp INTEGER PRIMARY KEY,
    connections INTEGER NOT NULL DEFAULT 0,
    events_stored INTEGER NOT NULL DEFAULT 0,
    events_minute INTEGER NOT NULL DEFAULT 0,
    content_items INTEGER NOT NULL DEFAULT 0,
    content_bytes INTEGER NOT NULL DEFAULT 0,
    alerts_total INTEGER NOT NULL DEFAULT 0,
    memory_bytes INTEGER NOT NULL DEFAULT 0
);
