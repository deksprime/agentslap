//! Basic Agent Example
//!
//! Demonstrates creating and running an agent with all components.
//!
//! Run with:
//! ```bash
//! OPENAI_API_KEY=your-key cargo run -p agent-runtime --example basic_agent
//! # or
//! ANTHROPIC_API_KEY=your-key cargo run -p agent-runtime --example basic_agent -- anthropic
//! ```

use agent_llm::{OpenAIProvider, AnthropicProvider};
use agent_runtime::Agent;
use agent_session::InMemoryStore;
use agent_tools::{builtin::*, ToolRegistry};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    agent_core::logging::init_logging(agent_core::logging::LogConfig {
        level: "info".to_string(),
        json: false,
    });

    println!("🤖 Agent Runtime - Basic Example\n");

    // Determine provider from args
    let args: Vec<String> = env::args().collect();
    let provider_name = args.get(1).map(|s| s.as_str()).unwrap_or("openai");

    // Create tools
    let tools = ToolRegistry::new();
    tools.register(CalculatorTool)?;
    tools.register(EchoTool)?;
    tools.register(CurrentTimeTool)?;

    println!("Registered {} tools", tools.count());

    // Create session store
    let store = InMemoryStore::new();

    // Build agent (using concrete types to avoid Box issues)
    let agent = match provider_name {
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY not set");
            let provider = OpenAIProvider::new(api_key, "gpt-3.5-turbo")?;
            
            Agent::builder()
                .provider(provider)
                .tools(tools)
                .session_store(store)
                .session_id("demo-session")
                .system_message("You are a helpful assistant.")
                .max_iterations(5)
                .build()?
        }
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("ANTHROPIC_API_KEY not set");
            let provider = AnthropicProvider::new(api_key, "claude-sonnet-4-5")?;
            
            Agent::builder()
                .provider(provider)
                .tools(tools)
                .session_store(store)
                .session_id("demo-session")
                .system_message("You are a helpful assistant.")
                .max_iterations(5)
                .build()?
        }
        _ => {
            eprintln!("Unknown provider: {}", provider_name);
            std::process::exit(1);
        }
    };

    println!("Agent created successfully!\n");

    // Example conversations
    let questions = vec![
        "What is 15 + 27?",
        "What is the current time?",
        "Can you echo the word 'hello'?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("=== Question {} ===", i + 1);
        println!("User: {}", question);

        match agent.run(question).await {
            Ok(response) => {
                println!("Assistant: {}\n", response);
            }
            Err(e) => {
                eprintln!("Error: {}\n", e);
            }
        }
    }

    println!("✅ Agent demo complete!");

    Ok(())
}

