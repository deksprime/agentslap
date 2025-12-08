//! Session storage trait definition

use agent_llm::Conversation;
use async_trait::async_trait;

use crate::Result;

/// Metadata about a stored session
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// Session identifier
    pub id: String,
    /// Number of messages in the conversation
    pub message_count: usize,
    /// Estimated token count
    pub estimated_tokens: usize,
    /// When the session was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the session was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Trait for session storage backends
///
/// Implementations provide different storage strategies:
/// - In-memory for speed
/// - Cache with TTL for temporary storage
/// - Database for persistence
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Save a conversation to the store
    ///
    /// # Arguments
    /// * `session_id` - Unique identifier for the session
    /// * `conversation` - The conversation to save
    ///
    /// # Returns
    /// Ok(()) on success, Error if save fails
    async fn save(&self, session_id: &str, conversation: Conversation) -> Result<()>;

    /// Load a conversation from the store
    ///
    /// # Arguments
    /// * `session_id` - Unique identifier for the session
    ///
    /// # Returns
    /// The conversation if found, NotFound error otherwise
    async fn load(&self, session_id: &str) -> Result<Conversation>;

    /// Delete a conversation from the store
    ///
    /// # Arguments
    /// * `session_id` - Unique identifier for the session
    ///
    /// # Returns
    /// Ok(()) on success, NotFound if session doesn't exist
    async fn delete(&self, session_id: &str) -> Result<()>;

    /// Check if a session exists
    ///
    /// # Arguments
    /// * `session_id` - Unique identifier for the session
    ///
    /// # Returns
    /// true if the session exists, false otherwise
    async fn exists(&self, session_id: &str) -> Result<bool>;

    /// List all session IDs in the store
    ///
    /// # Returns
    /// Vector of session IDs
    async fn list_sessions(&self) -> Result<Vec<String>>;

    /// Get metadata for a session without loading the full conversation
    ///
    /// # Arguments
    /// * `session_id` - Unique identifier for the session
    ///
    /// # Returns
    /// Session metadata if found
    async fn get_metadata(&self, session_id: &str) -> Result<SessionMetadata>;

    /// Clear all sessions from the store
    ///
    /// # Returns
    /// Number of sessions cleared
    async fn clear(&self) -> Result<usize>;

    /// Get the name of this store (for debugging/logging)
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock store for testing
    struct MockStore;

    #[async_trait]
    impl SessionStore for MockStore {
        async fn save(&self, _session_id: &str, _conversation: Conversation) -> Result<()> {
            Ok(())
        }

        async fn load(&self, _session_id: &str) -> Result<Conversation> {
            Ok(Conversation::new())
        }

        async fn delete(&self, _session_id: &str) -> Result<()> {
            Ok(())
        }

        async fn exists(&self, _session_id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn list_sessions(&self) -> Result<Vec<String>> {
            Ok(vec!["session-1".to_string()])
        }

        async fn get_metadata(&self, session_id: &str) -> Result<SessionMetadata> {
            Ok(SessionMetadata {
                id: session_id.to_string(),
                message_count: 0,
                estimated_tokens: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }

        async fn clear(&self) -> Result<usize> {
            Ok(0)
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_mock_store() {
        let store = MockStore;
        assert_eq!(store.name(), "mock");
        
        let result = store.save("test", Conversation::new()).await;
        assert!(result.is_ok());
        
        let sessions = store.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
    }
}




