//! Tool trait definition

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Result, ToolSchema};

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution was successful
    pub success: bool,
    
    /// The result data (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create a successful result from serializable data
    pub fn success_data<T: Serialize>(data: T) -> Result<Self> {
        Ok(Self {
            success: true,
            data: Some(serde_json::to_value(data)?),
            error: None,
        })
    }

    /// Create an error result
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Trait for tools that can be called by agents
///
/// Tools are functions that agents can execute to perform actions
/// or gather information.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool's unique name
    ///
    /// This is used to identify and call the tool.
    fn name(&self) -> &str;

    /// Get a human-readable description of what the tool does
    ///
    /// This description is included in the LLM prompt to help
    /// the model understand when to use the tool.
    fn description(&self) -> &str;

    /// Get the JSON schema for the tool's parameters
    ///
    /// Returns a ToolSchema describing the expected parameters.
    fn parameters_schema(&self) -> ToolSchema;

    /// Execute the tool with given parameters
    ///
    /// # Arguments
    /// * `params` - JSON value containing the tool parameters
    ///
    /// # Returns
    /// A ToolResult containing the execution result or error
    async fn execute(&self, params: Value) -> Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success(serde_json::json!({"answer": 42}));
        assert!(result.success);
        assert!(result.data.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error.unwrap(), "Something went wrong");
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult::success(serde_json::json!({"value": 123}));
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ToolResult = serde_json::from_str(&json).unwrap();
        
        assert!(deserialized.success);
        assert!(deserialized.data.is_some());
    }

    #[test]
    fn test_success_data() {
        #[derive(Serialize)]
        struct Response {
            answer: i32,
        }

        let result = ToolResult::success_data(Response { answer: 42 }).unwrap();
        assert!(result.success);
        assert_eq!(result.data.unwrap()["answer"], 42);
    }
}

