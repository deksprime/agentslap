//! Streaming support for agent responses

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Events that can be emitted during agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Text content chunk from LLM
    TextChunk {
        /// The text content
        content: String,
    },

    /// LLM is requesting to use a tool
    ToolCallStart {
        /// Tool being called
        tool_name: String,
        /// Tool parameters
        parameters: Value,
    },

    /// Tool execution completed
    ToolCallEnd {
        /// Tool that was called
        tool_name: String,
        /// Whether execution was successful
        success: bool,
        /// Result data (if successful)
        result: Option<Value>,
        /// Error message (if failed)
        error: Option<String>,
    },

    /// Agent thinking/processing
    Thinking {
        /// What the agent is thinking about
        message: String,
    },

    /// Final response is complete
    Done {
        /// Total tokens used (if available)
        total_tokens: Option<usize>,
    },

    /// An error occurred
    Error {
        /// Error message
        message: String,
    },
}

impl AgentEvent {
    /// Create a text chunk event
    pub fn text<S: Into<String>>(content: S) -> Self {
        Self::TextChunk {
            content: content.into(),
        }
    }

    /// Create a tool call start event
    pub fn tool_call_start<S: Into<String>>(tool_name: S, parameters: Value) -> Self {
        Self::ToolCallStart {
            tool_name: tool_name.into(),
            parameters,
        }
    }

    /// Create a tool call end event
    pub fn tool_call_end<S: Into<String>>(
        tool_name: S,
        success: bool,
        result: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self::ToolCallEnd {
            tool_name: tool_name.into(),
            success,
            result,
            error,
        }
    }

    /// Create a thinking event
    pub fn thinking<S: Into<String>>(message: S) -> Self {
        Self::Thinking {
            message: message.into(),
        }
    }

    /// Create a done event
    pub fn done(total_tokens: Option<usize>) -> Self {
        Self::Done { total_tokens }
    }

    /// Create an error event
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_event() {
        let event = AgentEvent::text("Hello");
        assert!(matches!(event, AgentEvent::TextChunk { .. }));
    }

    #[test]
    fn test_tool_events() {
        let start = AgentEvent::tool_call_start("calc", serde_json::json!({"a": 5}));
        assert!(matches!(start, AgentEvent::ToolCallStart { .. }));

        let end = AgentEvent::tool_call_end("calc", true, Some(serde_json::json!(42)), None);
        assert!(matches!(end, AgentEvent::ToolCallEnd { .. }));
    }

    #[test]
    fn test_event_serialization() {
        let event = AgentEvent::text("Test");
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AgentEvent = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized, AgentEvent::TextChunk { .. }));
    }
}

