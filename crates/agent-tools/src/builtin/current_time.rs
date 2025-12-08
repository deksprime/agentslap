//! Current time tool

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;

use crate::{Result, Tool, ToolResult, ToolSchema};

/// Tool that returns the current date and time
pub struct CurrentTimeTool;

#[async_trait]
impl Tool for CurrentTimeTool {
    fn name(&self) -> &str {
        "current_time"
    }

    fn description(&self) -> &str {
        "Get the current date and time in ISO 8601 format"
    }

    fn parameters_schema(&self) -> ToolSchema {
        // No parameters needed
        ToolSchema::new()
            .with_description("No parameters required")
    }

    async fn execute(&self, _params: Value) -> Result<ToolResult> {
        let now = Utc::now();

        Ok(ToolResult::success(serde_json::json!({
            "timestamp": now.to_rfc3339(),
            "unix": now.timestamp(),
            "formatted": now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_current_time() {
        let tool = CurrentTimeTool;
        let result = tool.execute(serde_json::json!({})).await.unwrap();

        assert!(result.success);
        let data = result.data.unwrap();
        assert!(data["timestamp"].is_string());
        assert!(data["unix"].is_number());
        assert!(data["formatted"].is_string());
    }

    #[test]
    fn test_current_time_schema() {
        let tool = CurrentTimeTool;
        let schema = tool.parameters_schema();
        
        assert_eq!(schema.schema_type, "object");
    }
}

