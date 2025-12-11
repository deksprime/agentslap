//! Agent coordinator for managing multiple agents

use dashmap::DashMap;
use std::sync::Arc;
use serde_json::json;

use crate::{
    AgentAddress, AgentRegistry, MessageTransport, Result, AgentMessage,
    CommsError,
    hierarchy::{AgentHierarchy, AgentRole},
    delegation::{DelegationRequest, DelegationResult, EscalationRequest},
};

/// Agent handle tracking running state
pub struct AgentHandle {
    pub agent_id: String,
    pub role: AgentRole,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Coordinator for managing multi-agent collaboration
///
/// Provides:
/// - Agent lifecycle management (spawn, stop)
/// - Hierarchical relationships (coordinators, workers, peers)
/// - Message routing based on hierarchy
/// - Delegation patterns (task assignment, escalation, broadcast)
/// - Team-based communication
#[derive(Clone)]
pub struct AgentCoordinator {
    /// Agent registry for discovery
    registry: Arc<AgentRegistry>,
    
    /// Hierarchy tracking
    hierarchy: Arc<AgentHierarchy>,
    
    /// Message transport
    transport: Arc<dyn MessageTransport>,
    
    /// Running agents (agent_id -> handle)
    running_agents: Arc<DashMap<String, AgentHandle>>,
    
    /// Pending delegations (task_id -> delegation request)
    pending_delegations: Arc<DashMap<String, DelegationRequest>>,
}

// Implement CoordinatorHandle to allow passing to agent-runtime without circular dep
impl agent_context::CoordinatorHandle for AgentCoordinator {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AgentCoordinator {
    /// Create a new coordinator
    pub fn new(transport: Arc<dyn MessageTransport>) -> Self {
        Self {
            registry: Arc::new(AgentRegistry::new()),
            hierarchy: Arc::new(AgentHierarchy::new()),
            transport,
            running_agents: Arc::new(DashMap::new()),
            pending_delegations: Arc::new(DashMap::new()),
        }
    }

    /// Get the agent registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
    
    /// Get the hierarchy tracker
    pub fn hierarchy(&self) -> &AgentHierarchy {
        &self.hierarchy
    }
    
    /// Get the message transport
    pub fn transport(&self) -> &Arc<dyn MessageTransport> {
        &self.transport
    }

    /// Register a new agent with role
    pub fn register_agent(&self, address: AgentAddress, role: AgentRole) {
        // Register in registry with role name
        let role_name = match &role {
            AgentRole::Coordinator { .. } => Some("coordinator".to_string()),
            AgentRole::Worker { .. } => Some("worker".to_string()),
            AgentRole::Peer { .. } => Some("peer".to_string()),
            AgentRole::Standalone => Some("standalone".to_string()),
        };
        
        self.registry.register(address.clone(), role_name);
        
        // Register in hierarchy
        self.hierarchy.register(address.id.clone(), role.clone());
        
        // Track as running
        let agent_id = address.id.clone();
        let handle = AgentHandle {
            agent_id: address.id,
            role,
            started_at: chrono::Utc::now(),
        };
        self.running_agents.insert(handle.agent_id.clone(), handle);
        
        tracing::info!("Agent registered and running: {}", agent_id);
    }
    
    /// Unregister and stop an agent
    pub fn unregister_agent(&self, agent_id: &str) -> bool {
        if self.running_agents.remove(agent_id).is_some() {
            tracing::info!("Agent stopped: {}", agent_id);
            true
        } else {
            false
        }
    }

    /// Get count of running agents
    pub fn agent_count(&self) -> usize {
        self.running_agents.len()
    }
    
    /// List all running agent IDs
    pub fn list_agents(&self) -> Vec<String> {
        self.running_agents.iter().map(|e| e.key().clone()).collect()
    }
    
    /// Get agent handle
    pub fn get_agent(&self, agent_id: &str) -> Option<AgentHandle> {
        self.running_agents.get(agent_id).map(|h| AgentHandle {
            agent_id: h.agent_id.clone(),
            role: h.role.clone(),
            started_at: h.started_at,
        })
    }

    // ========== DELEGATION PATTERNS ==========

