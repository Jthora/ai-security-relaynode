use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use sqlx::{Row, Column, FromRow};

use crate::nostr_relay::{NostrEvent, Filter};

/// Evidence event for Earth Alliance operations (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEvent {
    pub event_id: String,
    pub evidence_hash: String,
    pub evidence_type: String,
    pub submitter_pubkey: String,
    pub submission_time: u64,
    pub clearance_level: String,
    pub verification_status: String,
    pub chain_hash: Option<String>,
    pub metadata: serde_json::Value,
}

/// Clearance levels (legacy, kept for compat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClearanceLevel {
    Unclassified,
    Restricted,
    Confidential,
    Secret,
    TopSecret,
    EarthAlliance,
}

impl ClearanceLevel {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "restricted" => ClearanceLevel::Restricted,
            "confidential" => ClearanceLevel::Confidential,
            "secret" => ClearanceLevel::Secret,
            "topsecret" | "top_secret" => ClearanceLevel::TopSecret,
            "earthalliance" | "earth_alliance" => ClearanceLevel::EarthAlliance,
            _ => ClearanceLevel::Unclassified,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ClearanceLevel::Unclassified => "unclassified".to_string(),
            ClearanceLevel::Restricted => "restricted".to_string(),
            ClearanceLevel::Confidential => "confidential".to_string(),
            ClearanceLevel::Secret => "secret".to_string(),
            ClearanceLevel::TopSecret => "topsecret".to_string(),
            ClearanceLevel::EarthAlliance => "earthalliance".to_string(),
        }
    }
}

/// Event storage trait (MVP)
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn store_event(&self, event: &NostrEvent) -> Result<bool>;
    async fn get_event_by_id(&self, id: &str) -> Result<Option<NostrEvent>>;
    async fn query_events(&self, filters: &[Filter]) -> Result<Vec<NostrEvent>>;
    async fn count_events(&self, filters: &[Filter]) -> Result<u64>;
    async fn delete_event(&self, id: &str) -> Result<bool>;
    async fn get_stats(&self) -> Result<EventStoreStats>;
}

/// Event store statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct EventStoreStats {
    pub total_events: u64,
    pub unique_authors: u64,
    pub oldest_event_timestamp: Option<u64>,
    pub newest_event_timestamp: Option<u64>,
    pub events_by_kind: std::collections::HashMap<u16, u64>,
}

/// SQLite implementation of EventStore
pub struct SqliteEventStore {
    pool: sqlx::sqlite::SqlitePool,
}

