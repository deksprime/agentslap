//! Layered session storage with fallback

use agent_llm::Conversation;
use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    error::SessionError,
    store::{SessionMetadata, SessionStore},
    Result,
};

/// Layered store combining multiple storage backends
///
/// Provides a caching hierarchy where reads check layers in order
/// and writes propagate to all layers.
///
/// Typical usage: Cache → Memory → Database
/// - Cache: Fast, temporary (TTL-based)
/// - Memory: Fast, process lifetime
/// - Database: Slow, persistent (Phase 6)
///
/// # Example
///
/// ```
/// use agent_session::{LayeredStore, InMemoryStore, CacheStore, SessionStore};
/// use agent_llm::Conversation;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let cache = CacheStore::new(Duration::from_secs(300));
///     let memory = InMemoryStore::new();
///     
///     let store = LayeredStore::new()
///         .with_layer(cache)  // Fast, temporary
///         .with_layer(memory); // Persistent in process
///     
///     // Writes go to all layers
///     store.save("session-1", Conversation::new()).await?;
///     
///     // Reads check cache first, then memory
///     let loaded = store.load("session-1").await?;
///     
///     Ok(())
/// }
/// ```
pub struct LayeredStore {
    /// Storage layers (checked in order for reads)
    layers: Vec<Arc<dyn SessionStore>>,
}

impl LayeredStore {
    /// Create a new layered store
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a storage layer
    ///
    /// Layers are checked in the order they're added.
    /// First layer = fastest (cache)
    /// Last layer = slowest (database)
    pub fn with_layer<S: SessionStore + 'static>(mut self, store: S) -> Self {
        self.layers.push(Arc::new(store));
        self
    }

    /// Get the number of layers
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

