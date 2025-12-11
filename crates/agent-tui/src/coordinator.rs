//! Coordinator setup

use agent_comms::{AgentCoordinator, InProcessTransport};
use agent_coordination::{AgentFactory, RoleConfig};
use agent_hitl::AutoApprove;
use agent_llm::{AnthropicProvider, OpenAIProvider};
use agent_runtime::Agent;
use agent_session::InMemoryStore;
use anyhow::Result;
use std::sync::Arc;

pub async fn setup_coordinator(
    openai_key: String,
    anthropic_key: Option<String>,
) -> Result<Arc<Agent>> {
    // Create infrastructure
    let transport = Arc::new(InProcessTransport::new());
    let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));
    let factory = AgentFactory::new(
        coordinator.clone(),
        Some(openai_key.clone()),
        anthropic_key.clone(),
    );

    // Register coordinator role
    factory.register_role(RoleConfig {
        role: "coordinator".to_string(),
        system_message: r#"You are an intelligent coordinator agent managing a team of specialists.

Your capabilities:
- Analyze user requests and determine if they require delegation
- Delegate complex tasks to specialist workers using the delegate_task tool
- Answer simple questions directly
- Coordinate responses from multiple workers
- Provide comprehensive answers by synthesizing worker outputs

When to delegate:
- Complex analysis requiring specialized knowledge
- Tasks that can be parallelized across workers
- Data processing that benefits from distributed work

When to answer directly:
- Simple questions about your capabilities
- General knowledge queries
- Conversational responses

Be helpful, concise, and efficient. Use tools when appropriate."#.to_string(),
        model: "gpt-4".to_string(),
        provider: "openai".to_string(),
        tools: vec![],
        can_spawn_agents: false,
        max_iterations: 5,
        hierarchy_role: "coordinator".to_string(),
        team: Some("main-team".to_string()),
        supervisor: None,
    });

    // Spawn coordinator
    let agent_id = factory.spawn_agent("coordinator", Some("main-coordinator".to_string())).await?;
    
    // Build the agent for direct use
    let provider = OpenAIProvider::new(&openai_key, "gpt-4")?;
    let tools = agent_tools::ToolRegistry::new();
    
    // Register all tools
    tools.register(agent_tools::builtin::CalculatorTool)?;
    tools.register(agent_tools::builtin::CurrentTimeTool)?;
    tools.register(agent_tools::builtin::EchoTool)?;
    tools.register(agent_tools::builtin::DelegateTaskTool)?;
    tools.register(agent_tools::builtin::EscalateTool)?;
    tools.register(agent_tools::builtin::BroadcastToTeamTool)?;
    tools.register(agent_tools::builtin::SendMessageTool)?;
    
    let agent = Agent::builder()
        .provider(provider)
        .tools(tools)
        .session_store(InMemoryStore::new())
        .session_id(&agent_id)
        .system_message(r#"You are an intelligent coordinator agent. Use tools when necessary."#)
        .max_iterations(5)
        .hitl(AutoApprove::new())
        .coordinator(coordinator.clone())
        .build()?;

    Ok(Arc::new(agent))
}