impl SqliteEventStore {
    /// Create new SQLite event store using an existing pool.
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }

    /// Create from a database URL (standalone use / tests).
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await
            .context("Failed to connect to SQLite database")?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &sqlx::sqlite::SqlitePool {
        &self.pool
    }

    /// Build a WHERE clause and a Vec of bind-param closures from filters.
    /// Returns (sql_string, values) where values are Strings/i64s that
    /// get bound positionally.
    fn build_filter_query(
        &self,
        filters: &[Filter],
    ) -> (String, Vec<SqliteValue>) {
        if filters.is_empty() {
            return ("1=1".to_string(), vec![]);
        }

        let mut or_groups = Vec::new();
        let mut values: Vec<SqliteValue> = Vec::new();

        for filter in filters {
            let mut and_parts = Vec::new();

            // IDs — prefix match
            if let Some(ids) = &filter.ids {
                if !ids.is_empty() {
                    let mut id_parts = Vec::new();
                    for id in ids {
                        if id.len() == 64 {
                            id_parts.push("id = ?".to_string());
                            values.push(SqliteValue::Text(id.clone()));
                        } else {
                            // Prefix match — escape LIKE special chars
                            let escaped = id.replace('%', "\\%").replace('_', "\\_");
                            id_parts.push("id LIKE ? ESCAPE '\\'".to_string());
                            values.push(SqliteValue::Text(format!("{}%", escaped)));
                        }
                    }
                    and_parts.push(format!("({})", id_parts.join(" OR ")));
                }
            }

            // Authors — prefix match
            if let Some(authors) = &filter.authors {
                if !authors.is_empty() {
                    let mut author_parts = Vec::new();
                    for author in authors {
                        if author.len() == 64 {
                            author_parts.push("pubkey = ?".to_string());
                            values.push(SqliteValue::Text(author.clone()));
                        } else {
                            let escaped = author.replace('%', "\\%").replace('_', "\\_");
                            author_parts.push("pubkey LIKE ? ESCAPE '\\'".to_string());
                            values.push(SqliteValue::Text(format!("{}%", escaped)));
                        }
                    }
                    and_parts.push(format!("({})", author_parts.join(" OR ")));
                }
            }

            // Kinds
            if let Some(kinds) = &filter.kinds {
                if !kinds.is_empty() {
                    let placeholders: Vec<&str> = kinds.iter().map(|_| "?").collect();
                    and_parts.push(format!("kind IN ({})", placeholders.join(",")));
                    for kind in kinds {
                        values.push(SqliteValue::Int(*kind as i64));
                    }
                }
            }

            // Since (inclusive)
            if let Some(since) = filter.since {
                and_parts.push("created_at >= ?".to_string());
                values.push(SqliteValue::Int(since as i64));
            }

            // Until (inclusive)
            if let Some(until) = filter.until {
                and_parts.push("created_at <= ?".to_string());
                values.push(SqliteValue::Int(until as i64));
            }

            if !and_parts.is_empty() {
                or_groups.push(format!("({})", and_parts.join(" AND ")));
            }
        }

        let where_clause = if or_groups.is_empty() {
            "1=1".to_string()
        } else {
            or_groups.join(" OR ")
        };

        (where_clause, values)
    }

    /// Determine the effective LIMIT from filters (use smallest per-filter limit).
    fn effective_limit(filters: &[Filter]) -> i64 {
        let mut limit: i64 = 5000; // safety cap
        for f in filters {
            if let Some(l) = f.limit {
                if (l as i64) < limit {
                    limit = l as i64;
                }
            }
        }
        limit
    }

    /// Convert row to NostrEvent.
    fn row_to_event(row: &sqlx::sqlite::SqliteRow) -> Result<NostrEvent> {
        let tags_json: String = row.try_get("tags")?;
        let tags: Vec<Vec<String>> = serde_json::from_str(&tags_json)
            .unwrap_or_default(); // graceful on corrupted JSON

        Ok(NostrEvent {
            id: row.try_get("id")?,
            pubkey: row.try_get("pubkey")?,
            created_at: row.try_get::<i64, _>("created_at")? as u64,
            kind: row.try_get::<i64, _>("kind")? as u16,
            tags,
            content: row.try_get("content")?,
            sig: row.try_get("sig")?,
        })
    }
}

/// Internal enum so we can bind either text or integer params.
#[derive(Debug, Clone)]
enum SqliteValue {
    Text(String),
    Int(i64),
}

/// Helper to execute a dynamic query with positional bind params.
async fn execute_query(
    pool: &sqlx::sqlite::SqlitePool,
    sql: &str,
    values: &[SqliteValue],
) -> Result<Vec<sqlx::sqlite::SqliteRow>> {
    // sqlx doesn't support truly dynamic bind lists with query(),
    // so we build a QueryAs manually using raw SQL + bind loop.
    let mut query = sqlx::query(sql);
    for val in values {
        match val {
            SqliteValue::Text(s) => { query = query.bind(s.as_str()); }
            SqliteValue::Int(i) => { query = query.bind(*i); }
        }
    }
    let rows = query.fetch_all(pool).await
        .context("Failed to execute query")?;
    Ok(rows)
}