impl Default for LayeredStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for LayeredStore {
    async fn save(&self, session_id: &str, conversation: Conversation) -> Result<()> {
        if self.layers.is_empty() {
            return Err(SessionError::storage("No storage layers configured"));
        }

        // Write-through: save to all layers
        let mut errors = Vec::new();

        for (i, layer) in self.layers.iter().enumerate() {
            if let Err(e) = layer.save(session_id, conversation.clone()).await {
                tracing::warn!(
                    "Failed to save to layer {} ({}): {}",
                    i,
                    layer.name(),
                    e
                );
                errors.push((layer.name(), e));
            }
        }

        // If all layers failed, return error
        if errors.len() == self.layers.len() {
            return Err(SessionError::storage(format!(
                "Failed to save to all layers: {:?}",
                errors
            )));
        }

        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Conversation> {
        if self.layers.is_empty() {
            return Err(SessionError::storage("No storage layers configured"));
        }

        // Read-through: check each layer in order
        for (i, layer) in self.layers.iter().enumerate() {
            match layer.load(session_id).await {
                Ok(conversation) => {
                    tracing::debug!(
                        "Loaded session {} from layer {} ({})",
                        session_id,
                        i,
                        layer.name()
                    );

                    // Write-back: populate earlier (faster) layers
                    for earlier_layer in self.layers.iter().take(i) {
                        if let Err(e) = earlier_layer.save(session_id, conversation.clone()).await
                        {
                            tracing::warn!(
                                "Failed to write-back to layer {}: {}",
                                earlier_layer.name(),
                                e
                            );
                        }
                    }

                    return Ok(conversation);
                }
                Err(SessionError::NotFound(_)) => {
                    // Try next layer
                    continue;
                }
                Err(e) => {
                    // Non-NotFound error, log but continue
                    tracing::warn!("Error loading from layer {}: {}", layer.name(), e);
                    continue;
                }
            }
        }

        // Not found in any layer
        Err(SessionError::not_found(session_id))
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        if self.layers.is_empty() {
            return Err(SessionError::storage("No storage layers configured"));
        }

        // Delete from all layers
        let mut found_in_any = false;

        for layer in &self.layers {
            match layer.delete(session_id).await {
                Ok(()) => {
                    found_in_any = true;
                }
                Err(SessionError::NotFound(_)) => {
                    // Not in this layer, continue
                    continue;
                }
                Err(e) => {
                    tracing::warn!("Error deleting from layer {}: {}", layer.name(), e);
                }
            }
        }

        if !found_in_any {
            return Err(SessionError::not_found(session_id));
        }

        Ok(())
    }

    async fn exists(&self, session_id: &str) -> Result<bool> {
        // Check if exists in any layer
        for layer in &self.layers {
            if layer.exists(session_id).await.unwrap_or(false) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        if self.layers.is_empty() {
            return Ok(vec![]);
        }

        // Collect sessions from all layers and deduplicate
        let mut all_sessions = std::collections::HashSet::new();

        for layer in &self.layers {
            if let Ok(sessions) = layer.list_sessions().await {
                all_sessions.extend(sessions);
            }
        }

        Ok(all_sessions.into_iter().collect())
    }

    async fn get_metadata(&self, session_id: &str) -> Result<SessionMetadata> {
        // Get metadata from first layer that has it
        for layer in &self.layers {
            if let Ok(metadata) = layer.get_metadata(session_id).await {
                return Ok(metadata);
            }
        }

        Err(SessionError::not_found(session_id))
    }

    async fn clear(&self) -> Result<usize> {
        let mut total_cleared = 0;

        for layer in &self.layers {
            if let Ok(count) = layer.clear().await {
                total_cleared += count;
            }
        }

        tracing::info!("Cleared {} sessions from layered store", total_cleared);
        Ok(total_cleared)
    }

    fn name(&self) -> &str {
        "layered"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CacheStore, InMemoryStore};
    use std::time::Duration;

    #[tokio::test]
    async fn test_layered_creation() {
        let store = LayeredStore::new()
            .with_layer(CacheStore::new(Duration::from_secs(60)))
            .with_layer(InMemoryStore::new());

        assert_eq!(store.layer_count(), 2);
    }

    #[tokio::test]
    async fn test_write_through() {
        let cache = CacheStore::new(Duration::from_secs(60));
        let memory = InMemoryStore::new();

        let store = LayeredStore::new()
            .with_layer(cache.clone())
            .with_layer(memory.clone());

        let conv = Conversation::builder().user("Test").build();
        store.save("test", conv).await.unwrap();

        // Should be in both layers
        assert!(cache.exists("test").await.unwrap());
        assert!(memory.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_read_from_first_available() {
        let cache = CacheStore::new(Duration::from_secs(60));
        let memory = InMemoryStore::new();

        let store = LayeredStore::new()
            .with_layer(cache.clone())
            .with_layer(memory.clone());

        // Save only to memory (simulate cache miss)
        let conv = Conversation::builder().user("Test").build();
        memory.save("test", conv.clone()).await.unwrap();

        // Load should find it in memory
        let loaded = store.load("test").await.unwrap();
        assert_eq!(loaded.len(), conv.len());

        // Should now be written back to cache
        assert!(cache.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_from_all_layers() {
        let cache = CacheStore::new(Duration::from_secs(60));
        let memory = InMemoryStore::new();

        let store = LayeredStore::new()
            .with_layer(cache.clone())
            .with_layer(memory.clone());

        let conv = Conversation::new();
        store.save("test", conv).await.unwrap();

        // Delete from layered store
        store.delete("test").await.unwrap();

        // Should be deleted from both
        assert!(!cache.exists("test").await.unwrap());
        assert!(!memory.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_exists_in_any_layer() {
        let cache = CacheStore::new(Duration::from_secs(60));
        let memory = InMemoryStore::new();

        let store = LayeredStore::new()
            .with_layer(cache.clone())
            .with_layer(memory.clone());

        // Save only to memory
        memory.save("test", Conversation::new()).await.unwrap();

        // Should be found via layered store
        assert!(store.exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_sessions_deduplicate() {
        let memory1 = InMemoryStore::new();
        let memory2 = InMemoryStore::new();

        let store = LayeredStore::new()
            .with_layer(memory1.clone())
            .with_layer(memory2.clone());

        // Add same session to both layers
        memory1.save("s1", Conversation::new()).await.unwrap();
        memory2.save("s1", Conversation::new()).await.unwrap();
        
        // Add unique sessions
        memory1.save("s2", Conversation::new()).await.unwrap();
        memory2.save("s3", Conversation::new()).await.unwrap();

        let sessions = store.list_sessions().await.unwrap();
        
        // Should have 3 unique sessions
        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_empty_store() {
        let store = LayeredStore::new();

        let result = store.save("test", Conversation::new()).await;
        assert!(result.is_err());

        let result = store.load("test").await;
        assert!(result.is_err());
    }
}

