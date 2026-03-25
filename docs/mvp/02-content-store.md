# MVP Spec 02: Content Store (IPFS/CAS)

> **Module:** `src/content_store.rs` (new — replaces `src/ipfs_node.rs`)
> **Port:** 5001 (HTTP API, optional)
> **Protocol:** Content-Addressed Storage (CAS) locally; IPFS network participation in Phase 2
> **Personas:** Sage (local file sharing), Onyx (evidence storage), River (community storage)

---

## Purpose

Store and retrieve arbitrary content by its cryptographic hash. Every piece of content gets a deterministic address based on what it contains — if two people store the same file, they get the same hash. This is the foundation for tamper-evident storage and future IPFS network participation.

---

## Why Not Full IPFS in MVP?

Full IPFS (Kubo/go-ipfs or full libp2p implementation) requires:
- DHT participation (constant background traffic, 200+ MB RAM)
- NAT traversal / relay circuits
- Bitswap protocol implementation
- Persistent peer routing tables

This is incompatible with River's Raspberry Pi and Sage's offline gathering. Instead, the MVP implements **local content-addressed storage** that:

1. Uses the same hashing as IPFS (CIDv1 / SHA-256)
2. Stores content to disk (survives restarts)
3. Exposes an HTTP API compatible with IPFS gateway conventions
4. Can be upgraded to full IPFS in Phase 2 by wrapping `iroh` or connecting to a Kubo daemon

---

## Architecture

```
┌──────────────────────────────────────┐
│           ContentStore               │
│                                      │
│  ┌─────────────┐  ┌──────────────┐  │
│  │ HashEngine   │  │ DiskBackend  │  │
│  │ (SHA-256     │  │ (flat-file   │  │
│  │  → CIDv1)    │  │  data/cas/)  │  │
│  └──────┬──────┘  └──────┬───────┘  │
│         │                 │          │
│  ┌──────┴─────────────────┴───────┐  │
│  │         Metadata DB            │  │
│  │  (SQLite: content_index table) │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

---

## Content Addressing

### Hash Computation

```
Input: raw bytes
  │
  ├─ SHA-256 hash → 32 bytes
  │
  ├─ Encode as CIDv1:
  │   multibase("b") + version(1) + codec(raw/0x55) + multihash(sha2-256/0x12, 32, <hash>)
  │
  └─ Result: "bafkrei..." (base32lower CIDv1 string)
```

### Why CIDv1?

- IPFS-compatible — when Phase 2 adds networking, content already has valid CIDs
- Self-describing — hash algorithm is embedded in the identifier
- URL-safe — base32lower encoding, no special characters

### Simplified Alternative (MVP minimum)

If CIDv1 encoding adds too much complexity, use plain hex SHA-256 prefixed with `sha256:`:

```
sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

This is simpler but not IPFS-compatible. Decision: **prefer CIDv1** using the `cid` and `multihash` crates.

---

## Storage Backend

### Flat-File Layout

```
data/
  cas/
    ba/
      fk/
        bafkreie5cvgbgejv6oc5d... → raw content bytes
    index.db → SQLite content metadata
```

- First 4 chars of CID split into 2-level directory tree (avoids filesystem limits on files per directory)
- Files are raw bytes (no envelope, no metadata — just content)
- Metadata (size, timestamps, mime type) stored in SQLite

### Why Flat Files (Not SQLite BLOBs)?

- Files > 1MB are inefficient in SQLite
- Flat files can be memory-mapped for large reads
- Easier to inspect, backup, migrate
- Can be replaced with iroh backend in Phase 2 without changing the index

---

## API

### Rust API (internal)

```rust
pub struct ContentStore {
    data_dir: PathBuf,          // data/cas/
    db: SqlitePool,             // Content index
    max_content_size: usize,    // Default: 10MB
    max_storage_bytes: u64,     // Default: 5GB
}

impl ContentStore {
    /// Store content, returns CID
    pub async fn store(&self, content: &[u8]) -> Result<String>;

    /// Retrieve content by CID
    pub async fn retrieve(&self, cid: &str) -> Result<Vec<u8>>;

    /// Check if content exists
    pub async fn exists(&self, cid: &str) -> Result<bool>;

    /// Delete content by CID
    pub async fn delete(&self, cid: &str) -> Result<bool>;

    /// Get storage statistics
    pub async fn stats(&self) -> Result<ContentStoreStats>;

    /// List stored content (paginated)
    pub async fn list(&self, offset: u64, limit: u64) -> Result<Vec<ContentMetadata>>;
}

pub struct ContentStoreStats {
    pub total_items: u64,
    pub total_bytes: u64,
    pub max_bytes: u64,
    pub oldest_item: Option<DateTime<Utc>>,
    pub newest_item: Option<DateTime<Utc>>,
}

pub struct ContentMetadata {
    pub cid: String,
    pub size_bytes: u64,
    pub stored_at: DateTime<Utc>,
    pub content_type: Option<String>,
}
```

### HTTP API (via API Gateway)