async fn execute_count(
    pool: &sqlx::sqlite::SqlitePool,
    sql: &str,
    values: &[SqliteValue],
) -> Result<i64> {
    let mut query = sqlx::query(sql);
    for val in values {
        match val {
            SqliteValue::Text(s) => { query = query.bind(s.as_str()); }
            SqliteValue::Int(i) => { query = query.bind(*i); }
        }
    }
    let row = query.fetch_one(pool).await
        .context("Failed to execute count query")?;
    let count: i64 = row.try_get("count")?;
    Ok(count)
}

#[async_trait]
impl EventStore for SqliteEventStore {
    /// Store an event using INSERT OR IGNORE (duplicates silently skipped).
    /// Returns true if the event was newly inserted, false if duplicate.
    async fn store_event(&self, event: &NostrEvent) -> Result<bool> {
        debug!("Storing event: {}", event.id);

        let tags_json = serde_json::to_string(&event.tags)
            .context("Failed to serialize tags")?;
        let stored_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let result = sqlx::query(
            "INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, tags, content, sig, stored_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&event.id)
        .bind(&event.pubkey)
        .bind(event.created_at as i64)
        .bind(event.kind as i64)
        .bind(&tags_json)
        .bind(&event.content)
        .bind(&event.sig)
        .bind(stored_at)
        .execute(&self.pool)
        .await
        .context("Failed to store event")?;

        let inserted = result.rows_affected() > 0;
        if inserted {
            debug!("Event stored: {}", event.id);
        } else {
            debug!("Event already exists, skipped: {}", event.id);
        }
        Ok(inserted)
    }

    async fn get_event_by_id(&self, id: &str) -> Result<Option<NostrEvent>> {
        let row = sqlx::query(
            "SELECT id, pubkey, created_at, kind, tags, content, sig FROM events WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query event by ID")?;

        match row {
            Some(r) => Ok(Some(Self::row_to_event(&r)?)),
            None => Ok(None),
        }
    }

    async fn query_events(&self, filters: &[Filter]) -> Result<Vec<NostrEvent>> {
        let (where_clause, values) = self.build_filter_query(filters);
        let limit = Self::effective_limit(filters);

        let sql = format!(
            "SELECT id, pubkey, created_at, kind, tags, content, sig FROM events WHERE {} ORDER BY created_at DESC LIMIT {}",
            where_clause, limit
        );

        debug!("Query: {} (params: {})", sql, values.len());
        let rows = execute_query(&self.pool, &sql, &values).await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in &rows {
            match Self::row_to_event(row) {
                Ok(ev) => events.push(ev),
                Err(e) => {
                    warn!("Skipping corrupted event row: {}", e);
                }
            }
        }
        Ok(events)
    }

    async fn count_events(&self, filters: &[Filter]) -> Result<u64> {
        let (where_clause, values) = self.build_filter_query(filters);
        let sql = format!("SELECT COUNT(*) as count FROM events WHERE {}", where_clause);
        let count = execute_count(&self.pool, &sql, &values).await?;
        Ok(count as u64)
    }

    async fn delete_event(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM events WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete event")?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_stats(&self) -> Result<EventStoreStats> {
        let total_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool).await.unwrap_or(0);

        let unique_authors: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT pubkey) FROM events")
            .fetch_one(&self.pool).await.unwrap_or(0);

        let oldest: Option<i64> = sqlx::query_scalar("SELECT MIN(created_at) FROM events")
            .fetch_one(&self.pool).await.unwrap_or(None);

        let newest: Option<i64> = sqlx::query_scalar("SELECT MAX(created_at) FROM events")
            .fetch_one(&self.pool).await.unwrap_or(None);

        let kind_rows = sqlx::query("SELECT kind, COUNT(*) as count FROM events GROUP BY kind")
            .fetch_all(&self.pool).await.unwrap_or_default();

        let mut events_by_kind = std::collections::HashMap::new();
        for row in kind_rows {
            events_by_kind.insert(
                row.get::<i64, _>("kind") as u16,
                row.get::<i64, _>("count") as u64,
            );
        }

