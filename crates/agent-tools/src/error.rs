//! Error types for tool operations

use agent_core::AgentError;

/// Result type for tool operations
pub type Result<T> = std::result::Result<T, ToolError>;

/// Errors that can occur during tool operations
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ExecutionError(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Tool timeout
    #[error("Tool execution timed out")]
    Timeout,

    /// Tool already registered
    #[error("Tool already registered: {0}")]
    AlreadyRegistered(String),

    /// Schema generation error
    #[error("Schema error: {0}")]
    SchemaError(String),

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl ToolError {
    /// Create an execution error
    pub fn execution<S: Into<String>>(msg: S) -> Self {
        Self::ExecutionError(msg.into())
    }

    /// Create an invalid parameters error
    pub fn invalid_params<S: Into<String>>(msg: S) -> Self {
        Self::InvalidParameters(msg.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(tool_name: S) -> Self {
        Self::NotFound(tool_name.into())
    }

    /// Create a schema error
    pub fn schema<S: Into<String>>(msg: S) -> Self {
        Self::SchemaError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ToolError::not_found("calculator");
        assert!(matches!(err, ToolError::NotFound(_)));
        assert_eq!(err.to_string(), "Tool not found: calculator");
    }

    #[test]
    fn test_execution_error() {
        let err = ToolError::execution("division by zero");
        assert!(matches!(err, ToolError::ExecutionError(_)));
    }

    #[test]
    fn test_invalid_params() {
        let err = ToolError::invalid_params("missing field 'x'");
        assert!(matches!(err, ToolError::InvalidParameters(_)));
    }
}

