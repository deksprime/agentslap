//! Streaming LLM completion example
//!
//! This example demonstrates how to use streaming to receive
//! responses as they're generated.
//!
//! Run with:
//! ```bash
//! # For OpenAI
//! OPENAI_API_KEY=your-key cargo run -p agent-llm --example streaming_completion
//!
//! # For Anthropic  
//! ANTHROPIC_API_KEY=your-key cargo run -p agent-llm --example streaming_completion -- anthropic
//! ```

use agent_llm::{create_provider, LLMProvider, Message};
use futures::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Determine provider from command line argument
    let args: Vec<String> = env::args().collect();
    let provider_name = args.get(1).map(|s| s.as_str()).unwrap_or("openai");

    println!("🤖 Agent LLM Example - Streaming Completion");
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

    // Create a conversation
    let messages = vec![
        Message::system("You are a creative storyteller."),
        Message::user("Tell me a very short story about a robot learning to code in Rust."),
    ];

    println!("Streaming response...\n");
    println!("---");

    // Stream the response
    let mut stream = provider.stream_message(messages).await?;

    let mut full_content = String::new();
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if !chunk.content.is_empty() {
                    print!("{}", chunk.content);
                    std::io::Write::flush(&mut std::io::stdout())?;
                    full_content.push_str(&chunk.content);
                }

                if chunk.is_final() {
                    println!("\n---\n");
                    if let Some(reason) = chunk.finish_reason {
                        println!("Stream finished: {}", reason);
                    }
                    break;
                }
            }
            Err(e) => {
                eprintln!("\n❌ Streaming error: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\n✅ Streaming complete!");
    println!("Total characters received: {}", full_content.len());

    Ok(())
}

