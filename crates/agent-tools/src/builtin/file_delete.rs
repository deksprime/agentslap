//! Dangerous file deletion tool (for testing HITL)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolRiskLevel, ToolSchema};

/// File deletion tool - DANGEROUS!
///
/// This tool is marked as Critical risk and should always require
/// human approval before execution.
///
/// **For testing HITL integration only** - not for production use!
pub struct FileDeleteTool;

#[derive(Debug, Deserialize, Serialize)]
struct FileDeleteParams {
    path: String,
}

#[async_trait]
impl Tool for FileDeleteTool {
    fn name(&self) -> &str {
        "file_delete"
    }

    fn description(&self) -> &str {
        "Delete a file from the filesystem (DANGEROUS - requires approval)"
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for file deletion")
            .with_properties(serde_json::json!({
                "path": property("string", "Path to the file to delete"),
            }))
            .with_required(vec!["path".to_string()])
    }

    fn risk_level(&self) -> ToolRiskLevel {
        ToolRiskLevel::Critical  // ← DANGEROUS!
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: FileDeleteParams = serde_json::from_value(params)
            .map_err(|e| crate::ToolError::invalid_params(e.to_string()))?;

        // SAFETY: This is a demo tool - would actually delete in production!
        // For now, just simulate
        tracing::warn!("SIMULATING file deletion: {}", params.path);

        Ok(ToolResult::success(serde_json::json!({
            "deleted": params.path,
            "simulated": true,
            "warning": "This tool requires approval in production!"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_delete_risk_level() {
        let tool = FileDeleteTool;
        assert_eq!(tool.risk_level(), ToolRiskLevel::Critical);
    }

    #[tokio::test]
    async fn test_file_delete_execution() {
        let tool = FileDeleteTool;
        let params = serde_json::json!({
            "path": "/tmp/test.txt"
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data.unwrap()["deleted"], "/tmp/test.txt");
    }

    #[test]
    fn test_schema() {
        let tool = FileDeleteTool;
        let schema = tool.parameters_schema();
        assert!(schema.properties.is_some());
        assert_eq!(schema.required.as_ref().unwrap().len(), 1);
    }
}