    /// Delegate task from coordinator to worker
    ///
    /// # Arguments
    /// * `coordinator_id` - ID of the delegating coordinator
    /// * `worker_id` - ID of the target worker (or None to auto-select)
    /// * `description` - Task description
    ///
    /// # Returns
    /// Task ID for tracking
    pub async fn delegate_task(
        &self,
        coordinator_id: &str,
        worker_id: Option<&str>,
        description: String,
    ) -> Result<String> {
        // Verify coordinator role
        let role = self.hierarchy.get_role(coordinator_id)
            .ok_or_else(|| CommsError::AgentNotFound(coordinator_id.to_string()))?;
        
        if !role.is_coordinator() {
            return Err(CommsError::Permission("Only coordinators can delegate tasks".to_string()));
        }

        // Select worker
        let target_worker = if let Some(wid) = worker_id {
            // Verify it's a subordinate
            if !self.hierarchy.is_subordinate(wid, coordinator_id) {
                return Err(CommsError::Permission(
                    format!("{} is not a subordinate of {}", wid, coordinator_id)
                ));
            }
            wid.to_string()
        } else {
            // Auto-select first available subordinate
            let subordinates = self.hierarchy.get_subordinates(coordinator_id);
            subordinates.first()
                .ok_or_else(|| CommsError::other("No subordinates available"))?
                .clone()
        };

        // Create delegation request
        let delegation = DelegationRequest::new(coordinator_id, &target_worker, description);
        let task_id = delegation.task_id.clone();
        
        // Track pending delegation
        self.pending_delegations.insert(task_id.clone(), delegation.clone());

        // Send delegation message
        let message = AgentMessage::new(
            coordinator_id,
            &target_worker,
            json!({
                "type": "task_delegation",
                "delegation": delegation,
            }),
        );

        let addr = AgentAddress::local(&target_worker);
        self.transport.send(&addr, message).await?;

        tracing::info!("Task {} delegated from {} to {}", task_id, coordinator_id, target_worker);

        Ok(task_id)
    }

    /// Report task completion from worker to coordinator
    pub async fn report_task_result(
        &self,
        task_id: &str,
        worker_id: &str,
        result: DelegationResult,
    ) -> Result<()> {
        // Get delegation request
        let delegation = self.pending_delegations.remove(task_id)
            .ok_or_else(|| CommsError::other(format!("Task {} not found", task_id)))?
            .1;

        // Send result to coordinator
        let message = AgentMessage::new(
            worker_id,
            &delegation.from_coordinator,
            json!({
                "type": "task_result",
                "result": result,
            }),
        );

        let addr = AgentAddress::local(&delegation.from_coordinator);
        self.transport.send(&addr, message).await?;

        tracing::info!("Task {} completed by {}", task_id, worker_id);

        Ok(())
    }

    /// Escalate issue from worker to supervisor
    pub async fn escalate(
        &self,
        worker_id: &str,
        escalation: EscalationRequest,
    ) -> Result<String> {
        // Verify worker has supervisor
        let supervisor = self.hierarchy.get_supervisor(worker_id)
            .ok_or_else(|| CommsError::other(format!("Worker {} has no supervisor", worker_id)))?;

        // Send escalation
        let message = AgentMessage::new(
            worker_id,
            &supervisor,
            json!({
                "type": "escalation",
                "escalation": escalation,
            }),
        );

        let addr = AgentAddress::local(&supervisor);
        self.transport.send(&addr, message).await?;

        tracing::info!("Escalation from {} to {}", worker_id, supervisor);

        Ok(escalation.id)
    }

    /// Broadcast message to entire team
    pub async fn broadcast_to_team(
        &self,
        from_agent: &str,
        team: &str,
        content: serde_json::Value,
    ) -> Result<usize> {
        let members = self.hierarchy.get_team_members(team);
        
        if members.is_empty() {
            return Err(CommsError::other(format!("Team {} not found or empty", team)));
        }

        // Create broadcast message
        let message = AgentMessage::broadcast(from_agent, team, content);
        
        // Broadcast via transport
        self.transport.broadcast(team, message).await?;

        tracing::info!("Broadcast from {} to team {} ({} members)", from_agent, team, members.len());

        Ok(members.len())
    }

    /// Send request and wait for response
    pub async fn request_response(
        &self,
        from_agent: &str,
        to_agent: &str,
        content: serde_json::Value,
    ) -> Result<AgentMessage> {
        // Verify both agents exist
        self.registry.lookup(from_agent)
            .ok_or_else(|| CommsError::AgentNotFound(from_agent.to_string()))?;
        
        let to_addr = self.registry.lookup(to_agent)
            .ok_or_else(|| CommsError::AgentNotFound(to_agent.to_string()))?;

        // Create request message
        let request = AgentMessage::request(from_agent, to_agent, content);

        // Send via transport's request method (handles response waiting)
        let response = self.transport.request(&to_addr, request).await?;

        Ok(response)
    }

    // ========== TEAM MANAGEMENT ==========

