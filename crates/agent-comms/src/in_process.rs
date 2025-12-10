//! In-process transport using Tokio channels

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, broadcast};
use tokio::time::timeout;

use crate::{
    AgentAddress, AgentMessage, MessageTransport, Result, CommsError, MessageType,
};

/// In-process message transport
///
/// Uses Tokio channels for fast, in-memory communication between agents.
#[derive(Clone)]
pub struct InProcessTransport {
    /// Agent mailboxes (agent_id -> channel)
    mailboxes: Arc<DashMap<String, mpsc::UnboundedSender<AgentMessage>>>,
    
    /// Pending requests (request_id -> response channel)  
    /// Public for testing
    pub pending_requests: Arc<DashMap<String, oneshot::Sender<AgentMessage>>>,
    
    /// Topics for pub/sub (topic -> broadcast sender)
    topics: Arc<DashMap<String, broadcast::Sender<AgentMessage>>>,
}

impl InProcessTransport {
    pub fn new() -> Self {
        Self {
            mailboxes: Arc::new(DashMap::new()),
            pending_requests: Arc::new(DashMap::new()),
            topics: Arc::new(DashMap::new()),
        }
    }

    /// Register an agent's mailbox
    pub fn register_mailbox(&self, agent_id: &str) -> mpsc::UnboundedReceiver<AgentMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.mailboxes.insert(agent_id.to_string(), tx);
        tracing::debug!("Registered mailbox for agent: {}", agent_id);
        rx
    }

    /// Get subscriber for a topic
    pub fn get_topic_subscriber(&self, topic: &str) -> broadcast::Receiver<AgentMessage> {
        let sender = self.topics.entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            });
        sender.subscribe()
    }
}

impl Default for InProcessTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageTransport for InProcessTransport {
    async fn send(&self, to: &AgentAddress, message: AgentMessage) -> Result<()> {
        let mailbox = self.mailboxes.get(&to.id)
            .ok_or_else(|| CommsError::AgentNotFound(to.id.clone()))?;

        mailbox.send(message)
            .map_err(|e| CommsError::delivery_failed(e.to_string()))?;

        Ok(())
    }

    async fn receive(&self, agent_id: &str) -> Result<AgentMessage> {
        // This would need the receiver - simplified for now
        Err(CommsError::transport("receive() needs receiver from register_mailbox"))
    }

    async fn request(&self, to: &AgentAddress, mut message: AgentMessage) -> Result<AgentMessage> {
        // Ensure it's a request
        message.msg_type = MessageType::Request;
        let request_id = message.request_id.clone()
            .unwrap_or_else(|| message.id.clone());

        // Create response channel
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(request_id.clone(), tx);

        // Send request
        self.send(to, message.clone()).await?;

        // Wait for response with timeout
        let response = timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| {
                self.pending_requests.remove(&request_id);
                CommsError::Timeout(Duration::from_secs(30))
            })?
            .map_err(|_| CommsError::delivery_failed("Response channel closed"))?;

        Ok(response)
    }

    async fn broadcast(&self, topic: &str, message: AgentMessage) -> Result<()> {
        let sender = self.topics.entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            });

        sender.send(message)
            .map_err(|e| CommsError::delivery_failed(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(&self, agent_id: &str, topic: &str) -> Result<()> {
        // Subscription management would be more complex in real impl
        tracing::debug!("Agent {} subscribed to topic: {}", agent_id, topic);
        Ok(())
    }

    fn name(&self) -> &str {
        "in_process"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = InProcessTransport::new();
        assert_eq!(transport.name(), "in_process");
    }

    #[tokio::test]
    async fn test_mailbox_registration() {
        let transport = InProcessTransport::new();
        let mut rx = transport.register_mailbox("agent-1");

        let msg = AgentMessage::new("agent-2", "agent-1", serde_json::json!({"test": true}));
        let addr = AgentAddress::local("agent-1");

        transport.send(&addr, msg.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.from, "agent-2");
        assert_eq!(received.content["test"], true);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let transport = InProcessTransport::new();
        
        // Create subscribers
        let mut sub1 = transport.get_topic_subscriber("test-topic");
        let mut sub2 = transport.get_topic_subscriber("test-topic");

        // Broadcast message
        let msg = AgentMessage::broadcast("sender", "test-topic", serde_json::json!({"broadcast": true}));
        transport.broadcast("test-topic", msg).await.unwrap();

        // Both should receive
        let recv1 = sub1.recv().await.unwrap();
        let recv2 = sub2.recv().await.unwrap();

        assert_eq!(recv1.content["broadcast"], true);
        assert_eq!(recv2.content["broadcast"], true);
    }
}

