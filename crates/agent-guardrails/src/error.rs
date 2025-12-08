//! Error types for guardrails

use agent_core::AgentError;

/// Result type for guardrail operations
pub type Result<T> = std::result::Result<T, GuardrailError>;

/// Errors that can occur in guardrail operations
#[derive(Debug, thiserror::Error)]
pub enum GuardrailError {
    /// Generic guardrail error
    #[error("Guardrail error: {0}")]
    Error(String),

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl GuardrailError {
    /// Create a guardrail error
    pub fn error<S: Into<String>>(msg: S) -> Self {
        Self::Error(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = GuardrailError::error("test error");
        assert!(matches!(err, GuardrailError::Error(_)));
    }
}

