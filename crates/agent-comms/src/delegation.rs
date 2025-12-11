//! Coordination patterns for multi-agent collaboration

use serde::{Deserialize, Serialize};

/// Coordination pattern for multi-agent interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoordinationPattern {
    /// Broadcast message to all team members (1 → many, no response)
    Broadcast,
    
    /// Request-response pattern (1 → 1, wait for response)
    RequestResponse,
    
    /// Task delegation from coordinator to worker (1 → 1, fire & forget)
    TaskDelegation,
    
    /// Peer collaboration (n ↔ n, equal status)
    PeerCollaboration,
    
    /// Escalation from worker to supervisor (1 → 1, request help)
    Escalation,
}

/// Task delegation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRequest {
    /// Task ID for tracking
    pub task_id: String,
    
    /// Task description
    pub description: String,
    
    /// Coordinator agent ID
    pub from_coordinator: String,
    
    /// Target worker agent ID
    pub to_worker: String,
    
    /// Optional priority (1-10, 10 = highest)
    #[serde(default)]
    pub priority: u8,
    
    /// Optional deadline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Additional context/parameters
    #[serde(default)]
    pub context: serde_json::Value,
}

impl DelegationRequest {
    pub fn new(
        from_coordinator: impl Into<String>,
        to_worker: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            task_id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            from_coordinator: from_coordinator.into(),
            to_worker: to_worker.into(),
            priority: 5,
            deadline: None,
            context: serde_json::Value::Null,
        }
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }
    
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }
}

/// Task result from worker to coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResult {
    /// Original task ID
    pub task_id: String,
    
    /// Success status
    pub success: bool,
    
    /// Result data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Worker agent ID
    pub from_worker: String,
}

impl DelegationResult {
    pub fn success(task_id: impl Into<String>, from_worker: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            task_id: task_id.into(),
            success: true,
            result: Some(result),
            error: None,
            from_worker: from_worker.into(),
        }
    }
    
    pub fn failure(task_id: impl Into<String>, from_worker: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            success: false,
            result: None,
            error: Some(error.into()),
            from_worker: from_worker.into(),
        }
    }
}

/// Escalation request from worker to supervisor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRequest {
    /// Escalation ID
    pub id: String,
    
    /// Worker agent ID
    pub from_worker: String,
    
    /// Supervisor agent ID
    pub to_supervisor: String,
    
    /// Reason for escalation
    pub reason: String,
    
    /// Context/details
    pub context: serde_json::Value,
    
    /// Original task ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

impl EscalationRequest {
    pub fn new(
        from_worker: impl Into<String>,
        to_supervisor: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_worker: from_worker.into(),
            to_supervisor: to_supervisor.into(),
            reason: reason.into(),
            context: serde_json::Value::Null,
            task_id: None,
        }
    }
    
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }
    
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_request_creation() {
        let req = DelegationRequest::new("coord-1", "worker-1", "Process user data")
            .with_priority(8);
        
        assert_eq!(req.from_coordinator, "coord-1");
        assert_eq!(req.to_worker, "worker-1");
        assert_eq!(req.priority, 8);
    }

    #[test]
    fn test_delegation_result_success() {
        let result = DelegationResult::success(
            "task-123",
            "worker-1",
            serde_json::json!({"count": 42}),
        );
        
        assert!(result.success);
        assert!(result.result.is_some());
        assert_eq!(result.result.unwrap()["count"], 42);
    }

    #[test]
    fn test_delegation_result_failure() {
        let result = DelegationResult::failure("task-123", "worker-1", "Out of memory");
        
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "Out of memory");
    }

    #[test]
    fn test_escalation_request() {
        let esc = EscalationRequest::new("worker-1", "coord-1", "Need more resources")
            .with_task("task-456");
        
        assert_eq!(esc.from_worker, "worker-1");
        assert_eq!(esc.task_id.unwrap(), "task-456");
    }
}
