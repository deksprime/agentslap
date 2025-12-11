//! Error types for agent communication

use agent_core::AgentError;

/// Result type for communication operations
pub type Result<T> = std::result::Result<T, CommsError>;

/// Errors in agent communication
#[derive(Debug, thiserror::Error)]
pub enum CommsError {
    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Message delivery failed
    #[error("Message delivery failed: {0}")]
    DeliveryFailed(String),

    /// Timeout waiting for response
    #[error("Request timed out after {0:?}")]
    Timeout(std::time::Duration),
    
    /// Permission denied (hierarchy/role violation)
    #[error("Permission denied: {0}")]
    Permission(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl CommsError {
    /// Create a transport error
    pub fn transport<S: Into<String>>(msg: S) -> Self {
        Self::Transport(msg.into())
    }

    /// Create a delivery failed error
    pub fn delivery_failed<S: Into<String>>(msg: S) -> Self {
        Self::DeliveryFailed(msg.into())
    }
    
    /// Create a generic other error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}



