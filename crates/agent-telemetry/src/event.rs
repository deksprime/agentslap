//! Telemetry event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Telemetry events emitted by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TelemetryEvent {
    /// Agent lifecycle events
    AgentStarted {
        agent_id: String,
        role: String,
        timestamp: DateTime<Utc>,
    },
    
    AgentStopped {
        agent_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// LLM interaction events
    LlmRequestSent {
        agent_id: String,
        model: String,
        message_count: usize,
        timestamp: DateTime<Utc>,
    },
    
    LlmResponseReceived {
        agent_id: String,
        model: String,
        tokens_used: usize,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },

    /// Tool execution events
    ToolCallStarted {
        agent_id: String,
        tool_name: String,
        parameters: Value,
        timestamp: DateTime<Utc>,
    },
    
    ToolCallCompleted {
        agent_id: String,
        tool_name: String,
        success: bool,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },

    /// Inter-agent communication
    MessageSent {
        from: String,
        to: String,
        message_type: String,
        timestamp: DateTime<Utc>,
    },
    
    MessageReceived {
        agent_id: String,
        from: String,
        timestamp: DateTime<Utc>,
    },

    /// Guardrail events
    GuardrailTriggered {
        agent_id: String,
        guardrail: String,
        action: String,
        severity: String,
        timestamp: DateTime<Utc>,
    },

    /// HITL events
    ApprovalRequested {
        agent_id: String,
        action: String,
        risk_level: String,
        timestamp: DateTime<Utc>,
    },
    
    ApprovalResponded {
        agent_id: String,
        decision: String,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },

    /// Session events
    SessionCreated {
        agent_id: String,
        session_id: String,
        timestamp: DateTime<Utc>,
    },
    
    SessionSaved {
        agent_id: String,
        session_id: String,
        message_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Error events
    ErrorOccurred {
        agent_id: String,
        error: String,
        context: Option<Value>,
        timestamp: DateTime<Utc>,
    },

    /// Custom events
    Custom {
        agent_id: String,
        event_name: String,
        data: Value,
        timestamp: DateTime<Utc>,
    },
}

impl TelemetryEvent {
    /// Get the agent ID associated with this event
    pub fn agent_id(&self) -> &str {
        match self {
            Self::AgentStarted { agent_id, .. } => agent_id,
            Self::AgentStopped { agent_id, .. } => agent_id,
            Self::LlmRequestSent { agent_id, .. } => agent_id,
            Self::LlmResponseReceived { agent_id, .. } => agent_id,
            Self::ToolCallStarted { agent_id, .. } => agent_id,
            Self::ToolCallCompleted { agent_id, .. } => agent_id,
            Self::MessageSent { from, .. } => from,
            Self::MessageReceived { agent_id, .. } => agent_id,
            Self::GuardrailTriggered { agent_id, .. } => agent_id,
            Self::ApprovalRequested { agent_id, .. } => agent_id,
            Self::ApprovalResponded { agent_id, .. } => agent_id,
            Self::SessionCreated { agent_id, .. } => agent_id,
            Self::SessionSaved { agent_id, .. } => agent_id,
            Self::ErrorOccurred { agent_id, .. } => agent_id,
            Self::Custom { agent_id, .. } => agent_id,
        }
    }

    /// Get the timestamp of this event
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Self::AgentStarted { timestamp, .. } => timestamp,
            Self::AgentStopped { timestamp, .. } => timestamp,
            Self::LlmRequestSent { timestamp, .. } => timestamp,
            Self::LlmResponseReceived { timestamp, .. } => timestamp,
            Self::ToolCallStarted { timestamp, .. } => timestamp,
            Self::ToolCallCompleted { timestamp, .. } => timestamp,
            Self::MessageSent { timestamp, .. } => timestamp,
            Self::MessageReceived { timestamp, .. } => timestamp,
            Self::GuardrailTriggered { timestamp, .. } => timestamp,
            Self::ApprovalRequested { timestamp, .. } => timestamp,
            Self::ApprovalResponded { timestamp, .. } => timestamp,
            Self::SessionCreated { timestamp, .. } => timestamp,
            Self::SessionSaved { timestamp, .. } => timestamp,
            Self::ErrorOccurred { timestamp, .. } => timestamp,
            Self::Custom { timestamp, .. } => timestamp,
        }
    }

    // Convenience constructors
    pub fn agent_started(agent_id: impl Into<String>, role: impl Into<String>) -> Self {
        Self::AgentStarted {
            agent_id: agent_id.into(),
            role: role.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn llm_request(agent_id: impl Into<String>, model: impl Into<String>, message_count: usize) -> Self {
        Self::LlmRequestSent {
            agent_id: agent_id.into(),
            model: model.into(),
            message_count,
            timestamp: Utc::now(),
        }
    }

    pub fn tool_started(agent_id: impl Into<String>, tool_name: impl Into<String>, params: Value) -> Self {
        Self::ToolCallStarted {
            agent_id: agent_id.into(),
            tool_name: tool_name.into(),
            parameters: params,
            timestamp: Utc::now(),
        }
    }

    pub fn error(agent_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self::ErrorOccurred {
            agent_id: agent_id.into(),
            error: error.into(),
            context: None,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = TelemetryEvent::agent_started("agent-1", "coordinator");
        assert_eq!(event.agent_id(), "agent-1");
    }

    #[test]
    fn test_event_serialization() {
        let event = TelemetryEvent::llm_request("agent-1", "gpt-4", 3);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TelemetryEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.agent_id(), "agent-1");
    }

    #[test]
    fn test_all_event_types_have_agent_id() {
        let events = vec![
            TelemetryEvent::agent_started("a", "r"),
            TelemetryEvent::llm_request("a", "m", 1),
            TelemetryEvent::tool_started("a", "t", serde_json::json!({})),
            TelemetryEvent::error("a", "e"),
        ];

        for event in events {
            assert!(!event.agent_id().is_empty());
        }
    }
}

