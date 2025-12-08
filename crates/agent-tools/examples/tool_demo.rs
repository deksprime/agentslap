//! Tool System Demo
//!
//! Demonstrates the tool calling system with built-in tools.
//!
//! Run with:
//! ```bash
//! cargo run -p agent-tools --example tool_demo
//! ```

use agent_tools::{builtin::*, ToolRegistry, ToolResult};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️  Tool System Demo\n");

    // Create registry and register built-in tools
    let registry = ToolRegistry::new();
    
    registry.register(CalculatorTool)?;
    registry.register(EchoTool)?;
    registry.register(CurrentTimeTool)?;

    println!("=== Registered Tools ===");
    for tool_name in registry.list_tools() {
        if let Some(tool) = registry.get_tool(&tool_name) {
            println!("  • {}: {}", tool.name(), tool.description());
        }
    }
    println!("Total tools: {}\n", registry.count());

    // Example 1: Calculator - Addition
    println!("=== Example 1: Calculator (Add) ===");
    let params = json!({
        "operation": "add",
        "a": 15.5,
        "b": 7.3,
    });

    let result = registry.execute("calculator", params).await?;
    print_result(&result);

    // Example 2: Calculator - Division
    println!("=== Example 2: Calculator (Divide) ===");
    let params = json!({
        "operation": "divide",
        "a": 100.0,
        "b": 4.0,
    });

    let result = registry.execute("calculator", params).await?;
    print_result(&result);

    // Example 3: Calculator - Division by Zero (Error Handling)
    println!("=== Example 3: Calculator (Division by Zero) ===");
    let params = json!({
        "operation": "divide",
        "a": 10.0,
        "b": 0.0,
    });

    let result = registry.execute("calculator", params).await?;
    print_result(&result);

    // Example 4: Echo Tool
    println!("=== Example 4: Echo ===");
    let params = json!({
        "text": "Hello from the tool system!",
    });

    let result = registry.execute("echo", params).await?;
    print_result(&result);

    // Example 5: Current Time
    println!("=== Example 5: Current Time ===");
    let params = json!({});

    let result = registry.execute("current_time", params).await?;
    print_result(&result);

    // Example 6: Tool Schema (for LLM integration)
    println!("=== Example 6: Tool Schemas for LLM ===");
    
    println!("\nOpenAI Function Format:");
    let openai_funcs = registry.to_openai_functions();
    for func in &openai_funcs {
        println!("{}", serde_json::to_string_pretty(func)?);
    }

    println!("\nAnthropic Tool Format:");
    let anthropic_tools = registry.to_anthropic_tools();
    for tool in anthropic_tools.iter().take(1) {
        println!("{}", serde_json::to_string_pretty(tool)?);
    }

    // Example 7: Invalid Parameters
    println!("\n=== Example 7: Invalid Parameters ===");
    let invalid_params = json!({
        "operation": "add",
        "x": 5.0,  // Wrong field name
        "y": 3.0,
    });

    match registry.execute("calculator", invalid_params).await {
        Ok(_) => println!("✗ Should have failed"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    // Example 8: Nonexistent Tool
    println!("\n=== Example 8: Nonexistent Tool ===");
    match registry.execute("nonexistent", json!({})).await {
        Ok(_) => println!("✗ Should have failed"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    println!("\n✅ Tool system demo complete!");

    Ok(())
}

fn print_result(result: &ToolResult) {
    if result.success {
        println!("✓ Success!");
        if let Some(data) = &result.data {
            println!("  Result: {}", serde_json::to_string_pretty(data).unwrap());
        }
    } else {
        println!("✗ Error: {}", result.error.as_ref().unwrap());
    }
    println!();
}

