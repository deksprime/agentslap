//! Database-backed session storage using SQLite

use agent_llm::Conversation;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{
    error::SessionError,
    store::{SessionMetadata, SessionStore},
    Result,
};

/// SQLite-backed session store
///
/// Provides persistent storage for sessions across application restarts.
/// Database file is a single SQLite file.
///
/// # Features
///
/// - Persistent storage (survives restarts)
/// - ACID transactions
/// - Concurrent access (WAL mode)
/// - Automatic migrations
/// - Cross-platform (Linux, macOS, Windows)
///
/// # Example
///
/// ```no_run
/// use agent_session::{DatabaseStore, SessionStore};
/// use agent_llm::Conversation;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = DatabaseStore::new("/tmp/sessions.db").await?;
///     
///     let conv = Conversation::new();
///     store.save("session-1", conv).await?;
///     
///     // Later (even after restart)
///     let loaded = store.load("session-1").await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct DatabaseStore {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl DatabaseStore {
    /// Create a new database store
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file (will be created if doesn't exist)
    ///
    /// # Behavior
    /// - Creates parent directories if they don't exist (permissions: 755)
    /// - Sets database file permissions to 600 (owner read/write only)
    /// - Runs embedded migrations automatically
    /// - Enables WAL mode for better concurrency
    ///
    /// # Errors
    /// - Returns error if path is not absolute
    /// - Returns error if cannot create directories
    /// - Returns error if cannot connect to database
    /// - Returns error if migrations fail
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();

        // Require absolute path for safety
        if !db_path.is_absolute() {
            return Err(SessionError::storage(format!(
                "Database path must be absolute, got: {}",
                db_path.display()
            )));
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SessionError::storage(format!(
                        "Failed to create database directory {}: {}",
                        parent.display(),
                        e
                    )))?;

                // Set directory permissions (755)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(0o755);
                    std::fs::set_permissions(parent, perms)
                        .map_err(|e| SessionError::storage(format!(
                            "Failed to set directory permissions: {}",
                            e
                        )))?;
                }

                tracing::info!("Created database directory: {}", parent.display());
            }
        }

        // Connect to database
        let db_url = format!("sqlite://{}", db_path.display());
        
        let options = SqliteConnectOptions::from_str(&db_url)
            .map_err(|e| SessionError::storage(format!("Invalid database URL: {}", e)))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| SessionError::storage(format!("Failed to connect to database: {}", e)))?;

        // Run migrations (embedded in binary)
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| SessionError::storage(format!("Migration failed: {}", e)))?;

        // Set PRAGMAs (must be outside transaction, so after migrations)
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(|e| SessionError::storage(format!("Failed to set WAL mode: {}", e)))?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .map_err(|e| SessionError::storage(format!("Failed to enable foreign keys: {}", e)))?;

        sqlx::query("PRAGMA busy_timeout = 5000")
            .execute(&pool)
            .await
            .map_err(|e| SessionError::storage(format!("Failed to set busy timeout: {}", e)))?;

        tracing::info!("Database initialized: {}", db_path.display());

        // Set database file permissions (600) - owner only
        #[cfg(unix)]
        {
            if db_path.exists() {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o600);
                std::fs::set_permissions(db_path, perms)
                    .map_err(|e| SessionError::storage(format!(
                        "Failed to set database permissions: {}",
                        e
                    )))?;
            }
        }

        Ok(Self {
            pool,
            db_path: db_path.to_path_buf(),
        })
    }

    /// Get the path to the database file
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Get database statistics
    pub async fn stats(&self) -> Result<DatabaseStats> {
        let row: (i64, Option<i64>, Option<i64>) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as count,
                SUM(message_count) as total_messages,
                SUM(estimated_tokens) as total_tokens
            FROM sessions
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?;

        Ok(DatabaseStats {
            session_count: row.0 as usize,
            total_messages: row.1.unwrap_or(0) as usize,
            total_tokens: row.2.unwrap_or(0) as usize,
        })
    }

    /// Vacuum the database to reclaim space
    pub async fn vacuum(&self) -> Result<()> {
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await
            .map_err(|e| SessionError::storage(e.to_string()))?;

        tracing::info!("Database vacuumed: {}", self.db_path.display());
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Total number of sessions
    pub session_count: usize,
    /// Total messages across all sessions
    pub total_messages: usize,
    /// Total estimated tokens
    pub total_tokens: usize,
}

