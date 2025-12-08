//! In-memory session storage using DashMap

use agent_llm::Conversation;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    error::SessionError,
    store::{SessionMetadata, SessionStore},
    Result,
};

/// In-memory session store using concurrent HashMap
///
/// This store keeps all sessions in memory and is thread-safe.
/// Sessions are lost when the application restarts.
///
/// # Example
///
/// ```
/// use agent_session::{InMemoryStore, SessionStore};
/// use agent_llm::Conversation;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = InMemoryStore::new();
///     
///     let conv = Conversation::new();
///     store.save("session-1", conv).await?;
///     
///     let loaded = store.load("session-1").await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct InMemoryStore {
    /// Concurrent HashMap storing conversations
    sessions: Arc<DashMap<String, Conversation>>,
    /// Maximum number of sessions to store
    max_capacity: Option<usize>,
}

impl InMemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            max_capacity: None,
        }
    }

    /// Create a new in-memory store with capacity limit
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            max_capacity: Some(max_capacity),
        }
    }

    /// Get the current number of sessions
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Check if capacity limit is reached
    fn is_at_capacity(&self) -> bool {
        if let Some(max) = self.max_capacity {
            self.sessions.len() >= max
        } else {
            false
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemoryStore {
    async fn save(&self, session_id: &str, conversation: Conversation) -> Result<()> {
        // Check capacity before inserting new session
        if !self.sessions.contains_key(session_id) && self.is_at_capacity() {
            return Err(SessionError::CapacityExceeded);
        }

        self.sessions.insert(session_id.to_string(), conversation);
        tracing::debug!("Saved session: {}", session_id);
        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Conversation> {
        self.sessions
            .get(session_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| SessionError::not_found(session_id))
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| SessionError::not_found(session_id))?;
        tracing::debug!("Deleted session: {}", session_id);
        Ok(())
    }

    async fn exists(&self, session_id: &str) -> Result<bool> {
        Ok(self.sessions.contains_key(session_id))
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        Ok(self.sessions.iter().map(|entry| entry.key().clone()).collect())
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
        let count = self.sessions.len();
        self.sessions.clear();
        tracing::info!("Cleared {} sessions from memory store", count);
        Ok(count)
    }

    fn name(&self) -> &str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_load() {
        let store = InMemoryStore::new();
        let conv = Conversation::builder()
            .system("Test")
            .user("Hello")
            .build();

        let session_id = "test-session";
        store.save(session_id, conv.clone()).await.unwrap();

        let loaded = store.load(session_id).await.unwrap();
        assert_eq!(loaded.len(), conv.len());
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let store = InMemoryStore::new();
        let result = store.load("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryStore::new();
        let conv = Conversation::new();

        store.save("test", conv).await.unwrap();
        assert!(store.exists("test").await.unwrap());

        store.delete("test").await.unwrap();
        assert!(!store.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let store = InMemoryStore::new();

        store.save("session-1", Conversation::new()).await.unwrap();
        store.save("session-2", Conversation::new()).await.unwrap();

        let sessions = store.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session-1".to_string()));
        assert!(sessions.contains(&"session-2".to_string()));
    }

    #[tokio::test]
    async fn test_clear() {
        let store = InMemoryStore::new();

        store.save("s1", Conversation::new()).await.unwrap();
        store.save("s2", Conversation::new()).await.unwrap();

        let count = store.clear().await.unwrap();
        assert_eq!(count, 2);
        assert!(store.is_empty());
    }

    #[tokio::test]
    async fn test_capacity_limit() {
        let store = InMemoryStore::with_capacity(2);

        store.save("s1", Conversation::new()).await.unwrap();
        store.save("s2", Conversation::new()).await.unwrap();

        // Third insert should fail
        let result = store.save("s3", Conversation::new()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SessionError::CapacityExceeded));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let store = Arc::new(InMemoryStore::new());
        let mut handles = vec![];

        // Spawn 10 concurrent tasks
        for i in 0..10 {
            let store = Arc::clone(&store);
            let handle = tokio::spawn(async move {
                let session_id = format!("session-{}", i);
                let conv = Conversation::builder()
                    .user(format!("Message {}", i))
                    .build();

                store.save(&session_id, conv).await.unwrap();
                store.load(&session_id).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(store.len(), 10);
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let store = InMemoryStore::new();
        let conv = Conversation::builder()
            .system("System")
            .user("User")
            .build();

        store.save("test", conv).await.unwrap();

        let metadata = store.get_metadata("test").await.unwrap();
        assert_eq!(metadata.id, "test");
        assert_eq!(metadata.message_count, 2);
        assert!(metadata.estimated_tokens > 0);
    }
}

