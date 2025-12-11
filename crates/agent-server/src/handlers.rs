//! HTTP request handlers - COMPLETE IMPLEMENTATION

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::{Event, KeepAlive, Sse}},
    Json,
};
use agent_hitl::AutoApprove;
use agent_llm::{AnthropicProvider, OpenAIProvider};
use agent_runtime::{Agent, AgentEvent};
use agent_session::InMemoryStore;
use agent_coordination::RoleConfig;
use agent_comms::AgentRole;
use futures::stream::Stream;
use std::sync::Arc;
use std::convert::Infallible;
use futures::StreamExt;

use crate::{
    models::*,
    state::{AgentInfo, AppState},
};

/// Health check
pub async fn health() -> &'static str {
    "OK"
}

//==============================================================================
// ROLE MANAGEMENT
//==============================================================================

/// Register a new role template
pub async fn register_role(
    State(state): State<AppState>,
    Json(req): Json<RoleRequest>,
) -> Result<Json<RoleResponse>, (StatusCode, String)> {
    let config = RoleConfig {
        role: req.role.clone(),
        system_message: req.system_message,
        model: req.model.clone(),
        provider: req.provider.clone(),
        tools: req.tools,
        can_spawn_agents: req.can_spawn_agents.unwrap_or(false),
        max_iterations: req.max_iterations.unwrap_or(10),
        hierarchy_role: req.hierarchy_role.clone(),
        team: req.team,
        supervisor: req.supervisor,
    };
    
    // Register with factory
    state.factory.register_role(config.clone());
    
    // Store in state
    state.role_configs.insert(req.role.clone(), config);
    
    Ok(Json(RoleResponse {
        role: req.role,
        model: req.model,
        provider: req.provider,
        hierarchy_role: req.hierarchy_role,
    }))
}

/// List all registered roles
pub async fn list_roles(
    State(state): State<AppState>,
) -> Json<Vec<RoleResponse>> {
    let roles: Vec<_> = state.role_configs.iter()
        .map(|entry| {
            let config = entry.value();
            RoleResponse {
                role: config.role.clone(),
                model: config.model.clone(),
                provider: config.provider.clone(),
                hierarchy_role: config.hierarchy_role.clone(),
            }
        })
        .collect();
    
    Json(roles)
}

/// Get role details
pub async fn get_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> Result<Json<RoleRequest>, (StatusCode, String)> {
    let config = state.role_configs.get(&role_name)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Role not found".to_string()))?;
    
    Ok(Json(RoleRequest {
        role: config.role.clone(),
        system_message: config.system_message.clone(),
        model: config.model.clone(),
        provider: config.provider.clone(),
        tools: config.tools.clone(),
        can_spawn_agents: Some(config.can_spawn_agents),
        max_iterations: Some(config.max_iterations),
        hierarchy_role: config.hierarchy_role.clone(),
        team: config.team.clone(),
        supervisor: config.supervisor.clone(),
    }))
}

/// Delete a role
pub async fn delete_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.role_configs.remove(&role_name)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Role not found".to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

//==============================================================================
// AGENT MANAGEMENT
//==============================================================================

/// Create a new agent with full configuration
pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentResponse>, (StatusCode, String)> {
    // If role specified, use factory to spawn
    if let Some(role_name) = req.role.clone() {
        return create_agent_from_role(state, &role_name, req).await;
    }
    
    // Otherwise create standalone agent directly
    create_agent_direct(state, req).await
}

/// Create agent using AgentFactory and role
async fn create_agent_from_role(
    state: AppState,
    role_name: &str,
    req: CreateAgentRequest,
) -> Result<Json<AgentResponse>, (StatusCode, String)> {
    let agent_id = req.name.clone()
        .unwrap_or_else(|| format!("{}-{}", role_name, uuid::Uuid::new_v4().to_string()[..8].to_string()));
    
    // Spawn agent via factory
    let spawned_id = state.factory.spawn_agent(role_name, Some(agent_id.clone())).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Get role config for response
    let role_config = state.role_configs.get(role_name)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Role not found".to_string()))?;
    
    // Get hierarchy info
    let hierarchy_role = state.coordinator.hierarchy().get_role(&spawned_id)
        .map(|r| r.clone())
        .unwrap_or(AgentRole::Standalone);
    
    let (team, supervisor, subordinates) = match &hierarchy_role {
        AgentRole::Coordinator { subordinates, team } => (Some(team.clone()), None, subordinates.clone()),
        AgentRole::Worker { supervisor, team } => (Some(team.clone()), Some(supervisor.clone()), vec![]),
        AgentRole::Peer { team, .. } => (Some(team.clone()), None, vec![]),
        AgentRole::Standalone => (None, None, vec![]),
    };
    
    Ok(Json(AgentResponse {
        id: spawned_id,
        name: role_config.role.clone(),
        model: role_config.model.clone(),
        created_at: chrono::Utc::now(),
        message_count: 0,
        role: Some(role_name.to_string()),
        hierarchy_role: role_config.hierarchy_role.clone(),
        team,
        supervisor,
        subordinates,
        tools: role_config.tools.clone(),
        max_iterations: role_config.max_iterations,
    }))
}

