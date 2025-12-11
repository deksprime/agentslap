//! HTTP request handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, KeepAlive, Sse}},
    Json,
};
use agent_hitl::AutoApprove;
use agent_llm::{AnthropicProvider, OpenAIProvider};
use agent_runtime::{Agent, AgentEvent};
use agent_session::InMemoryStore;
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

/// Create a new agent
pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentResponse>, (StatusCode, String)> {
    let agent_id = uuid::Uuid::new_v4().to_string();
    let name = req.name.unwrap_or_else(|| format!("agent-{}", &agent_id[..8]));
    let model = req.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    let system_message = req.system_message.unwrap_or_else(|| 
        "You are a helpful assistant. Use tools when appropriate.".to_string()
    );

    // Build agent
    let tools = agent_tools::ToolRegistry::new();
    tools.register(agent_tools::builtin::CalculatorTool)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    tools.register(agent_tools::builtin::CurrentTimeTool)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    tools.register(agent_tools::builtin::EchoTool)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let agent = if model.starts_with("gpt") {
        let provider = OpenAIProvider::new(&state.openai_key, &model)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        Agent::builder()
            .provider(provider)
            .tools(tools)
            .session_store(InMemoryStore::new())
            .session_id(&agent_id)
            .system_message(&system_message)
            .max_iterations(5)
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
            .session_store(InMemoryStore::new())
            .session_id(&agent_id)
            .system_message(&system_message)
            .max_iterations(5)
            .hitl(AutoApprove::new())
            .coordinator(state.coordinator.clone())
            .build()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        return Err((StatusCode::BAD_REQUEST, "Unsupported model".to_string()));
    };

    let info = AgentInfo {
        id: agent_id.clone(),
        name: name.clone(),
        model: model.clone(),
        agent: Arc::new(agent),
        created_at: chrono::Utc::now(),
        messages: Arc::new(dashmap::DashMap::new()),
    };

    state.agents.insert(agent_id.clone(), info);

    Ok(Json(AgentResponse {
        id: agent_id,
        name,
        model,
        created_at: chrono::Utc::now(),
        message_count: 0,
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

    // Create stream
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