#[async_trait]
impl SessionStore for DatabaseStore {
    async fn save(&self, session_id: &str, conversation: Conversation) -> Result<()> {
        let conversation_json = serde_json::to_string(&conversation)
            .map_err(|e| SessionError::Serialization(e))?;

        let summary = conversation.summary();
        let metadata_json = if conversation.metadata.is_null() {
            None
        } else {
            Some(conversation.metadata.to_string())
        };

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, conversation_json, created_at, updated_at,
                message_count, estimated_tokens, metadata_json
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                conversation_json = excluded.conversation_json,
                updated_at = excluded.updated_at,
                message_count = excluded.message_count,
                estimated_tokens = excluded.estimated_tokens,
                metadata_json = excluded.metadata_json
            "#
        )
        .bind(session_id)
        .bind(conversation_json)
        .bind(conversation.created_at.timestamp())
        .bind(conversation.updated_at.timestamp())
        .bind(summary.message_count as i64)
        .bind(summary.estimated_tokens as i64)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?;

        tracing::debug!("Saved session to database: {}", session_id);
        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Conversation> {
        let row: (String,) = sqlx::query_as(
            r#"
            SELECT conversation_json
            FROM sessions
            WHERE id = ?
            "#
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?
        .ok_or_else(|| SessionError::not_found(session_id))?;

        serde_json::from_str(&row.0)
            .map_err(|e| SessionError::Serialization(e))
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE id = ?
            "#
        )
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(SessionError::not_found(session_id));
        }

        tracing::debug!("Deleted session from database: {}", session_id);
        Ok(())
    }

    async fn exists(&self, session_id: &str) -> Result<bool> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM sessions
            WHERE id = ?
            "#
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?;

        Ok(row.0 > 0)
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM sessions
            ORDER BY updated_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?;

        Ok(rows.into_iter().map(|row| row.0).collect())
    }

    async fn get_metadata(&self, session_id: &str) -> Result<SessionMetadata> {
        let row: (String, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT id, message_count, estimated_tokens, created_at, updated_at
            FROM sessions
            WHERE id = ?
            "#
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?
        .ok_or_else(|| SessionError::not_found(session_id))?;

        Ok(SessionMetadata {
            id: row.0,
            message_count: row.1 as usize,
            estimated_tokens: row.2 as usize,
            created_at: chrono::DateTime::from_timestamp(row.3, 0)
                .unwrap_or_else(chrono::Utc::now),
            updated_at: chrono::DateTime::from_timestamp(row.4, 0)
                .unwrap_or_else(chrono::Utc::now),
        })
    }

    async fn clear(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SessionError::storage(e.to_string()))?;

        let count = result.rows_affected() as usize;
        tracing::info!("Cleared {} sessions from database", count);
        Ok(count)
    }

    fn name(&self) -> &str {
        "database"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> DatabaseStore {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_agent_{}.db", uuid::Uuid::new_v4()));
        DatabaseStore::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_database_creation() {
        let store = create_test_db().await;
        assert_eq!(store.name(), "database");
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let store = create_test_db().await;
        let conv = Conversation::builder()
            .system("Test")
            .user("Hello")
            .build();

        store.save("test-session", conv.clone()).await.unwrap();
        let loaded = store.load("test-session").await.unwrap();

        assert_eq!(loaded.len(), conv.len());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = create_test_db().await;
        let conv = Conversation::new();

        store.save("test", conv).await.unwrap();
        assert!(store.exists("test").await.unwrap());

        store.delete("test").await.unwrap();
        assert!(!store.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let store = create_test_db().await;

        store.save("s1", Conversation::new()).await.unwrap();
        store.save("s2", Conversation::new()).await.unwrap();

        let sessions = store.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let store = create_test_db().await;
        let conv = Conversation::builder()
            .system("System")
            .user("User")
            .build();

        store.save("test", conv).await.unwrap();

        let metadata = store.get_metadata("test").await.unwrap();
        assert_eq!(metadata.id, "test");
        assert_eq!(metadata.message_count, 2);
    }

    #[tokio::test]
    async fn test_update_session() {
        let store = create_test_db().await;
        let mut conv = Conversation::builder()
            .system("Original")
            .build();

        store.save("test", conv.clone()).await.unwrap();

        conv.add_user("New message");
        store.save("test", conv.clone()).await.unwrap();

        let loaded = store.load("test").await.unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    async fn test_clear() {
        let store = create_test_db().await;

        store.save("s1", Conversation::new()).await.unwrap();
        store.save("s2", Conversation::new()).await.unwrap();

        let count = store.clear().await.unwrap();
        assert_eq!(count, 2);
        assert!(!store.exists("s1").await.unwrap());
    }

    #[tokio::test]
    async fn test_stats() {
        let store = create_test_db().await;

        let conv = Conversation::builder()
            .system("Test")
            .user("Message")
            .build();

        store.save("s1", conv.clone()).await.unwrap();
        store.save("s2", conv).await.unwrap();

        let stats = store.stats().await.unwrap();
        assert_eq!(stats.session_count, 2);
        assert!(stats.total_messages > 0);
    }

    #[tokio::test]
    async fn test_vacuum() {
        let store = create_test_db().await;
        store.vacuum().await.unwrap();
    }
}

