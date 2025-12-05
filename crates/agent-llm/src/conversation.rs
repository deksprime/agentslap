//! Conversation management for multi-turn interactions
//!
//! This module provides conversation management capabilities for tracking
//! multi-turn interactions with LLMs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Message, MessageRole};

/// A conversation consisting of multiple messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for this conversation
    pub id: String,
    
    /// Messages in this conversation
    messages: Vec<Message>,
    
    /// When this conversation was created
    pub created_at: DateTime<Utc>,
    
    /// When this conversation was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Optional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Conversation {
    /// Create a new empty conversation
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Create a conversation with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Create a builder for fluent conversation construction
    pub fn builder() -> ConversationBuilder {
        ConversationBuilder::new()
    }
    
    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }
    
    /// Add a system message
    pub fn add_system(&mut self, content: impl Into<String>) {
        self.add_message(Message::system(content));
    }
    
    /// Add a user message
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content));
    }
    
    /// Add an assistant message
    pub fn add_assistant(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content));
    }
    
    /// Get all messages in the conversation
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    
    /// Get a mutable reference to messages
    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        self.updated_at = Utc::now();
        &mut self.messages
    }
    
    /// Get the number of messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Check if conversation is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Get the last message
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }
    
    /// Get the first message (usually system message)
    pub fn first_message(&self) -> Option<&Message> {
        self.messages.first()
    }
    
    /// Get the system message if it exists
    pub fn system_message(&self) -> Option<&Message> {
        self.messages
            .iter()
            .find(|m| m.role == MessageRole::System)
    }
    
    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.updated_at = Utc::now();
    }
    
    /// Get approximate token count for this conversation
    ///
    /// This uses a simple heuristic: characters / 4 for English text.
    /// For more accurate counting, use a proper tokenizer.
    pub fn estimate_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(estimate_message_tokens)
            .sum()
    }
    
    /// Truncate conversation to fit within token limit
    ///
    /// Uses a strategy that keeps the system message (if any) and recent messages.
    pub fn truncate_to_tokens(&mut self, max_tokens: usize) {
        let system_msg = self.messages
            .iter()
            .position(|m| m.role == MessageRole::System);
        
        let mut current_tokens = 0;
        let mut keep_indices = Vec::new();
        
        // Always keep system message if it exists
        if let Some(idx) = system_msg {
            current_tokens += estimate_message_tokens(&self.messages[idx]);
            keep_indices.push(idx);
        }
        
        // Add recent messages working backwards
        for (idx, message) in self.messages.iter().enumerate().rev() {
            if Some(idx) == system_msg {
                continue; // Already counted
            }
            
            let msg_tokens = estimate_message_tokens(message);
            if current_tokens + msg_tokens <= max_tokens {
                current_tokens += msg_tokens;
                keep_indices.push(idx);
            } else {
                break;
            }
        }
        
        // Sort indices and keep only those messages
        keep_indices.sort_unstable();
        let kept_messages: Vec<_> = keep_indices
            .into_iter()
            .map(|idx| self.messages[idx].clone())
            .collect();
        
        self.messages = kept_messages;
        self.updated_at = Utc::now();
    }
    
    /// Create a summary of the conversation
    pub fn summary(&self) -> ConversationSummary {
        ConversationSummary {
            id: self.id.clone(),
            message_count: self.messages.len(),
            estimated_tokens: self.estimate_tokens(),
            has_system_message: self.system_message().is_some(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating conversations fluently
pub struct ConversationBuilder {
    conversation: Conversation,
}

impl ConversationBuilder {
    /// Create a new conversation builder
    pub fn new() -> Self {
        Self {
            conversation: Conversation::new(),
        }
    }
    
    /// Set a specific ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.conversation.id = id.into();
        self
    }
    
    /// Add a system message
    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.conversation.add_system(content);
        self
    }
    
    /// Add a user message
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.conversation.add_user(content);
        self
    }
    
    /// Add an assistant message
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.conversation.add_assistant(content);
        self
    }
    
    /// Add a message
    pub fn message(mut self, message: Message) -> Self {
        self.conversation.add_message(message);
        self
    }
    
    /// Add multiple messages
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        for message in messages {
            self.conversation.add_message(message);
        }
        self
    }
    
    /// Set metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.conversation.metadata = metadata;
        self
    }
    
    /// Build the conversation
    pub fn build(self) -> Conversation {
        self.conversation
    }
}