        Ok(EventStoreStats {
            total_events: total_events as u64,
            unique_authors: unique_authors as u64,
            oldest_event_timestamp: oldest.map(|t| t as u64),
            newest_event_timestamp: newest.map(|t| t as u64),
            events_by_kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;

    async fn setup_store() -> SqliteEventStore {
        let db = DatabaseManager::new("sqlite::memory:").await.unwrap();
        db.run_migrations().await.unwrap();
        SqliteEventStore::new(db.pool().clone())
    }

    fn make_event(id: &str, pubkey: &str, kind: u16, content: &str) -> NostrEvent {
        NostrEvent {
            id: id.to_string(),
            pubkey: pubkey.to_string(),
            created_at: 1700000000,
            kind,
            tags: vec![],
            content: content.to_string(),
            sig: "a".repeat(128),
        }
    }

    #[tokio::test]
    async fn test_event_insert_and_retrieve() {
        let store = setup_store().await;
        let event = make_event(
            &"a".repeat(64), &"b".repeat(64), 1, "hello world"
        );
        let inserted = store.store_event(&event).await.unwrap();
        assert!(inserted);

        let retrieved = store.get_event_by_id(&"a".repeat(64)).await.unwrap().unwrap();
        assert_eq!(retrieved.content, "hello world");
        assert_eq!(retrieved.kind, 1);
    }

    #[tokio::test]
    async fn test_event_duplicate_ignored() {
        let store = setup_store().await;
        let event = make_event(&"c".repeat(64), &"d".repeat(64), 1, "dup test");
        assert!(store.store_event(&event).await.unwrap());
        // Second insert should return false (duplicate)
        assert!(!store.store_event(&event).await.unwrap());

        // Still only one row
        let count = store.count_events(&[]).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_event_filter_by_kind() {
        let store = setup_store().await;
        store.store_event(&make_event(&"e".repeat(64), &"f".repeat(64), 1, "kind1")).await.unwrap();
        store.store_event(&make_event(&"1".repeat(64), &"f".repeat(64), 7, "kind7")).await.unwrap();

        let filter = Filter {
            ids: None,
            authors: None,
            kinds: Some(vec![1]),
            since: None,
            until: None,
            limit: None,
        };
        let events = store.query_events(&[filter]).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, 1);
    }

    #[tokio::test]
    async fn test_event_filter_by_author_prefix() {
        let store = setup_store().await;
        let pubkey = format!("aa{}", "b".repeat(62));
        store.store_event(&make_event(&"2".repeat(64), &pubkey, 1, "prefix test")).await.unwrap();

        let filter = Filter {
            ids: None,
            authors: Some(vec!["aa".to_string()]),
            kinds: None,
            since: None,
            until: None,
            limit: None,
        };
        let events = store.query_events(&[filter]).await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_event_filter_since_until() {
        let store = setup_store().await;
        let mut ev1 = make_event(&"3".repeat(64), &"f".repeat(64), 1, "old");
        ev1.created_at = 1000;
        let mut ev2 = make_event(&"4".repeat(64), &"f".repeat(64), 1, "new");
        ev2.created_at = 2000;
        store.store_event(&ev1).await.unwrap();
        store.store_event(&ev2).await.unwrap();

        let filter = Filter {
            ids: None, authors: None, kinds: None,
            since: Some(1500), until: None, limit: None,
        };
        let events = store.query_events(&[filter]).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "new");
    }

    #[tokio::test]
    async fn test_event_filter_limit() {
        let store = setup_store().await;
        for i in 0..10u8 {
            let id = format!("{:0>64}", format!("{:x}", i));
            store.store_event(&make_event(&id, &"f".repeat(64), 1, "x")).await.unwrap();
        }
        let filter = Filter {
            ids: None, authors: None, kinds: None,
            since: None, until: None, limit: Some(3),
        };
        let events = store.query_events(&[filter]).await.unwrap();
        assert_eq!(events.len(), 3);
    }
}

