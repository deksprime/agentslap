//! Agent coordinator for managing multiple agents

use dashmap::DashMap;
use std::sync::Arc;

use crate::{AgentAddress, AgentRegistry, MessageTransport, Result};

/// Coordinator for managing agent lifecycle and communication
pub struct AgentCoordinator {
    registry: Arc<AgentRegistry>,
    transport: Arc<dyn MessageTransport>,
    agents: Arc<DashMap<String, ()>>, // Would store actual agent instances
}

impl AgentCoordinator {
    pub fn new(transport: Arc<dyn MessageTransport>) -> Self {
        Self {
            registry: Arc::new(AgentRegistry::new()),
            transport,
            agents: Arc::new(DashMap::new()),
        }
    }

    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }

    pub fn register_agent(&self, address: AgentAddress, role: Option<String>) {
        self.registry.register(address.clone(), role);
        self.agents.insert(address.id, ());
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}



