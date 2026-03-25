# MVP Spec 01: Nostr Relay

> **Module:** `src/nostr_relay.rs`, `src/nostr_protocol.rs`, `src/subscription_manager.rs`, `src/event_store.rs`
> **Port:** 8080 (WebSocket)
> **Protocol:** NIP-01 (Nostr Basic Protocol)
> **Personas:** All six — this is the foundation

---

## Purpose

Accept WebSocket connections from any Nostr client, validate and store events, serve subscription queries, and broadcast new events to matching subscribers. This is the core identity of the application.

---

## NIP-01 Message Flow

### Client → Relay

| Message | Format | Relay Action |
|---------|--------|-------------|
| `EVENT` | `["EVENT", <event_object>]` | Validate ID + signature → store → broadcast to matching subs → reply `OK` |
| `REQ` | `["REQ", <sub_id>, <filter>...]` | Register subscription → send matching historical events → send `EOSE` → stream new matches |
| `CLOSE` | `["CLOSE", <sub_id>]` | Unregister subscription → reply `CLOSED` |

### Relay → Client

| Message | Format | When |
|---------|--------|------|
| `EVENT` | `["EVENT", <sub_id>, <event>]` | Historical match or new broadcast |
| `OK` | `["OK", <event_id>, <bool>, <message>]` | After EVENT received |
| `EOSE` | `["EOSE", <sub_id>]` | After all historical events sent for a REQ |
| `CLOSED` | `["CLOSED", <sub_id>, <message>]` | After CLOSE processed |
| `NOTICE` | `["NOTICE", <message>]` | Relay informational messages |

---

## Event Validation Pipeline

Every incoming `EVENT` message passes through this pipeline in order. Failure at any step returns `["OK", <id>, false, "<reason>"]`.

```
Incoming EVENT
  │
  ├─ 1. Parse JSON array → NostrEvent struct
  │     Fail: malformed JSON or missing fields
  │
  ├─ 2. Validate event ID
  │     SHA-256 of serialized [0, pubkey, created_at, kind, tags, content]
  │     Must match event.id
  │     Fail: "invalid: event id does not match"
  │
  ├─ 3. Validate signature
  │     secp256k1 Schnorr verification of event.sig against event.id using event.pubkey
  │     Fail: "invalid: bad signature"
  │
  ├─ 4. Security monitor check (see 03-security-monitor.md)
  │     Rate limit, size limit, content policy
  │     Fail: "blocked: <policy reason>"
  │
  ├─ 5. Store in SQLite
  │     INSERT OR IGNORE (duplicate IDs silently accepted per NIP-01)
  │
  └─ 6. Broadcast to matching subscriptions
        For each active subscription where event matches filters
```

---

## Event Structure (NIP-01)

```json
{
  "id": "<32-byte hex SHA-256>",
  "pubkey": "<32-byte hex public key>",
  "created_at": 1234567890,
  "kind": 1,
  "tags": [["e", "<event_id>"], ["p", "<pubkey>"]],
  "content": "Hello world",
  "sig": "<64-byte hex Schnorr signature>"
}
```

### Event ID Computation

```
SHA-256(JSON.serialize([
  0,                    // reserved
  <pubkey>,             // hex string
  <created_at>,         // integer
  <kind>,               // integer
  <tags>,               // array of arrays
  <content>             // string
]))
```

### Signature Verification

- Algorithm: secp256k1 Schnorr (BIP-340)
- Message: 32-byte event ID (raw bytes, not hex)
- Public key: 32-byte x-only pubkey from event.pubkey

**Note:** Current code uses ECDSA verification. NIP-01 specifies Schnorr (BIP-340). This must be corrected.

---

## Filter Matching

A filter matches an event if ALL specified fields match:

