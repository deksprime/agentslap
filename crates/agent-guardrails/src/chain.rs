//! Guardrail chain for executing multiple guardrails

use std::sync::Arc;

use crate::{
    guardrail::{Guardrail, InputContext, OutputContext, ToolCallContext},
    violation::ViolationAction,
    Result, Violation,
};

/// Chain of guardrails that execute in sequence
///
/// Supports both fail-fast (stop on first violation) and
/// collect-all (gather all violations) modes.
#[derive(Clone)]
pub struct GuardrailChain {
    guardrails: Vec<Arc<dyn Guardrail>>,
    fail_fast: bool,
}

impl GuardrailChain {
    /// Create a new guardrail chain
    pub fn new() -> Self {
        Self {
            guardrails: Vec::new(),
            fail_fast: false,
        }
    }

    /// Add a guardrail to the chain
    pub fn with_guardrail<G: Guardrail + 'static>(mut self, guardrail: G) -> Self {
        self.guardrails.push(Arc::new(guardrail));
        self
    }

    /// Set fail-fast mode
    ///
    /// If true, stops on first violation.
    /// If false, collects all violations.
    pub fn fail_fast(mut self, enabled: bool) -> Self {
        self.fail_fast = enabled;
        self
    }

    /// Get the number of guardrails in the chain
    pub fn len(&self) -> usize {
        self.guardrails.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.guardrails.is_empty()
    }

    /// Check input against all guardrails
    pub async fn check_input(&self, context: &InputContext) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for guardrail in &self.guardrails {
            if let Some(violation) = guardrail.check_input(context).await? {
                tracing::warn!(
                    "Guardrail {} triggered on input: {}",
                    guardrail.name(),
                    violation.message
                );
                
                violations.push(violation.clone());
                
                if self.fail_fast && violation.action == ViolationAction::Block {
                    break;
                }
            }
        }

        Ok(violations)
    }

    /// Check output against all guardrails
    pub async fn check_output(&self, context: &OutputContext) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for guardrail in &self.guardrails {
            if let Some(violation) = guardrail.check_output(context).await? {
                tracing::warn!(
                    "Guardrail {} triggered on output: {}",
                    guardrail.name(),
                    violation.message
                );
                
                violations.push(violation.clone());
                
                if self.fail_fast && violation.action == ViolationAction::Block {
                    break;
                }
            }
        }

        Ok(violations)
    }

    /// Check tool call against all guardrails
    pub async fn check_tool_call(&self, context: &ToolCallContext) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for guardrail in &self.guardrails {
            if let Some(violation) = guardrail.check_tool_call(context).await? {
                tracing::warn!(
                    "Guardrail {} triggered on tool call: {}",
                    guardrail.name(),
                    violation.message
                );
                
                violations.push(violation.clone());
                
                if self.fail_fast && violation.action == ViolationAction::Block {
                    break;
                }
            }
        }

        Ok(violations)
    }

    /// Check if any violations should block execution
    pub fn has_blocking_violations(violations: &[Violation]) -> bool {
        violations.iter().any(|v| v.should_block())
    }
}

impl Default for GuardrailChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::{ViolationAction, ViolationSeverity};

    struct AlwaysBlockGuardrail;

    #[async_trait]
    impl Guardrail for AlwaysBlockGuardrail {
        fn name(&self) -> &str {
            "always_block"
        }

        async fn check_input(&self, _context: &InputContext) -> Result<Option<Violation>> {
            Ok(Some(Violation::new(
                "always_block",
                ViolationSeverity::High,
                "Always blocks",
                ViolationAction::Block,
            )))
        }

        async fn check_output(&self, _context: &OutputContext) -> Result<Option<Violation>> {
            Ok(None)
        }

        async fn check_tool_call(&self, _context: &ToolCallContext) -> Result<Option<Violation>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_chain_creation() {
        let chain = GuardrailChain::new();
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());
    }

    #[tokio::test]
    async fn test_add_guardrail() {
        let chain = GuardrailChain::new()
            .with_guardrail(AlwaysBlockGuardrail);

        assert_eq!(chain.len(), 1);
    }

    #[tokio::test]
    async fn test_check_input_with_violation() {
        let chain = GuardrailChain::new()
            .with_guardrail(AlwaysBlockGuardrail);

        let context = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: None,
        };

        let violations = chain.check_input(&context).await.unwrap();
        assert_eq!(violations.len(), 1);
        assert!(GuardrailChain::has_blocking_violations(&violations));
    }

    #[tokio::test]
    async fn test_fail_fast_mode() {
        let chain = GuardrailChain::new()
            .with_guardrail(AlwaysBlockGuardrail)
            .with_guardrail(AlwaysBlockGuardrail)
            .fail_fast(true);

        let context = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: None,
        };

        let violations = chain.check_input(&context).await.unwrap();
        assert_eq!(violations.len(), 1); // Stopped after first
    }

    #[tokio::test]
    async fn test_collect_all_mode() {
        let chain = GuardrailChain::new()
            .with_guardrail(AlwaysBlockGuardrail)
            .with_guardrail(AlwaysBlockGuardrail)
            .fail_fast(false);

        let context = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: None,
        };

        let violations = chain.check_input(&context).await.unwrap();
        assert_eq!(violations.len(), 2); // Collected all
    }
}

