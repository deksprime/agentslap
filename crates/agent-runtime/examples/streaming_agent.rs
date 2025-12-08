//! Streaming Agent Example
//!
//! Demonstrates real-time streaming responses from the agent,
//! including tool calls and results.
//!
//! Run with:
//! ```bash
//! # Normal (info logging)
//! OPENAI_API_KEY=your-key cargo run -p agent-runtime --example streaming_agent
//!
//! # With debug logging to see stream chunks
//! RUST_LOG=debug OPENAI_API_KEY=your-key cargo run -p agent-runtime --example streaming_agent
//! ```

use agent_llm::OpenAIProvider;
use agent_runtime::{Agent, AgentEvent};
use agent_session::InMemoryStore;
use agent_tools::{builtin::*, ToolRegistry};
use futures::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set log level from env var or default to info
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    
    agent_core::logging::init_logging(agent_core::logging::LogConfig {
        level: log_level,
        json: false,
    });

    println!("🌊 Agent Streaming Example\n");

    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let provider = OpenAIProvider::new(api_key, "gpt-3.5-turbo")?;

    let tools = ToolRegistry::new();
    tools.register(CalculatorTool)?;
    tools.register(CurrentTimeTool)?;
    tools.register(EchoTool)?;

    let store = InMemoryStore::new();

    let agent = Agent::builder()
        .provider(provider)
        .tools(tools)
        .session_store(store)
        .session_id("streaming-demo")
        .system_message("You are a helpful assistant. Use tools when appropriate.")
        .max_iterations(5)
        .build()?;

    let questions = vec![
        "What is 123 + 456123123 and the echo of the answer?",
        "Tell me an exceptionally long joke about programming. This joke should be at least 1000 characters long.",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("=== Question {} ===", i + 1);
        println!("User: {}\n", question);
        println!("Assistant: ");

        let mut stream = agent.run_stream(question).await?;

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => match event {
                    AgentEvent::TextChunk { content } => {
                        print!("{}", content);
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    AgentEvent::ToolCallStart { tool_name, parameters } => {
                        println!("\n[Tool Call: {} with params: {}]", tool_name, parameters);
                    }
                    AgentEvent::ToolCallEnd { tool_name, success, result, error } => {
                        if success {
                            println!("[Tool Result: {} → {:?}]", tool_name, result);
                        } else {
                            println!("[Tool Error: {} → {:?}]", tool_name, error);
                        }
                    }
                    AgentEvent::Thinking { message } => {
                        println!("\n[Thinking: {}]", message);
                    }
                    AgentEvent::Done { total_tokens } => {
                        println!("\n[Done");
                        if let Some(tokens) = total_tokens {
                            print!(" - {} tokens", tokens);
                        }
                        println!("]");
                    }
                    AgentEvent::Error { message } => {
                        eprintln!("\n[Error: {}]", message);
                    }
                },
                Err(e) => {
                    eprintln!("\nStream error: {}", e);
                    break;
                }
            }
        }

        println!("\n");
    }

    println!("✅ Streaming demo complete!");

    Ok(())
}

