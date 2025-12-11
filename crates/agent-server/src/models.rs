//! Request/Response models

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: Option<String>,
    pub model: Option<String>,  // "gpt-4", "gpt-3.5-turbo", "claude-sonnet-4"
    pub system_message: Option<String>,
    
    // Role-based spawning
    pub role: Option<String>,  // Role name from registered roles
    
    // Hierarchy
    pub hierarchy_role: Option<String>,  // "coordinator", "worker", "peer", "standalone"
    pub team: Option<String>,
    pub supervisor: Option<String>,  // For workers
    
    // Configuration
    pub tools: Option<Vec<String>>,  // Tool names to enable
    pub max_iterations: Option<usize>,
    pub session_backend: Option<String>,  // "memory", "database", "cache"
    pub hitl_strategy: Option<String>,  // "auto", "prompt"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub model: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
    
    // Hierarchy info
    pub role: Option<String>,
    pub hierarchy_role: String,
    pub team: Option<String>,
    pub supervisor: Option<String>,
    pub subordinates: Vec<String>,
    
    // Configuration
    pub tools: Vec<String>,
    pub max_iterations: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub role: String,  // "user" or "agent"
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    TextChunk { content: String },
    ToolCall { name: String, params: String },
    ToolResult { name: String, success: bool },
    Done { total_tokens: Option<usize> },
    Error { message: String },
}

// Role management
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleRequest {
    pub role: String,
    pub system_message: String,
    pub model: String,
    pub provider: String,
    pub tools: Vec<String>,
    pub can_spawn_agents: Option<bool>,
    pub max_iterations: Option<usize>,
    pub hierarchy_role: String,
    pub team: Option<String>,
    pub supervisor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleResponse {
    pub role: String,
    pub model: String,
    pub provider: String,
    pub hierarchy_role: String,
}

// Coordination operations
#[derive(Debug, Serialize, Deserialize)]
pub struct DelegateRequest {
    pub task_description: String,
    pub worker_id: Option<String>,  // Auto-select if not specified
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelegateResponse {
    pub task_id: String,
    pub worker_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EscalateRequest {
    pub reason: String,
    pub context: Option<serde_json::Value>,
    pub task_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EscalateResponse {
    pub escalation_id: String,
    pub supervisor_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub message: String,
    pub team: Option<String>,  // Use agent's team if not specified
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastResponse {
    pub team: String,
    pub recipients: usize,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamResponse {
    pub team: String,
    pub members: Vec<String>,
    pub coordinators: Vec<String>,
    pub workers: Vec<String>,
}

