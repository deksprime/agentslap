//! Error types for HITL

use agent_core::AgentError;

/// Result type for HITL operations
pub type Result<T> = std::result::Result<T, HitlError>;

/// Errors that can occur in HITL operations
#[derive(Debug, thiserror::Error)]
pub enum HitlError {
    /// Approval timeout
    #[error("Approval request timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Approval denied
    #[error("Approval denied: {0}")]
    Denied(String),

    /// Invalid request
    #[error("Invalid approval request: {0}")]
    InvalidRequest(String),

    /// Strategy error
    #[error("Approval strategy error: {0}")]
    StrategyError(String),

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl HitlError {
    /// Create a strategy error
    pub fn strategy<S: Into<String>>(msg: S) -> Self {
        Self::StrategyError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = HitlError::strategy("test error");
        assert!(matches!(err, HitlError::StrategyError(_)));
    }
}

