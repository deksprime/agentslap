//! Agent implementation

use agent_llm::{Conversation, LLMProvider};
use agent_session::SessionStore;
use agent_tools::{ToolRegistry, ToolResult};
use std::sync::Arc;

use crate::{error::AgentRuntimeError, parser, Result};

/// Configuration for agent behavior
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maximum iterations in agent loop (prevents infinite loops)
    pub max_iterations: usize,
    
    /// Session ID for conversation persistence
    pub session_id: Option<String>,
    
    /// System message for the agent
    pub system_message: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            session_id: None,
            system_message: Some("You are a helpful AI assistant.".to_string()),
        }
    }
}

/// Main agent struct combining all components
pub struct Agent {
    /// LLM provider
    provider: Arc<dyn LLMProvider>,
    
    /// Tool registry
    tools: Arc<ToolRegistry>,
    
    /// Session store
    session_store: Arc<dyn SessionStore>,
    
    /// Agent configuration
    config: AgentConfig,
}

impl Agent {
    /// Create a new agent builder
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Run the agent with a user message
    ///
    /// This is the main entry point for agent execution.
    /// It will:
    /// 1. Load or create conversation from session
    /// 2. Add user message
    /// 3. Call LLM with tools
    /// 4. Parse and execute any tool calls
    /// 5. Loop until final answer
    /// 6. Save conversation to session
    /// 7. Return final response
    pub async fn run(&self, user_message: &str) -> Result<String> {
        // Load or create conversation
        let mut conversation = self.load_or_create_conversation().await?;

        // Add user message
        conversation.add_user(user_message);
        tracing::info!("User: {}", user_message);

        // Run agent loop
        let mut iterations = 0;

        loop {
            iterations += 1;

            if iterations > self.config.max_iterations {
                tracing::error!("Max iterations ({}) exceeded", self.config.max_iterations);
                return Err(AgentRuntimeError::MaxIterationsExceeded(
                    self.config.max_iterations,
                ));
            }

            tracing::debug!("Iteration {}/{}", iterations, self.config.max_iterations);

            // Get tools in provider format
            let tools_available = self.tools.count() > 0;
            
            let raw_response = if tools_available {
                // Get tools based on provider type
                let tools = match self.provider.name() {
                    "openai" => self.tools.to_openai_functions(),
                    "anthropic" => self.tools.to_anthropic_tools(),
                    _ => vec![],
                };

                // Send with tools
                tracing::debug!("Sending {} tools to LLM", tools.len());
                self.provider
                    .send_message_with_tools(conversation.messages().to_vec(), tools)
                    .await?
            } else {
                // No tools, use regular message
                let response = self.provider.send_message(conversation.messages().to_vec()).await?;
                conversation.add_assistant(&response.content);
                self.save_conversation(&conversation).await?;
                tracing::info!("Assistant: {}", response.content);
                return Ok(response.content);
            };

            // Parse tool calls based on provider
            let tool_calls = match self.provider.name() {
                "openai" => {
                    // Check if there are tool calls in the response
                    if let Some(tool_calls_json) = raw_response.get("choices")
                        .and_then(|v| v.get(0))
                        .and_then(|v| v.get("message"))
                        .and_then(|v| v.get("tool_calls"))
                    {
                        parser::parse_openai_tool_calls(&serde_json::json!({
                            "tool_calls": tool_calls_json
                        })).ok()
                    } else {
                        None
                    }
                }
                "anthropic" => {
                    // Anthropic puts tool calls in content blocks
                    parser::parse_anthropic_tool_calls(&raw_response).ok()
                }
                _ => None,
            };

            // Execute tools if LLM requested them
            if let Some(calls) = tool_calls {
                if !calls.is_empty() {
                    tracing::info!("LLM requested {} tool(s)", calls.len());

                    // Add assistant message saying it's using tools
                    conversation.add_assistant("I'll use some tools to help answer that.");

                    // Execute each tool
                    for tool_call in &calls {
                        tracing::info!("Executing tool: {}", tool_call.name);
                        
                        let result = self.execute_tool_call(tool_call).await?;
                        
                        // Format tool result as user message (simulating tool return)
                        let result_text = if result.success {
                            format!("Tool '{}' returned: {}", 
                                tool_call.name,
                                result.data.as_ref()
                                    .map(|d| serde_json::to_string_pretty(d).unwrap_or_default())
                                    .unwrap_or_default()
                            )
                        } else {
                            format!("Tool '{}' failed: {}", 
                                tool_call.name,
                                result.error.as_ref().unwrap_or(&"Unknown error".to_string())
                            )
                        };

                        conversation.add_user(&result_text);
                        tracing::info!("Tool result added to conversation");
                    }

                    // Loop continues - LLM will see tool results and respond
                    continue;
                }
            }

            // No tool calls - extract final response
            let content = match self.provider.name() {
                "openai" => {
                    raw_response.get("choices")
                        .and_then(|v| v.get(0))
                        .and_then(|v| v.get("message"))
                        .and_then(|v| v.get("content"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                }
                "anthropic" => {
                    raw_response.get("content")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                }
                _ => String::new(),
            };

            conversation.add_assistant(&content);
            self.save_conversation(&conversation).await?;
            tracing::info!("Assistant: {}", content);

            return Ok(content);
        }
    }

    /// Load conversation from session or create new one
    async fn load_or_create_conversation(&self) -> Result<Conversation> {
        if let Some(session_id) = &self.config.session_id {
            match self.session_store.load(session_id).await {
                Ok(conv) => {
                    tracing::debug!("Loaded conversation from session: {}", session_id);
                    return Ok(conv);
                }
                Err(e) => {
                    tracing::warn!("Failed to load session {}: {}", session_id, e);
                    // Fall through to create new
                }
            }
        }

        // Create new conversation
        let mut conversation = Conversation::new();
        
        if let Some(system_msg) = &self.config.system_message {
            conversation.add_system(system_msg);
        }

        Ok(conversation)
    }

    /// Save conversation to session
    async fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        if let Some(session_id) = &self.config.session_id {
            self.session_store
                .save(session_id, conversation.clone())
                .await?;
            tracing::debug!("Saved conversation to session: {}", session_id);
        }
        Ok(())
    }

    /// Execute a tool call
    async fn execute_tool_call(&self, tool_call: &parser::ToolCall) -> Result<ToolResult> {
        tracing::info!("Executing tool: {} with params: {}", tool_call.name, tool_call.parameters);

        self.tools
            .execute(&tool_call.name, tool_call.parameters.clone())
            .await
            .map_err(|e| e.into())
    }
}

/// Builder for constructing an Agent
pub struct AgentBuilder {
    provider: Option<Arc<dyn LLMProvider>>,
    tools: Option<Arc<ToolRegistry>>,
    session_store: Option<Arc<dyn SessionStore>>,
    config: AgentConfig,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new() -> Self {
        Self {
            provider: None,
            tools: None,
            session_store: None,
            config: AgentConfig::default(),
        }
    }

