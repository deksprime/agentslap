//! Agent factory for dynamic agent spawning

use agent_llm::{OpenAIProvider, AnthropicProvider};
use agent_session::InMemoryStore;
use agent_tools::ToolRegistry;
use agent_hitl::AutoApprove;
use agent_comms::{AgentAddress, AgentCoordinator, AgentRole};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{Result, CoordinationError, orchestrator};

/// Role configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// Role name
    pub role: String,
    
    /// System message
    pub system_message: String,
    
    /// LLM model to use
    pub model: String,
    
    /// LLM provider (openai, anthropic)
    pub provider: String,
    
    /// Tools available to this role
    pub tools: Vec<String>,
    
    /// Can this agent spawn other agents?
    #[serde(default)]
    pub can_spawn_agents: bool,
    
    /// Max iterations
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    
    /// Hierarchical role (coordinator, worker, peer, standalone)
    #[serde(default = "default_hierarchy_role")]
    pub hierarchy_role: String,
    
    /// Team name if part of a team
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    
    /// Supervisor ID if this is a worker
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervisor: Option<String>,
}

fn default_max_iterations() -> usize {
    10
}

fn default_hierarchy_role() -> String {
    "standalone".to_string()
}

/// Factory for creating and managing agents dynamically
pub struct AgentFactory {
    /// Role templates
    role_configs: Arc<DashMap<String, RoleConfig>>,
    
    /// Agent coordinator for multi-agent management
    coordinator: Arc<AgentCoordinator>,
    
    /// API keys
    openai_key: Option<String>,
    anthropic_key: Option<String>,
}

impl AgentFactory {
    /// Create a new agent factory
    pub fn new(
        coordinator: Arc<AgentCoordinator>,
        openai_key: Option<String>,
        anthropic_key: Option<String>,
    ) -> Self {
        Self {
            role_configs: Arc::new(DashMap::new()),
            coordinator,
            openai_key,
            anthropic_key,
        }
    }

    /// Register a role configuration
    pub fn register_role(&self, config: RoleConfig) {
        tracing::info!("Registered role template: {}", config.role);
        self.role_configs.insert(config.role.clone(), config);
    }

