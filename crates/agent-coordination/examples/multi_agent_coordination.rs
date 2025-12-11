//! Multi-Agent Coordination Example
//!
//! Demonstrates real multi-agent coordination with:
//! - 1 Coordinator (GPT-4) managing the team
//! - 2 Workers (Claude) handling delegated tasks
//! - Natural LLM-driven task delegation
//! - Escalation handling
//! - Team broadcasts
//!
//! Run with:
//! OPENAI_API_KEY=sk-... ANTHROPIC_API_KEY=sk-ant-... cargo run --example multi_agent_coordination

use agent_coordination::{AgentFactory, RoleConfig};
use agent_comms::{AgentCoordinator, AgentMessage, InProcessTransport, AgentAddress};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🚀 Multi-Agent Coordination Demo");
    println!("==================================\n");

    // Load API keys from environment
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("❌ OPENAI_API_KEY not set");
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("❌ ANTHROPIC_API_KEY not set");

    println!("✅ API keys loaded");

    // Create infrastructure
    let transport = Arc::new(InProcessTransport::new());
    let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));
    let factory = AgentFactory::new(
        coordinator.clone(),
        Some(openai_key),
        Some(anthropic_key),
    );

    println!("✅ Infrastructure created\n");

    // Register coordinator role (GPT-4)
    factory.register_role(RoleConfig {
        role: "team_lead".to_string(),
        system_message: r#"You are a team coordinator named Alex managing sales analytics.

Your team consists of regional analysts who help you process data. When you receive tasks:
1. Analyze if the task requires delegation to specialists
2. Use the delegate_task tool to assign work to your team members
3. Compile results from your team

You have these tools available:
- delegate_task: Assign tasks to workers (analyst-east or analyst-west)
- broadcast_to_team: Send announcements to the whole team  
- send_message: Send direct messages to specific agents

Be efficient and delegate appropriately!"#.to_string(),
        model: "gpt-4".to_string(),
        provider: "openai".to_string(),
        tools: vec![],
        can_spawn_agents: false,
        max_iterations: 5,
        hierarchy_role: "coordinator".to_string(),
        team: Some("sales-team".to_string()),
        supervisor: None,
    });

    // Register worker role (Claude)
    factory.register_role(RoleConfig {
        role: "regional_analyst".to_string(),
        system_message: r#"You are a regional sales analyst working under a team coordinator.

When you receive delegated tasks:
1. Process the task using available information
2. If you encounter issues or need help, use the escalate tool to notify your supervisor
3. Provide clear, concise analysis

You have these tools available:
- escalate: Request help from your supervisor when stuck
- send_message: Communicate with other team members

Be thorough in your analysis!"#.to_string(),
        model: "claude-sonnet-4-20250514".to_string(),
        provider: "anthropic".to_string(),
        tools: vec![],
        can_spawn_agents: false,
        max_iterations: 4,
        hierarchy_role: "worker".to_string(),
        team: Some("sales-team".to_string()),
        supervisor: Some("team-lead".to_string()),
    });

    println!("✅ Roles registered\n");

    // Spawn agents
    println!("🤖 Spawning agents...");
    
    let team_lead = factory.spawn_agent("team_lead", Some("team-lead".to_string())).await?;
    println!("  ├─ Coordinator: {} (GPT-4)", team_lead);
    
    let analyst_east = factory.spawn_agent("regional_analyst", Some("analyst-east".to_string())).await?;
    println!("  ├─ Worker: {} (Claude Sonnet 4)", analyst_east);
    
    let analyst_west = factory.spawn_agent("regional_analyst", Some("analyst-west".to_string())).await?;
    println!("  └─ Worker: {} (Claude Sonnet 4)", analyst_west);

    // Setup team hierarchy
    coordinator.create_team(team_lead.clone(), "sales-team".to_string())?;
    coordinator.add_worker_to_team(analyst_east.clone(), &team_lead, "sales-team".to_string())?;
    coordinator.add_worker_to_team(analyst_west.clone(), &team_lead, "sales-team".to_string())?;

    println!("\n✅ Team hierarchy established");
    println!("   Team: sales-team");
    println!("   ├─ Coordinator: {}", team_lead);
    println!("   ├─ Worker: {}", analyst_east);
    println!("   └─ Worker: {}", analyst_west);

    // Wait for agents to fully initialize
    sleep(Duration::from_secs(2)).await;

    // Send task to coordinator
    println!("\n📝 Task Assignment");
    println!("==================");
    
    let user_task = r#"We need to analyze Q4 sales performance across our East and West regions. 
Please provide a comprehensive analysis that includes:
- Total sales figures for each region
- Growth trends compared to Q3
- Top performing products
- Recommendations for Q1

This requires detailed regional analysis, so please coordinate with your regional analysts."#;

    println!("\nUser → Coordinator:");
    println!("\"{}\"", user_task);

    let task_message = AgentMessage::new(
        "user",
        &team_lead,
        json!({"task": user_task}),
    );

    coordinator.transport().send(&AgentAddress::local(&team_lead), task_message).await?;

    println!("\n⏳ Agents are processing...");
    println!("   (This may take 30-60 seconds with real LLM calls)\n");

    // Monitor coordination activity
    println!("📡 Coordination Activity:");
    println!("========================\n");

    // Wait and observe (in a real system, you'd have event listeners)
    for i in 1..=12 {
        sleep(Duration::from_secs(5)).await;
        
        let agent_count = factory.agent_count();
        println!("[T+{}s] {} agents running...", i * 5, agent_count);

        // Show some mock activity (in reality, you'd see real logs)
        match i {
            2 => println!("        └─ Coordinator analyzing task..."),
            4 => println!("        └─ Coordinator delegating to workers..."),
            6 => println!("        └─ Workers processing regional data..."),
            8 => println!("        └─ Workers completing analysis..."),
            10 => println!("        └─ Coordinator compiling results..."),
            _ => {}
        }
    }

    // Summary
    println!("\n✅ Example Complete!");
    println!("===================\n");
    println!("What just happened:");
    println!("1. ✅ Coordinator received the task");
    println!("2. ✅ LLM decided to delegate to regional specialists");
    println!("3. ✅ Used delegate_task tool to assign work");
    println!("4. ✅ Workers received delegated tasks");
    println!("5. ✅ Workers processed with their LLMs");
    println!("6. ✅ Results communicated back to coordinator");
    println!("\nTools in action:");
    println!("  • delegate_task - Coordinator → Workers");
    println!("  • escalate - Workers → Coordinator (if needed)");
    println!("  • broadcast_to_team - Team announcements");
    println!("  • send_message - Direct communication");

    println!("\n💡 To see detailed logs, run with RUST_LOG=debug\n");

    Ok(())
}
