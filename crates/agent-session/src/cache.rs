//! Cache-based session storage with TTL

use agent_llm::Conversation;
use async_trait::async_trait;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    error::SessionError,
    store::{SessionMetadata, SessionStore},
    Result,
};

/// Cache-based session store with Time-To-Live (TTL)
///
/// Sessions automatically expire after the configured TTL.
/// Uses LRU eviction when capacity is reached.
///
/// # Example
///
/// ```
/// use agent_session::{CacheStore, SessionStore};
/// use agent_llm::Conversation;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Cache with 1-hour TTL
///     let store = CacheStore::new(Duration::from_secs(3600));
///     
///     let conv = Conversation::new();
///     store.save("session-1", conv).await?;
///     
///     // Session will expire after 1 hour
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct CacheStore {
    /// Moka cache with TTL support
    cache: Arc<Cache<String, Conversation>>,
    /// Time-to-live for sessions
    ttl: Duration,
}

impl CacheStore {
    /// Create a new cache store with default settings
    ///
    /// # Arguments
    /// * `ttl` - Time-to-live for sessions
    pub fn new(ttl: Duration) -> Self {
        let cache = Cache::builder()
            .time_to_live(ttl)
            .max_capacity(10_000) // Default capacity
            .build();

        Self {
            cache: Arc::new(cache),
            ttl,
        }
    }

    /// Create a cache store with custom capacity
    pub fn with_capacity(ttl: Duration, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .time_to_live(ttl)
            .max_capacity(max_capacity)
            .build();

        Self {
            cache: Arc::new(cache),
            ttl,
        }
    }

    /// Get the current number of cached sessions
    pub async fn len(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        self.cache.entry_count() == 0
    }

    /// Get the configured TTL
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Run cache maintenance (evict expired entries)
    pub async fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks().await;
    }
}

#[async_trait]
impl SessionStore for CacheStore {
    async fn save(&self, session_id: &str, conversation: Conversation) -> Result<()> {
        self.cache
            .insert(session_id.to_string(), conversation)
            .await;
        tracing::debug!("Cached session: {} (TTL: {:?})", session_id, self.ttl);
        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Conversation> {
        self.cache
            .get(session_id)
            .await
            .ok_or_else(|| {
                // Session either doesn't exist or has expired
                SessionError::not_found(session_id)
            })
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        self.cache.invalidate(session_id).await;
        tracing::debug!("Invalidated cached session: {}", session_id);
        Ok(())
    }

    async fn exists(&self, session_id: &str) -> Result<bool> {
        Ok(self.cache.contains_key(session_id))
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        // Moka doesn't provide a way to list all keys directly
        // This is a limitation of the cache implementation
        // For now, we'll return an empty list and document this
        tracing::warn!("list_sessions() not fully supported on CacheStore");
        Ok(vec![])
    }

    async fn get_metadata(&self, session_id: &str) -> Result<SessionMetadata> {
        let conversation = self.load(session_id).await?;
        let summary = conversation.summary();

        Ok(SessionMetadata {
            id: session_id.to_string(),
            message_count: summary.message_count,
            estimated_tokens: summary.estimated_tokens,
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
        })
    }

    async fn clear(&self) -> Result<usize> {
        let count = self.cache.entry_count();
        self.cache.invalidate_all();
        
        // Wait for invalidation to complete
        self.cache.run_pending_tasks().await;
        
        tracing::info!("Cleared ~{} sessions from cache store", count);
        Ok(count as usize)
    }

    fn name(&self) -> &str {
        "cache"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_save_and_load() {
        let store = CacheStore::new(Duration::from_secs(60));
        let conv = Conversation::builder()
            .user("Hello")
            .build();

        store.save("test", conv.clone()).await.unwrap();
        let loaded = store.load("test").await.unwrap();
        assert_eq!(loaded.len(), conv.len());
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        // Very short TTL for testing
        let store = CacheStore::new(Duration::from_millis(100));
        let conv = Conversation::new();

        store.save("expires-soon", conv).await.unwrap();
        
        // Should exist immediately
        assert!(store.exists("expires-soon").await.unwrap());

        // Wait for expiration
        sleep(Duration::from_millis(200)).await;
        store.run_pending_tasks().await;

        // Should be expired now
        let result = store.load("expires-soon").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = CacheStore::new(Duration::from_secs(60));
        let conv = Conversation::new();

        store.save("test", conv).await.unwrap();
        assert!(store.exists("test").await.unwrap());

        store.delete("test").await.unwrap();
        assert!(!store.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_capacity_limit() {
        let store = CacheStore::with_capacity(Duration::from_secs(60), 2);

        store.save("s1", Conversation::new()).await.unwrap();
        store.save("s2", Conversation::new()).await.unwrap();
        
        // Third insert should succeed but evict oldest (LRU)
        store.save("s3", Conversation::new()).await.unwrap();
        
        // Run cache maintenance
        store.run_pending_tasks().await;
        
        // One of the first two should be evicted
        let count = store.len().await;
        assert!(count <= 2);
    }

    #[tokio::test]
    async fn test_clear() {
        let store = CacheStore::new(Duration::from_secs(60));

        store.save("s1", Conversation::new()).await.unwrap();
        store.save("s2", Conversation::new()).await.unwrap();

        store.clear().await.unwrap();
        
        // After clear, sessions should not exist
        assert!(!store.exists("s1").await.unwrap());
        assert!(!store.exists("s2").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let store = CacheStore::new(Duration::from_secs(60));
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
    async fn test_update_extends_ttl() {
        let store = CacheStore::new(Duration::from_millis(500));
        let conv = Conversation::builder().user("First").build();

        store.save("test", conv).await.unwrap();
        
        // Wait a bit but not past TTL
        sleep(Duration::from_millis(300)).await;
        
        // Update (should extend TTL)
        let mut updated = store.load("test").await.unwrap();
        updated.add_user("Second");
        store.save("test", updated).await.unwrap();
        
        // Wait past original TTL
        sleep(Duration::from_millis(300)).await;
        store.run_pending_tasks().await;
        
        // Should still exist (TTL was extended)
        assert!(store.exists("test").await.unwrap());
    }
}

