//! Basic Session Storage Example
//!
//! Demonstrates using different storage backends for sessions.
//!
//! Run with:
//! ```bash
//! cargo run -p agent-session --example basic_storage
//! ```

use agent_llm::Conversation;
use agent_session::{CacheStore, InMemoryStore, SessionStore};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Session Storage Demo\n");

    // Example 1: In-Memory Store
    println!("=== Example 1: InMemoryStore ===");
    let memory_store = InMemoryStore::new();

    let conv1 = Conversation::builder()
        .system("You are a helpful assistant")
        .user("What is Rust?")
        .assistant("Rust is a systems programming language...")
        .build();

    // Save session
    memory_store.save("session-1", conv1.clone()).await?;
    println!("✓ Saved session-1 to memory store");

    // Load session
    let loaded = memory_store.load("session-1").await?;
    println!("✓ Loaded session-1: {} messages", loaded.len());

    // Check exists
    assert!(memory_store.exists("session-1").await?);
    println!("✓ Session exists");

    // Get metadata without loading full conversation
    let metadata = memory_store.get_metadata("session-1").await?;
    println!("✓ Session metadata:");
    println!("  - Messages: {}", metadata.message_count);
    println!("  - Tokens: ~{}", metadata.estimated_tokens);
    println!("  - Created: {}", metadata.created_at);
    println!();

    // Example 2: Cache Store with TTL
    println!("=== Example 2: CacheStore with TTL ===");
    let cache_store = CacheStore::new(Duration::from_secs(300)); // 5-minute TTL

    let conv2 = Conversation::builder()
        .system("You are a coding expert")
        .user("Explain async/await")
        .build();

    cache_store.save("cached-session", conv2).await?;
    println!("✓ Saved to cache with 5-minute TTL");

    assert!(cache_store.exists("cached-session").await?);
    println!("✓ Session exists in cache");

    let loaded_from_cache = cache_store.load("cached-session").await?;
    println!("✓ Loaded from cache: {} messages", loaded_from_cache.len());
    println!();

    // Example 3: Multiple Sessions
    println!("=== Example 3: Multiple Sessions ===");
    let store = InMemoryStore::new();

    for i in 1..=5 {
        let conv = Conversation::builder()
            .system(format!("Agent {}", i))
            .user(format!("Question {}", i))
            .build();

        store.save(&format!("session-{}", i), conv).await?;
    }

    let sessions = store.list_sessions().await?;
    println!("✓ Stored {} sessions", sessions.len());

    for session_id in &sessions {
        let meta = store.get_metadata(session_id).await?;
        println!("  - {}: {} messages, ~{} tokens",
            session_id, meta.message_count, meta.estimated_tokens);
    }
    println!();

    // Example 4: Session Lifecycle
    println!("=== Example 4: Session Lifecycle ===");
    let lifecycle_store = InMemoryStore::new();

    // Create
    let conv = Conversation::builder()
        .system("Session lifecycle demo")
        .user("Initial message")
        .build();
    lifecycle_store.save("lifecycle", conv).await?;
    println!("✓ Created session");

    // Update
    let mut loaded = lifecycle_store.load("lifecycle").await?;
    loaded.add_user("Updated message");
    lifecycle_store.save("lifecycle", loaded).await?;
    println!("✓ Updated session");

    // Read
    let reloaded = lifecycle_store.load("lifecycle").await?;
    println!("✓ Read session: {} messages", reloaded.len());

    // Delete
    lifecycle_store.delete("lifecycle").await?;
    println!("✓ Deleted session");

    assert!(!lifecycle_store.exists("lifecycle").await?);
    println!("✓ Session no longer exists");
    println!();

    // Example 5: Capacity Limits
    println!("=== Example 5: Capacity Limits ===");
    let limited_store = InMemoryStore::with_capacity(3);

    for i in 1..=3 {
        limited_store.save(&format!("s{}", i), Conversation::new()).await?;
    }
    println!("✓ Stored 3 sessions (at capacity)");

    // Try to store a 4th - should fail
    let result = limited_store.save("s4", Conversation::new()).await;
    match result {
        Err(agent_session::SessionError::CapacityExceeded) => {
            println!("✓ Capacity limit enforced (rejected 4th session)");
        }
        _ => println!("✗ Expected capacity error"),
    }
    println!();

    // Example 6: Clear All
    println!("=== Example 6: Clear All Sessions ===");
    let clear_store = InMemoryStore::new();

    clear_store.save("s1", Conversation::new()).await?;
    clear_store.save("s2", Conversation::new()).await?;
    clear_store.save("s3", Conversation::new()).await?;

    println!("✓ Created 3 sessions");

    let cleared = clear_store.clear().await?;
    println!("✓ Cleared {} sessions", cleared);

    assert!(clear_store.is_empty());
    println!("✓ Store is now empty");
    println!();

    println!("✅ All examples complete!");

    Ok(())
}

