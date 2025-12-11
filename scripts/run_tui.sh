#!/bin/bash
# Run the Agent TUI with API keys

set -e

echo "🚀 Starting Agent TUI..."
echo ""

# Check for API keys
if [ -z "$OPENAI_API_KEY" ]; then
    echo "❌ OPENAI_API_KEY not set"
    echo "   export OPENAI_API_KEY=sk-..."
    exit 1
fi

echo "✅ API keys found"
echo ""
echo "Controls:"
echo "  - Type your message and press Enter"
echo "  - Use Up/Down arrows to scroll"
echo "  - Press Ctrl+Q to quit"
echo ""

cargo run -p agent-tui