    /// Create a new team with a coordinator
    pub fn create_team(
        &self,
        coordinator_id: String,
        team_name: String,
    ) -> Result<()> {
        // Check if coordinator exists
        if !self.running_agents.contains_key(&coordinator_id) {
            return Err(CommsError::AgentNotFound(coordinator_id));
        }

        // Register as coordinator
        let role = AgentRole::Coordinator {
            subordinates: vec![],
            team: team_name.clone(),
        };
        
        self.hierarchy.register(coordinator_id.clone(), role);

        tracing::info!("Team {} created with coordinator {}", team_name, coordinator_id);

        Ok(())
    }

    /// Add worker to team under coordinator
    pub fn add_worker_to_team(
        &self,
        worker_id: String,
        coordinator_id: &str,
        team_name: String,
    ) -> Result<()> {
        // Verify coordinator exists and is a coordinator
        let coord_role = self.hierarchy.get_role(coordinator_id)
            .ok_or_else(|| CommsError::AgentNotFound(coordinator_id.to_string()))?;

        if !coord_role.is_coordinator() {
            return Err(CommsError::Permission(
                format!("{} is not a coordinator", coordinator_id)
            ));
        }

        // Register worker with supervisor
        let worker_role = AgentRole::Worker {
            supervisor: coordinator_id.to_string(),
            team: team_name.clone(),
        };
        
        self.hierarchy.register(worker_id.clone(), worker_role);

        // Add to coordinator's subordinates
        self.hierarchy.add_subordinate(coordinator_id, worker_id.clone());

        tracing::info!("Worker {} added to team {} under {}", worker_id, team_name, coordinator_id);

        Ok(())
    }

    /// Get all coordinators
    pub fn get_coordinators(&self) -> Vec<String> {
        self.hierarchy.get_coordinators()
    }

    /// Get all workers
    pub fn get_workers(&self) -> Vec<String> {
        self.hierarchy.get_workers()
    }

    /// Get subordinates of a coordinator
    pub fn get_subordinates(&self, coordinator_id: &str) -> Vec<String> {
        self.hierarchy.get_subordinates(coordinator_id)
    }

    /// Get team members
    pub fn get_team_members(&self, team: &str) -> Vec<String> {
        self.hierarchy.get_team_members(team)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InProcessTransport;

    #[test]
    fn test_coordinator_creation() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = AgentCoordinator::new(transport);
        
        assert_eq!(coordinator.agent_count(), 0);
    }

    #[test]
    fn test_agent_registration() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = AgentCoordinator::new(transport);

        let role = AgentRole::Coordinator {
            subordinates: vec![],
            team: "team-alpha".to_string(),
        };

        coordinator.register_agent(AgentAddress::local("coord-1"), role);

        assert_eq!(coordinator.agent_count(), 1);
        assert!(coordinator.get_agent("coord-1").is_some());
    }

    #[test]
    fn test_create_team() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = AgentCoordinator::new(transport);

        // Register coordinator first
        let role = AgentRole::Standalone;
        coordinator.register_agent(AgentAddress::local("coord-1"), role);

        // Create team
        coordinator.create_team("coord-1".to_string(), "team-alpha".to_string()).unwrap();

        // Verify role changed
        let role = coordinator.hierarchy().get_role("coord-1").unwrap();
        assert!(role.is_coordinator());
    }

    #[test]
    fn test_add_worker_to_team() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = AgentCoordinator::new(transport);

        // Setup coordinator
        let role = AgentRole::Coordinator {
            subordinates: vec![],
            team: "team-alpha".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("coord-1"), role);

        // Add worker
        coordinator.add_worker_to_team(
            "worker-1".to_string(),
            "coord-1",
            "team-alpha".to_string(),
        ).unwrap();

        // Verify hierarchy
        assert!(coordinator.hierarchy().is_subordinate("worker-1", "coord-1"));
        assert_eq!(coordinator.get_subordinates("coord-1").len(), 1);
    }

    #[tokio::test]
    async fn test_delegation() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = AgentCoordinator::new(transport.clone());

        // Setup coordinator
        let coord_role = AgentRole::Coordinator {
            subordinates: vec!["worker-1".to_string()],
            team: "team-alpha".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("coord-1"), coord_role);

        // Setup worker with mailbox
        let worker_role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-alpha".to_string(),
        };
        coordinator.register_agent(AgentAddress::local("worker-1"), worker_role);
        transport.register_mailbox("worker-1");

        // Delegate task
        let task_id = coordinator.delegate_task(
            "coord-1",
            Some("worker-1"),
            "Process data".to_string(),
        ).await.unwrap();

        assert!(!task_id.is_empty());
    }
}

