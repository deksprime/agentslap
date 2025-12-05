# agent-llm

LLM Provider Abstraction for the Agent Runtime.

## Features

- **Multi-Provider Support**: OpenAI, Anthropic (Claude)
- **Streaming**: Real-time response streaming with Server-Sent Events
- **Retry Logic**: Automatic retry with exponential backoff
- **Error Handling**: Comprehensive error types with retry-ability detection
- **Async**: Built on Tokio for high performance
- **Conversation Management**: Multi-turn conversation tracking with context management
- **Token Counting**: Approximate token estimation for context window management
- **Builder Pattern**: Fluent API for constructing conversations

## Usage

### Building Conversations

```rust
use agent_llm::Conversation;

// Fluent builder pattern
let conversation = Conversation::builder()
    .system("You are a helpful coding assistant")
    .user("How do I create a vector in Rust?")
    .assistant("You can use Vec::new() or the vec! macro")
    .user("Show me an example")
    .build();

// Manual construction
let mut conv = Conversation::new();
conv.add_system("You are helpful");
conv.add_user("Hello!");
conv.add_assistant("Hi there!");

// Token estimation
println!("Estimated tokens: {}", conv.estimate_tokens());

// Truncate to fit context window
conv.truncate_to_tokens(4000);
```

### Basic Completion

```rust
use agent_llm::{OpenAIProvider, LLMProvider, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = OpenAIProvider::new("your-api-key", "gpt-3.5-turbo")?;
    
    let messages = vec![
        Message::system("You are helpful"),
        Message::user("Hello!"),
    ];
    
    let response = provider.send_message(messages).await?;
    println!("{}", response.content);
    
    Ok(())
}
```

### Streaming Completion

```rust
use agent_llm::{OpenAIProvider, LLMProvider, Message};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = OpenAIProvider::new("your-api-key", "gpt-3.5-turbo")?;
    
    let messages = vec![Message::user("Tell me a story")];
    
    let mut stream = provider.stream_message(messages).await?;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        print!("{}", chunk.content);
    }
    
    Ok(())
}
```

### Provider Abstraction

```rust
use agent_llm::{create_provider, LLMProvider, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Provider is determined at runtime
    let provider = create_provider("openai", "api-key", "gpt-3.5-turbo")?;
    // or
    let provider = create_provider("anthropic", "api-key", "claude-3-opus-20240229")?;
    
    let response = provider.send_message(vec![Message::user("Hello")]).await?;
    println!("{}", response.content);
    
    Ok(())
}
```

## Supported Models

### OpenAI
- `gpt-4o` (Latest - multimodal)
- `gpt-4-turbo` (Latest GPT-4)
- `gpt-4` (Legacy)
- `gpt-3.5-turbo` (Fast, cost-effective)
- And all other OpenAI chat models

### Anthropic
- `claude-opus-4-5` (Latest - Nov 2025, 200K context)
- `claude-sonnet-4-5` (Latest - Sept 2025)
- `claude-3-opus-20240229` (Claude 3)
- `claude-3-sonnet-20240229` (Claude 3)
- `claude-3-haiku-20240307` (Claude 3)

## Environment Variables

API keys should be provided via environment variables:

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
```

## Examples

Run examples with:

```bash
# Basic completion (OpenAI)
OPENAI_API_KEY=your-key cargo run -p agent-llm --example basic_completion

# Basic completion (Anthropic)
ANTHROPIC_API_KEY=your-key cargo run -p agent-llm --example basic_completion -- anthropic

# Streaming
OPENAI_API_KEY=your-key cargo run -p agent-llm --example streaming_completion
```

## Error Handling

The crate provides comprehensive error handling with retry support:

```rust
use agent_llm::{LLMError, Result};

async fn example() -> Result<()> {
    // Errors are automatically retried for transient failures
    // (rate limits, server errors, timeouts)
    
    match provider.send_message(messages).await {
        Ok(response) => println!("{}", response.content),
        Err(LLMError::RateLimitExceeded(retry_after)) => {
            // Handle rate limiting
        }
        Err(LLMError::Timeout) => {
            // Handle timeout
        }
        Err(e) => {
            // Other errors
        }
    }
    
    Ok(())
}
```

## Features

- **Automatic Retries**: Exponential backoff for transient errors
- **Rate Limit Handling**: Respects retry-after headers
- **Timeout Support**: Configurable request timeouts
- **Streaming**: Server-Sent Events for real-time responses
- **Type Safety**: Comprehensive Rust types for all API interactions

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

