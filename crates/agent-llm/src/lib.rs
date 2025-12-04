//! LLM Provider Abstraction
//!
//! This crate provides a unified interface for interacting with different
//! Large Language Model providers (OpenAI, Anthropic, etc.).
//!
//! # Example
//!
//! ```no_run
//! use agent_llm::{OpenAIProvider, LLMProvider, Message, MessageRole};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = OpenAIProvider::new("your-api-key", "gpt-4")?;
//!     
//!     let messages = vec![
//!         Message::new(MessageRole::User, "Hello, how are you?"),
//!     ];
//!     
//!     let response = provider.send_message(messages).await?;
//!     println!("Response: {}", response.content);
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod provider;
pub mod types;

// Provider implementations
pub mod openai;
pub mod anthropic;

// Re-exports
pub use error::{LLMError, Result};
pub use provider::LLMProvider;
pub use types::{Message, MessageRole, Response, StreamChunk};

pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;

/// Create a provider from configuration
pub fn create_provider(provider_name: &str, api_key: &str, model: &str) -> Result<Box<dyn LLMProvider>> {
    match provider_name.to_lowercase().as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new(api_key, model)?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(api_key, model)?)),
        _ => Err(LLMError::UnsupportedProvider(provider_name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider_openai() {
        let result = create_provider("openai", "test-key", "gpt-4");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_provider_anthropic() {
        let result = create_provider("anthropic", "test-key", "claude-opus-4-5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_provider_unknown() {
        let result = create_provider("unknown", "test-key", "model");
        assert!(result.is_err());
        if let Err(LLMError::UnsupportedProvider(name)) = result {
            assert_eq!(name, "unknown");
        }
    }
}