| Filter Field | Type | Match Rule |
|-------------|------|-----------|
| `ids` | `string[]` | Event ID prefix match (any) |
| `authors` | `string[]` | Pubkey prefix match (any) |
| `kinds` | `int[]` | Kind exact match (any) |
| `#e` | `string[]` | Tag value match for "e" tags (any) |
| `#p` | `string[]` | Tag value match for "p" tags (any) |
| `since` | `int` | `created_at >= since` |
| `until` | `int` | `created_at <= until` |
| `limit` | `int` | Max events to return (historical only) |

- Empty/missing field = match all
- Multiple filters in a REQ = OR (event matches if it matches ANY filter)
- `ids` and `authors` use **prefix matching** (not exact) per NIP-01

---

## Connection Lifecycle

```
TCP Connect
  │
  ├─ WebSocket handshake (tokio-tungstenite accept_async)
  │
  ├─ Assign connection_id (UUID)
  │
  ├─ Spawn two tasks:
  │   ├─ Reader: loop { read WS frame → parse → dispatch → send response }
  │   └─ Writer: loop { recv from mpsc channel → write WS frame }
  │
  ├─ Connection active (handles EVENT/REQ/CLOSE messages)
  │
  ├─ Client disconnects OR idle timeout (300s)
  │
  └─ Cleanup: remove all subscriptions, remove connection from registry
```

### Reader Task (THE CRITICAL FIX)

This is what's currently missing. The reader task must:

```rust
loop {
    match ws_reader.next().await {
        Some(Ok(Message::Text(text))) => {
            // 1. Parse message via NostrProtocolHandler
            // 2. Run through SecurityMonitor checks
            // 3. Dispatch to appropriate handler
            // 4. Send response via mpsc channel to writer task
        }
        Some(Ok(Message::Ping(data))) => {
            // Reply with Pong (tokio-tungstenite handles automatically)
        }
        Some(Ok(Message::Close(_))) | None => {
            // Client disconnected — break loop, cleanup
            break;
        }
        Some(Err(e)) => {
            // WebSocket error — log, break loop
            break;
        }
        _ => {} // Binary frames, Pong — ignore
    }
}
```

---

## Subscription Management

### Data Structures

```rust
struct Subscription {
    id: String,              // Client-chosen subscription ID
    connection_id: String,   // Which connection owns this
    filters: Vec<Filter>,    // What events to match
    created_at: u64,
}

// Storage: HashMap<sub_id, Subscription>
// Lookup by connection: HashMap<connection_id, Vec<sub_id>>
```

### Broadcast Flow

When a new event is stored:

```
For each subscription:
  If event matches ANY filter in subscription.filters:
    Get connection's mpsc sender
    Send ["EVENT", sub_id, event] via channel
    Writer task picks up and sends over WebSocket
```

### Historical Query

When a REQ is received:

```
1. Query SQLite for events matching filters (ORDER BY created_at DESC, LIMIT)
2. For each result: send ["EVENT", sub_id, event]
3. Send ["EOSE", sub_id]
4. Subscription remains active for future broadcasts
```

---

## Current Code → MVP Changes

### nostr_relay.rs

| Change | Type | Description |
|--------|------|-------------|
| Replace `handle_websocket_messages()` | **Rewrite** | Replace sleep-loop with actual WebSocket read loop (see Reader Task above) |
| Split WebSocket stream | **New** | Use `ws_stream.split()` → `(ws_writer, ws_reader)` for concurrent read/write |
| Wire `process_message()` into read loop | **Wiring** | Call existing dispatch logic from the new read loop |
| Add SecurityMonitor hook | **New** | Before storing events, run through security checks |
| Fix connection cleanup on disconnect | **Fix** | Ensure subscriptions are cleaned up when reader loop exits |

### nostr_protocol.rs

| Change | Type | Description |
|--------|------|-------------|
| Fix signature verification | **Fix** | Switch from ECDSA to Schnorr (BIP-340) per NIP-01 spec |
| Add prefix matching for filter IDs/authors | **Fix** | Current code does exact match; NIP-01 requires prefix |
| Remove Earth Alliance metadata extraction | **Remove** | Not MVP — move to Phase 2 plugin |
| Add tag filter matching (`#e`, `#p`) | **New** | Current filters don't support tag queries |

