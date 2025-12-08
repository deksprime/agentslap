//! Error types for session management

use agent_core::AgentError;

/// Result type for session operations
pub type Result<T> = std::result::Result<T, SessionError>;

/// Errors that can occur during session operations
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// Session not found
    #[error("Session not found: {0}")]
    NotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Session expired
    #[error("Session expired: {0}")]
    Expired(String),

    /// Invalid session ID
    #[error("Invalid session ID: {0}")]
    InvalidId(String),

    /// Storage capacity exceeded
    #[error("Storage capacity exceeded")]
    CapacityExceeded,

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl SessionError {
    /// Create a storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Self::Storage(msg.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(session_id: S) -> Self {
        Self::NotFound(session_id.into())
    }

    /// Create an expired error
    pub fn expired<S: Into<String>>(session_id: S) -> Self {
        Self::Expired(session_id.into())
    }

    /// Create an invalid ID error
    pub fn invalid_id<S: Into<String>>(session_id: S) -> Self {
        Self::InvalidId(session_id.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SessionError::not_found("test-session");
        assert!(matches!(err, SessionError::NotFound(_)));
        assert_eq!(err.to_string(), "Session not found: test-session");
    }

    #[test]
    fn test_storage_error() {
        let err = SessionError::storage("disk full");
        assert!(matches!(err, SessionError::Storage(_)));
    }

    #[test]
    fn test_expired_error() {
        let err = SessionError::expired("old-session");
        assert!(matches!(err, SessionError::Expired(_)));
    }
}




