//! Request/Response models

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: Option<String>,
    pub model: Option<String>,  // "gpt-4", "gpt-3.5-turbo", "claude-sonnet-4"
    pub system_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub model: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
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
