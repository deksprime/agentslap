//! Error types for LLM operations

use agent_core::AgentError;

/// Result type for LLM operations
pub type Result<T> = std::result::Result<T, LLMError>;

/// Errors that can occur during LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// API returned an error
    #[error("API error: {0}")]
    ApiError(String),

    /// Failed to parse API response
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Unsupported provider
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Retry after: {0:?}")]
    RateLimitExceeded(Option<u64>),

    /// Streaming error
    #[error("Streaming error: {0}")]
    StreamError(String),

    /// Timeout
    #[error("Request timed out")]
    Timeout,

    /// Generic error from agent-core
    #[error(transparent)]
    CoreError(#[from] AgentError),
}

impl LLMError {
    /// Create an API error
    pub fn api_error<S: Into<String>>(msg: S) -> Self {
        Self::ApiError(msg.into())
    }

    /// Create a parse error
    pub fn parse_error<S: Into<String>>(msg: S) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a config error
    pub fn config_error<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a stream error
    pub fn stream_error<S: Into<String>>(msg: S) -> Self {
        Self::StreamError(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::HttpError(_) | Self::RateLimitExceeded(_) | Self::Timeout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = LLMError::api_error("test error");
        assert!(matches!(err, LLMError::ApiError(_)));
        assert_eq!(err.to_string(), "API error: test error");
    }

    #[test]
    fn test_is_retryable() {
        assert!(LLMError::Timeout.is_retryable());
        assert!(LLMError::RateLimitExceeded(None).is_retryable());
        assert!(!LLMError::ConfigError("test".to_string()).is_retryable());
    }
}