    /// Set the LLM provider
    pub fn provider<P: LLMProvider + 'static>(mut self, provider: P) -> Self {
        self.provider = Some(Arc::new(provider));
        self
    }

    /// Set the tool registry
    pub fn tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = Some(Arc::new(tools));
        self
    }

    /// Set the session store
    pub fn session_store<S: SessionStore + 'static>(mut self, store: S) -> Self {
        self.session_store = Some(Arc::new(store));
        self
    }

    /// Set the agent configuration
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Set max iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    /// Set session ID
    pub fn session_id<S: Into<String>>(mut self, id: S) -> Self {
        self.config.session_id = Some(id.into());
        self
    }

    /// Set system message
    pub fn system_message<S: Into<String>>(mut self, msg: S) -> Self {
        self.config.system_message = Some(msg.into());
        self
    }

    /// Build the agent
    pub fn build(self) -> Result<Agent> {
        let provider = self
            .provider
            .ok_or_else(|| AgentRuntimeError::config("LLM provider not set"))?;

        let tools = self.tools.unwrap_or_else(|| Arc::new(ToolRegistry::new()));

        let session_store = self
            .session_store
            .ok_or_else(|| AgentRuntimeError::config("Session store not set"))?;

        Ok(Agent {
            provider,
            tools,
            session_store,
            config: self.config,
        })
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_llm::OpenAIProvider;
    use agent_session::InMemoryStore;
    use agent_tools::builtin::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert!(config.system_message.is_some());
    }

    #[test]
    fn test_agent_builder() {
        let provider = OpenAIProvider::new("test-key", "gpt-4").unwrap();
        let tools = ToolRegistry::new();
        tools.register(CalculatorTool).unwrap();
        let store = InMemoryStore::new();

        let agent = Agent::builder()
            .provider(provider)
            .tools(tools)
            .session_store(store)
            .max_iterations(20)
            .session_id("test-session")
            .system_message("Test agent")
            .build();

        assert!(agent.is_ok());
        let agent = agent.unwrap();
        assert_eq!(agent.config.max_iterations, 20);
        assert_eq!(agent.config.session_id.unwrap(), "test-session");
    }

    #[test]
    fn test_builder_missing_provider() {
        let store = InMemoryStore::new();
        
        let result = Agent::builder()
            .session_store(store)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_store() {
        let provider = OpenAIProvider::new("test-key", "gpt-4").unwrap();
        
        let result = Agent::builder()
            .provider(provider)
            .build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_or_create_conversation() {
        let provider = OpenAIProvider::new("test-key", "gpt-4").unwrap();
        let store = InMemoryStore::new();

        let agent = Agent::builder()
            .provider(provider)
            .session_store(store.clone())
            .session_id("test")
            .system_message("Hello")
            .build()
            .unwrap();

        // Should create new conversation
        let conv = agent.load_or_create_conversation().await.unwrap();
        assert_eq!(conv.len(), 1); // Just system message
        assert!(conv.system_message().is_some());
    }
}

