//! Common types for LLM interactions

use serde::{Deserialize, Serialize};

/// Role of a message in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions)
    System,
    /// User message
    User,
    /// Assistant message (LLM response)
    Assistant,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

/// Response from an LLM
#[derive(Debug, Clone)]
pub struct Response {
    /// The generated content
    pub content: String,
    /// Model that generated the response
    pub model: String,
    /// Token usage information
    pub usage: Option<TokenUsage>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens
    pub total_tokens: u32,
}

/// A chunk from a streaming response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Content delta (incremental text)
    pub content: String,
    /// Model that generated this chunk
    pub model: Option<String>,
    /// Finish reason if this is the last chunk
    pub finish_reason: Option<String>,
}

impl StreamChunk {
    /// Create a new stream chunk
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            model: None,
            finish_reason: None,
        }
    }

    /// Check if this is the last chunk
    pub fn is_final(&self) -> bool {
        self.finish_reason.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_helpers() {
        let system = Message::system("You are helpful");
        assert_eq!(system.role, MessageRole::System);

        let user = Message::user("Hello");
        assert_eq!(user.role, MessageRole::User);

        let assistant = Message::assistant("Hi there");
        assert_eq!(assistant.role, MessageRole::Assistant);
    }

    #[test]
    fn test_stream_chunk() {
        let mut chunk = StreamChunk::new("Hello");
        assert_eq!(chunk.content, "Hello");
        assert!(!chunk.is_final());

        chunk.finish_reason = Some("stop".to_string());
        assert!(chunk.is_final());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("Hello");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.content, deserialized.content);
        assert_eq!(msg.role, deserialized.role);
    }
}




