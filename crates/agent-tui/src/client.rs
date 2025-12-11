//! HTTP client for agent-server API

use anyhow::Result;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio_stream::Stream;

/// Agent server API client
pub struct AgentClient {
    base_url: String,
    client: reqwest::Client,
}

/// Request to create an agent
#[derive(Debug, Serialize)]
pub struct CreateAgentRequest {
    pub model: String,
}

/// Agent response from server
#[derive(Debug, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub model: String,
}

/// Message to send to agent
#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// Stream event from SSE
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    TextChunk { content: String },
    ToolCall { name: String, params: String },
    ToolResult { name: String, success: bool },
    Done { total_tokens: Option<usize> },
    Error { message: String },
}

/// Message response from server
#[derive(Debug, Clone, Deserialize)]
pub struct MessageResponse {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AgentClient {
    /// Create a new agent client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new agent
    pub async fn create_agent(&self, model: &str) -> Result<AgentResponse> {
        let url = format!("{}/agents", self.base_url);
        let req = CreateAgentRequest {
            model: model.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;

        let agent = response.json::<AgentResponse>().await?;
        Ok(agent)
    }

    /// Stream a message to an agent (SSE)
    pub async fn stream_message(
        &self,
        agent_id: &str,
        content: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let url = format!("{}/agents/{}/stream", self.base_url, agent_id);
        let req = SendMessageRequest {
            content: content.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;

        // Convert response to SSE stream
        let stream = response
            .bytes_stream()
            .eventsource()
            .map(|event| {
                match event {
                    Ok(event) => {
                        // Parse the data field as JSON
                        serde_json::from_str::<StreamEvent>(&event.data)
                            .map_err(|e| anyhow::anyhow!("Failed to parse event: {}", e))
                    }
                    Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
                }
            });

        Ok(Box::pin(stream))
    }

    /// Get message history
    pub async fn get_messages(&self, agent_id: &str) -> Result<Vec<MessageResponse>> {
        let url = format!("{}/agents/{}/messages", self.base_url, agent_id);

        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;

        let messages = response.json::<Vec<MessageResponse>>().await?;
        Ok(messages)
    }
}
