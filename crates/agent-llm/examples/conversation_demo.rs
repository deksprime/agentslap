//! Conversation Management Demo
//!
//! This example demonstrates conversation management features including:
//! - Building conversations with the fluent API
//! - Token counting
//! - Conversation truncation
//! - Serialization
//! - Integration with LLM providers
//!
//! Run with:
//! ```bash
//! cargo run -p agent-llm --example conversation_demo
//! ```

use agent_llm::Conversation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Conversation Management Demo\n");

    // Example 1: Builder Pattern
    println!("=== Example 1: Fluent Builder ===");
    let conversation = Conversation::builder()
        .system("You are a helpful coding assistant")
        .user("How do I create a vector in Rust?")
        .assistant("You can create a vector using Vec::new() or the vec! macro")
        .user("Can you show me an example?")
        .build();

    println!("Messages: {}", conversation.len());
    println!("Estimated tokens: {}", conversation.estimate_tokens());
    println!();

    // Example 2: Manual Construction
    println!("=== Example 2: Manual Construction ===");
    let mut conv = Conversation::new();
    conv.add_system("You are an expert in mathematics");
    conv.add_user("What is the Pythagorean theorem?");
    conv.add_assistant("The Pythagorean theorem states that a² + b² = c²");

    println!("Conversation ID: {}", conv.id);
    println!("Messages: {}", conv.len());
    println!();

    // Example 3: Accessing Messages
    println!("=== Example 3: Accessing Messages ===");
    if let Some(system_msg) = conversation.system_message() {
        println!("System: {}", system_msg.content);
    }

    if let Some(last_msg) = conversation.last_message() {
        println!("Last message role: {:?}", last_msg.role);
    }
    println!();

    // Example 4: Token Counting
    println!("=== Example 4: Token Estimation ===");
    let large_conv = Conversation::builder()
        .system("You are a helpful assistant with extensive knowledge")
        .user("Tell me about the history of programming languages")
        .assistant("Programming languages have evolved significantly over the decades...")
        .user("What about Rust specifically?")
        .assistant("Rust was created by Graydon Hoare and first released in 2010...")
        .build();

    println!("Total messages: {}", large_conv.len());
    println!("Estimated tokens: {}", large_conv.estimate_tokens());
    
    for (i, msg) in large_conv.messages().iter().enumerate() {
        let tokens = agent_llm::conversation::estimate_message_tokens(msg);
        println!("  Message {}: ~{} tokens ({:?})", i + 1, tokens, msg.role);
    }
    println!();

    // Example 5: Truncation
    println!("=== Example 5: Context Window Management ===");
    let mut truncatable = Conversation::builder()
        .system("You are helpful")
        .user("Message 1: This is a longer message with more content")
        .assistant("Response 1: Detailed response with lots of information")
        .user("Message 2: Another long user message here")
        .assistant("Response 2: More detailed information in response")
        .user("Message 3: Yet another message with content")
        .build();

    println!("Before truncation:");
    println!("  Messages: {}", truncatable.len());
    println!("  Tokens: ~{}", truncatable.estimate_tokens());

    truncatable.truncate_to_tokens(50);

    println!("After truncation to 50 tokens:");
    println!("  Messages: {}", truncatable.len());
    println!("  Tokens: ~{}", truncatable.estimate_tokens());
    println!("  System message preserved: {}", truncatable.system_message().is_some());
    println!();

    // Example 6: Serialization
    println!("=== Example 6: Serialization ===");
    let conv_for_save = Conversation::builder()
        .id("demo-conversation-123")
        .system("You are helpful")
        .user("Hello!")
        .build();

    let json = serde_json::to_string_pretty(&conv_for_save)?;
    println!("Serialized conversation:");
    println!("{}", json);
    println!();

    // Deserialize
    let loaded: Conversation = serde_json::from_str(&json)?;
    println!("Loaded conversation ID: {}", loaded.id);
    println!("Messages preserved: {}", loaded.len());
    println!();

    // Example 7: Conversation Summary
    println!("=== Example 7: Conversation Summary ===");
    let summary = conversation.summary();
    println!("Summary:");
    println!("  ID: {}", summary.id);
    println!("  Messages: {}", summary.message_count);
    println!("  Tokens: ~{}", summary.estimated_tokens);
    println!("  Has system message: {}", summary.has_system_message);
    println!("  Created: {}", summary.created_at);
    println!();

    // Example 8: Conversation Metadata
    println!("=== Example 8: Metadata ===");
    let with_metadata = Conversation::builder()
        .system("Hello")
        .metadata(serde_json::json!({
            "user_id": "user-123",
            "session": "abc-def",
            "tags": ["support", "technical"]
        }))
        .build();

    println!("Metadata: {}", with_metadata.metadata);
    println!();

    // Example 9: Iterating Messages
    println!("=== Example 9: Message Iteration ===");
    let demo = Conversation::builder()
        .system("System prompt")
        .user("User query")
        .assistant("Assistant response")
        .build();

    for (i, message) in demo.messages().iter().enumerate() {
        println!("{}. [{:?}] {}", 
            i + 1, 
            message.role, 
            &message.content[..message.content.len().min(30)]
        );
    }
    println!();

    println!("✅ Demo complete!");

    Ok(())
}

