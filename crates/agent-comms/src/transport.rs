//! Message transport trait

use async_trait::async_trait;

use crate::{AgentAddress, AgentMessage, Result};

/// Transport layer for agent messages
///
/// Implementations handle HOW messages are delivered.
/// Agents use this interface without knowing the underlying mechanism.
#[async_trait]
pub trait MessageTransport: Send + Sync {
    /// Send a message to an agent
    async fn send(&self, to: &AgentAddress, message: AgentMessage) -> Result<()>;

    /// Receive a message (blocking until available)
    async fn receive(&self, agent_id: &str) -> Result<AgentMessage>;

    /// Send request and wait for response
    async fn request(&self, to: &AgentAddress, message: AgentMessage) -> Result<AgentMessage>;

    /// Broadcast message to a topic
    async fn broadcast(&self, topic: &str, message: AgentMessage) -> Result<()>;

    /// Subscribe to a topic
    async fn subscribe(&self, agent_id: &str, topic: &str) -> Result<()>;

    /// Get transport name
    fn name(&self) -> &str;
    
    /// Downcast to concrete type for specific transport features
    fn as_any(&self) -> &dyn std::any::Any;
}



