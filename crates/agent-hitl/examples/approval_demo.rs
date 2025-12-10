//! Approval Strategies Demo
//!
//! Demonstrates all HITL approval strategies.
//!
//! Run with:
//! ```bash
//! cargo run -p agent-hitl --example approval_demo
//! ```

use agent_hitl::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🤚 Human-in-the-Loop Demo\n");

    // Example 1: Auto-Approve (Testing/Trusted)
    println!("=== Example 1: AutoApprove ===");
    let auto = AutoApprove::new();
    
    let req = ApprovalRequest::new("Delete file", RiskLevel::High)
        .with_timeout(Duration::from_secs(10));

    let response = auto.request_approval(req).await?;
    println!("Response: {:?}", response);
    println!("✓ Auto-approved (use for testing/trusted environments)\n");

    // Example 2: Mock Approval (E2E Testing)
    println!("=== Example 2: MockApproval (Testing) ===");
    
    // Mock that approves first 2, then denies
    let mock = MockApproval::approve_n_times(2);
    
    for i in 1..=3 {
        let req = ApprovalRequest::new(format!("Action {}", i), RiskLevel::Medium)
            .with_timeout(Duration::from_secs(10));
        
        let response = mock.request_approval(req).await?;
        println!("Request {}: {:?}", i, response);
    }
    println!("✓ Mock approved 2, then denied (perfect for testing)\n");

    // Example 3: Callback Approval (Async/Remote)
    println!("=== Example 3: CallbackApproval (Async) ===");
    let callback = CallbackApproval::new();
    let callback_clone = callback.clone();

    let req = ApprovalRequest::new("Send email", RiskLevel::Medium)
        .with_context(serde_json::json!({
            "to": "user@example.com",
            "subject": "Test"
        }))
        .with_timeout(Duration::from_secs(2));

    let request_id = req.id.clone();
    println!("Request ID: {}", request_id);
    println!("Pending requests: {}", callback.pending_count());

    // Simulate external approval after delay
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("  [External system approving...]");
        callback_clone.submit_response(&request_id, ApprovalResponse::Approved).unwrap();
    });

    let response = callback.request_approval(req).await?;
    println!("Response: {:?}", response);
    println!("✓ Callback approval (for web apps, bots, webhooks)\n");

    // Example 4: Different Risk Levels
    println!("=== Example 4: Risk Levels ===");
    
    let risk_levels = vec![
        (RiskLevel::Low, "Read file"),
        (RiskLevel::Medium, "Write file"),
        (RiskLevel::High, "Delete file"),
        (RiskLevel::Critical, "Execute code"),
    ];

    for (risk, action) in risk_levels {
        let req = ApprovalRequest::new(action, risk);
        println!("{:?} risk: {}", risk, req.action);
    }
    println!("✓ Risk-based approval triggering\n");

    // Example 5: Audit Trail
    println!("=== Example 5: Audit Trail ===");
    let audit = ApprovalAudit::new();

    let req1 = ApprovalRequest::new("Action 1", RiskLevel::Low)
        .with_timeout(Duration::from_secs(10));
    let req1_id = req1.id.clone();
    
    audit.record(req1, ApprovalResponse::Approved, 150);

    let req2 = ApprovalRequest::new("Action 2", RiskLevel::High)
        .with_timeout(Duration::from_secs(10));
    
    audit.record(req2, ApprovalResponse::Denied { reason: Some("Too risky".to_string()) }, 200);

    println!("Total audit records: {}", audit.count());
    
    let record = audit.get(&req1_id).unwrap();
    println!("Record 1: {} ({}ms)", record.request.action, record.duration_ms);
    println!("✓ Full audit trail for compliance\n");

    // Example 6: Timeout Handling
    println!("=== Example 6: Timeout Handling ===");
    let callback_timeout = CallbackApproval::new();

    let req = ApprovalRequest::new("Time-sensitive action", RiskLevel::Medium)
        .with_timeout(Duration::from_millis(100));  // Very short timeout

    let response = callback_timeout.request_approval(req).await?;
    match response {
        ApprovalResponse::Timeout => println!("✓ Correctly timed out"),
        _ => println!("✗ Should have timed out"),
    }
    println!();

    // Example 7: Modified Response
    println!("=== Example 7: Modified Response ===");
    let mock_modify = MockApproval::always_modify(serde_json::json!({
        "modified": true,
        "new_param": "safer_value"
    }));

    let req = ApprovalRequest::new("Risky operation", RiskLevel::High)
        .with_context(serde_json::json!({"param": "dangerous_value"}))
        .with_timeout(Duration::from_secs(10));

    let response = mock_modify.request_approval(req).await?;
    match response {
        ApprovalResponse::Modified { modified_context } => {
            println!("✓ Human modified the request:");
            println!("  {}", serde_json::to_string_pretty(&modified_context)?);
        }
        _ => println!("✗ Expected modified response"),
    }
    println!();

    println!("✅ HITL demo complete!");
    println!("\n💡 Integration with Agent:");
    println!("  1. Add ApprovalStrategy to Agent::builder()");
    println!("  2. Agent checks before dangerous operations");
    println!("  3. Use MockApproval for automated E2E tests");
    println!("  4. Use CallbackApproval for production web apps");
    println!("  5. Use ConsoleApproval for CLI agents");

    Ok(())
}

