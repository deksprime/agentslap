//! Tool registry for managing and executing tools

use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

use crate::{error::ToolError, tool::Tool, Result, ToolResult};

/// Registry for managing tools
///
/// Provides a central place to register, lookup, and execute tools.
/// Thread-safe and can be shared across async tasks.
#[derive(Clone)]
pub struct ToolRegistry {
    /// Map of tool name to tool implementation
    tools: Arc<DashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
        }
    }

    /// Register a tool
    ///
    /// # Arguments
    /// * `tool` - The tool to register
    ///
    /// # Returns
    /// Error if a tool with the same name is already registered
    pub fn register<T: Tool + 'static>(&self, tool: T) -> Result<()> {
        let name = tool.name().to_string();
        
        if self.tools.contains_key(&name) {
            return Err(ToolError::AlreadyRegistered(name.clone()));
        }

        self.tools.insert(name.clone(), Arc::new(tool));
        tracing::debug!("Registered tool: {}", name);
        Ok(())
    }

    /// Register a tool, replacing if it exists
    pub fn register_or_replace<T: Tool + 'static>(&self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name.clone(), Arc::new(tool));
        tracing::debug!("Registered/replaced tool: {}", name);
    }

    /// Check if a tool is registered
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).map(|entry| Arc::clone(entry.value()))
    }

    /// Execute a tool by name
    ///
    /// # Arguments
    /// * `name` - Name of the tool to execute
    /// * `params` - JSON parameters for the tool
    ///
    /// # Returns
    /// The result of the tool execution
    pub async fn execute(&self, name: &str, params: Value) -> Result<ToolResult> {
        let tool = self
            .get_tool(name)
            .ok_or_else(|| ToolError::not_found(name))?;

        tracing::info!("Executing tool: {} with params: {}", name, params);

        match tool.execute(params).await {
            Ok(result) => {
                tracing::debug!("Tool {} executed successfully", name);
                Ok(result)
            }
            Err(e) => {
                tracing::error!("Tool {} execution failed: {}", name, e);
                Err(e)
            }
        }
    }

    /// List all registered tool names
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get the number of registered tools
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Get all tools formatted for OpenAI function calling
    pub fn to_openai_functions(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|entry| {
                let tool = entry.value();
                let schema = tool.parameters_schema();
                schema.to_openai_function(tool.name(), tool.description())
            })
            .collect()
    }

    /// Get all tools formatted for Anthropic tool use
    pub fn to_anthropic_tools(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|entry| {
                let tool = entry.value();
                let schema = tool.parameters_schema();
                schema.to_anthropic_tool(tool.name(), tool.description())
            })
            .collect()
    }

    /// Clear all registered tools
    pub fn clear(&self) {
        let count = self.tools.len();
        self.tools.clear();
        tracing::info!("Cleared {} tools from registry", count);
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::ToolSchema;

    // Mock tool for testing
    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_tool"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn parameters_schema(&self) -> ToolSchema {
            ToolSchema::new()
        }

        async fn execute(&self, _params: Value) -> Result<ToolResult> {
            Ok(ToolResult::success(serde_json::json!({"result": "mocked"})))
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_tool() {
        let registry = ToolRegistry::new();
        let tool = MockTool;

        registry.register(tool).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.has_tool("mock_tool"));
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = ToolRegistry::new();

        registry.register(MockTool).unwrap();
        let result = registry.register(MockTool);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::AlreadyRegistered(_)));
    }

    #[test]
    fn test_register_or_replace() {
        let registry = ToolRegistry::new();

        registry.register_or_replace(MockTool);
        registry.register_or_replace(MockTool); // Should not error

        assert_eq!(registry.count(), 1);
    }

    #[tokio::test]
    async fn test_execute_tool() {
        let registry = ToolRegistry::new();
        registry.register(MockTool).unwrap();

        let result = registry.execute("mock_tool", serde_json::json!({})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_execute_nonexistent_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("nonexistent", serde_json::json!({})).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::NotFound(_)));
    }

    #[test]
    fn test_list_tools() {
        let registry = ToolRegistry::new();
        registry.register(MockTool).unwrap();

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert!(tools.contains(&"mock_tool".to_string()));
    }

    #[test]
    fn test_clear() {
        let registry = ToolRegistry::new();
        registry.register(MockTool).unwrap();

        assert_eq!(registry.count(), 1);
        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_get_tool() {
        let registry = ToolRegistry::new();
        registry.register(MockTool).unwrap();

        let tool = registry.get_tool("mock_tool");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "mock_tool");
    }
}

