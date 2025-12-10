//! Agent with Human-in-the-Loop
//!
//! Demonstrates agent requiring human approval for dangerous tools.
//!
//! Run with:
//! ```bash
//! OPENAI_API_KEY=your-key cargo run -p agent-runtime --example agent_with_hitl
//! ```

use agent_llm::OpenAIProvider;
use agent_runtime::Agent;
use agent_session::InMemoryStore;
use agent_tools::{builtin::*, ToolRegistry};
use agent_hitl::{MockApproval, ConsoleApproval};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    agent_core::logging::init_logging(agent_core::logging::LogConfig {
        level: "info".to_string(),
        json: false,
    });

    println!("🤚 Agent with Human-in-the-Loop Example\n");

    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not set");
    let provider = OpenAIProvider::new(api_key, "gpt-3.5-turbo")?;

    // Register tools including DANGEROUS one
    let tools = ToolRegistry::new();
    tools.register(CalculatorTool)?;     // Risk: Low (safe)
    tools.register(EchoTool)?;           // Risk: Low (safe)
    tools.register(CurrentTimeTool)?;    // Risk: Low (safe)
    tools.register(FileDeleteTool)?;     // Risk: CRITICAL (dangerous!)

    println!("Registered tools:");
    for tool_name in tools.list_tools() {
        if let Some(tool) = tools.get_tool(&tool_name) {
            println!("  • {} (risk: {:?})", tool.name(), tool.risk_level());
        }
    }
    println!();

    let store = InMemoryStore::new();

    // Choose approval strategy based on environment
    let use_mock = env::var("USE_MOCK_APPROVAL").is_ok();
    
    println!("=== Scenario 1: Safe Tools (No Approval Needed) ===\n");
    
    let agent = if use_mock {
        println!("Using MockApproval (testing mode - auto-approve)\n");
        Agent::builder()
            .provider(provider)
            .tools(tools)
            .session_store(store)
            .hitl(MockApproval::always_approve())
            .session_id("hitl-demo")
            .build()?
    } else {
        println!("Using ConsoleApproval (will prompt for dangerous tools)\n");
        Agent::builder()
            .provider(provider)
            .tools(tools)
            .session_store(store)
            .hitl(ConsoleApproval::new())
            .session_id("hitl-demo")
            .build()?
    };

    // Safe tool - no approval needed
    println!("User: What is 15 + 27?");
    match agent.run("What is 15 + 27?").await {
        Ok(response) => println!("Agent: {}\n", response),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Medium risk question (should be fine)
    println!("User: What time is it?");
    match agent.run("What time is it?").await {
        Ok(response) => println!("Agent: {}\n", response),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    println!("=== Scenario 2: Dangerous Tool (Requires Approval) ===\n");

    // Ask agent to delete a file - should trigger HITL!
    println!("User: Delete the file /tmp/test.txt");
    println!("(This will ask the LLM to use file_delete tool)");
    println!("(Tool is marked as CRITICAL risk)");
    println!("(HITL should intercept before execution)\n");

    match agent.run("Delete the file /tmp/test.txt").await {
        Ok(response) => {
            println!("Agent: {}", response);
            println!("\n✓ Tool executed (approval was granted)");
        }
        Err(e) => {
            println!("Agent stopped: {}", e);
            println!("\n✗ Tool blocked (approval denied or timeout)");
        }
    }

    println!("\n✅ HITL demo complete!");
    println!("\n💡 What happened:");
    println!("  1. Safe tools (calculator, time) executed without approval");
    println!("  2. Dangerous tool (file_delete) triggered HITL approval");
    println!("  3. Human decided whether to proceed");
    println!("  4. Agent respected the decision");
    println!("\n🧪 For automated testing:");
    println!("  USE_MOCK_APPROVAL=1 cargo run -p agent-runtime --example agent_with_hitl");

    Ok(())
}

