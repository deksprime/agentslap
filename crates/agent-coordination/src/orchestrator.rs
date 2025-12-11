//! Agent orchestrator - manages active agent message loops
//!
//! THIS IS POINT 3: Making agents actually run with background message loops

use agent_runtime::Agent;
use agent_comms::{AgentCoordinator, AgentMessage};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Handle to a running agent
pub struct AgentRunHandle {
    pub agent_id: String,
    pub task_handle: JoinHandle<()>,
}

impl AgentRunHandle {
    /// Stop the agent
    pub fn stop(self) {
        self.task_handle.abort();
        tracing::info!("Stopped agent: {}", self.agent_id);
    }
}

/// Spawn an agent with an active message loop
///
/// The agent will:
/// 1. Register its mailbox with the transport
/// 2. Listen for incoming messages
/// 3. Process each message with the agent's LLM
/// 4. Send responses back
/// 5. Track tasks and maintain conversation state
///
/// This is the ACTUAL implementation that makes agents run!
pub fn spawn_agent_with_loop(
    agent: Agent,
    agent_id: String,
    coordinator: Arc<AgentCoordinator>,
) -> AgentRunHandle {
    let agent_id_clone = agent_id.clone();
    
    // Spawn the agent's message processing loop
    let task_handle = tokio::spawn(async move {
        // Register mailbox for this agent
        let transport = coordinator.transport();
        
        // Get mailbox receiver if using InProcessTransport
        // For in-process transport, we need to register and get receiver
        let mut receiver = if let Some(in_process) = transport.as_any().downcast_ref::<agent_comms::InProcessTransport>() {
            in_process.register_mailbox(&agent_id_clone)
        } else {
            tracing::warn!("Transport type doesn't support direct mailbox registration. Agent {} will not receive messages.", agent_id_clone);
            return;
        };
        
        tracing::info!("Agent {} message loop started", agent_id_clone);
        
        // Main message processing loop
        loop {
            match receiver.recv().await {
                Some(message) => {
                    tracing::debug!("Agent {} received message from {}", agent_id_clone, message.from);
                    
                    // Process message based on type
                    match process_message(&agent, &message, &coordinator).await {
                        Ok(_) => {
                            tracing::debug!("Agent {} processed message successfully", agent_id_clone);
                        }
                        Err(e) => {
                            tracing::error!("Agent {} error processing message: {}", agent_id_clone, e);
                            
                            // Send error response for requests
                            if message.msg_type == agent_comms::MessageType::Request {
                                if let Err(send_err) = send_error_response(&message, &coordinator, e.to_string()).await {
                                    tracing::error!("Failed to send error response: {}", send_err);
                                }
                            }
                        }
                    }
                }
                None => {
                    tracing::info!("Agent {} mailbox closed, stopping", agent_id_clone);
                    break;
                }
            }
        }
        
        // Unregister agent when loop ends
        coordinator.unregister_agent(&agent_id_clone);
        tracing::info!("Agent {} stopped", agent_id_clone);
    });
    
    AgentRunHandle {
        agent_id,
        task_handle,
    }
}

/// Process a received message with the agent
async fn process_message(
    agent: &Agent,
    message: &AgentMessage,
    coordinator: &Arc<AgentCoordinator>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Extract message content as text
    let user_message = message.content.to_string();
    
    // Check message type
    match message.msg_type {
        agent_comms::MessageType::Request => {
            // Process with agent and send response
            let response_text = agent.run(&user_message).await
                .map_err(|e| format!("Agent execution failed: {}", e))?;
            
            // Determine sender
            let from_id = message.to.as_ref().unwrap_or(&message.from);
            let to_id = &message.from;
            let req_id = &message.id;
            
            // Create response message
            let response = AgentMessage::response(
                from_id.as_str(),
                to_id.as_str(),
                serde_json::json!({"response": response_text}),
                req_id.as_str(),
            );
            
            // Send response via coordinator's transport
            let to_addr = agent_comms::AgentAddress::local(to_id);
            coordinator.transport().send(&to_addr, response).await?;
            
            Ok(())
        }
        
        agent_comms::MessageType::Send => {
            // One-way message - process but don't respond
            let _response = agent.run(&user_message).await
                .map_err(|e| format!("Agent execution failed: {}", e))?;
            
            tracing::debug!("Processed one-way message, no response sent");
            Ok(())
        }
        
        agent_comms::MessageType::Response => {
            // This is a response to our previous request
            // For now, just log it
            // In a full implementation, this would resolve a pending future
            tracing::debug!("Received response to previous request");
            
            // If there's a pending request, resolve it
            if let Some(request_id) = &message.request_id {
                // Try to complete the pending request
                if let Some(pending_tx) = coordinator.transport().as_any()
                    .downcast_ref::<agent_comms::InProcessTransport>()
                    .and_then(|t| t.pending_requests.remove(request_id))
                {
                    let _ = pending_tx.1.send(message.clone());
                }
            }
            
            Ok(())
        }
        
        agent_comms::MessageType::Broadcast => {
            // Broadcast message - process but don't respond unless specific action needed
            let _response = agent.run(&user_message).await
                .map_err(|e| format!("Agent execution failed: {}", e))?;
            
            tracing::debug!("Processed broadcast message");
            Ok(())
        }
    }
}

/// Send error response for a failed message
async fn send_error_response(
    original_message: &AgentMessage,
    coordinator: &Arc<AgentCoordinator>,
    error: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let unknown_str = "unknown".to_string();
    let from_id = original_message.to.as_ref().unwrap_or(&unknown_str);
    let to_id = &original_message.from;
    let req_id = &original_message.id;
    
    let response = AgentMessage::response(
        from_id.as_str(),
        to_id.as_str(),
        serde_json::json!({
            "error": error,
            "success": false,
        }),
        req_id.as_str(),
    );
    
    let to_addr = agent_comms::AgentAddress::local(to_id);
    coordinator.transport().send(&to_addr, response).await?;
    
    Ok(())
}
