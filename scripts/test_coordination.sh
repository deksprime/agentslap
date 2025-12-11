#!/bin/bash
# Helper script for testing multi-agent coordination

set -e

echo "🧪 Multi-Agent Coordination Test"
echo "=================================="
echo ""

# Check for API keys
if [ -z "$OPENAI_API_KEY" ]; then
    echo "❌ OPENAI_API_KEY not set"
    echo "   export OPENAI_API_KEY=sk-..."
    exit 1
fi

if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ ANTHROPIC_API_KEY not set"
    echo "   export ANTHROPIC_API_KEY=sk-ant-..."
    exit 1
fi

echo "✅ API keys found"
echo ""

# Build
echo "📦 Building project..."
cargo build --release --example multi_agent_coordination

echo ""
echo "🚀 Running multi-agent coordination example..."
echo ""

# Run
cargo run --release --example multi_agent_coordination

echo ""
echo "✅ Test complete!"
