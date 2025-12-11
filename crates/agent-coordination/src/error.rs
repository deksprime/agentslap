//! Error types for agent coordination

use agent_core::AgentError;

/// Result type for coordination operations
pub type Result<T> = std::result::Result<T, CoordinationError>;

/// Errors in agent coordination
#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Agent spawn error
    #[error("Agent spawn failed: {0}")]
    SpawnFailed(String),
    
    /// Communication error
    #[error("Communication error: {0}")]
    Communication(#[from] agent_comms::CommsError),
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
    
    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl CoordinationError {
    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create a spawn failed error
    pub fn spawn_failed<S: Into<String>>(msg: S) -> Self {
        Self::SpawnFailed(msg.into())
    }
    
    /// Create a generic other error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}
