# Agent TUI (HTTP Client)

**Pure HTTP client** for agent-server with streaming support.

## Quick Start

```bash
# Terminal 1: Start server
export OPENAI_API_KEY=sk-...
cargo run -p agent-server

# Terminal 2: Start TUI
cargo run -p agent-tui
```

TUI will:
1. Connect to `http://localhost:3000`
2. Create a GPT-4 agent via API
3. Stream responses in real-time

## Architecture

```
TUI (HTTP Client)          agent-server
    ↓ HTTP                     ↓
POST /agents            Create agent
POST /agents/:id/stream  ← SSE streaming
```

## Features

✅ Pure HTTP client (~300 LOC vs 500+)  
✅ Server-Sent Events (SSE) streaming  
✅ Character-by-character display  
✅ Tool call visualization  
✅ Clean separation - no business logic  

## Controls

- **Type** - Enter message
- **Enter** - Send
- **↑/↓** - Scroll
- **Ctrl+Q** - Quit

## Implementation

- [client.rs](file:///home/deks/new_agents/agentslap/crates/agent-tui/src/client.rs) - HTTP wrapper with reqwest
- [app.rs](file:///home/deks/new_agents/agentslap/crates/agent-tui/src/app.rs) - State management
- [ui.rs](file:///home/deks/new_agents/agentslap/crates/agent-tui/src/ui.rs) - Ratatui rendering

**No agent dependencies!** Pure UI code.
