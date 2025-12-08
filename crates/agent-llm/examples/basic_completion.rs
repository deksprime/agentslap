//! Basic LLM completion example
//!
//! This example demonstrates how to use the LLM provider abstraction
//! to send messages and receive responses.
//!
//! Run with:
//! ```bash
//! # For OpenAI
//! OPENAI_API_KEY=your-key cargo run -p agent-llm --example basic_completion
//!
//! # For Anthropic
//! ANTHROPIC_API_KEY=your-key cargo run -p agent-llm --example basic_completion -- anthropic
//! ```

use agent_llm::{create_provider, LLMProvider, Message};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Determine provider from command line argument
    let args: Vec<String> = env::args().collect();
    let provider_name = args.get(1).map(|s| s.as_str()).unwrap_or("openai");

    println!("🤖 Agent LLM Example - Basic Completion");
    println!("Provider: {}\n", provider_name);

    // Create provider based on selection
    let provider: Box<dyn LLMProvider> = match provider_name {
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY environment variable not set");
            let model = "gpt-3.5-turbo";
            println!("Using OpenAI model: {}", model);
            create_provider("openai", &api_key, model)?
        }
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY environment variable not set");
            let model = "claude-sonnet-4-5"; // Latest Claude Sonnet 4.5 (Sept 2025)
            println!("Using Anthropic model: {}", model);
            create_provider("anthropic", &api_key, model)?
        }
        _ => {
            eprintln!("Unknown provider: {}. Use 'openai' or 'anthropic'", provider_name);
            std::process::exit(1);
        }
    };

    // Create a simple conversation
    let messages = vec![
        Message::system("You are a helpful assistant that provides concise answers."),
        Message::user("What is Rust programming language in one sentence?"),
    ];

    println!("Sending message...\n");

    // Send message and get response
    match provider.send_message(messages).await {
        Ok(response) => {
            println!("✅ Response received:");
            println!("Model: {}", response.model);
            println!("Content: {}\n", response.content);

            if let Some(usage) = response.usage {
                println!("Token usage:");
                println!("  Prompt: {} tokens", usage.prompt_tokens);
                println!("  Completion: {} tokens", usage.completion_tokens);
                println!("  Total: {} tokens", usage.total_tokens);
            }

            if let Some(reason) = response.finish_reason {
                println!("Finish reason: {}", reason);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

