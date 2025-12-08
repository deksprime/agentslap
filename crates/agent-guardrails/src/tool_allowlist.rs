//! Tool allowlist guardrail

use async_trait::async_trait;
use std::collections::HashSet;

use crate::{
    guardrail::{Guardrail, InputContext, OutputContext, ToolCallContext},
    violation::{ViolationAction, ViolationSeverity},
    Result, Violation,
};

/// Tool allowlist guardrail
///
/// Only allows specified tools to be executed.
pub struct ToolAllowlist {
    /// Set of allowed tool names
    allowed_tools: HashSet<String>,
}

impl ToolAllowlist {
    /// Create a new tool allowlist
    ///
    /// # Arguments
    /// * `allowed` - List of allowed tool names
    pub fn new(allowed: Vec<String>) -> Self {
        Self {
            allowed_tools: allowed.into_iter().collect(),
        }
    }

    /// Add an allowed tool
    pub fn allow(mut self, tool_name: impl Into<String>) -> Self {
        self.allowed_tools.insert(tool_name.into());
        self
    }

    /// Check if a tool is allowed
    pub fn is_allowed(&self, tool_name: &str) -> bool {
        self.allowed_tools.contains(tool_name)
    }

    /// Get list of allowed tools
    pub fn allowed_tools(&self) -> Vec<String> {
        self.allowed_tools.iter().cloned().collect()
    }
}

#[async_trait]
impl Guardrail for ToolAllowlist {
    fn name(&self) -> &str {
        "tool_allowlist"
    }

    async fn check_input(&self, _context: &InputContext) -> Result<Option<Violation>> {
        // Tool allowlist doesn't check inputs
        Ok(None)
    }

    async fn check_output(&self, _context: &OutputContext) -> Result<Option<Violation>> {
        // Tool allowlist doesn't check outputs
        Ok(None)
    }

    async fn check_tool_call(&self, context: &ToolCallContext) -> Result<Option<Violation>> {
        if !self.is_allowed(&context.tool_name) {
            let msg = format!("Tool '{}' is not in allowlist", context.tool_name);
            return Ok(Some(
                Violation::new(
                    "tool_allowlist",
                    ViolationSeverity::High,
                    &msg,
                    ViolationAction::Block,
                )
                .with_context(serde_json::json!({
                    "tool": context.tool_name,
                    "allowed_tools": self.allowed_tools()
                }))
            ));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allowed_tool() {
        let allowlist = ToolAllowlist::new(vec!["calculator".to_string(), "echo".to_string()]);

        let context = ToolCallContext {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({}),
            user_id: None,
        };

        let violation = allowlist.check_tool_call(&context).await.unwrap();
        assert!(violation.is_none());
    }

    #[tokio::test]
    async fn test_blocked_tool() {
        let allowlist = ToolAllowlist::new(vec!["calculator".to_string()]);

        let context = ToolCallContext {
            tool_name: "dangerous_tool".to_string(),
            parameters: serde_json::json!({}),
            user_id: None,
        };

        let violation = allowlist.check_tool_call(&context).await.unwrap();
        assert!(violation.is_some());
        assert!(violation.unwrap().should_block());
    }

    #[test]
    fn test_is_allowed() {
        let allowlist = ToolAllowlist::new(vec!["calc".to_string()]);
        assert!(allowlist.is_allowed("calc"));
        assert!(!allowlist.is_allowed("other"));
    }

    #[test]
    fn test_allow_method() {
        let allowlist = ToolAllowlist::new(vec![])
            .allow("tool1")
            .allow("tool2");

        assert!(allowlist.is_allowed("tool1"));
        assert!(allowlist.is_allowed("tool2"));
        assert_eq!(allowlist.allowed_tools().len(), 2);
    }
}

