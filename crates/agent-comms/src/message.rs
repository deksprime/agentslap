//! Agent messages

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type of message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    /// One-way message (no response expected)
    Send,
    
    /// Request (response expected)
    Request,
    
    /// Response to a request
    Response,
    
    /// Broadcast to multiple agents
    Broadcast,
}

/// Message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Message ID (for correlation)
    pub id: String,
    
    /// Message type
    pub msg_type: MessageType,
    
    /// From agent ID
    pub from: String,
    
    /// To agent ID (None for broadcast)
    pub to: Option<String>,
    
    /// Topic (for pub/sub)
    pub topic: Option<String>,
    
    /// Message payload
    pub content: Value,
    
    /// Request ID (for request/response correlation)
    pub request_id: Option<String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AgentMessage {
    /// Create a new message
    pub fn new<S: Into<String>>(from: S, to: S, content: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: MessageType::Send,
            from: from.into(),
            to: Some(to.into()),
            topic: None,
            content,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a request message
    pub fn request<S: Into<String>>(from: S, to: S, content: Value) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id: id.clone(),
            msg_type: MessageType::Request,
            from: from.into(),
            to: Some(to.into()),
            topic: None,
            content,
            request_id: Some(id), // Request ID = Message ID
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a response message
    pub fn response<S: Into<String>>(from: S, to: S, content: Value, request_id: S) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: MessageType::Response,
            from: from.into(),
            to: Some(to.into()),
            topic: None,
            content,
            request_id: Some(request_id.into()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a broadcast message
    pub fn broadcast<S: Into<String>>(from: S, topic: S, content: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: MessageType::Broadcast,
            from: from.into(),
            to: None,
            topic: Some(topic.into()),
            content,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = AgentMessage::new("agent-1", "agent-2", serde_json::json!({"test": true}));
        assert_eq!(msg.from, "agent-1");
        assert_eq!(msg.to.unwrap(), "agent-2");
        assert_eq!(msg.msg_type, MessageType::Send);
    }

    #[test]
    fn test_request_message() {
        let msg = AgentMessage::request("a", "b", serde_json::json!({"query": "status"}));
        assert_eq!(msg.msg_type, MessageType::Request);
        assert!(msg.request_id.is_some());
    }

    #[test]
    fn test_broadcast_message() {
        let msg = AgentMessage::broadcast("coordinator", "tasks", serde_json::json!({"task": "new"}));
        assert_eq!(msg.msg_type, MessageType::Broadcast);
        assert!(msg.topic.is_some());
        assert!(msg.to.is_none());
    }

    #[test]
    fn test_message_serialization() {
        let msg = AgentMessage::new("a", "b", serde_json::json!({"x": 1}));
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: AgentMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.from, "a");
        assert_eq!(deserialized.content["x"], 1);
    }
}



