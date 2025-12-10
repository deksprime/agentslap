//! Session Management
//!
//! This crate provides session storage and management capabilities
//! with pluggable backends (in-memory, cache, database).
//!
//! # Example
//!
//! ```no_run
//! use agent_session::{InMemoryStore, SessionStore};
//! use agent_llm::Conversation;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = InMemoryStore::new();
//!     
//!     let conversation = Conversation::builder()
//!         .system("You are helpful")
//!         .user("Hello")
//!         .build();
//!     
//!     // Save session
//!     store.save("session-1", conversation).await?;
//!     
//!     // Load session
//!     let loaded = store.load("session-1").await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod store;
pub mod memory;
pub mod cache;
pub mod layered;

// Database support (optional feature)
#[cfg(feature = "database")]
pub mod database;

// Re-exports
pub use error::{SessionError, Result};
pub use store::SessionStore;
pub use memory::InMemoryStore;
pub use cache::CacheStore;
pub use layered::LayeredStore;

#[cfg(feature = "database")]
pub use database::{DatabaseStore, DatabaseStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all main types are accessible
        let _ = std::mem::size_of::<InMemoryStore>();
        let _ = std::mem::size_of::<CacheStore>();
    }
}
