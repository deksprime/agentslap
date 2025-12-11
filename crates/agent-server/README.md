# Agent Server

REST API server for managing agents and handling message conversations.

## Quick Start

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...  # Optional

cargo run -p agent-server
```

Server starts on `http://localhost:3000`

## API Endpoints

### Create Agent
```bash
POST /agents
Content-Type: application/json

{
  "name": "my-agent",
  "model": "gpt-4",  # or "gpt-3.5-turbo", "claude-sonnet-4"
  "system_message": "You are a helpful assistant"
}

Response: {
  "id": "uuid",
  "name": "my-agent",
  "model": "gpt-4",
  "created_at": "2025-12-11T...",
  "message_count": 0
}
```

### List Agents
```bash
GET /agents

Response: [{ agent objects }]
```

### Get Agent
```bash
GET /agents/:id

Response: { agent object }
```

### Delete Agent
```bash
DELETE /agents/:id

Response: 204 No Content
```

### Send Message (Non-Streaming)
```bash
POST /agents/:id/messages
Content-Type: application/json

{
  "content": "Hello!"
}

Response: {
  "role": "agent",
  "content": "Response text",
  "timestamp": "..."
}
```

### Send Message (Streaming - SSE)
```bash
POST /agents/:id/stream
Content-Type: application/json

{
  "content": "Complex question"
}

Response: text/event-stream
data: {"type":"text_chunk","content":"Hello"}
data: {"type":"tool_call","name":"calculator","params":"..."}
data: {"type":"tool_result","name":"calculator","success":true}
data: {"type":"done","total_tokens":150}
```

### Get Message History
```bash
GET /agents/:id/messages

Response: [
  {
    "role": "user",
    "content": "...",
    "timestamp": "..."
  },
  ...
]
```

## Event Types (SSE)

- `text_chunk` - Streaming text content
- `tool_call` - Tool being executed
- `tool_result` - Tool execution complete
- `done` - Response complete
- `error` - Error occurred

## Example Usage

```bash
# Create agent
AGENT_ID=$(curl -X POST http://localhost:3000/agents \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4"}' | jq -r '.id')

# Send streaming message
curl -N -X POST http://localhost:3000/agents/$AGENT_ID/stream \
  -H "Content-Type: application/json" \
  -d '{"content":"What is 2+2?"}'

# Get history
curl http://localhost:3000/agents/$AGENT_ID/messages
```

## Architecture

```
Client (TUI/Web/CLI)
    ↓ HTTP/SSE
Agent Server (Axum)
    ↓
Agent Runtime + Coordination
```

The server is a thin API layer - all agent logic stays in the core crates.
