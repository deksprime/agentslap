//! Guardrails Demo
//!
//! Demonstrates all guardrails and the chain system.
//!
//! Run with:
//! ```bash
//! cargo run -p agent-guardrails --example guardrails_demo
//! ```

use agent_guardrails::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Guardrails System Demo\n");

    // Create individual guardrails
    let content_filter = ContentFilter::new(vec![
        "badword".to_string(),
        "inappropriate".to_string(),
    ]);

    let rate_limiter = RateLimiter::new(5, Duration::from_secs(60));
    
    let cost_limiter = CostLimiter::new(1000).warn_threshold(0.8);
    
    let tool_allowlist = ToolAllowlist::new(vec![
        "calculator".to_string(),
        "echo".to_string(),
    ]);

    // Create guardrail chain
    let chain = GuardrailChain::new()
        .with_guardrail(content_filter)
        .with_guardrail(rate_limiter)
        .with_guardrail(cost_limiter)
        .with_guardrail(tool_allowlist);

    println!("Created guardrail chain with {} guardrails\n", chain.len());

    // Test 1: Clean input (should pass)
    println!("=== Test 1: Clean Input ===");
    let ctx = guardrail::InputContext {
        message: "What is 2 + 2?".to_string(),
        user_id: Some("user1".to_string()),
        session_id: Some("session1".to_string()),
    };

    let violations = chain.check_input(&ctx).await?;
    if violations.is_empty() {
        println!("✓ Input passed all guardrails\n");
    } else {
        println!("✗ Violations: {:?}\n", violations);
    }

    // Test 2: Blocked content
    println!("=== Test 2: Blocked Content ===");
    let ctx = guardrail::InputContext {
        message: "This contains badword in it".to_string(),
        user_id: Some("user1".to_string()),
        session_id: None,
    };

    let violations = chain.check_input(&ctx).await?;
    for v in &violations {
        println!("✗ Violation:");
        println!("  Guardrail: {}", v.guardrail);
        println!("  Severity: {:?}", v.severity);
        println!("  Message: {}", v.message);
        println!("  Action: {:?}", v.action);
    }
    println!();

    // Test 3: Rate limiting
    println!("=== Test 3: Rate Limiting ===");
    let ctx = guardrail::InputContext {
        message: "request".to_string(),
        user_id: Some("user2".to_string()),
        session_id: None,
    };

    for i in 1..=7 {
        let violations = chain.check_input(&ctx).await?;
        if violations.is_empty() {
            println!("  Request {}: ✓ Allowed", i);
        } else {
            println!("  Request {}: ✗ Rate limited", i);
        }
    }
    println!();

    // Test 4: Cost limiting
    println!("=== Test 4: Cost Limiting ===");
    let limiter = CostLimiter::new(100).warn_threshold(0.8);
    
    // Simulate token usage
    limiter.reset("session_cost");
    println!("Budget: 100 tokens, Warn at: 80%");
    println!("Current usage: {} tokens\n", limiter.get_usage("session_cost"));

    // Simulate approaching limit
    for usage in &[0, 50, 85, 100, 105] {
        // Reset and set usage (would be tracked by check_output in real usage)
        limiter.reset("session_cost");
        // Simulate the usage
        for _ in 0..*usage {
            limiter.usage.insert("session_cost".to_string(), *usage);
        }
        
        let ctx = guardrail::InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: Some("session_cost".to_string()),
        };

        let violation = limiter.check_input(&ctx).await?;
        if violation.is_none() {
            println!("  {} tokens: ✓ OK", usage);
        } else {
            let v = violation.unwrap();
            println!("  {} tokens: {:?} - {}", usage, v.action, v.message);
        }
    }
    println!();

    // Test 5: Tool allowlist
    println!("=== Test 5: Tool Allowlist ===");
    let allowlist = ToolAllowlist::new(vec!["calculator".to_string()]);
    
    let allowed_ctx = guardrail::ToolCallContext {
        tool_name: "calculator".to_string(),
        parameters: serde_json::json!({}),
        user_id: None,
    };

    let blocked_ctx = guardrail::ToolCallContext {
        tool_name: "delete_file".to_string(),
        parameters: serde_json::json!({}),
        user_id: None,
    };

    let v1 = allowlist.check_tool_call(&allowed_ctx).await?;
    if v1.is_none() {
        println!("✓ calculator: Allowed");
    }

    let v2 = allowlist.check_tool_call(&blocked_ctx).await?;
    if let Some(v) = v2 {
        println!("✗ delete_file: Blocked - {}", v.message);
    }
    println!();

    // Test 6: Fail-fast vs Collect-all
    println!("=== Test 6: Chain Modes ===");
    
    let ctx = guardrail::InputContext {
        message: "Contains badword".to_string(),
        user_id: Some("user3".to_string()),
        session_id: None,
    };

    // Fail-fast
    let chain_failfast = GuardrailChain::new()
        .with_guardrail(ContentFilter::new(vec!["badword".to_string()]))
        .with_guardrail(ContentFilter::new(vec!["other".to_string()]))
        .fail_fast(true);

    let violations = chain_failfast.check_input(&ctx).await?;
    println!("Fail-fast mode: {} violation(s) (stops at first block)", violations.len());

    // Collect-all
    let chain_collect = GuardrailChain::new()
        .with_guardrail(ContentFilter::new(vec!["badword".to_string()]))
        .with_guardrail(ContentFilter::new(vec!["badword".to_string()]))
        .fail_fast(false);

    let violations = chain_collect.check_input(&ctx).await?;
    println!("Collect-all mode: {} violation(s) (collects all)\n", violations.len());

    println!("✅ Guardrails demo complete!");

    Ok(())
}

