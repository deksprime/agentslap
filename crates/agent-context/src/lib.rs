//! Agent Context - Shared types for multi-agent coordination
//!
//! This crate provides foundational types used across agent-runtime, agent-comms,
//! and agent-tools to enable proper dependency management and avoid circular dependencies.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Agent identifier and network address
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        AgentId(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        AgentId(s.to_string())
    }
}

/// Trait for accessing agent coordinator (type-erased to avoid circular deps)
/// 
/// This allows agent-runtime to hold a reference to the coordinator
/// without depending on agent-comms directly.
pub trait CoordinatorHandle: Send + Sync {
    /// Get the coordinator as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

// Implement for Arc<T> where T implements CoordinatorHandle
impl<T: CoordinatorHandle + ?Sized> CoordinatorHandle for Arc<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        (**self).as_any()
    }
}
