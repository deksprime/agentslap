//! Agent hierarchy and relationship tracking

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Agent role within a hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentRole {
    /// Coordinator agent that manages subordinates
    Coordinator {
        /// IDs of subordinate agents
        subordinates: Vec<String>,
        /// Team or group identifier
        team: String,
    },
    
    /// Worker agent with a supervisor
    Worker {
        /// ID of supervisor agent
        supervisor: String,
        /// Team or group identifier
        team: String,
    },
    
    /// Peer agent (no hierarchy, equal collaboration)
    Peer {
        /// Team or group identifier
        team: String,
        /// Peer agent IDs
        peers: Vec<String>,
    },
    
    /// Standalone agent (no relationships)
    Standalone,
}

impl AgentRole {
    pub fn is_coordinator(&self) -> bool {
        matches!(self, AgentRole::Coordinator { .. })
    }
    
    pub fn is_worker(&self) -> bool {
        matches!(self, AgentRole::Worker { .. })
    }
    
    pub fn team(&self) -> Option<&str> {
        match self {
            AgentRole::Coordinator { team, .. } => Some(team),
            AgentRole::Worker { team, .. } => Some(team),
            AgentRole::Peer { team, .. } => Some(team),
            AgentRole::Standalone => None,
        }
    }
    
    pub fn subordinates(&self) -> &[String] {
        match self {
            AgentRole::Coordinator { subordinates, .. } => subordinates,
            _ => &[],
        }
    }
}

/// Tracks hierarchical relationships between agents
#[derive(Clone)]
pub struct AgentHierarchy {
    /// Agent ID -> Role mapping
    roles: Arc<DashMap<String, AgentRole>>,
    
    /// Team -> Agent IDs mapping
    teams: Arc<DashMap<String, Vec<String>>>,
}

impl AgentHierarchy {
    pub fn new() -> Self {
        Self {
            roles: Arc::new(DashMap::new()),
            teams: Arc::new(DashMap::new()),
        }
    }
    
    /// Register an agent with a role
    pub fn register(&self, agent_id: String, role: AgentRole) {
        // Add to team mapping
        if let Some(team) = role.team() {
            self.teams.entry(team.to_string())
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }
        
        self.roles.insert(agent_id.clone(), role.clone());
        tracing::debug!("Registered agent {} with role: {:?}", agent_id, role);
    }
    
    /// Get agent's role
    pub fn get_role(&self, agent_id: &str) -> Option<AgentRole> {
        self.roles.get(agent_id).map(|r| r.clone())
    }
    
    /// Check if agent is subordinate of another
    pub fn is_subordinate(&self, agent_id: &str, supervisor_id: &str) -> bool {
        if let Some(role) = self.get_role(agent_id) {
            match role {
                AgentRole::Worker { supervisor, .. } => supervisor == supervisor_id,
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Get all subordinates of a coordinator
    pub fn get_subordinates(&self, coordinator_id: &str) -> Vec<String> {
        self.get_role(coordinator_id)
            .and_then(|role| match role {
                AgentRole::Coordinator { subordinates, .. } => Some(subordinates),
                _ => None,
            })
            .unwrap_or_default()
    }
    
    /// Get supervisor of a worker
    pub fn get_supervisor(&self, worker_id: &str) -> Option<String> {
        self.get_role(worker_id).and_then(|role| match role {
            AgentRole::Worker { supervisor, .. } => Some(supervisor),
            _ => None,
        })
    }
    
    /// Get all agents in a team
    pub fn get_team_members(&self, team: &str) -> Vec<String> {
        self.teams.get(team)
            .map(|members| members.clone())
            .unwrap_or_default()
    }
    
    /// Add subordinate to a coordinator
    pub fn add_subordinate(&self, coordinator_id: &str, subordinate_id: String) -> bool {
        if let Some(mut role) = self.roles.get_mut(coordinator_id) {
            match role.value_mut() {
                AgentRole::Coordinator { subordinates, .. } => {
                    if !subordinates.contains(&subordinate_id) {
                        subordinates.push(subordinate_id.clone());
                        tracing::debug!("Added subordinate {} to {}", subordinate_id, coordinator_id);
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
    
    /// Remove subordinate from a coordinator
    pub fn remove_subordinate(&self, coordinator_id: &str, subordinate_id: &str) -> bool {
        if let Some(mut role) = self.roles.get_mut(coordinator_id) {
            match role.value_mut() {
                AgentRole::Coordinator { subordinates, .. } => {
                    if let Some(pos) = subordinates.iter().position(|id| id == subordinate_id) {
                        subordinates.remove(pos);
                        tracing::debug!("Removed subordinate {} from {}", subordinate_id, coordinator_id);
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
    
    /// Get all coordinators
    pub fn get_coordinators(&self) -> Vec<String> {
        self.roles.iter()
            .filter(|entry| entry.value().is_coordinator())
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Get all workers
    pub fn get_workers(&self) -> Vec<String> {
        self.roles.iter()
            .filter(|entry| entry.value().is_worker())
            .map(|entry| entry.key().clone())
            .collect()
    }
}

impl Default for AgentHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_role() {
        let hierarchy = AgentHierarchy::new();
        let role = AgentRole::Coordinator {
            subordinates: vec!["worker-1".to_string(), "worker-2".to_string()],
            team: "team-alpha".to_string(),
        };
        
        hierarchy.register("coord-1".to_string(), role.clone());
        
        let retrieved = hierarchy.get_role("coord-1").unwrap();
        assert!(retrieved.is_coordinator());
        assert_eq!(retrieved.subordinates().len(), 2);
    }

    #[test]
    fn test_worker_role() {
        let hierarchy = AgentHierarchy::new();
        let role = AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-alpha".to_string(),
        };
        
        hierarchy.register("worker-1".to_string(), role);
        
        assert!(hierarchy.is_subordinate("worker-1", "coord-1"));
        assert_eq!(hierarchy.get_supervisor("worker-1"), Some("coord-1".to_string()));
    }

    #[test]
    fn test_add_remove_subordinate() {
        let hierarchy = AgentHierarchy::new();
        let role = AgentRole::Coordinator {
            subordinates: vec![],
            team: "team-alpha".to_string(),
        };
        
        hierarchy.register("coord-1".to_string(), role);
        
        assert!(hierarchy.add_subordinate("coord-1", "worker-1".to_string()));
        assert_eq!(hierarchy.get_subordinates("coord-1").len(), 1);
        
        assert!(hierarchy.remove_subordinate("coord-1", "worker-1"));
        assert_eq!(hierarchy.get_subordinates("coord-1").len(), 0);
    }

    #[test]
    fn test_team_members() {
        let hierarchy = AgentHierarchy::new();
        
        hierarchy.register("agent-1".to_string(), AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-alpha".to_string(),
        });
        
        hierarchy.register("agent-2".to_string(), AgentRole::Worker {
            supervisor: "coord-1".to_string(),
            team: "team-alpha".to_string(),
        });
        
        let members = hierarchy.get_team_members("team-alpha");
        assert_eq!(members.len(), 2);
    }
}
