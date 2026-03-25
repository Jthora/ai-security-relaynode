// Database initialization and versioned migration management
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite, Row};
use tracing::{info, error, warn};
use anyhow::Result;

pub struct DatabaseManager {
    pool: SqlitePool,
}

/// Represents a single migration step.
struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

/// All migrations in order. Add new entries at the end.
fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "MVP schema",
            sql: include_str!("migrations/001_mvp_schema.sql"),
        },
        // Legacy investigation tables — kept so existing DBs don't break.
        Migration {
            version: 2,
            name: "Investigation tables",
            sql: include_str!("migrations/add_investigation_tables_simple.sql"),
        },
    ]
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create database if it doesn't exist
        if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
            info!("Creating database: {}", database_url);
            Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;

        // Enable WAL mode for better concurrent read performance
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await
            .ok(); // non-fatal if it fails

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");

        // Ensure schema_version table exists (bootstrap)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        // Determine current version
        let current_version: i64 = sqlx::query("SELECT COALESCE(MAX(version), 0) as v FROM schema_version")
            .fetch_one(&self.pool)
            .await
            .map(|row| row.get::<i64, _>("v"))
            .unwrap_or(0);

        let all_migrations = migrations();
        let pending: Vec<&Migration> = all_migrations.iter()
            .filter(|m| m.version > current_version)
            .collect();

        if pending.is_empty() {
            info!("Database at version {}, no pending migrations", current_version);
            return Ok(());
        }

        info!("Database at version {}, applying {} pending migration(s)", current_version, pending.len());

        for migration in &pending {
            info!("Applying migration v{}: {}", migration.version, migration.name);

            // Run each migration inside a transaction for atomicity
            let mut tx = self.pool.begin().await?;

            // Split SQL by semicolons and execute each statement
            let statements: Vec<&str> = migration.sql
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && !s.starts_with("--"))
                .collect();

            for statement in &statements {
                // Strip line-level comments
                let clean: String = statement.lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.starts_with("--")
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let clean = clean.trim();
                if clean.is_empty() { continue; }

                match sqlx::query(clean).execute(&mut *tx).await {
                    Ok(_) => {}
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("already exists") || msg.contains("duplicate column") {
                            warn!("Migration v{}: skipped (already applied): {}", migration.version, msg);
                        } else {
                            error!("Migration v{} failed on statement: {}", migration.version, clean);
                            return Err(anyhow::anyhow!(
                                "Migration v{} '{}' failed: {}", migration.version, migration.name, e
                            ));
                        }
                    }
                }
            }

            // Record the version
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            sqlx::query("INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (?, ?)")
                .bind(migration.version)
                .bind(now)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            info!("Migration v{} applied successfully", migration.version);
        }

        info!("Database migrations completed (now at version {})",
              pending.last().map(|m| m.version).unwrap_or(current_version));
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Insert a relay stats snapshot.
    pub async fn insert_stats_snapshot(
        &self,
        connections: i64,
        events_stored: i64,
        events_minute: i64,
        content_items: i64,
        content_bytes: i64,
        alerts_total: i64,
        memory_bytes: i64,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO relay_stats (timestamp, connections, events_stored, events_minute, content_items, content_bytes, alerts_total, memory_bytes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(now)
        .bind(connections)
        .bind(events_stored)
        .bind(events_minute)
        .bind(content_items)
        .bind(content_bytes)
        .bind(alerts_total)
        .bind(memory_bytes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get investigation count (legacy compat).
    pub async fn get_investigation_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM investigations")
            .fetch_one(&self.pool)
            .await?;
        let count: i64 = row.get("count");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_migrations_run() {
        // Use in-memory SQLite for test
        let db = DatabaseManager::new("sqlite::memory:").await.unwrap();
        db.run_migrations().await.unwrap();

        // Verify MVP tables exist
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='events'")
            .fetch_optional(db.pool())
            .await
            .unwrap();
        assert!(result.is_some(), "events table should exist");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='content_index'")
            .fetch_optional(db.pool())
            .await
            .unwrap();
        assert!(result.is_some(), "content_index table should exist");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='security_alerts'")
            .fetch_optional(db.pool())
            .await
            .unwrap();
        assert!(result.is_some(), "security_alerts table should exist");

        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='relay_stats'")
            .fetch_optional(db.pool())
            .await
            .unwrap();
        assert!(result.is_some(), "relay_stats table should exist");

        // Verify idempotency — run again, no errors
        db.run_migrations().await.unwrap();
    }

    #[tokio::test]
    async fn test_stats_snapshot_insert() {
        let db = DatabaseManager::new("sqlite::memory:").await.unwrap();
        db.run_migrations().await.unwrap();

        db.insert_stats_snapshot(5, 100, 10, 3, 1024, 2, 50_000_000).await.unwrap();

        let row = sqlx::query("SELECT connections, events_stored FROM relay_stats ORDER BY timestamp DESC LIMIT 1")
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert_eq!(row.get::<i64, _>("connections"), 5);
        assert_eq!(row.get::<i64, _>("events_stored"), 100);
    }
}
