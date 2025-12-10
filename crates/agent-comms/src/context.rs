//! Agent context (organizational awareness)

use std::sync::Arc;

use crate::{AgentAddress, MessageTransport};

/// Agent's organizational context
///
/// Provides awareness of:
/// - Who am I
/// - Who is my supervisor
/// - Who are my subordinates
/// - How do I communicate
pub struct AgentContext {
    /// My address
    pub my_address: AgentAddress,
    
    /// My supervisor (if any)
    pub supervisor: Option<AgentAddress>,
    
    /// My subordinates
    pub subordinates: Vec<AgentAddress>,
    
    /// My peers (same level)
    pub peers: Vec<AgentAddress>,
    
    /// Transport for communication
    pub transport: Arc<dyn MessageTransport>,
    
    /// My role (optional)
    pub role: Option<String>,
}

impl AgentContext {
    pub fn new(my_address: AgentAddress, transport: Arc<dyn MessageTransport>) -> Self {
        Self {
            my_address,
            supervisor: None,
            subordinates: Vec::new(),
            peers: Vec::new(),
            transport,
            role: None,
        }
    }

    pub fn with_supervisor(mut self, supervisor: AgentAddress) -> Self {
        self.supervisor = Some(supervisor);
        self
    }

    pub fn with_subordinates(mut self, subordinates: Vec<AgentAddress>) -> Self {
        self.subordinates = subordinates;
        self
    }

    pub fn with_role<S: Into<String>>(mut self, role: S) -> Self {
        self.role = Some(role.into());
        self
    }
}



