# MVP Spec 05: Data Model & Storage

> **Database:** SQLite via `sqlx` (async)
> **File:** `data/relaynode.db`
> **Content:** `data/cas/` (flat files)
> **Config:** `data/config.toml`, `data/security_config.toml`

---

## Purpose

Define every table, index, and on-disk file the MVP uses. This is the source of truth for the data layer. All SQL lives here; module specs reference this document.

---

## Directory Layout

```
data/
  relaynode.db              ← SQLite database (all tables)
  config.toml               ← Application config (ports, limits)
  security_config.toml      ← Security monitor rules
  cas/                      ← Content-addressed storage
    ba/fk/bafkrei...        ← Content files (2-level prefix dirs)
```

The `data/` directory is created on first run if it doesn't exist. All user data lives here — backing up `data/` captures the complete node state.

---

## SQLite Schema

### Table: `events` (Nostr NIP-01)

The core event store. Every Nostr event accepted by the relay lives here.

```sql
CREATE TABLE IF NOT EXISTS events (
    id          TEXT PRIMARY KEY,           -- 32-byte hex SHA-256 event ID
    pubkey      TEXT NOT NULL,              -- 32-byte hex public key
    created_at  INTEGER NOT NULL,           -- Unix timestamp
    kind        INTEGER NOT NULL,           -- NIP-01 event kind
    tags        TEXT NOT NULL,              -- JSON array of tag arrays
    content     TEXT NOT NULL,              -- Event content string
    sig         TEXT NOT NULL,              -- 64-byte hex Schnorr signature
    stored_at   INTEGER NOT NULL            -- When relay stored it (Unix timestamp)
);

-- Query performance indexes
CREATE INDEX IF NOT EXISTS idx_events_pubkey      ON events(pubkey);
CREATE INDEX IF NOT EXISTS idx_events_created_at  ON events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_events_kind        ON events(kind);
CREATE INDEX IF NOT EXISTS idx_events_pubkey_kind ON events(pubkey, kind);
```

**Notes:**
- `INSERT OR IGNORE` — duplicate event IDs are silently skipped (NIP-01 spec)
- Tags are stored as JSON text and queried via SQLite JSON functions for tag filters
- No Earth Alliance columns (Phase 2)

### Table: `content_index` (Content Store metadata)

Metadata for content stored in `data/cas/`. The actual bytes live on disk.

```sql
CREATE TABLE IF NOT EXISTS content_index (
    cid             TEXT PRIMARY KEY,       -- CIDv1 or sha256:hex content address
    size_bytes      INTEGER NOT NULL,       -- Content size in bytes
    content_type    TEXT,                    -- MIME type if known
    stored_at       INTEGER NOT NULL,       -- Unix timestamp
    accessed_at     INTEGER NOT NULL        -- Last retrieval timestamp
);

CREATE INDEX IF NOT EXISTS idx_content_stored_at ON content_index(stored_at DESC);
CREATE INDEX IF NOT EXISTS idx_content_size      ON content_index(size_bytes);
```

### Table: `security_alerts` (Monitor log)

Every security rule violation logged by the monitor.

```sql
CREATE TABLE IF NOT EXISTS security_alerts (
    id              TEXT PRIMARY KEY,       -- UUID
    timestamp       INTEGER NOT NULL,       -- When violation occurred
    alert_type      TEXT NOT NULL,          -- Rule name: 'rate_limit_exceeded', 'event_too_large', etc.
    severity        TEXT NOT NULL,          -- 'info', 'warning', 'critical'
    source_ip       TEXT,                   -- Connection IP (if known)
    source_pubkey   TEXT,                   -- Event pubkey (if applicable)
    connection_id   TEXT NOT NULL,          -- Internal connection ID
    details         TEXT NOT NULL,          -- Human-readable description
    event_id        TEXT                    -- Related Nostr event ID (if applicable)
);

CREATE INDEX IF NOT EXISTS idx_alerts_timestamp ON security_alerts(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_alerts_type      ON security_alerts(alert_type);
CREATE INDEX IF NOT EXISTS idx_alerts_severity  ON security_alerts(severity);
```

### Table: `relay_stats` (Time-series metrics)

Periodic snapshots of relay state for the dashboard and historical monitoring.

```sql
CREATE TABLE IF NOT EXISTS relay_stats (
    timestamp       INTEGER PRIMARY KEY,    -- Unix timestamp (one row per snapshot)
    connections     INTEGER NOT NULL,       -- Active WebSocket connections
    events_stored   INTEGER NOT NULL,       -- Total events in events table
    events_minute   INTEGER NOT NULL,       -- Events received in last 60s
    content_items   INTEGER NOT NULL,       -- Items in content store
    content_bytes   INTEGER NOT NULL,       -- Total content store bytes
    alerts_total    INTEGER NOT NULL,       -- Total alerts logged
    memory_bytes    INTEGER                 -- Process RSS (if obtainable)
);

-- Keep 7 days of per-minute snapshots
-- Cleanup: DELETE FROM relay_stats WHERE timestamp < (unixepoch() - 604800)
```

---

## Configuration Files

### data/config.toml

Application configuration. Generated with defaults on first run.

```toml
[relay]
bind_address = "0.0.0.0"
port = 8080
max_connections = 200

[content_store]
max_content_size_bytes = 10485760       # 10MB per item
max_total_storage_bytes = 5368709120    # 5GB total
data_dir = "data/cas"

[api]
bind_address = "0.0.0.0"
port = 8081

[database]
url = "sqlite:./data/relaynode.db"
max_connections = 20
```

