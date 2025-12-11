# Agent TUI

Interactive terminal interface for chatting with a coordinator agent that can spawn workers and delegate tasks.

## Features

- 💬 **Real-time Chat** - Talk to a GPT-4 coordinator agent
- 🌊 **Streaming Responses** - See agent responses appear character-by-character
- 🔧 **Tool Visualization** - See when tools are being called
- 🎨 **Beautiful UI** - Built with ratatui for a polished experience
- 🤖 **Multi-Agent Coordination** - Coordinator can delegate to worker agents

## Usage

```bash
# Set your OpenAI API key
export OPENAI_API_KEY=sk-...

# Optional: Anthropic for worker agents
export ANTHROPIC_API_KEY=sk-ant-...

# Run the TUI
cargo run -p agent-tui

# Or use the helper script
./scripts/run_tui.sh
```

## Controls

- **Type** - Enter your message
- **Enter** - Send message
- **↑/↓** - Scroll through conversation history
- **Ctrl+Q** - Quit

## What Happens

1. Coordinator agent spawns at startup with all coordination tools
2. You type a message and press Enter
3. Coordinator analyzes your request
4. If complex, coordinator delegates to specialist workers using `delegate_task` tool
5. Response streams back in real-time
6. Tool calls are shown inline with visual indicators

## Example Session

```
You: Analyze Q4 sales performance across East and West regions

Agent: [streaming...]
I'll delegate this analysis to our regional specialists...
[🔧 Tool: delegate_task {...}]
[✓ delegate_task]
Based on the analysis from our regional teams...
[complete response]
```

## Features Demonstrated

- ✅ Real coordinator agent (no mocks)
- ✅ Streaming responses
- ✅ Message history
- ✅ Tool call visualization
- ✅ Multi-agent coordination
- ✅ Clean ratatui UI