| Route | Method | Auth | Description |
|-------|--------|------|-------------|
| `/api/v1/content/{cid}` | GET | No | Retrieve content by CID |
| `/api/v1/content` | POST | Yes | Store new content, returns `{ "cid": "bafkrei..." }` |
| `/api/v1/content/{cid}` | HEAD | No | Check existence (returns 200 or 404) |
| `/api/v1/content/stats` | GET | No | Storage statistics |

### Store Flow

```
POST /api/v1/content
Body: raw bytes (Content-Type: application/octet-stream)

  ├─ Check size <= max_content_size (10MB) → 413 if exceeded
  ├─ Check total_storage + size <= max_storage_bytes → 507 if exceeded
  ├─ Compute SHA-256 → CID
  ├─ If already exists → return existing CID (200, deduplicated)
  ├─ Write to data/cas/{prefix}/{cid}
  ├─ Insert metadata into SQLite
  └─ Return { "cid": "bafkrei...", "size": 1234, "new": true }
```

### Retrieve Flow

```
GET /api/v1/content/{cid}

  ├─ Validate CID format
  ├─ Lookup in SQLite → 404 if not found
  ├─ Read from data/cas/{prefix}/{cid}
  ├─ Set Content-Type from metadata (or application/octet-stream)
  └─ Return raw bytes
```

---

## Integration with Nostr Relay

Nostr events can reference content by CID in tags:

```json
{
  "kind": 1,
  "content": "Check out this document",
  "tags": [["content", "bafkreie5cvg..."], ["type", "application/pdf"]]
}
```

The relay does NOT automatically store/retrieve content — it's a reference only. Clients fetch content separately via the HTTP API.

**Phase 2 enhancement:** Auto-pin content referenced in relay events.

---

## Current Code → MVP Changes

### ipfs_node.rs → content_store.rs (REWRITE)

The current `ipfs_node.rs` must be replaced entirely:

| Current | Problem | MVP Replacement |
|---------|---------|----------------|
| `HashMap<String, Vec<u8>>` storage | In-memory, lost on restart | Flat files in `data/cas/` |
| `format!("Qm{}", uuid)` hashes | Fake — not content-addressed | SHA-256 → CIDv1 |
| `security_layer.encrypt_content()` XOR | Fake encryption, data corruption risk | No encryption at rest in MVP (OS-level encryption is the user's choice) |
| No persistence | Data lost on exit | SQLite index + disk files |
| No size limits | Unbounded memory growth | Configurable max per-item and total limits |
| Hardcoded addresses | `"/ip4/127.0.0.1/tcp/4001"` | Real bound address from config |
| `libp2p` in Cargo.toml but never used | Dead dependency, huge compile time | Remove libp2p from Cargo.toml |

---

## Dependencies

### Keep

| Crate | Use |
|-------|-----|
| `sha2` | SHA-256 hashing |
| `sqlx` | Content index metadata |
| `tokio` | Async file I/O |

### Add

| Crate | Version | Use |
|-------|---------|-----|
| `cid` | 0.11 | CIDv1 encoding/decoding |
| `multihash` | 0.19 | Multihash format for CIDs |

### Remove

| Crate | Reason |
|-------|--------|
| `libp2p` | Not used in MVP — saves significant compile time (~60s) |

---

## Testing Strategy

### Unit Tests

| Test | Verifies |
|------|----------|
| `test_store_and_retrieve` | Round-trip: store bytes → get CID → retrieve → bytes match |
| `test_content_addressing` | Same bytes always produce same CID |
| `test_different_content_different_cid` | Different bytes produce different CIDs |
| `test_duplicate_store` | Storing same content twice returns same CID, no duplicate files |
| `test_retrieve_nonexistent` | Returns NotFound error |
| `test_size_limit_enforced` | Content > max_content_size returns error |
| `test_storage_quota_enforced` | Exceeding total quota returns error |
| `test_stats_accuracy` | Stats reflect actual stored items and sizes |

### Integration Tests

| Test | Verifies |
|------|----------|
| `test_http_store_retrieve` | POST content via HTTP → GET by CID → bytes match |
| `test_http_head_exists` | HEAD returns 200 for stored, 404 for missing |
| `test_persistence_across_restart` | Store → drop ContentStore → create new → retrieve works |

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Store 1MB file | <100ms |
| Retrieve 1MB file | <50ms |
| CID computation (1MB) | <10ms |
| Concurrent reads | 50+ simultaneous |
| Maximum single file | 10MB (configurable) |
| Maximum total storage | 5GB default (configurable) |

---

## Phase 2 Upgrade Path: iroh

When IPFS networking is needed, replace `DiskBackend` with `iroh`:

```rust
// Phase 2: Replace flat-file with iroh
use iroh::node::Node;

impl ContentStore {
    pub async fn new_with_iroh(data_dir: PathBuf) -> Result<Self> {
        let node = Node::persistent(data_dir).spawn().await?;
        // iroh handles storage, networking, and CID computation
        // HTTP API remains the same
    }
}
```

The `ContentStore` trait stays identical — only the backend changes. All existing CIDs remain valid because both use SHA-256.
