# agent-runtime

Agent Runtime - The composition layer that ties all components together.

## What This Is

This crate combines:
- **agent-llm**: LLM providers and conversations
- **agent-session**: Session storage
- **agent-tools**: Tool calling system
- **agent-core**: Foundation

Into a complete **Agent** that can:
- Have conversations with LLMs
- Execute tools
- Persist sessions
- Handle errors gracefully

## Current Implementation Status

### ✅ What Works
- Agent struct with builder pattern
- LLM integration (OpenAI, Anthropic)
- Session load/save (automatic)
- Conversation management
- Tool registration
- **Tool calling with LLM** (OpenAI and Anthropic)
- Tool execution loop
- Max iteration safeguard

### 🔄 What's Simplified
- Tool results are added as user messages (not official tool_result format)
- Anthropic tool calling needs testing with real API
- Streaming with tools not yet implemented

## Usage

### Basic Agent

```rust
use agent_runtime::Agent;
use agent_llm::OpenAIProvider;
use agent_session::InMemoryStore;
use agent_tools::{ToolRegistry, builtin::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = OpenAIProvider::new(api_key, "gpt-4o")?;
    
    // Register tools
    let tools = ToolRegistry::new();
    tools.register(CalculatorTool)?;
    tools.register(CurrentTimeTool)?;
    
    // Create storage
    let store = InMemoryStore::new();
    
    // Build agent
    let agent = Agent::builder()
        .provider(provider)
        .tools(tools)
        .session_store(store)
        .session_id("user-123")
        .max_iterations(10)
        .build()?;
    
    // Run agent
    let response = agent.run("What is 25 + 17?").await?;
    println!("{}", response);
    // Agent will:
    // 1. See calculator tool is available
    // 2. LLM decides to use it
    // 3. Execute calculator(25, 17)
    // 4. Get result: 42
    // 5. Return: "The answer is 42"
    
    Ok(())
}
```

## How It Works

### Agent Loop

```
1. Load conversation from session (or create new)
2. Add user message
3. Send to LLM WITH tools list
4. LLM response:
   - If tool calls → Execute tools → Add results → Loop back to step 3
   - If final answer → Return to user
5. Save conversation to session
```

### Tool Calling Flow

```
User: "What is the current time?"
  ↓
Agent sends to LLM with tools: [current_time, calculator, echo]
  ↓
LLM: "I'll use the current_time tool"
  ↓
Agent executes: current_time()
  ↓
Tool returns: {"timestamp": "2025-12-08T15:30:00Z"}
  ↓
Agent sends result back to LLM
  ↓
LLM: "The current time is 3:30 PM UTC on December 8th, 2025"
  ↓
User gets final answer
```

## Configuration

```rust
use agent_runtime::AgentConfig;

let config = AgentConfig {
    max_iterations: 10,      // Prevent infinite loops
    session_id: Some("user-session".to_string()),
    system_message: Some("You are helpful".to_string()),
};

let agent = Agent::builder()
    .config(config)
    // ... other components
    .build()?;
```

## Testing

Currently the agent works with:
- ✅ OpenAI (gpt-3.5-turbo, gpt-4, gpt-4o)
- ✅ Anthropic (claude-sonnet-4-5, claude-opus-4-5)

Both support tool calling!

## Examples

### basic_agent.rs

Full working agent with tools:

```bash
# With OpenAI
OPENAI_API_KEY=sk-... cargo run -p agent-runtime --example basic_agent

# With Anthropic  
ANTHROPIC_API_KEY=sk-ant-... cargo run -p agent-runtime --example basic_agent -- anthropic
```

**Note**: The example will actually call the real LLM API and execute tools!

## Architecture

### No Circular Dependencies

```
agent-core (foundation)
    ↑
agent-llm, agent-session, agent-tools
    ↑
agent-runtime ← This crate (composition)
    ↑
agent-cli
```

Clean dependency graph - no cycles!

## Error Handling

All errors from all layers are handled:

```rust
match agent.run("query").await {
    Ok(response) => println!("{}", response),
    Err(AgentRuntimeError::LLM(e)) => // Handle LLM errors,
    Err(AgentRuntimeError::Tool(e)) => // Handle tool errors,
    Err(AgentRuntimeError::Session(e)) => // Handle storage errors,
    Err(AgentRuntimeError::MaxIterationsExceeded(n)) => // Too many iterations,
    Err(e) => // Other errors,
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

