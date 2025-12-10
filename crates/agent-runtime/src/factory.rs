//! Agent factory for dynamic agent spawning

use agent_llm::{LLMProvider, OpenAIProvider, AnthropicProvider};
use agent_session::{SessionStore, InMemoryStore};
use agent_tools::ToolRegistry;
use agent_hitl::{ApprovalStrategy, AutoApprove};
use agent_comms::{AgentAddress, AgentRegistry, MessageTransport};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::{Agent, AgentBuilder, AgentConfig, Result};

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
}

fn default_max_iterations() -> usize {
    10
}

/// Factory for creating and managing agents dynamically
pub struct AgentFactory {
    /// Role templates
    role_configs: Arc<DashMap<String, RoleConfig>>,
    
    /// Running agents (agent_id -> handle)
    agents: Arc<DashMap<String, JoinHandle<()>>>,
    
    /// Agent registry
    registry: Arc<AgentRegistry>,
    
    /// Transport for communication
    transport: Arc<dyn MessageTransport>,
    
    /// API keys
    openai_key: Option<String>,
    anthropic_key: Option<String>,
    
    /// Session store template
    session_store_factory: Arc<dyn Fn() -> Arc<dyn SessionStore> + Send + Sync>,
}

impl AgentFactory {
    /// Create a new agent factory
    pub fn new(
        transport: Arc<dyn MessageTransport>,
        openai_key: Option<String>,
        anthropic_key: Option<String>,
    ) -> Self {
        Self {
            role_configs: Arc::new(DashMap::new()),
            agents: Arc::new(DashMap::new()),
            registry: Arc::new(AgentRegistry::new()),
            transport,
            openai_key,
            anthropic_key,
            session_store_factory: Arc::new(|| Arc::new(InMemoryStore::new())),
        }
    }

    /// Register a role configuration
    pub fn register_role(&self, config: RoleConfig) {
        tracing::info!("Registered role: {}", config.role);
        self.role_configs.insert(config.role.clone(), config);
    }

    /// Spawn an agent with a specific role
    ///
    /// Returns the agent ID
    pub fn spawn_agent(&self, role: &str, agent_id: Option<String>) -> Result<String> {
        let role_config = self.role_configs.get(role)
            .ok_or_else(|| crate::AgentRuntimeError::config(format!("Unknown role: {}", role)))?
            .clone();

        let agent_id = agent_id.unwrap_or_else(|| {
            format!("{}-{}", role, uuid::Uuid::new_v4().to_string()[..8].to_string())
        });

        // Create tools registry
        let tools = ToolRegistry::new();
        tools.register(agent_tools::builtin::CalculatorTool)
            .map_err(|e| crate::AgentRuntimeError::Tool(e))?;
        tools.register(agent_tools::builtin::EchoTool)
            .map_err(|e| crate::AgentRuntimeError::Tool(e))?;
        tools.register(agent_tools::builtin::CurrentTimeTool)
            .map_err(|e| crate::AgentRuntimeError::Tool(e))?;

        // Build agent with concrete provider type (use match for each type)
        let _agent = match role_config.provider.as_str() {
            "openai" => {
                let key = self.openai_key.as_ref()
                    .ok_or_else(|| crate::AgentRuntimeError::config("OpenAI key not set"))?;
                let provider = OpenAIProvider::new(key, &role_config.model)
                    .map_err(|e| crate::AgentRuntimeError::config(format!("Provider error: {}", e)))?;
                
                Agent::builder()
                    .provider(provider)
                    .tools(tools.clone())
                    .session_store(InMemoryStore::new())
                    .hitl(AutoApprove::new())
                    .session_id(&agent_id)
                    .system_message(&role_config.system_message)
                    .max_iterations(role_config.max_iterations)
                    .build()?
            }
            "anthropic" => {
                let key = self.anthropic_key.as_ref()
                    .ok_or_else(|| crate::AgentRuntimeError::config("Anthropic key not set"))?;
                let provider = AnthropicProvider::new(key, &role_config.model)
                    .map_err(|e| crate::AgentRuntimeError::config(format!("Provider error: {}", e)))?;
                
                Agent::builder()
                    .provider(provider)
                    .tools(tools)
                    .session_store(InMemoryStore::new())
                    .hitl(AutoApprove::new())
                    .session_id(&agent_id)
                    .system_message(&role_config.system_message)
                    .max_iterations(role_config.max_iterations)
                    .build()?
            }
            _ => return Err(crate::AgentRuntimeError::config("Unknown provider")),
        };

        // Register in registry
        self.registry.register(
            AgentAddress::local(&agent_id),
            Some(role.to_string())
        );

        tracing::info!("Spawned agent: {} (role: {})", agent_id, role);

        // For now, just store the agent (actual background loop would be more complex)
        // This is the foundation - can extend to run message loops
        
        Ok(agent_id)
    }

    /// Get count of running agents
    pub fn agent_count(&self) -> usize {
        self.registry.count()
    }

    /// List all running agents
    pub fn list_agents(&self) -> Vec<String> {
        self.registry.list_all()
    }

    /// Get agents by role
    pub fn get_agents_by_role(&self, role: &str) -> Vec<AgentAddress> {
        self.registry.find_by_role(role)
    }

    /// Get registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_comms::InProcessTransport;

    #[test]
    fn test_factory_creation() {
        let transport = Arc::new(InProcessTransport::new());
        let factory = AgentFactory::new(transport, Some("test-key".to_string()), None);
        
        assert_eq!(factory.agent_count(), 0);
    }

    #[test]
    fn test_register_role() {
        let transport = Arc::new(InProcessTransport::new());
        let factory = AgentFactory::new(transport, Some("test-key".to_string()), None);

        let role = RoleConfig {
            role: "test_specialist".to_string(),
            system_message: "You are a test specialist".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            provider: "openai".to_string(),
            tools: vec![],
            can_spawn_agents: false,
            max_iterations: 10,
        };

        factory.register_role(role);
        assert!(factory.role_configs.contains_key("test_specialist"));
    }
}