/// Create agent directly without role
async fn create_agent_direct(
    state: AppState,
    req: CreateAgentRequest,
) -> Result<Json<AgentResponse>, (StatusCode, String)> {
    let agent_id = req.name.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let model = req.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    let system_message = req.system_message.unwrap_or_else(|| 
        "You are a helpful assistant. Use tools when appropriate.".to_string()
    );
    let max_iterations = req.max_iterations.unwrap_or(5);
    
    // Build tools registry
    let tools = agent_tools::ToolRegistry::new();
    
    // Register requested tools or defaults
    let tool_names = req.tools.unwrap_or_else(|| vec![
        "CalculatorTool".to_string(),
        "CurrentTimeTool".to_string(),
        "EchoTool".to_string(),
        "DelegateTaskTool".to_string(),
        "EscalateTool".to_string(),
        "BroadcastToTeamTool".to_string(),
        "SendMessageTool".to_string(),
    ]);
    
    for tool_name in &tool_names {
        match tool_name.as_str() {
            "CalculatorTool" => tools.register(agent_tools::builtin::CalculatorTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            "CurrentTimeTool" => tools.register(agent_tools::builtin::CurrentTimeTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            "EchoTool" => tools.register(agent_tools::builtin::EchoTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            "DelegateTaskTool" => tools.register(agent_tools::builtin::DelegateTaskTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            "EscalateTool" => tools.register(agent_tools::builtin::EscalateTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            "BroadcastToTeamTool" => tools.register(agent_tools::builtin::BroadcastToTeamTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            "SendMessageTool" => tools.register(agent_tools::builtin::SendMessageTool)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            _ => {} // Ignore unknown tools
        }
    }
    
    // Session backend (only InMemoryStore supported for now)
    let session_store = InMemoryStore::new();

    // Build agent
    let agent = if model.starts_with("gpt") {
        let provider = OpenAIProvider::new(&state.openai_key, &model)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        Agent::builder()
            .provider(provider)
            .tools(tools)
            .session_store(session_store)
            .session_id(&agent_id)
            .system_message(&system_message)
            .max_iterations(max_iterations)
            .hitl(AutoApprove::new())
            .coordinator(state.coordinator.clone())
            .build()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else if model.starts_with("claude") {
        let key = state.anthropic_key.as_ref()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Anthropic API key not configured".to_string()))?;
        let provider = AnthropicProvider::new(key, &model)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        Agent::builder()
            .provider(provider)
            .tools(tools)
            .session_store(session_store)
            .session_id(&agent_id)
            .system_message(&system_message)
            .max_iterations(max_iterations)
            .hitl(AutoApprove::new())
            .coordinator(state.coordinator.clone())
            .build()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        return Err((StatusCode::BAD_REQUEST, "Unsupported model".to_string()));
    };

    let hierarchy_role = req.hierarchy_role.unwrap_or_else(|| "standalone".to_string());
    
    // Build AgentRole for coordinator registration
    let agent_role = match hierarchy_role.as_str() {
        "coordinator" => AgentRole::Coordinator {
            subordinates: vec![],
            team: req.team.clone().unwrap_or_else(|| "default".to_string()),
        },
        "worker" => {
            let supervisor = req.supervisor.clone()
                .ok_or_else(|| (StatusCode::BAD_REQUEST, "Worker requires supervisor".to_string()))?;
            AgentRole::Worker {
                supervisor,
                team: req.team.clone().unwrap_or_else(|| "default".to_string()),
            }
        },
        "peer" => AgentRole::Peer {
            team: req.team.clone().unwrap_or_else(|| "default".to_string()),
            peers: vec![],
        },
        _ => AgentRole::Standalone,
    };
    
    // Register in coordinator hierarchy
    let agent_address = agent_comms::AgentAddress::local(&agent_id);
    state.coordinator.register_agent(agent_address, agent_role);
    
    let info = AgentInfo {
        id: agent_id.clone(),
        name: agent_id.clone(),
        model: model.clone(),
        agent: Arc::new(agent),
        created_at: chrono::Utc::now(),
        messages: Arc::new(dashmap::DashMap::new()),
        role: None,
        hierarchy_role: hierarchy_role.clone(),
        team: req.team.clone(),
        supervisor: req.supervisor.clone(),
        subordinates: vec![],
        tools: tool_names.clone(),
        max_iterations,
    };

    state.agents.insert(agent_id.clone(), info);

    Ok(Json(AgentResponse {
        id: agent_id.clone(),
        name: agent_id.clone(),
        model,
        created_at: chrono::Utc::now(),
        message_count: 0,
        role: None,
        hierarchy_role,
        team: req.team,
        supervisor: req.supervisor,
        subordinates: vec![],
        tools: tool_names,
        max_iterations,
    }))
}

/// List all agents
pub async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<AgentResponse>> {
    let agents: Vec<_> = state.agents.iter()
        .map(|entry| AgentResponse {
            id: entry.id.clone(),
            name: entry.name.clone(),
            model: entry.model.clone(),
            created_at: entry.created_at,
            message_count: entry.messages.len(),
            role: entry.role.clone(),
            hierarchy_role: entry.hierarchy_role.clone(),
            team: entry.team.clone(),
            supervisor: entry.supervisor.clone(),
            subordinates: entry.subordinates.clone(),
            tools: entry.tools.clone(),
            max_iterations: entry.max_iterations,
        })
        .collect();
    
    Json(agents)
}

/// Get agent details
pub async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentResponse>, (StatusCode, String)> {
    let agent = state.agents.get(&agent_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    
    Ok(Json(AgentResponse {
        id: agent.id.clone(),
        name: agent.name.clone(),
        model: agent.model.clone(),
        created_at: agent.created_at,
        message_count: agent.messages.len(),
        role: agent.role.clone(),
        hierarchy_role: agent.hierarchy_role.clone(),
        team: agent.team.clone(),
        supervisor: agent.supervisor.clone(),
        subordinates: agent.subordinates.clone(),
        tools: agent.tools.clone(),
        max_iterations: agent.max_iterations,
    }))
}

/// Delete/stop an agent
pub async fn delete_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.agents.remove(&agent_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    
    Ok(StatusCode::NO_CONTENT)
}

//==============================================================================
// MESSAGE HANDLING
//==============================================================================

/// Send message and get response (non-streaming)
pub async fn send_message(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let agent_info = state.agents.get(&agent_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    // Store user message
    let user_msg = MessageResponse {
        role: "user".to_string(),
        content: req.content.clone(),
        timestamp: chrono::Utc::now(),
    };
    agent_info.messages.insert(agent_info.messages.len(), user_msg);

    // Get response
    let response = agent_info.agent.run(&req.content).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let agent_msg = MessageResponse {
        role: "agent".to_string(),
        content: response,
        timestamp: chrono::Utc::now(),
    };
    agent_info.messages.insert(agent_info.messages.len(), agent_msg.clone());

    Ok(Json(agent_msg))
}

/// Get message history
pub async fn get_messages(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Vec<MessageResponse>>, (StatusCode, String)> {
    let agent_info = state.agents.get(&agent_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    let messages: Vec<_> = agent_info.messages.iter()
        .map(|entry| entry.value().clone())
        .collect();

    Ok(Json(messages))
}

/// Send message with streaming response (SSE)
pub async fn stream_message(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let agent_info = state.agents.get(&agent_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    let agent = agent_info.agent.clone();
    let content = req.content.clone();

    let stream = async_stream::stream! {
        match agent.run_stream(&content).await {
            Ok(mut event_stream) => {
                while let Some(event_result) = event_stream.next().await {
                    match event_result {
                        Ok(event) => {
                            let stream_event = match event {
                                AgentEvent::TextChunk { content } => {
                                    StreamEvent::TextChunk { content }
                                }
                                AgentEvent::ToolCallStart { tool_name, parameters } => {
                                    StreamEvent::ToolCall {
                                        name: tool_name,
                                        params: parameters.to_string(),
                                    }
                                }
                                AgentEvent::ToolCallEnd { tool_name, success, .. } => {
                                    StreamEvent::ToolResult {
                                        name: tool_name,
                                        success,
                                    }
                                }
                                AgentEvent::Done { total_tokens } => {
                                    StreamEvent::Done { total_tokens }
                                }
                                AgentEvent::Error { message } => {
                                    StreamEvent::Error { message }
                                }
                                AgentEvent::Thinking { .. } => continue,
                            };
                            
                            let json = serde_json::to_string(&stream_event).unwrap();
                            yield Ok(Event::default().data(json));
                        }
                        Err(e) => {
                            let error_event = StreamEvent::Error { message: e.to_string() };
                            let json = serde_json::to_string(&error_event).unwrap();
                            yield Ok(Event::default().data(json));
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let error_event = StreamEvent::Error { message: e.to_string() };
                let json = serde_json::to_string(&error_event).unwrap();
                yield Ok(Event::default().data(json));
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

//==============================================================================
// COORDINATION OPERATIONS
//==============================================================================

/// Delegate task to worker
pub async fn delegate_task(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<DelegateRequest>,
) -> Result<Json<DelegateResponse>, (StatusCode, String)> {
    let task_id = state.coordinator.delegate_task(
        &agent_id,
        req.worker_id.as_deref(),
        req.task_description.clone(),
    ).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Get actual worker ID
    let worker_id = req.worker_id.unwrap_or_else(|| "auto-selected".to_string());
    
    Ok(Json(DelegateResponse {
        task_id,
        worker_id,
        status: "delegated".to_string(),
    }))
}

/// Escalate to supervisor
pub async fn escalate(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<EscalateRequest>,
) -> Result<Json<EscalateResponse>, (StatusCode, String)> {
    let supervisor = state.coordinator.hierarchy().get_supervisor(&agent_id)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Agent has no supervisor".to_string()))?;
    
    let mut escalation = agent_comms::EscalationRequest::new(
        &agent_id,
        &supervisor,
        &req.reason,
    );
    
    if let Some(task_id) = req.task_id {
        escalation = escalation.with_task(task_id);
    }
    
    if let Some(context) = req.context {
        escalation = escalation.with_context(context);
    }
    
    let escalation_id = state.coordinator.escalate(&agent_id, escalation).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(EscalateResponse {
        escalation_id,
        supervisor_id: supervisor,
        status: "escalated".to_string(),
    }))
}

/// Broadcast to team
pub async fn broadcast_to_team(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(req): Json<BroadcastRequest>,
) -> Result<Json<BroadcastResponse>, (StatusCode, String)> {
    // Get agent's team if not specified
    let team = if let Some(team_name) = req.team {
        team_name
    } else {
        let agent_info = state.agents.get(&agent_id)
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
        agent_info.team.clone()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Agent has no team".to_string()))?
    };
    
    let recipients = state.coordinator.broadcast_to_team(
        &agent_id,
        &team,
        serde_json::json!({"broadcast": req.message}),
    ).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(BroadcastResponse {
        team,
        recipients,
        status: "broadcast".to_string(),
    }))
}

//==============================================================================
// TEAM MANAGEMENT
//==============================================================================

/// List all teams
pub async fn list_teams(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    let teams: std::collections::HashSet<_> = state.agents.iter()
        .filter_map(|entry| entry.team.clone())
        .collect();
    
    Json(teams.into_iter().collect())
}

/// Get team members
pub async fn get_team(
    State(state): State<AppState>,
    Path(team_name): Path<String>,
) -> Result<Json<TeamResponse>, (StatusCode, String)> {
    let members: Vec<_> = state.agents.iter()
        .filter(|entry| entry.team.as_ref() == Some(&team_name))
        .map(|entry| entry.id.clone())
        .collect();
    
    if members.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Team not found".to_string()));
    }
    
    let coordinators: Vec<_> = state.agents.iter()
        .filter(|entry| {
            entry.team.as_ref() == Some(&team_name) &&
            entry.hierarchy_role == "coordinator"
        })
        .map(|entry| entry.id.clone())
        .collect();
    
    let workers: Vec<_> = state.agents.iter()
        .filter(|entry| {
            entry.team.as_ref() == Some(&team_name) &&
            entry.hierarchy_role == "worker"
        })
        .map(|entry| entry.id.clone())
        .collect();
    
    Ok(Json(TeamResponse {
        team: team_name,
        members,
        coordinators,
        workers,
    }))
}
