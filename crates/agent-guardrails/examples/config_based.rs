//! Configuration-Based Guardrails
//!
//! Shows how to configure guardrails from a config file/struct
//! instead of hardcoding in code.
//!
//! Run with:
//! ```bash
//! cargo run -p agent-guardrails --example config_based
//! ```

use agent_guardrails::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Configuration-Based Guardrails Demo\n");

    // Example 1: Load from code (simulating config file)
    println!("=== Example 1: From Configuration ===");
    
    let config = GuardrailConfig {
        enabled: true,
        content_filter: config::ContentFilterConfig {
            enabled: true,
            blocked_phrases: vec![
                "password".to_string(),
                "secret".to_string(),
                "api_key".to_string(),
            ],
            blocked_patterns: vec![
                r"\d{3}-\d{2}-\d{4}".to_string(),  // SSN
                r"sk-[a-zA-Z0-9]{32,}".to_string(), // API keys
            ],
            case_sensitive: false,
        },
        rate_limiter: config::RateLimiterConfig {
            enabled: true,
            max_requests: 10,
            window_seconds: 60,
        },
        cost_limiter: config::CostLimiterConfig {
            enabled: true,
            max_tokens: 5000,
            warn_threshold: 0.75,
        },
        tool_allowlist: config::ToolAllowlistConfig {
            enabled: true,
            allowed_tools: vec![
                "calculator".to_string(),
                "echo".to_string(),
                "current_time".to_string(),
            ],
        },
    };

    // Build guardrail chain from config
    let chain = config.build_chain()?;

    if let Some(chain) = chain {
        println!("✓ Built guardrail chain with {} guardrails\n", chain.len());

        // Test with safe input
        let ctx = guardrail::InputContext {
            message: "What is 2 + 2?".to_string(),
            user_id: Some("alice".to_string()),
            session_id: Some("session-1".to_string()),
        };

        let violations = chain.check_input(&ctx).await?;
        println!("Safe input: {} violations", violations.len());

        // Test with blocked content
        let ctx = guardrail::InputContext {
            message: "My password is secret123".to_string(),
            user_id: Some("bob".to_string()),
            session_id: None,
        };

        let violations = chain.check_input(&ctx).await?;
        println!("Blocked content: {} violation(s)", violations.len());
        for v in &violations {
            println!("  - {}: {}", v.guardrail, v.message);
        }
        println!();

        // Test with SSN pattern
        let ctx = guardrail::InputContext {
            message: "My SSN is 123-45-6789".to_string(),
            user_id: Some("charlie".to_string()),
            session_id: None,
        };

        let violations = chain.check_input(&ctx).await?;
        if !violations.is_empty() {
            println!("SSN detected: {} violation(s)", violations.len());
            for v in &violations {
                println!("  - {}", v.message);
            }
        }
        println!();

        // Test tool allowlist
        let allowed_tool = guardrail::ToolCallContext {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({}),
            user_id: None,
        };

        let blocked_tool = guardrail::ToolCallContext {
            tool_name: "delete_database".to_string(),
            parameters: serde_json::json!({}),
            user_id: None,
        };

        let v1 = chain.check_tool_call(&allowed_tool).await?;
        println!("calculator tool: {}", if v1.is_empty() { "✓ Allowed" } else { "✗ Blocked" });

        let v2 = chain.check_tool_call(&blocked_tool).await?;
        println!("delete_database tool: {}", if v2.is_empty() { "✓ Allowed" } else { "✗ Blocked" });
    } else {
        println!("No guardrails configured");
    }

    // Example 2: Serialization (save/load config)
    println!("\n=== Example 2: Configuration Serialization ===");
    
    let json = serde_json::to_string_pretty(&config)?;
    println!("Configuration as JSON:");
    println!("{}", json);

    println!("\n✅ Configuration-based guardrails demo complete!");
    println!("\n💡 In your app:");
    println!("  1. Load config from config.toml or environment");
    println!("  2. Call config.build_chain()");
    println!("  3. Add to Agent::builder().guardrails(chain)");
    println!("  4. All checks happen automatically!");

    Ok(())
}

