# Agent Server - Full Feature API Reference

Complete REST API server for multi-agent coordination with **19 endpoints** covering all capabilities.

## Quick Start

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...  # Optional

cargo run -p agent-server
```

Server starts on `http://localhost:3000`

---

## 1. Role Management (4 endpoints)

### Register Role Template
```bash
POST /roles
Content-Type: application/json

{
  "role": "coordinator",
  "system_message": "You are a coordinator managing a team",
  "model": "gpt-4",
  "provider": "openai",
  "tools": ["DelegateTaskTool", "BroadcastToTeamTool", "SendMessageTool"],
  "hierarchy_role": "coordinator",
  "team": "sales-team",
  "max_iterations": 5
}
```

### List Roles
```bash
GET /roles
```

### Get Role Details
```bash
GET /roles/:role_name
```

### Delete Role
```bash
DELETE /roles/:role_name
```

---

## 2. Agent Management (4 endpoints)

### Create Agent (Enhanced)

**Option A: From Role Template**
```bash
POST /agents
Content-Type: application/json

{
  "role": "coordinator",  # Use registered role
  "name": "team-lead"
}
```

**Option B: Direct Creation (Full Control)**
```bash
POST /agents
Content-Type: application/json

{
  "name": "my-agent",
  "model": "gpt-4",
  "system_message": "Custom prompt",
  "hierarchy_role": "worker",
  "team": "sales-team",
  "supervisor": "coord-123",
  "tools": ["CalculatorTool", "DelegateTaskTool", "EscalateTool"],
  "max_iterations": 10,
  "session_backend": "memory"
}
```

### List Agents
```bash
GET /agents

Response: [
  {
    "id": "uuid",
    "name": "...",
    "model": "gpt-4",
    "role": "coordinator",
    "hierarchy_role": "coordinator",
    "team": "sales-team",
    "supervisor": null,
    "subordinates": ["worker-1", "worker-2"],
    "tools": ["DelegateTaskTool", ...],
    "max_iterations": 5,
    "message_count": 0,
    "created_at": "..."
  }
]
```

### Get Agent
```bash
GET /agents/:agent_id
```

### Delete Agent
```bash
DELETE /agents/:agent_id
```

---

## 3. Messaging (3 endpoints)

### Send Message (Non-Streaming)
```bash
POST /agents/:agent_id/messages
Content-Type: application/json

{
  "content": "Hello!"
}
```

### Send Message (Streaming SSE)
```bash
POST /agents/:agent_id/stream
Content-Type: application/json

{
  "content": "Analyze Q4 sales"
}

# SSE Stream Response:
data: {"type":"text_chunk","content":"I'll "}
data: {"type":"tool_call","name":"delegate_task","params":"..."}
data: {"type":"tool_result","name":"delegate_task","success":true}
data: {"type":"text_chunk","content":"analyze"}
data: {"type":"done","total_tokens":150}
```

### Get Message History
```bash
GET /agents/:agent_id/messages
```

---

## 4. Coordination Operations (3 endpoints)

### Delegate Task
```bash
POST /agents/:agent_id/delegate
Content-Type: application/json

{
  "task_description": "Analyze regional sales data",
  "worker_id": "worker-123"  # Optional, auto-selects if omitted
}

Response: {
  "task_id": "uuid",
  "worker_id": "worker-123",
  "status": "delegated"
}
```

### Escalate to Supervisor
```bash
POST /agents/:agent_id/escalate
Content-Type: application/json

{
  "reason": "Need additional resources",
  "context": {"resource": "database_access"},
  "task_id": "optional-task-id"
}

Response: {
  "escalation_id": "uuid",
  "supervisor_id": "coord-123",
  "status": "escalated"
}
```

### Broadcast to Team
```bash
POST /agents/:agent_id/broadcast
Content-Type: application/json

{
  "message": "Team meeting at 3pm",
  "team": "sales-team"  # Optional, uses agent's team if omitted
}

Response: {
  "team": "sales-team",
  "recipients": 5,
  "status": "broadcast"
}
```

---

## 5. Team Management (2 endpoints)

### List All Teams
```bash
GET /teams

Response: ["sales-team", "eng-team", "support-team"]
```

### Get Team Details
```bash
GET /teams/:team_name

Response: {
  "team": "sales-team",
  "members": ["coord-1", "worker-1", "worker-2"],
  "coordinators": ["coord-1"],
  "workers": ["worker-1", "worker-2"]
}
```

---

## Available Tools

### Basic Tools
- `CalculatorTool` - Math calculations
- `CurrentTimeTool` - Get current time
- `EchoTool` - Echo input

### Coordination Tools
- `DelegateTaskTool` - Delegate to subordinate (coordinators only)
- `EscalateTool` - Escalate to supervisor (workers only)
- `BroadcastToTeamTool` - Broadcast to team
- `SendMessageTool` - Direct agent-to-agent messaging

---

## Example Workflows

### 1. Create Coordinator-Worker Hierarchy

```bash
# 1. Register coordinator role
POST /roles { ... coordinator config ... }

# 2. Create coordinator
COORD_ID=$(POST /agents {"role":"coordinator"})

# 3. Create workers
POST /agents {
  "hierarchy_role": "worker",
  "team": "sales-team",
  "supervisor": "$COORD_ID"
}

# 4. Get team info
GET /teams/sales-team
```

### 2. Delegate and Track Task

```bash
# Coordinator delegates
POST /agents/$COORD_ID/delegate {
  "task_description": "Analyze Q4 data",
  "worker_id": "$WORKER_ID"
}
# Returns task_id

# Worker escalates if needed
POST /agents/$WORKER_ID/escalate {
  "reason": "Missing data access",
  "task_id": "..."
}
```

### 3. Team Communication

```bash
# Coordinator broadcasts
POST /agents/$COORD_ID/broadcast {
  "message": "Deadline extended to Friday"
}

# Check who received it
GET /teams/sales-team
```

---

## Architecture

```
Client (TUI/Web/CLI)
    ↓ HTTP/SSE
Agent Server (Axum)
    ↓
Agent Factory + Coordination
    ↓
Agent Runtime + Tools
```

**All 19 endpoints expose the full multi-agent coordination system via HTTP!**

---

## Test It

```bash
# Full test suite
./scripts/test_full_api.sh

# Simple test
./scripts/test_api.sh
```

---

## Features

✅ Role-based agent spawning  
✅ Coordinator/Worker hierarchies  
✅ Task delegation  
✅ Escalation  
✅ Team broadcasting  
✅ Streaming responses (SSE)  
✅ Message history  
✅ All 7 coordination tools  
✅ OpenAI + Anthropic support  
✅ Full CRUD for agents & roles  

**The server is now a complete multi-agent backend!**
