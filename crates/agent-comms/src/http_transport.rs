//! HTTP-based transport for cross-process communication

use async_trait::async_trait;
use reqwest::Client;

use crate::{AgentAddress, AgentMessage, MessageTransport, Result, CommsError};

/// HTTP transport for agent communication
///
/// Enables agents to communicate across processes and networks.
pub struct HttpTransport {
    client: Client,
    base_url: String,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    fn get_agent_url(&self, agent_id: &str) -> String {
        format!("{}/api/agents/{}/messages", self.base_url, agent_id)
    }
}

#[async_trait]
impl MessageTransport for HttpTransport {
    async fn send(&self, to: &AgentAddress, message: AgentMessage) -> Result<()> {
        let url = self.get_agent_url(&to.id);
        
        self.client
            .post(&url)
            .json(&message)
            .send()
            .await
            .map_err(|e| CommsError::transport(e.to_string()))?;

        Ok(())
    }

    async fn receive(&self, _agent_id: &str) -> Result<AgentMessage> {
        // Would poll or use long-polling
        Err(CommsError::transport("receive() not implemented for HTTP transport"))
    }

    async fn request(&self, to: &AgentAddress, message: AgentMessage) -> Result<AgentMessage> {
        let url = format!("{}/api/agents/{}/request", self.base_url, to.id);
        
        let http_response = self.client
            .post(&url)
            .json(&message)
            .send()
            .await
            .map_err(|e| CommsError::transport(e.to_string()))?;

        let response: AgentMessage = http_response
            .json()
            .await
            .map_err(|e| CommsError::transport(format!("JSON parse error: {}", e)))?;

        Ok(response)
    }

    async fn broadcast(&self, topic: &str, message: AgentMessage) -> Result<()> {
        let url = format!("{}/api/topics/{}", self.base_url, topic);
        
        self.client
            .post(&url)
            .json(&message)
            .send()
            .await
            .map_err(|e| CommsError::transport(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(&self, _agent_id: &str, _topic: &str) -> Result<()> {
        // Would use WebSocket for subscriptions
        Ok(())
    }

    fn name(&self) -> &str {
        "http"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_creation() {
        let transport = HttpTransport::new("http://localhost:8080");
        assert_eq!(transport.name(), "http");
    }

    #[test]
    fn test_get_agent_url() {
        let transport = HttpTransport::new("http://localhost:8080");
        let url = transport.get_agent_url("agent-1");
        assert_eq!(url, "http://localhost:8080/api/agents/agent-1/messages");
    }
}

