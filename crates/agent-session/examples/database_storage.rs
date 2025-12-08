//! Database Storage Example
//!
//! Demonstrates persistent storage using SQLite.
//! Sessions survive application restarts!
//!
//! Run with:
//! ```bash
//! cargo run -p agent-session --features database --example database_storage
//! ```

#[cfg(feature = "database")]
use agent_llm::Conversation;
#[cfg(feature = "database")]
use agent_session::{DatabaseStore, SessionStore};
#[cfg(feature = "database")]
use std::env;

#[cfg(feature = "database")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Database Storage Demo\n");

    // Get database path (use temp for demo)
    let db_path = env::temp_dir().join("agent_demo_sessions.db");
    println!("Database: {}\n", db_path.display());

    // Create database store
    let store = DatabaseStore::new(&db_path).await?;
    println!("✓ Database initialized");
    println!("✓ Migrations applied");
    println!("✓ WAL mode enabled\n");

    // Create conversations
    let conv1 = Conversation::builder()
        .system("You are a helpful assistant")
        .user("What is Rust?")
        .assistant("Rust is a systems programming language...")
        .build();

    let conv2 = Conversation::builder()
        .system("You are a coding expert")
        .user("How do I use async/await?")
        .build();

    // Save sessions
    println!("=== Saving Sessions ===");
    store.save("session-1", conv1).await?;
    println!("✓ Saved session-1");

    store.save("session-2", conv2).await?;
    println!("✓ Saved session-2\n");

    // Get stats
    println!("=== Database Statistics ===");
    let stats = store.stats().await?;
    println!("Total sessions: {}", stats.session_count);
    println!("Total messages: {}", stats.total_messages);
    println!("Total tokens: ~{}\n", stats.total_tokens);

    // List sessions
    println!("=== All Sessions ===");
    let sessions = store.list_sessions().await?;
    for session_id in &sessions {
        let meta = store.get_metadata(session_id).await?;
        println!("  • {}: {} messages, ~{} tokens",
            session_id, meta.message_count, meta.estimated_tokens);
    }
    println!();

    // Simulate restart - close and reopen database
    println!("=== Simulating Application Restart ===");
    drop(store);
    println!("✓ Closed database\n");

    println!("=== Reopening Database ===");
    let store = DatabaseStore::new(&db_path).await?;
    println!("✓ Database reopened\n");

    // Sessions should still be there!
    println!("=== Loading Persisted Sessions ===");
    let loaded = store.load("session-1").await?;
    println!("✓ Loaded session-1: {} messages", loaded.len());

    assert!(store.exists("session-2").await?);
    println!("✓ session-2 still exists\n");

    // Update a session
    println!("=== Updating Session ===");
    let mut conv = store.load("session-1").await?;
    conv.add_user("Follow-up question");
    store.save("session-1", conv).await?;
    println!("✓ Updated session-1\n");

    // Verify update persisted
    let reloaded = store.load("session-1").await?;
    println!("✓ Verified: now has {} messages\n", reloaded.len());

    // Cleanup
    println!("=== Cleanup ===");
    let cleared = store.clear().await?;
    println!("✓ Cleared {} sessions", cleared);

    // Vacuum database
    store.vacuum().await?;
    println!("✓ Database vacuumed\n");

    println!("✅ Database storage demo complete!");
    println!("\nDatabase file remains at: {}", db_path.display());
    println!("You can inspect it with: sqlite3 {}", db_path.display());

    Ok(())
}

#[cfg(not(feature = "database"))]
fn main() {
    eprintln!("ERROR: This example requires the 'database' feature");
    eprintln!("Run with: cargo run -p agent-session --features database --example database_storage");
    std::process::exit(1);
}


