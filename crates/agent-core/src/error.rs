//! Error types for the agent runtime
//!
//! This module defines a comprehensive error type that can represent
//! all possible errors in the agent system.

/// Result type alias for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;

/// Main error type for the agent runtime
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration parsing errors
    #[error("Config parse error: {0}")]
    ConfigParse(#[from] config::ConfigError),

    /// Generic errors with context
    #[error("{0}")]
    Other(String),
}

impl AgentError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AgentError::config("test error");
        assert!(matches!(err, AgentError::Config(_)));
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AgentError::from(io_err);
        assert!(matches!(err, AgentError::Io(_)));
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(returns_result().unwrap(), 42);
    }
}