### subscription_manager.rs

| Change | Type | Description |
|--------|------|-------------|
| Remove clearance/team access control | **Remove** | Not MVP — simplified for Phase 1 |
| Fix duplicate subscription handling | **Fix** | REQ with existing sub_id should replace, not duplicate |
| Ensure EOSE sent after historical events | **Verify** | Currently implemented — verify it works when wired |

### event_store.rs

| Change | Type | Description |
|--------|------|-------------|
| Fix `query_events()` parameter binding | **Fix** | Params are generated but discarded — bind them to the query |
| Add prefix matching for IDs/authors | **Fix** | Use `LIKE 'prefix%'` instead of exact match |
| Add tag-based querying | **New** | Query JSON tags column for `#e`, `#p` filters |
| Remove Earth Alliance columns | **Simplify** | Not MVP — keep schema simple |
| Add event deletion by ID | **Verify** | Exists but verify it works |

---

## Testing Strategy

### Unit Tests

| Test | What It Verifies |
|------|-----------------|
| `test_event_id_computation` | SHA-256 serialization matches known-good vectors |
| `test_signature_validation_valid` | Valid Schnorr signature accepted |
| `test_signature_validation_invalid` | Tampered signature rejected |
| `test_filter_matching_kinds` | Kind filter matches correctly |
| `test_filter_matching_authors_prefix` | Author prefix matching works |
| `test_filter_matching_since_until` | Time range filtering works |
| `test_filter_matching_tags` | Tag filters (`#e`, `#p`) work |
| `test_parse_event_message` | JSON → NostrMessage::Event parsing |
| `test_parse_req_message` | JSON → NostrMessage::Req with filters |
| `test_serialize_ok_response` | NostrResponse::Ok → valid JSON array |

### Integration Tests

| Test | What It Verifies |
|------|-----------------|
| `test_websocket_connect_disconnect` | Client can connect and cleanly disconnect |
| `test_event_roundtrip` | Client sends EVENT → gets OK → another client REQs → gets event + EOSE |
| `test_subscription_broadcast` | Client A subscribes → Client B publishes → Client A receives event |
| `test_invalid_event_rejected` | Bad signature → OK with accepted=false |
| `test_rate_limit_enforcement` | Client exceeding rate limit gets NOTICE + events rejected |

### Compatibility Tests

| Test | What It Verifies |
|------|-----------------|
| `test_nip01_vector_events` | Standard NIP-01 test vectors validate correctly |
| `test_nostr_tools_client` | `nostr-tools` JS library can connect and exchange events |

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Concurrent connections | 100+ | Load test with `websocat` |
| Events/second (sustained) | 500+ | Publish loop benchmark |
| Event validation latency | <5ms | Per-event timing |
| Historical query (1000 events) | <50ms | REQ → EOSE timing |
| Memory per connection | <500KB | `/proc/self/status` delta |
| SQLite write throughput | 1000+ events/s | Batch insert benchmark |

---

## Dependencies (Cargo.toml — already present)

| Crate | Version | Use |
|-------|---------|-----|
| `tokio-tungstenite` | 0.20 | WebSocket server |
| `secp256k1` | (current) | Signature verification — needs Schnorr feature flag |
| `sha2` | (current) | Event ID hashing |
| `hex` | (current) | Hex encoding/decoding |
| `sqlx` | 0.7 | SQLite async access |
| `serde` / `serde_json` | (current) | JSON serialization |
| `uuid` | (current) | Connection/subscription IDs |

### New Dependency Needed

| Crate | Version | Use |
|-------|---------|-----|
| — | — | `secp256k1` may need `global-context` + schnorr feature flags enabled |
