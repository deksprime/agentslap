//! Cost limiting guardrail based on token usage

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    guardrail::{Guardrail, InputContext, OutputContext, ToolCallContext},
    violation::{ViolationAction, ViolationSeverity},
    Result, Violation,
};

/// Cost limiter guardrail
///
/// Tracks token usage and blocks when budget is exceeded.
pub struct CostLimiter {
    /// Maximum tokens allowed per session/user
    max_tokens: usize,
    /// Token usage per session (public for testing/manual tracking)
    pub usage: Arc<DashMap<String, usize>>,
    /// Warning threshold (percentage of max)
    warn_threshold: f64,
}

impl CostLimiter {
    /// Create a new cost limiter
    ///
    /// # Arguments
    /// * `max_tokens` - Maximum tokens allowed
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            usage: Arc::new(DashMap::new()),
            warn_threshold: 0.8, // Warn at 80%
        }
    }

    /// Set warning threshold (0.0 to 1.0)
    pub fn warn_threshold(mut self, threshold: f64) -> Self {
        self.warn_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Get current usage for a session
    pub fn get_usage(&self, session_id: &str) -> usize {
        self.usage.get(session_id).map(|v| *v).unwrap_or(0)
    }

    /// Get remaining budget
    pub fn get_remaining(&self, session_id: &str) -> usize {
        self.max_tokens.saturating_sub(self.get_usage(session_id))
    }

    /// Reset usage for a session
    pub fn reset(&self, session_id: &str) {
        self.usage.remove(session_id);
    }
}

impl Clone for CostLimiter {
    fn clone(&self) -> Self {
        Self {
            max_tokens: self.max_tokens,
            usage: Arc::clone(&self.usage),
            warn_threshold: self.warn_threshold,
        }
    }
}

#[async_trait]
impl Guardrail for CostLimiter {
    fn name(&self) -> &str {
        "cost_limiter"
    }

    async fn check_input(&self, context: &InputContext) -> Result<Option<Violation>> {
        let session_id = context.session_id.as_deref().unwrap_or("default");
        let usage = self.get_usage(session_id);

        // Check if budget exceeded
        if usage >= self.max_tokens {
            let msg = format!("Token budget exceeded: {} / {}", usage, self.max_tokens);
            return Ok(Some(
                Violation::new(
                    "cost_limiter",
                    ViolationSeverity::High,
                    &msg,
                    ViolationAction::Block,
                )
            ));
        }

        // Warn if approaching limit
        if usage as f64 >= self.max_tokens as f64 * self.warn_threshold {
            let msg = format!("Approaching token limit: {} / {} ({:.0}%)", 
                usage, self.max_tokens, 
                (usage as f64 / self.max_tokens as f64) * 100.0);
            return Ok(Some(
                Violation::new(
                    "cost_limiter",
                    ViolationSeverity::Medium,
                    &msg,
                    ViolationAction::Warn,
                )
            ));
        }

        Ok(None)
    }

    async fn check_output(&self, context: &OutputContext) -> Result<Option<Violation>> {
        if let Some(tokens_used) = context.tokens_used {
            // This is called after generation, so we track usage
            // In a real system, we'd get session_id from context
            let session_id = "default"; // Would come from context in real implementation
            
            self.usage
                .entry(session_id.to_string())
                .and_modify(|v| *v += tokens_used)
                .or_insert(tokens_used);

            tracing::debug!("Tracked {} tokens for session {}", tokens_used, session_id);
        }

        Ok(None)
    }

    async fn check_tool_call(&self, _context: &ToolCallContext) -> Result<Option<Violation>> {
        // Cost limiter doesn't check tool calls directly
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_limiter_allows_within_budget() {
        let limiter = CostLimiter::new(1000);

        let context = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: Some("session1".to_string()),
        };

        let violation = limiter.check_input(&context).await.unwrap();
        assert!(violation.is_none());
    }

    #[tokio::test]
    async fn test_cost_limiter_warns_at_threshold() {
        let limiter = CostLimiter::new(100).warn_threshold(0.8);

        // Simulate 85 tokens used
        limiter.usage.insert("session1".to_string(), 85);

        let context = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: Some("session1".to_string()),
        };

        let violation = limiter.check_input(&context).await.unwrap();
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.action, ViolationAction::Warn);
    }

    #[tokio::test]
    async fn test_cost_limiter_blocks_when_exceeded() {
        let limiter = CostLimiter::new(100);

        // Simulate 100 tokens used
        limiter.usage.insert("session1".to_string(), 100);

        let context = InputContext {
            message: "test".to_string(),
            user_id: None,
            session_id: Some("session1".to_string()),
        };

        let violation = limiter.check_input(&context).await.unwrap();
        assert!(violation.is_some());
        assert!(violation.unwrap().should_block());
    }

    #[test]
    fn test_get_usage_and_remaining() {
        let limiter = CostLimiter::new(1000);
        limiter.usage.insert("session1".to_string(), 300);

        assert_eq!(limiter.get_usage("session1"), 300);
        assert_eq!(limiter.get_remaining("session1"), 700);
    }

    #[test]
    fn test_reset_usage() {
        let limiter = CostLimiter::new(1000);
        limiter.usage.insert("session1".to_string(), 500);

        limiter.reset("session1");
        assert_eq!(limiter.get_usage("session1"), 0);
    }
}

