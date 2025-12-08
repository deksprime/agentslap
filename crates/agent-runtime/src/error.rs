//! Error types for agent runtime

use agent_core::AgentError;
use agent_llm::LLMError;
use agent_session::SessionError;
use agent_tools::ToolError;

/// Result type for agent runtime operations
pub type Result<T> = std::result::Result<T, AgentRuntimeError>;

/// Errors that can occur during agent execution
#[derive(Debug, thiserror::Error)]
pub enum AgentRuntimeError {
    /// LLM provider error
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),

    /// Session storage error
    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    /// Tool execution error
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    /// Max iterations exceeded
    #[error("Max iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),

    /// No response from LLM
    #[error("No response from LLM")]
    NoResponse,

    /// Tool call parsing error
    #[error("Failed to parse tool call: {0}")]
    ToolCallParse(String),

    /// Agent not configured properly
    #[error("Agent configuration error: {0}")]
    Configuration(String),

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl AgentRuntimeError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a tool call parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::ToolCallParse(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AgentRuntimeError::config("missing provider");
        assert!(matches!(err, AgentRuntimeError::Configuration(_)));
    }

    #[test]
    fn test_max_iterations() {
        let err = AgentRuntimeError::MaxIterationsExceeded(10);
        assert!(err.to_string().contains("10"));
    }
}

