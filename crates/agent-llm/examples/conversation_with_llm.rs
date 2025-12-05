//! Conversation with LLM Integration
//!
//! This example demonstrates using conversations with actual LLM providers
//! to maintain context across multiple turns.
//!
//! Run with:
//! ```bash
//! OPENAI_API_KEY=your-key cargo run -p agent-llm --example conversation_with_llm
//! # or
//! ANTHROPIC_API_KEY=your-key cargo run -p agent-llm --example conversation_with_llm -- anthropic
//! ```

use agent_llm::{create_provider, Conversation, LLMProvider};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💬 Multi-Turn Conversation with LLM\n");

    // Determine provider from command line argument
    let args: Vec<String> = env::args().collect();
    let provider_name = args.get(1).map(|s| s.as_str()).unwrap_or("openai");

    // Create provider
    let provider: Box<dyn LLMProvider> = match provider_name {
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY environment variable not set");
            let model = "gpt-3.5-turbo";
            println!("Using OpenAI: {}", model);
            create_provider("openai", &api_key, model)?
        }
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY environment variable not set");
            let model = "claude-sonnet-4-5";
            println!("Using Anthropic: {}", model);
            create_provider("anthropic", &api_key, model)?
        }
        _ => {
            eprintln!("Unknown provider: {}", provider_name);
            std::process::exit(1);
        }
    };

    // Build a conversation with context
    let mut conversation = Conversation::builder()
        .system("You are a helpful assistant. Keep responses concise (2-3 sentences max).")
        .user("What is Rust programming language?")
        .build();

    println!("=== Turn 1 ===");
    println!("User: What is Rust programming language?");

    // Send first message
    let response1 = provider.send_message(conversation.messages().to_vec()).await?;
    println!("Assistant: {}\n", response1.content);

    // Add response to conversation
    conversation.add_assistant(&response1.content);

    // Show conversation state
    println!("📊 Conversation state:");
    println!("   Messages: {}", conversation.len());
    println!("   Tokens: ~{}\n", conversation.estimate_tokens());

    // Second turn - follow-up question
    conversation.add_user("What makes it different from C++?");

    println!("=== Turn 2 ===");
    println!("User: What makes it different from C++?");

    let response2 = provider.send_message(conversation.messages().to_vec()).await?;
    println!("Assistant: {}\n", response2.content);

    conversation.add_assistant(&response2.content);

    println!("📊 Conversation state:");
    println!("   Messages: {}", conversation.len());
    println!("   Tokens: ~{}\n", conversation.estimate_tokens());

    // Third turn - another follow-up
    conversation.add_user("Can you give me a simple code example?");

    println!("=== Turn 3 ===");
    println!("User: Can you give me a simple code example?");

    let response3 = provider.send_message(conversation.messages().to_vec()).await?;
    println!("Assistant: {}\n", response3.content);

    conversation.add_assistant(&response3.content);

    // Final conversation state
    println!("📊 Final conversation state:");
    println!("   ID: {}", conversation.id);
    println!("   Messages: {}", conversation.len());
    println!("   Estimated tokens: ~{}", conversation.estimate_tokens());

    // Show token usage if available
    if let Some(usage) = response3.usage {
        println!("\n💰 Token usage (last response):");
        println!("   Prompt: {} tokens", usage.prompt_tokens);
        println!("   Completion: {} tokens", usage.completion_tokens);
        println!("   Total: {} tokens", usage.total_tokens);
    }

    // Save conversation to JSON
    let json = serde_json::to_string_pretty(&conversation)?;
    println!("\n📝 Conversation JSON:");
    println!("{}", json);

    // Demonstrate context window management
    println!("\n🔧 Context Window Management:");
    let mut demo_conv = conversation.clone();
    println!("   Original: {} messages, ~{} tokens", 
        demo_conv.len(), demo_conv.estimate_tokens());

    demo_conv.truncate_to_tokens(100);
    println!("   Truncated: {} messages, ~{} tokens", 
        demo_conv.len(), demo_conv.estimate_tokens());
    println!("   System message preserved: {}", demo_conv.system_message().is_some());

    println!("\n✅ Multi-turn conversation complete!");

    Ok(())
}

