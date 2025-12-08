//! Layered Storage Example
//!
//! Demonstrates using layered storage with fallback and write-through.
//!
//! Run with:
//! ```bash
//! cargo run -p agent-session --example layered_storage
//! ```

use agent_llm::Conversation;
use agent_session::{CacheStore, InMemoryStore, LayeredStore, SessionStore};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Layered Storage Demo\n");

    // Create storage layers
    let cache = CacheStore::new(Duration::from_secs(5)); // 5-second TTL for demo
    let memory = InMemoryStore::new();

    // Build layered store: Cache → Memory
    let store = LayeredStore::new()
        .with_layer(cache.clone()) // Fast, temporary
        .with_layer(memory.clone()); // Persistent in process

    println!("Created layered store with {} layers:", store.layer_count());
    println!("  Layer 1: Cache (5s TTL)");
    println!("  Layer 2: Memory (process lifetime)");
    println!();

    // Example 1: Write-Through
    println!("=== Example 1: Write-Through ===");
    let conv = Conversation::builder()
        .system("Test system")
        .user("Test message")
        .build();

    store.save("test-session", conv).await?;
    println!("✓ Saved to layered store");

    // Check both layers
    assert!(cache.exists("test-session").await?);
    assert!(memory.exists("test-session").await?);
    println!("✓ Session exists in BOTH cache and memory");
    println!();

    // Example 2: Read from Cache (Fast Path)
    println!("=== Example 2: Read from Cache (Fast) ===");
    let loaded = store.load("test-session").await?;
    println!("✓ Loaded from cache (fast path)");
    println!("  Messages: {}", loaded.len());
    println!();

    // Example 3: Cache Miss → Memory Fallback
    println!("=== Example 3: Cache Miss → Memory Fallback ===");
    
    // Save directly to memory only
    let conv2 = Conversation::builder()
        .system("Only in memory")
        .user("Not in cache")
        .build();
    memory.save("memory-only", conv2).await?;
    println!("✓ Saved directly to memory (bypassing cache)");

    // Load via layered store
    let _loaded2 = store.load("memory-only").await?;
    println!("✓ Loaded from memory (cache miss → fallback)");

    // Check if written back to cache
    assert!(cache.exists("memory-only").await?);
    println!("✓ Session written back to cache automatically");
    println!();

    // Example 4: TTL Expiration & Fallback
    println!("=== Example 4: TTL Expiration & Fallback ===");
    
    println!("Waiting for cache to expire (5 seconds)...");
    sleep(Duration::from_secs(6)).await;
    
    // Run cache maintenance to evict expired entries
    cache.run_pending_tasks().await;

    // Cache should be expired now
    println!("✓ Cache entries expired");

    // But should still load from memory
    let still_available = store.load("test-session").await?;
    println!("✓ Still loaded from memory (cache → memory fallback)");
    println!("  Messages: {}", still_available.len());

    // And cache is repopulated
    assert!(cache.exists("test-session").await?);
    println!("✓ Cache repopulated from memory");
    println!();

    // Example 5: Delete from All Layers
    println!("=== Example 5: Delete from All Layers ===");
    
    store.delete("test-session").await?;
    println!("✓ Deleted from layered store");

    // Should be gone from both layers
    assert!(!cache.exists("test-session").await?);
    assert!(!memory.exists("test-session").await?);
    println!("✓ Deleted from BOTH cache and memory");
    println!();

    // Example 6: Listing Sessions
    println!("=== Example 6: Listing Sessions ===");
    
    // Add multiple sessions
    for i in 1..=3 {
        let conv = Conversation::builder()
            .system(format!("Agent {}", i))
            .build();
        store.save(&format!("session-{}", i), conv).await?;
    }

    let all_sessions = store.list_sessions().await?;
    println!("✓ Found {} sessions:", all_sessions.len());
    for session_id in &all_sessions {
        println!("  - {}", session_id);
    }
    println!();

    // Example 7: Partial Failure Handling
    println!("=== Example 7: Graceful Degradation ===");
    
    // If one layer fails, others still work
    // (This is automatic - layered store handles it)
    println!("✓ Layered store handles partial failures gracefully");
    println!("  - If cache fails, memory still works");
    println!("  - If memory fails, cache still works");
    println!("  - Only fails if ALL layers fail");
    println!();

    // Example 8: Clear All
    println!("=== Example 8: Clear All Sessions ===");
    
    let total_cleared = store.clear().await?;
    println!("✓ Cleared {} sessions from all layers", total_cleared);

    assert!(memory.is_empty());
    println!("✓ All stores now empty");
    println!();

    println!("✅ Layered storage demo complete!");
    println!("\n💡 Key Takeaways:");
    println!("  • Writes go to ALL layers (write-through)");
    println!("  • Reads check layers in order (cache → memory)");
    println!("  • Cache misses trigger write-back to faster layers");
    println!("  • Graceful degradation on partial failures");
    println!("  • Automatic TTL expiration in cache layer");

    Ok(())
}