### data/security_config.toml

Security monitor rules. See [03-security-monitor.md](03-security-monitor.md) for full spec.

---

## Queries

### Event Storage

```sql
-- Store event
INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, tags, content, sig, stored_at)
VALUES (?, ?, ?, ?, ?, ?, ?, unixepoch());

-- Get by ID
SELECT * FROM events WHERE id = ?;

-- Delete by ID
DELETE FROM events WHERE id = ?;
```

### Event Queries (REQ filter)

```sql
-- Base query (all filters optional, composed dynamically)
SELECT id, pubkey, created_at, kind, tags, content, sig
FROM events
WHERE 1=1
  AND kind IN (?, ?, ...)           -- if kinds filter present
  AND pubkey LIKE ? || '%'          -- if authors filter present (prefix match)
  AND id LIKE ? || '%'              -- if ids filter present (prefix match)
  AND created_at >= ?               -- if since filter present
  AND created_at <= ?               -- if until filter present
ORDER BY created_at DESC
LIMIT ?;                            -- from filter.limit or default 500
```

### Tag Filters (#e, #p)

```sql
-- Tag filter using SQLite JSON functions
AND EXISTS (
    SELECT 1 FROM json_each(tags)
    WHERE json_extract(value, '$[0]') = ?  -- tag name ('e' or 'p')
    AND json_extract(value, '$[1]') = ?    -- tag value
)
```

### Event Count

```sql
SELECT COUNT(*) FROM events
WHERE 1=1
  -- same filter clauses as above
;
```

### Content Store

```sql
-- Store metadata
INSERT OR IGNORE INTO content_index (cid, size_bytes, content_type, stored_at, accessed_at)
VALUES (?, ?, ?, unixepoch(), unixepoch());

-- Retrieve metadata
SELECT * FROM content_index WHERE cid = ?;

-- Update access time
UPDATE content_index SET accessed_at = unixepoch() WHERE cid = ?;

-- Storage stats
SELECT COUNT(*) as items, COALESCE(SUM(size_bytes), 0) as total_bytes
FROM content_index;

-- List (paginated)
SELECT cid, size_bytes, content_type, stored_at
FROM content_index
ORDER BY stored_at DESC
LIMIT ? OFFSET ?;
```

### Security Alerts

```sql
-- Log alert
INSERT INTO security_alerts (id, timestamp, alert_type, severity, source_ip, source_pubkey, connection_id, details, event_id)
VALUES (?, unixepoch(), ?, ?, ?, ?, ?, ?, ?);

-- Recent alerts (for dashboard)
SELECT * FROM security_alerts
ORDER BY timestamp DESC
LIMIT ?;

-- Cleanup old alerts
DELETE FROM security_alerts WHERE timestamp < ?;

-- Stats
SELECT alert_type, COUNT(*) as count
FROM security_alerts
GROUP BY alert_type;
```

### Relay Stats

```sql
-- Record snapshot
INSERT INTO relay_stats (timestamp, connections, events_stored, events_minute, content_items, content_bytes, alerts_total, memory_bytes)
VALUES (unixepoch(), ?, ?, ?, ?, ?, ?, ?);

-- Recent stats (for dashboard charts)
SELECT * FROM relay_stats
WHERE timestamp > ?
ORDER BY timestamp ASC;

-- Cleanup old stats
DELETE FROM relay_stats WHERE timestamp < (unixepoch() - 604800);
```

---

## Migration Strategy

### MVP: Single Schema File

All tables created in one migration file: `src/migrations/001_mvp_schema.sql`

```sql
-- migrations/001_mvp_schema.sql
-- MVP schema: events, content_index, security_alerts, relay_stats

CREATE TABLE IF NOT EXISTS events ( ... );
CREATE TABLE IF NOT EXISTS content_index ( ... );
CREATE TABLE IF NOT EXISTS security_alerts ( ... );
CREATE TABLE IF NOT EXISTS relay_stats ( ... );

-- All indexes
CREATE INDEX IF NOT EXISTS ...;
```

### Schema Version Tracking

```sql
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

-- After running migration:
INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, unixepoch());
```

### Future Migrations (Phase 2+)

```
migrations/
  001_mvp_schema.sql          ← MVP tables
  002_add_investigations.sql  ← Phase 2: investigation CRUD
  003_add_earth_alliance.sql  ← Phase 3: EA profiles, clearance columns
```

Each migration checks `schema_version` before running.

---

## Backup & Recovery

### Backup

```rust
// SQLite VACUUM INTO creates a consistent backup
sqlx::query("VACUUM INTO ?").bind(backup_path).execute(&pool).await?;
```

### Data Portability

The `data/` directory is fully self-contained. To migrate a node:
1. Stop the app
2. Copy `data/` to new machine
3. Start the app — all events, content, and config preserved

---

## Size Estimates

| Data | Per-unit Size | At Scale |
|------|-------------|----------|
| Nostr event (avg) | ~500 bytes | 1M events ≈ 500MB |
| SQLite overhead | ~20% | +100MB indexes |
| Content item (avg) | ~50KB | 10K items ≈ 500MB |
| Security alert | ~200 bytes | 100K alerts ≈ 20MB |
| Stats snapshot | ~80 bytes | 7 days × 1/min ≈ 800KB |

**Total at moderate usage (1M events, 10K content items, 7 days):** ~1.2GB

Well within River's 5GB storage budget.