impl Default for ConversationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary information about a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Conversation ID
    pub id: String,
    /// Number of messages
    pub message_count: usize,
    /// Estimated token count
    pub estimated_tokens: usize,
    /// Whether conversation has a system message
    pub has_system_message: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Estimate tokens for a single message
///
/// Uses a simple heuristic: characters / 4 for English text.
/// This is approximate and varies by tokenizer.
pub fn estimate_message_tokens(message: &Message) -> usize {
    // Simple heuristic: 1 token ≈ 4 characters for English
    // Add overhead for role and formatting
    let content_tokens = message.content.chars().count() / 4;
    let role_overhead = 4; // Approximate overhead for role formatting
    content_tokens + role_overhead
}

/// Estimate tokens for text content
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new();
        assert!(conv.is_empty());
        assert_eq!(conv.len(), 0);
        assert!(!conv.id.is_empty());
    }

    #[test]
    fn test_conversation_builder() {
        let conv = Conversation::builder()
            .system("You are helpful")
            .user("Hello")
            .assistant("Hi there!")
            .build();

        assert_eq!(conv.len(), 3);
        assert_eq!(conv.messages()[0].role, MessageRole::System);
        assert_eq!(conv.messages()[1].role, MessageRole::User);
        assert_eq!(conv.messages()[2].role, MessageRole::Assistant);
    }

    #[test]
    fn test_add_messages() {
        let mut conv = Conversation::new();
        conv.add_system("System");
        conv.add_user("User");
        conv.add_assistant("Assistant");

        assert_eq!(conv.len(), 3);
    }

    #[test]
    fn test_system_message() {
        let conv = Conversation::builder()
            .user("Hello")
            .system("Be helpful")
            .user("World")
            .build();

        assert!(conv.system_message().is_some());
        assert_eq!(conv.system_message().unwrap().content, "Be helpful");
    }

    #[test]
    fn test_token_estimation() {
        let message = Message::user("Hello, world!");
        let tokens = estimate_message_tokens(&message);
        assert!(tokens > 0);
        assert!(tokens < 20); // Should be small for short message
    }

    #[test]
    fn test_conversation_token_estimation() {
        let conv = Conversation::builder()
            .system("You are a helpful assistant")
            .user("Tell me a short joke")
            .build();

        let tokens = conv.estimate_tokens();
        assert!(tokens > 0);
    }

    #[test]
    fn test_truncate_to_tokens() {
        let mut conv = Conversation::builder()
            .system("This is a system message with some content")
            .user("This is the first user message with quite a bit of text to make it longer")
            .assistant("This is the first assistant response with lots of content")
            .user("This is the second user message also with extra text")
            .assistant("This is the second assistant response with more content")
            .user("This is the third user message with additional text")
            .build();

        let original_len = conv.len();
        
        // Truncate to a small limit (should keep system + recent messages)
        conv.truncate_to_tokens(30);

        // Should have fewer messages or same (if already under limit)
        assert!(conv.len() <= original_len);
        // Should be under the token limit
        assert!(conv.estimate_tokens() <= 35); // Allow small buffer
        // Should always keep system message
        assert!(conv.system_message().is_some());
    }

    #[test]
    fn test_conversation_clear() {
        let mut conv = Conversation::builder()
            .user("Hello")
            .build();

        assert!(!conv.is_empty());
        conv.clear();
        assert!(conv.is_empty());
    }

    #[test]
    fn test_conversation_summary() {
        let conv = Conversation::builder()
            .system("System")
            .user("Hello")
            .build();

        let summary = conv.summary();
        assert_eq!(summary.message_count, 2);
        assert!(summary.has_system_message);
        assert!(summary.estimated_tokens > 0);
    }

    #[test]
    fn test_serialization() {
        let conv = Conversation::builder()
            .id("test-123")
            .system("System")
            .user("Hello")
            .build();

        let json = serde_json::to_string(&conv).unwrap();
        let deserialized: Conversation = serde_json::from_str(&json).unwrap();

        assert_eq!(conv.id, deserialized.id);
        assert_eq!(conv.len(), deserialized.len());
    }

    #[test]
    fn test_last_and_first_message() {
        let conv = Conversation::builder()
            .system("First")
            .user("Middle")
            .assistant("Last")
            .build();

        assert_eq!(conv.first_message().unwrap().content, "First");
        assert_eq!(conv.last_message().unwrap().content, "Last");
    }
}

