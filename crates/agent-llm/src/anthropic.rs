//! Anthropic (Claude) provider implementation

use async_trait::async_trait;
use backoff::{future::retry, ExponentialBackoff};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
    error::{LLMError, Result},
    provider::{LLMProvider, ResponseStream},
    types::{Message, MessageRole, Response, StreamChunk, TokenUsage},
};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";
// API version - check https://docs.anthropic.com for latest
// Note: "2023-06-01" is stable and works with all Claude 4.x models
// Update to latest version as needed for new features
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic (Claude) API provider
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    ///
    /// # Arguments
    /// * `api_key` - Anthropic API key
    /// * `model` - Model to use
    ///   - Latest (2025): "claude-opus-4-5", "claude-sonnet-4-5"
    ///   - Claude 3: "claude-3-opus-20240229", "claude-3-sonnet-20240229", "claude-3-haiku-20240307"
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(LLMError::config_error("Anthropic API key cannot be empty"));
        }

        Ok(Self {
            client: Client::new(),
            api_key,
            model: model.into(),
            timeout: Duration::from_secs(60),
        })
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Convert our messages to Anthropic format
    fn format_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        // Anthropic uses system parameter separately
        let system = messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| m.content.clone());

        let messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|msg| AnthropicMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => unreachable!(), // filtered out
                },
                content: msg.content.clone(),
            })
            .collect();

        (system, messages)
    }

    /// Make a retryable API request
    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        request_body: &AnthropicRequest,
    ) -> Result<T> {
        let operation = || async {
            let response = self
                .client
                .post(format!("{}/messages", ANTHROPIC_API_BASE))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("Content-Type", "application/json")
                .timeout(self.timeout)
                .json(request_body)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        backoff::Error::Permanent(LLMError::Timeout)
                    } else {
                        backoff::Error::Transient {
                            err: LLMError::HttpError(e),
                            retry_after: None,
                        }
                    }
                })?;

            let status = response.status();

            // Handle rate limiting
            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after_secs: Option<u64> = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok());

                return Err(backoff::Error::Transient {
                    err: LLMError::RateLimitExceeded(retry_after_secs),
                    retry_after: retry_after_secs.map(Duration::from_secs),
                });
            }

            // Handle server errors (retryable)
            if status.is_server_error() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(backoff::Error::Transient {
                    err: LLMError::api_error(format!("Server error: {}", error_text)),
                    retry_after: None,
                });
            }

            // Handle client errors (not retryable)
            if status.is_client_error() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(backoff::Error::Permanent(LLMError::api_error(format!(
                    "Client error ({}): {}",
                    status, error_text
                ))));
            }

            // Parse successful response
            response
                .json::<T>()
                .await
                .map_err(|e| backoff::Error::Permanent(LLMError::parse_error(e.to_string())))
        };

        let backoff_config = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(30)),
            ..Default::default()
        };

        retry(backoff_config, operation).await
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn send_message(&self, messages: Vec<Message>) -> Result<Response> {
        let (system, formatted_messages) = self.format_messages(&messages);

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: formatted_messages,
            system,
            max_tokens: 4096, // Required by Anthropic
            stream: false,
            temperature: None,
        };

        let response: AnthropicResponse = self.make_request(&request).await?;

        let content = response
            .content
            .first()
            .ok_or_else(|| LLMError::parse_error("No content in response"))?;

        Ok(Response {
            content: content.text.clone(),
            model: response.model,
            usage: Some(TokenUsage {
                prompt_tokens: response.usage.input_tokens,
                completion_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            }),
            finish_reason: Some(response.stop_reason),
        })
    }

    async fn stream_message(&self, messages: Vec<Message>) -> Result<ResponseStream> {
        let (system, formatted_messages) = self.format_messages(&messages);

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: formatted_messages,
            system,
            max_tokens: 4096,
            stream: true,
            temperature: None,
        };

        let response = self
            .client
            .post(format!("{}/messages", ANTHROPIC_API_BASE))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(&request)
            .send()
            .await
            .map_err(LLMError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::api_error(format!(
                "API error ({}): {}",
                status,
                error_text
            )));
        }

        let stream = response
            .bytes_stream()
            .eventsource()
            .filter_map(|event| async move {
                match event {
                    Ok(event) => {
                        // Anthropic sends different event types
                        if event.event == "content_block_delta" {
                            match serde_json::from_str::<AnthropicStreamEvent>(&event.data) {
                                Ok(chunk) => {
                                    if let Some(delta) = chunk.delta {
                                        Some(Ok(StreamChunk {
                                            content: delta.text.unwrap_or_default(),
                                            model: None,
                                            finish_reason: None,
                                        }))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => Some(Err(LLMError::parse_error(e.to_string()))),
                            }
                        } else if event.event == "message_stop" {
                            Some(Ok(StreamChunk {
                                content: String::new(),
                                model: None,
                                finish_reason: Some("end_turn".to_string()),
                            }))
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(LLMError::stream_error(e.to_string()))),
                }
            });

        Ok(Box::pin(stream))
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

// Anthropic API types

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    model: String,
    content: Vec<AnthropicContent>,
    stop_reason: String,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    _type: String,
    delta: Option<AnthropicStreamDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key", "claude-sonnet-4-5");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.model(), "claude-sonnet-4-5");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_empty_api_key() {
        let provider = AnthropicProvider::new("", "claude-sonnet-4-5");
        assert!(provider.is_err());
    }

    #[test]
    fn test_message_formatting() {
        let provider = AnthropicProvider::new("test-key", "claude-sonnet-4-5").unwrap();
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi"),
        ];

        let (system, formatted) = provider.format_messages(&messages);
        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(formatted.len(), 2); // System message separated
        assert_eq!(formatted[0].role, "user");
        assert_eq!(formatted[1].role, "assistant");
    }

    #[test]
    fn test_with_timeout() {
        let provider = AnthropicProvider::new("test-key", "claude-sonnet-4-5")
            .unwrap()
            .with_timeout(Duration::from_secs(30));
        assert_eq!(provider.timeout, Duration::from_secs(30));
    }
}

