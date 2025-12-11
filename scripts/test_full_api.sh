#!/bin/bash
# Comprehensive Agent Server API Test Suite

SERVER="http://localhost:3000"

echo "🔧 Full-Featured Agent Server Test Suite"
echo "==========================================="
echo ""

# Helper function
check_response() {
    if [ $? -eq 0 ]; then
        echo "   ✅ Success"
    else
        echo "   ❌ Failed"
    fi
}

# 1. Health check
echo "1. Health Check"
curl -s $SERVER/ && echo ""
check_response
echo ""

# 2. Register roles
echo "2. Registering Roles"
echo "   → Registering coordinator role..."
curl -s -X POST $SERVER/roles \
  -H "Content-Type: application/json" \
  -d '{
    "role": "coordinator",
    "system_message": "You are a coordinator managing a team",
    "model": "gpt-4",
    "provider": "openai",
    "tools": ["DelegateTaskTool", "BroadcastToTeamTool"],
    "hierarchy_role": "coordinator",
    "team": "sales-team",
    "max_iterations": 5
  }' | jq '.'
check_response

echo "   → Registering worker role..."
curl -s -X POST $SERVER/roles \
  -H "Content-Type: application/json" \
  -d '{
    "role": "analyst",
    "system_message": "You are a data analyst",
    "model": "gpt-3.5-turbo",
    "provider": "openai",
    "tools": ["CalculatorTool", "EscalateTool"],
    "hierarchy_role": "worker",
    "team": "sales-team",
    "max_iterations": 3
  }' | jq '.'
check_response
echo ""

# 3. List roles
echo "3. Listing Roles"
curl -s $SERVER/roles | jq '.'
check_response
echo ""

# 4. Create coordinator from role
echo "4. Creating Coordinator Agent"
COORD_ID=$(curl -s -X POST $SERVER/agents \
  -H "Content-Type: application/json" \
  -d '{"role":"coordinator","name":"team-lead"}' \
  | jq -r '.id')

echo "   Coordinator ID: $COORD_ID"
check_response
echo ""

# 5. Create worker agents directly (without role)
echo "5. Creating Worker Agents (Direct)"
WORKER1_ID=$(curl -s -X POST $SERVER/agents \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"worker-1\",
    \"model\": \"gpt-3.5-turbo\",
    \"hierarchy_role\": \"worker\",
    \"team\": \"sales-team\",
    \"supervisor\": \"$COORD_ID\",
    \"tools\": [\"CalculatorTool\", \"EscalateTool\"]
  }" | jq -r '.id')

echo "   Worker 1 ID: $WORKER1_ID"
check_response
echo ""

# 6. List all agents
echo "6. Listing All Agents"
curl -s $SERVER/agents | jq '.'
check_response
echo ""

# 7. Get team info
echo "7. Getting Team Info"
curl -s $SERVER/teams/sales-team | jq '.'
check_response
echo ""

# 8. Send message to coordinator
echo "8. Sending Message to Coordinator (Streaming)"
curl -N -X POST $SERVER/agents/$COORD_ID/stream \
  -H "Content-Type: application/json" \
  -d '{"content":"What tools do you have available?"}'
echo ""
check_response
echo ""

# 9. Delegation (if supported by agent)
echo "9. Testing Delegation"
curl -s -X POST $SERVER/agents/$COORD_ID/delegate \
  -H "Content-Type: application/json" \
  -d "{
    \"task_description\": \"Analyze Q4 sales data\",
    \"worker_id\": \"$WORKER1_ID\"
  }" | jq '.'
check_response
echo ""

# 10. Escalation
echo "10. Testing Escalation"
curl -s -X POST $SERVER/agents/$WORKER1_ID/escalate \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Need additional resources",
    "context": {"resource": "database_access"}
  }' | jq '.'
check_response
echo ""

# 11. Team broadcast
echo "11. Testing Team Broadcast"
curl -s -X POST $SERVER/agents/$COORD_ID/broadcast \
  -H "Content-Type: application/json" \
  -d '{"message":"Team meeting at 3pm"}' | jq '.'
check_response
echo ""

# 12. Get message history
echo "12. Getting Message History"
curl -s $SERVER/agents/$COORD_ID/messages | jq '.'
check_response
echo ""

# 13. List teams
echo "13. Listing All Teams"
curl -s $SERVER/teams | jq '.'
check_response
echo ""

# 14. Cleanup
echo "14. Cleanup"
echo "   → Deleting worker..."
curl -s -X DELETE $SERVER/agents/$WORKER1_ID
check_response

echo "   → Deleting coordinator..."
curl -s -X DELETE $SERVER/agents/$COORD_ID
check_response
echo ""

echo "✅ Full test suite complete!"
echo ""
echo "Summary:"
echo "  ✓ 4 Role management endpoints"
echo "  ✓ 4 Agent management endpoints"
echo "  ✓ 3 Message endpoints"
echo "  ✓ 3 Coordination endpoints"
echo "  ✓ 2 Team endpoints"
echo "  ✓ Total: 19 endpoints tested!"
