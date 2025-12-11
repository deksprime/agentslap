//! Delegate task tool - allows coordinators to delegate tasks to workers

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolRiskLevel, ToolSchema, ToolContext, ToolError};

/// Tool for delegating tasks from coordinator to worker agents
///
/// Only available to coordinator agents. Allows natural delegation
/// through LLM conversation rather than programmatic API calls.
pub struct DelegateTaskTool;

#[derive(Debug, Deserialize, Serialize)]
struct DelegateTaskParams {
    /// The task description to delegate
    task_description: String,
    
    /// Optional specific worker ID (auto-selected if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    worker_id: Option<String>,
}

#[async_trait]
impl Tool for DelegateTaskTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "Delegate a task to a subordinate worker agent. Only available to coordinator agents. \
         Specify the task description and optionally a specific worker ID. If no worker is specified, \
         the system will auto-select an available subordinate."
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for task delegation")
            .with_properties(serde_json::json!({
                "task_description": property(
                    "string",
                    "Clear description of what the worker should do"
                ),
                "worker_id": property(
                    "string",
                    "Optional: ID of specific worker to delegate to (auto-selected if omitted)"
                ),
            }))
            .with_required(vec!["task_description".to_string()])
    }

    fn risk_level(&self) -> ToolRiskLevel {
        ToolRiskLevel::Medium  // Creates work for other agents
    }

    async fn execute_with_context(
        &self,
        params: Value,
        context: Option<&ToolContext>,
    ) -> Result<ToolResult> {
        // Parse parameters
        let params: DelegateTaskParams = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(e.to_string()))?;

        // Require context with coordinator
        let context = context.ok_or_else(|| {
            ToolError::execution("DelegateTaskTool requires execution context with coordinator")
        })?;

        #[cfg(feature = "coordination")]
        {
            let coordinator = context.coordinator.as_ref().ok_or_else(|| {
                ToolError::execution("Coordinator not available in context")
            })?;

            // Delegate task
            let task_id = coordinator
                .delegate_task(
                    &context.agent_id,
                    params.worker_id.as_deref(),
                    params.task_description.clone(),
                )
                .await
                .map_err(|e| ToolError::execution(format!("Delegation failed: {}", e)))?;

            tracing::info!(
                "Agent {} delegated task {} to worker",
                context.agent_id,
                task_id
            );

            Ok(ToolResult::success(serde_json::json!({
                "task_id": task_id,
                "task_description": params.task_description,
                "status": "delegated",
                "message": format!("Successfully delegated task {}", task_id),
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
    async fn test_delegate_task_success() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));

        // Setup coordinator
        let coord_role = AgentRole::Coordinator {
            subordinates: vec!["worker-1".to_string()],
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("coord-1"), coord_role);

        // Setup worker with mailbox
        let worker_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-1"), worker_role);
        transport.register_mailbox("worker-1");

        // Create tool and context
        let tool = DelegateTaskTool;
        let context = ToolContext::with_coordinator("coord-1", coordinator);

        // Execute delegation
        let params = serde_json::json!({
            "task_description": "Analyze sales data for Q4",
            "worker_id": "worker-1",
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(result.success);
        assert!(result.data.is_some());
        
        let data = result.data.unwrap();
        assert!(data["task_id"].is_string());
        assert_eq!(data["status"], "delegated");
    }

    #[tokio::test]
    async fn test_delegate_task_auto_select_worker() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));

        // Setup coordinator
        let coord_role = AgentRole::Coordinator {
            subordinates: vec!["worker-1".to_string()],
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("coord-1"), coord_role);

        // Setup worker
        let worker_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-1"), worker_role);
        transport.register_mailbox("worker-1");

        let tool = DelegateTaskTool;
        let context = ToolContext::with_coordinator("coord-1", coordinator);

        // Execute without worker_id (auto-select)
        let params = serde_json::json!({
            "task_description": "Process user feedback",
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_delegate_task_not_coordinator() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport));

        // Register as worker (NOT coordinator)
        let worker_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-test".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-1"), worker_role);

        let tool = DelegateTaskTool;
        let context = ToolContext::with_coordinator("worker-1", coordinator);

        let params = serde_json::json!({
            "task_description": "This should fail",
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
