//! Agent addressing

use serde::{Deserialize, Serialize};

/// Location where an agent runs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AgentLocation {
    /// Local (same process)
    Local,
    
    /// Different process on same machine
    Process {
        pid: u32,
    },
    
    /// Network (different machine)
    Network {
        host: String,
        port: u16,
    },
    
    /// Cluster (service discovery)
    Cluster {
        cluster_id: String,
    },
}

/// Universal agent address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentAddress {
    /// Agent identifier
    pub id: String,
    
    /// Where the agent runs
    pub location: AgentLocation,
}

impl AgentAddress {
    /// Create a local agent address
    pub fn local<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            location: AgentLocation::Local,
        }
    }

    /// Create a network agent address
    pub fn network<S: Into<String>>(id: S, host: S, port: u16) -> Self {
        Self {
            id: id.into(),
            location: AgentLocation::Network {
                host: host.into(),
                port,
            },
        }
    }

    /// Create a cluster agent address
    pub fn cluster<S: Into<String>>(id: S, cluster_id: S) -> Self {
        Self {
            id: id.into(),
            location: AgentLocation::Cluster {
                cluster_id: cluster_id.into(),
            },
        }
    }

    /// Check if this is a local agent
    pub fn is_local(&self) -> bool {
        matches!(self.location, AgentLocation::Local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_address() {
        let addr = AgentAddress::local("agent-1");
        assert_eq!(addr.id, "agent-1");
        assert!(addr.is_local());
    }

    #[test]
    fn test_network_address() {
        let addr = AgentAddress::network("agent-2", "192.168.1.100", 8080);
        assert_eq!(addr.id, "agent-2");
        assert!(!addr.is_local());
        
        match addr.location {
            AgentLocation::Network { host, port } => {
                assert_eq!(host, "192.168.1.100");
                assert_eq!(port, 8080);
            }
            _ => panic!("Expected Network location"),
        }
    }

    #[test]
    fn test_address_serialization() {
        let addr = AgentAddress::local("test");
        let json = serde_json::to_string(&addr).unwrap();
        let deserialized: AgentAddress = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, "test");
        assert!(deserialized.is_local());
    }
}



