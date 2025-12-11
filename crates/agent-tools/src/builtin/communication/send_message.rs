//! Send message tool - direct agent-to-agent messaging

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolRiskLevel, ToolSchema, ToolContext, ToolError};

/// Tool for sending direct messages to other agents
///
/// Supports both one-way messages and request-response pattern.
pub struct SendMessageTool;

#[derive(Debug, Deserialize, Serialize)]
struct SendMessageParams {
    /// ID of the recipient agent
    to_agent_id: String,
    
    /// The message content
    message: String,
    
    /// Whether to wait for a response (request-response pattern)
    #[serde(default)]
    wait_for_response: bool,
}

#[async_trait]
impl Tool for SendMessageTool {
    fn name(&self) -> &str {
        "send_message"
    }

    fn description(&self) -> &str {
        "Send a direct message to another agent. Can send one-way messages or wait for a response. \
         Use wait_for_response=true for request-response pattern."
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for sending a message to another agent")
            .with_properties(serde_json::json!({
                "to_agent_id": property(
                    "string",
                    "ID of the agent to send the message to"
                ),
                "message": property(
                    "string",
                    "The message content"
                ),
                "wait_for_response": property(
                    "boolean",
                    "Whether to wait for a response (default: false)"
                ),
            }))
            .with_required(vec!["to_agent_id".to_string(), "message".to_string()])
    }

    fn risk_level(&self) -> ToolRiskLevel {
        ToolRiskLevel::Low  // Just sends a message
    }

    async fn execute_with_context(
        &self,
        params: Value,
        context: Option<&ToolContext>,
    ) -> Result<ToolResult> {
        // Parse parameters
        let params: SendMessageParams = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(e.to_string()))?;

        // Require context with coordinator
        let context = context.ok_or_else(|| {
            ToolError::execution("SendMessageTool requires execution context with coordinator")
        })?;

        #[cfg(feature = "coordination")]
        {
            let coordinator = context.coordinator.as_ref().ok_or_else(|| {
                ToolError::execution("Coordinator not available in context")
            })?;

            if params.wait_for_response {
                // Request-response pattern
                let response = coordinator
                    .request_response(
                        &context.agent_id,
                        &params.to_agent_id,
                        serde_json::json!({"message": params.message}),
                    )
                    .await
                    .map_err(|e| ToolError::execution(format!("Request failed: {}", e)))?;

                tracing::info!(
                    "Agent {} sent request to {} and received response",
                    context.agent_id,
                    params.to_agent_id
                );

                Ok(ToolResult::success(serde_json::json!({
                    "to": params.to_agent_id,
                    "sent_message": params.message,
                    "response": response.content,
                    "pattern": "request-response",
                })))
            } else {
                // One-way message
                let message = agent_comms::AgentMessage::new(
                    &context.agent_id,
                    &params.to_agent_id,
                    serde_json::json!({"message": params.message}),
                );

                let to_addr = agent_comms::AgentAddress::local(&params.to_agent_id);
                
                coordinator.transport()
                    .send(&to_addr, message)
                    .await
                    .map_err(|e| ToolError::execution(format!("Send failed: {}", e)))?;

                tracing::info!(
                    "Agent {} sent message to {}",
                    context.agent_id,
                    params.to_agent_id
                );

                Ok(ToolResult::success(serde_json::json!({
                    "to": params.to_agent_id,
                    "message": params.message,
                    "pattern": "one-way",
                    "status": "sent",
                })))
            }
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
    async fn test_send_message_one_way() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));

        // Register agents
        coordinator.register_agent(
            AgentAddress::local("agent-1"),
            AgentRole::Standalone,
        );
        coordinator.register_agent(
            AgentAddress::local("agent-2"),
            AgentRole::Standalone,
        );
        transport.register_mailbox("agent-2");

        let tool = SendMessageTool;
        let context = ToolContext::with_coordinator("agent-1", coordinator);

        let params = serde_json::json!({
            "to_agent_id": "agent-2",
            "message": "Hello from agent-1",
            "wait_for_response": false,
        });

        let result = tool.execute_with_context(params, Some(&context)).await.unwrap();
        assert!(result.success);
        
        let data = result.data.unwrap();
        assert_eq!(data["to"], "agent-2");
        assert_eq!(data["pattern"], "one-way");
    }
}
