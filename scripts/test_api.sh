#!/bin/bash
# Test Agent Server API

SERVER="http://localhost:3000"

echo "🧪 Testing Agent Server API"
echo "=============================="
echo ""

# 1. Health check
echo "1. Health Check"
curl -s $SERVER/ && echo ""
echo ""

# 2. Create agent
echo "2. Creating Agent"
AGENT_ID=$(curl -s -X POST $SERVER/agents \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-3.5-turbo","name":"test-agent"}' \
  | jq -r '.id')

echo "   Agent ID: $AGENT_ID"
echo ""

# 3. List agents
echo "3. Listing Agents"
curl -s $SERVER/agents | jq '.'
echo ""

# 4. Send message (non-streaming)
echo "4. Sending Message (non-streaming)"
curl -s -X POST $SERVER/agents/$AGENT_ID/messages \
  -H "Content-Type: application/json" \
  -d '{"content":"What is 2+2?"}' \
  | jq '.'
echo ""

# 5. Get message history
echo "5. Getting Message History"
curl -s $SERVER/agents/$AGENT_ID/messages | jq '.'
echo ""

# 6. Test streaming
echo "6. Streaming Message (SSE)"
curl -N -X POST $SERVER/agents/$AGENT_ID/stream \
  -H "Content-Type: application/json" \
  -d '{"content":"Tell me a short joke"}'
echo ""
echo ""

# 7. Delete agent
echo "7. Deleting Agent"
curl -s -X DELETE $SERVER/agents/$AGENT_ID
echo "   Deleted"
echo ""

echo "✅ Tests complete!"
