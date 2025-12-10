//! Agent registry for discovery

use dashmap::DashMap;
use std::sync::Arc;

use crate::AgentAddress;

/// Registry for agent discovery
#[derive(Clone)]
pub struct AgentRegistry {
    agents: Arc<DashMap<String, AgentAddress>>,
    roles: Arc<DashMap<String, Vec<String>>>, // role -> agent IDs
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            roles: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, address: AgentAddress, role: Option<String>) {
        self.agents.insert(address.id.clone(), address.clone());
        
        if let Some(role) = role {
            self.roles.entry(role).or_insert_with(Vec::new).push(address.id.clone());
        }
        
        tracing::debug!("Registered agent: {}", address.id);
    }

    pub fn lookup(&self, id: &str) -> Option<AgentAddress> {
        self.agents.get(id).map(|a| a.clone())
    }

    pub fn find_by_role(&self, role: &str) -> Vec<AgentAddress> {
        self.roles.get(role)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.lookup(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }

    pub fn list_all(&self) -> Vec<String> {
        self.agents.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}



