//! LLM Provider trait definition

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::{Message, Response, Result, StreamChunk};

/// Type alias for a stream of chunks
pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

/// Trait for LLM providers
///
/// Implementations provide a unified interface for different LLM services
/// like OpenAI, Anthropic, etc.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a message and wait for the complete response
    ///
    /// # Arguments
    /// * `messages` - Conversation history including the new message
    ///
    /// # Returns
    /// The complete response from the LLM
    ///
    /// # Example
    /// ```no_run
    /// use agent_llm::{LLMProvider, Message};
    ///
    /// async fn example(provider: &dyn LLMProvider) -> Result<(), Box<dyn std::error::Error>> {
    ///     let messages = vec![Message::user("Hello!")];
    ///     let response = provider.send_message(messages).await?;
    ///     println!("{}", response.content);
    ///     Ok(())
    /// }
    /// ```
    async fn send_message(&self, messages: Vec<Message>) -> Result<Response>;

    /// Send a message with tools and get raw JSON response
    ///
    /// This is used for tool calling - returns the raw API response
    /// which may contain tool calls.
    ///
    /// # Arguments
    /// * `messages` - Conversation history
    /// * `tools` - Tools available to the LLM
    ///
    /// # Returns
    /// Raw JSON response from the API
    async fn send_message_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value>;

    /// Send a message and stream the response as it's generated
    ///
    /// # Arguments
    /// * `messages` - Conversation history including the new message
    ///
    /// # Returns
    /// A stream of response chunks
    ///
    /// # Example
    /// ```no_run
    /// use agent_llm::{LLMProvider, Message};
    /// use futures::StreamExt;
    ///
    /// async fn example(provider: &dyn LLMProvider) -> Result<(), Box<dyn std::error::Error>> {
    ///     let messages = vec![Message::user("Tell me a story")];
    ///     let mut stream = provider.stream_message(messages).await?;
    ///     
    ///     while let Some(chunk) = stream.next().await {
    ///         let chunk = chunk?;
    ///         print!("{}", chunk.content);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn stream_message(&self, messages: Vec<Message>) -> Result<ResponseStream>;

    /// Get the model name/identifier
    fn model(&self) -> &str;

    /// Get the provider name
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockProvider;

    #[async_trait]
    impl LLMProvider for MockProvider {
        async fn send_message(&self, _messages: Vec<Message>) -> Result<Response> {
            Ok(Response {
                content: "Mock response".to_string(),
                model: "mock-model".to_string(),
                usage: None,
                finish_reason: Some("stop".to_string()),
            })
        }

        async fn send_message_with_tools(
            &self,
            _messages: Vec<Message>,
            _tools: Vec<serde_json::Value>,
        ) -> Result<serde_json::Value> {
            Ok(serde_json::json!({
                "choices": [{
                    "message": {
                        "content": "Mock response with tools"
                    }
                }]
            }))
        }

        async fn stream_message(&self, _messages: Vec<Message>) -> Result<ResponseStream> {
            use futures::stream;
            let chunks = vec![
                Ok(StreamChunk::new("Hello")),
                Ok(StreamChunk::new(" world")),
            ];
            Ok(Box::pin(stream::iter(chunks)))
        }

        fn model(&self) -> &str {
            "mock-model"
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider;
        let response = provider.send_message(vec![Message::user("test")]).await.unwrap();
        assert_eq!(response.content, "Mock response");
        assert_eq!(provider.model(), "mock-model");
        assert_eq!(provider.name(), "mock");
    }
}