    /// Spawn an agent with a specific role and start its message loop
    ///
    /// Returns the agent ID
    pub async fn spawn_agent(&self, role: &str, agent_id: Option<String>) -> Result<String> {
        let role_config = self.role_configs.get(role)
            .ok_or_else(|| CoordinationError::config(format!("Unknown role: {}", role)))?
            .clone();

        let agent_id = agent_id.unwrap_or_else(|| {
            format!("{}-{}", role, uuid::Uuid::new_v4().to_string()[..8].to_string())
        });

        // Create tools registry
        let tools = ToolRegistry::new();
        tools.register(agent_tools::builtin::CalculatorTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;
        tools.register(agent_tools::builtin::EchoTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;
        tools.register(agent_tools::builtin::CurrentTimeTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;

        // Register communication tools for multi-agent coordination
        tools.register(agent_tools::builtin::DelegateTaskTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;
        tools.register(agent_tools::builtin::EscalateTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;
        tools.register(agent_tools::builtin::BroadcastToTeamTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;
        tools.register(agent_tools::builtin::SendMessageTool)
            .map_err(|e| CoordinationError::other(format!("Tool registration failed: {}", e)))?;
        
        tracing::info!("Registered {} tools including coordination tools for agent {}", tools.count(), agent_id);

        // Build agent with concrete provider type
        let agent = match role_config.provider.as_str() {
            "openai" => {
                let key = self.openai_key.as_ref()
                    .ok_or_else(|| CoordinationError::config("OpenAI key not set"))?;
                let provider = OpenAIProvider::new(key, &role_config.model)
                    .map_err(|e| CoordinationError::config(format!("Provider error: {}", e)))?;
                
                agent_runtime::AgentBuilder::new()
                    .provider(provider)
                    .tools(tools.clone())
                    .session_store(InMemoryStore::new())
                    .hitl(AutoApprove::new())
                    .session_id(&agent_id)
                    .system_message(&role_config.system_message)
                    .max_iterations(role_config.max_iterations)
                    .coordinator(self.coordinator.clone())
                    .build()
                    .map_err(|e| CoordinationError::other(format!("Agent build failed: {}", e)))?
            }
            "anthropic" => {
                let key = self.anthropic_key.as_ref()
                    .ok_or_else(|| CoordinationError::config("Anthropic key not set"))?;
                let provider = AnthropicProvider::new(key, &role_config.model)
                    .map_err(|e| CoordinationError::config(format!("Provider error: {}", e)))?;
                
                agent_runtime::AgentBuilder::new()
                    .provider(provider)
                    .tools(tools)
                    .session_store(InMemoryStore::new())
                    .hitl(AutoApprove::new())
                    .session_id(&agent_id)
                    .system_message(&role_config.system_message)
                    .max_iterations(role_config.max_iterations)
                    .coordinator(self.coordinator.clone())
                    .build()
                    .map_err(|e| CoordinationError::other(format!("Agent build failed: {}", e)))?
            }
            _ => return Err(CoordinationError::config("Unknown provider")),
        };

        // Determine hierarchical role
        let hierarchy_role = self.create_hierarchy_role(&role_config)?;

        // Register in coordinator with hierarchy
        let address = AgentAddress::local(&agent_id);
        self.coordinator.register_agent(address, hierarchy_role);

        // Spawn agent with message loop
        orchestrator::spawn_agent_with_loop(
            agent,
            agent_id.clone(),
            self.coordinator.clone(),
        );

        tracing::info!("Spawned agent: {} (role: {})", agent_id, role);

        Ok(agent_id)
    }
    
    /// Create AgentRole from RoleConfig
    fn create_hierarchy_role(&self, config: &RoleConfig) -> Result<AgentRole> {
        match config.hierarchy_role.as_str() {
            "coordinator" => {
                Ok(AgentRole::Coordinator {
                    subordinates: vec![],
                    team: config.team.clone().unwrap_or_else(|| "default".to_string()),
                })
            }
            "worker" => {
                let supervisor = config.supervisor.clone()
                    .ok_or_else(|| CoordinationError::config("Worker role requires supervisor"))?;
                Ok(AgentRole::Worker {
                    supervisor,
                    team: config.team.clone().unwrap_or_else(|| "default".to_string()),
                })
            }
            "peer" => {
                Ok(AgentRole::Peer {
                    team: config.team.clone().unwrap_or_else(|| "default".to_string()),
                    peers: vec![],
                })
            }
            "standalone" | _ => {
                Ok(AgentRole::Standalone)
            }
        }
    }

    /// Get count of running agents
    pub fn agent_count(&self) -> usize {
        self.coordinator.agent_count()
    }

    /// List all running agents
    pub fn list_agents(&self) -> Vec<String> {
        self.coordinator.list_agents()
    }

    /// Get agents by role
    pub fn get_agents_by_role(&self, role: &str) -> Vec<AgentAddress> {
        self.coordinator.registry().find_by_role(role)
    }
    
    /// Get the coordinator
    pub fn coordinator(&self) -> &Arc<AgentCoordinator> {
        &self.coordinator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_comms::InProcessTransport;

    #[test]
    fn test_factory_creation() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport));
        let factory = AgentFactory::new(coordinator, Some("test-key".to_string()), None);
        
        assert_eq!(factory.agent_count(), 0);
    }

    #[test]
    fn test_register_role() {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport));
        let factory = AgentFactory::new(coordinator, Some("test-key".to_string()), None);

        let role = RoleConfig {
            role: "test_specialist".to_string(),
            system_message: "You are a test specialist".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            provider: "openai".to_string(),
            tools: vec![],
            can_spawn_agents: false,
            max_iterations: 10,
            hierarchy_role: "standalone".to_string(),
            team: None,
            supervisor: None,
        };

        factory.register_role(role);
        assert!(factory.role_configs.contains_key("test_specialist"));
    }
}
