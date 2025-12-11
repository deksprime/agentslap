//! Broadcast to team tool - send messages to all team members

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolRiskLevel, ToolSchema, ToolContext, ToolError};

/// Tool for broadcasting messages to all team members
///
/// Available to all agents that are part of a team.
pub struct BroadcastToTeamTool;

#[derive(Debug, Deserialize, Serialize)]
struct BroadcastParams {
    /// The message to broadcast
    message: String,
    
    /// Optional team name (defaults to agent's own team)
    #[serde(skip_serializing_if = "Option::is_none")]
    team: Option<String>,
}

#[async_trait]
impl Tool for BroadcastToTeamTool {
    fn name(&self) -> &str {
        "broadcast_to_team"
    }

    fn description(&self) -> &str {
        "Broadcast a message to all members of your team. The message will be sent to all agents \
         in the team simultaneously. Optionally specify a different team name."
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for team broadcast")
            .with_properties(serde_json::json!({
                "message": property(
                    "string",
                    "The message to broadcast to the team"
                ),
                "team": property(
                    "string",
                    "Optional: specific team to broadcast to (defaults to your team)"
                ),
            }))
            .with_required(vec!["message".to_string()])
    }

    fn risk_level(&self) -> ToolRiskLevel {
        ToolRiskLevel::Low  // Just sends broadcast message
    }

    async fn execute_with_context(
        &self,
        params: Value,
        context: Option<&ToolContext>,
    ) -> Result<ToolResult> {
        // Parse parameters
        let params: BroadcastParams = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(e.to_string()))?;

        // Require context with coordinator
        let context = context.ok_or_else(|| {
            ToolError::execution("BroadcastToTeamTool requires execution context with coordinator")
        })?;

        #[cfg(feature = "coordination")]
        {
            let coordinator = context.coordinator.as_ref().ok_or_else(|| {
                ToolError::execution("Coordinator not available in context")
            })?;

            // Determine team to broadcast to
            let team = if let Some(team_name) = params.team {
                team_name
            } else {
                // Get agent's own team
                let role = coordinator.hierarchy().get_role(&context.agent_id)
                    .ok_or_else(|| ToolError::execution("Agent not found in hierarchy"))?;
                
                role.team().ok_or_else(|| {
                    ToolError::execution("Agent is not part of a team")
                })?.to_string()
            };

            // Broadcast message
            let recipient_count = coordinator
                .broadcast_to_team(
                    &context.agent_id,
                    &team,
                    serde_json::json!({"broadcast": params.message}),
                )
                .await
                .map_err(|e| ToolError::execution(format!("Broadcast failed: {}", e)))?;

            tracing::info!(
                "Agent {} broadcast to team {} ({} recipients)",
                context.agent_id,
                team,
                recipient_count
            );

            Ok(ToolResult::success(serde_json::json!({
                "team": team,
                "recipients": recipient_count,
                "message": params.message,
                "status": "broadcast",
            })))
        }

        #[cfg(not(feature = "coordination"))]
        {
            Ok(ToolResult::error(
                "Coordination feature is not enabled. Rebuild with --features coordination"
            ))
        }
    }
}

#[cfg(test)]
#[cfg(feature = "coordination")]
mod tests {
    use super::*;
    use agent_comms::{AgentCoordinator, InProcessTransport, AgentRole, AgentAddress};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_broadcast_to_team() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));

        // Setup team with multiple agents
        let coord_role = AgentRole::Coordinator {
            subordinates: vec!["worker-1".to_string(), "worker-2".to_string()],
            team: "team-alpha".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("coord-1"), coord_role);

        let worker1_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-alpha".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-1"), worker1_role);

        let worker2_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-alpha".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-2"), worker2_role);

        let tool = BroadcastToTeamTool;
        let context = ToolContext::with_coordinator("coord-1", coordinator);

        let params = serde_json::json!({
            "message": "Team meeting at 3pm",
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(result.success);
        
        let data = result.data.unwrap();
        assert_eq!(data["team"], "team-alpha");
        assert_eq!(data["recipients"], 3); // coordinator + 2 workers
    }
}
