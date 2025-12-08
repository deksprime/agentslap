//! Echo tool for testing

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolSchema};

/// Echo tool that returns its input unchanged
///
/// Useful for testing tool calling functionality.
pub struct EchoTool;

#[derive(Debug, Deserialize, Serialize)]
struct EchoParams {
    text: String,
}

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the provided text (useful for testing)"
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for echo")
            .with_properties(serde_json::json!({
                "text": property("string", "The text to echo back"),
            }))
            .with_required(vec!["text".to_string()])
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: EchoParams = serde_json::from_value(params)
            .map_err(|e| crate::ToolError::invalid_params(e.to_string()))?;

        Ok(ToolResult::success(serde_json::json!({
            "echoed": params.text,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo() {
        let echo = EchoTool;
        let params = serde_json::json!({
            "text": "Hello, world!"
        });

        let result = echo.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data.unwrap()["echoed"], "Hello, world!");
    }

    #[tokio::test]
    async fn test_echo_invalid_params() {
        let echo = EchoTool;
        let params = serde_json::json!({
            "wrong_field": "value"
        });

        let result = echo.execute(params).await;
        assert!(result.is_err());
    }
}

