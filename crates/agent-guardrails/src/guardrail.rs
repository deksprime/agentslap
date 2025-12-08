//! Guardrail trait definition

use async_trait::async_trait;
use serde_json::Value;

use crate::{Result, Violation};

/// Context for input validation
#[derive(Debug, Clone)]
pub struct InputContext {
    /// The user message
    pub message: String,
    /// Optional user ID
    pub user_id: Option<String>,
    /// Optional session ID
    pub session_id: Option<String>,
}

/// Context for output validation
#[derive(Debug, Clone)]
pub struct OutputContext {
    /// The LLM response
    pub response: String,
    /// Model that generated the response
    pub model: String,
    /// Tokens used (if available)
    pub tokens_used: Option<usize>,
}

/// Context for tool call validation
#[derive(Debug, Clone)]
pub struct ToolCallContext {
    /// Tool being called
    pub tool_name: String,
    /// Tool parameters
    pub parameters: Value,
    /// User who initiated
    pub user_id: Option<String>,
}

/// Trait for implementing guardrails
///
/// Guardrails validate inputs, outputs, and tool calls to ensure
/// safe and compliant agent behavior.
#[async_trait]
pub trait Guardrail: Send + Sync {
    /// Get the name of this guardrail
    fn name(&self) -> &str;

    /// Check user input
    ///
    /// # Returns
    /// `None` if input is allowed, `Some(Violation)` if blocked
    async fn check_input(&self, context: &InputContext) -> Result<Option<Violation>>;

    /// Check LLM output
    ///
    /// # Returns
    /// `None` if output is allowed, `Some(Violation)` if blocked
    async fn check_output(&self, context: &OutputContext) -> Result<Option<Violation>>;

    /// Check tool call
    ///
    /// # Returns
    /// `None` if tool call is allowed, `Some(Violation)` if blocked
    async fn check_tool_call(&self, context: &ToolCallContext) -> Result<Option<Violation>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestGuardrail;

    #[async_trait]
    impl Guardrail for TestGuardrail {
        fn name(&self) -> &str {
            "test"
        }

        async fn check_input(&self, _context: &InputContext) -> Result<Option<Violation>> {
            Ok(None)
        }

        async fn check_output(&self, _context: &OutputContext) -> Result<Option<Violation>> {
            Ok(None)
        }

        async fn check_tool_call(&self, _context: &ToolCallContext) -> Result<Option<Violation>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_guardrail_trait() {
        let guard = TestGuardrail;
        assert_eq!(guard.name(), "test");

        let input_ctx = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: None,
        };

        let result = guard.check_input(&input_ctx).await.unwrap();
        assert!(result.is_none());
    }
}

