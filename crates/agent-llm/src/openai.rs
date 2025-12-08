//! OpenAI provider implementation

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

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// OpenAI API provider
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `model` - Model to use
    ///   - Latest: "gpt-4o", "gpt-4-turbo"
    ///   - Legacy: "gpt-4", "gpt-3.5-turbo"
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(LLMError::config_error("OpenAI API key cannot be empty"));
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

    /// Convert our messages to OpenAI format
    fn format_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }

    /// Make a retryable API request
    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        request_body: &OpenAIRequest,
    ) -> Result<T> {
        let operation = || async {
            let response = self
                .client
                .post(format!("{}/chat/completions", OPENAI_API_BASE))
                .header("Authorization", format!("Bearer {}", self.api_key))
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
impl LLMProvider for OpenAIProvider {
    async fn send_message_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.format_messages(&messages),
            stream: false,
            temperature: None,
            max_tokens: None,
            tools: Some(tools),
            tool_choice: Some("auto".to_string()),
        };

        self.make_request(&request).await
    }

    async fn send_message(&self, messages: Vec<Message>) -> Result<Response> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.format_messages(&messages),
            stream: false,
            temperature: None,
            max_tokens: None,
            tools: None,
            tool_choice: None,
        };

        let response: OpenAIResponse = self.make_request(&request).await?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| LLMError::parse_error("No choices in response"))?;

        Ok(Response {
            content: choice.message.content.clone().unwrap_or_default(),
            model: response.model,
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            finish_reason: choice.finish_reason.clone(),
        })
    }

    async fn stream_message(&self, messages: Vec<Message>) -> Result<ResponseStream> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.format_messages(&messages),
            stream: true,
            temperature: None,
            max_tokens: None,
            tools: None,
            tool_choice: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", OPENAI_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            .map(|event| match event {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        Ok(StreamChunk {
                            content: String::new(),
                            model: None,
                            finish_reason: Some("stop".to_string()),
                        })
                    } else {
                        let chunk: OpenAIStreamChunk = serde_json::from_str(&event.data)
                            .map_err(|e| LLMError::parse_error(e.to_string()))?;

                        let delta = chunk
                            .choices
                            .first()
                            .ok_or_else(|| LLMError::parse_error("No choices in stream chunk"))?;

                        Ok(StreamChunk {
                            content: delta.delta.content.clone().unwrap_or_default(),
                            model: Some(chunk.model),
                            finish_reason: delta.finish_reason.clone(),
                        })
                    }
                }
                Err(e) => Err(LLMError::stream_error(e.to_string())),
            });

        Ok(Box::pin(stream))
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn name(&self) -> &str {
        "openai"
    }
}

// OpenAI API types

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAIProvider::new("test-key", "gpt-4");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.model(), "gpt-4");
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_empty_api_key() {
        let provider = OpenAIProvider::new("", "gpt-4");
        assert!(provider.is_err());
    }

    #[test]
    fn test_message_formatting() {
        let provider = OpenAIProvider::new("test-key", "gpt-4").unwrap();
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];

        let formatted = provider.format_messages(&messages);
        assert_eq!(formatted.len(), 2);
        assert_eq!(formatted[0].role, "system");
        assert_eq!(formatted[1].role, "user");
    }

    #[test]
    fn test_with_timeout() {
        let provider = OpenAIProvider::new("test-key", "gpt-4")
            .unwrap()
            .with_timeout(Duration::from_secs(30));
        assert_eq!(provider.timeout, Duration::from_secs(30));
    }
}

