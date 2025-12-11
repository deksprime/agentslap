//! Escalate tool - allows workers to escalate issues to their supervisor

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolRiskLevel, ToolSchema, ToolContext, ToolError};

/// Tool for worker agents to escalate issues to their supervisor
///
/// Only available to worker agents (those with a supervisor).
/// Allows natural escalation through LLM conversation.
pub struct EscalateTool;

#[derive(Debug, Deserialize, Serialize)]
struct EscalateParams {
    /// Reason for escalation
    reason: String,
    
    /// Additional context or details
    #[serde(default)]
    context: Value,
    
    /// Optional task ID if escalating about a specific delegated task
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
}

#[async_trait]
impl Tool for EscalateTool {
    fn name(&self) -> &str {
        "escalate"
    }

    fn description(&self) -> &str {
        "Escalate an issue or request to your supervisor. Only available to worker agents. \
         Provide a clear reason for escalation and any relevant context."
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for escalation")
            .with_properties(serde_json::json!({
                "reason": property(
                    "string",
                    "Clear explanation of why you're escalating"
                ),
                "context": property(
                    "object",
                    "Additional details or context"
                ),
                "task_id": property(
                    "string",
                    "Optional: ID of the task you're escalating about"
                ),
            }))
            .with_required(vec!["reason".to_string()])
    }

    fn risk_level(&self) -> ToolRiskLevel {
        ToolRiskLevel::Low  // Just sends a message to supervisor
    }

    async fn execute_with_context(
        &self,
        params: Value,
        context: Option<&ToolContext>,
    ) -> Result<ToolResult> {
        // Parse parameters
        let params: EscalateParams = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(e.to_string()))?;

        // Require context with coordinator
        let context = context.ok_or_else(|| {
            ToolError::execution("EscalateTool requires execution context with coordinator")
        })?;

        #[cfg(feature = "coordination")]
        {
            let coordinator = context.coordinator.as_ref().ok_or_else(|| {
                ToolError::execution("Coordinator not available in context")
            })?;

            // Create escalation request
            let mut escalation = agent_comms::EscalationRequest::new(
                &context.agent_id,
                "", // Supervisor will be determined from hierarchy
                &params.reason,
            );
            
            if let Some(task_id) = params.task_id {
                escalation = escalation.with_task(task_id);
            }
            
            escalation = escalation.with_context(params.context.clone());

            // Get supervisor ID first
            let supervisor = coordinator.hierarchy().get_supervisor(&context.agent_id)
                .ok_or_else(|| ToolError::execution(format!(
                    "Agent {} is not a worker or has no supervisor",
                    context.agent_id
                )))?;

            // Update escalation with supervisor
            escalation.to_supervisor = supervisor.clone();

            // Perform escalation
            let escalation_id = coordinator
                .escalate(&context.agent_id, escalation)
                .await
                .map_err(|e| ToolError::execution(format!("Escalation failed: {}", e)))?;

            tracing::info!(
                "Agent {} escalated to supervisor {}: {}",
                context.agent_id,
                supervisor,
                params.reason
            );

            Ok(ToolResult::success(serde_json::json!({
                "escalation_id": escalation_id,
                "supervisor": supervisor,
                "reason": params.reason,
                "status": "escalated",
                "message": format!("Successfully escalated to {}", supervisor),
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
    async fn test_escalate_success() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));

        // Setup coordinator
        let coord_role = AgentRole::Coordinator {
            subordinates: vec!["worker-1".to_string()],
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("coord-1"), coord_role);
        transport.register_mailbox("coord-1");

        // Setup worker
        let worker_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-1"), worker_role);

        let tool = EscalateTool;
        let context = ToolContext::with_coordinator("worker-1", coordinator);

        let params = serde_json::json!({
            "reason": "Need additional resources to complete task",
            "context": {"resource_type": "database_access"},
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(result.success);
        
        let data = result.data.unwrap();
        assert_eq!(data["supervisor"], "coord-1");
        assert_eq!(data["status"], "escalated");
    }

    #[tokio::test]
    async fn test_escalate_no_supervisor() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport));

        // Register as standalone (no supervisor)
        let standalone_role = AgentRole::Standalone;
        coordinator.register_agent(AgentAddress::local("agent-1"), standalone_role);

        let tool = EscalateTool;
        let context = ToolContext::with_coordinator("agent-1", coordinator);

        let params = serde_json::json!({
            "reason": "This should fail - no supervisor",
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
