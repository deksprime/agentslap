//! Rate limiting guardrail using token bucket algorithm

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{
    guardrail::{Guardrail, InputContext, OutputContext, ToolCallContext},
    violation::{ViolationAction, ViolationSeverity},
    Result, Violation,
};

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

/// Rate limiter guardrail
///
/// Limits the number of requests per time window using token bucket algorithm.
pub struct RateLimiter {
    /// Maximum tokens (requests) per window
    max_tokens: f64,
    /// Time window for refill
    window: Duration,
    /// Per-user token buckets
    buckets: Arc<DashMap<String, TokenBucket>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed
    /// * `window` - Time window for the limit
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_tokens: max_requests as f64,
            window,
            buckets: Arc::new(DashMap::new()),
        }
    }

    /// Check if request is allowed for a user
    fn check_rate(&self, user_id: &str) -> bool {
        let now = Instant::now();

        let mut bucket_ref = self.buckets.entry(user_id.to_string()).or_insert(TokenBucket {
            tokens: self.max_tokens,
            last_refill: now,
        });

        let bucket = bucket_ref.value_mut();

        // Refill tokens based on time passed
        let elapsed = now.duration_since(bucket.last_refill);
        let refill = (elapsed.as_secs_f64() / self.window.as_secs_f64()) * self.max_tokens;
        bucket.tokens = (bucket.tokens + refill).min(self.max_tokens);
        bucket.last_refill = now;

        // Check if we have tokens
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            max_tokens: self.max_tokens,
            window: self.window,
            buckets: Arc::clone(&self.buckets),
        }
    }
}

#[async_trait]
impl Guardrail for RateLimiter {
    fn name(&self) -> &str {
        "rate_limiter"
    }

    async fn check_input(&self, context: &InputContext) -> Result<Option<Violation>> {
        let user_id = context.user_id.as_deref().unwrap_or("default");

        if !self.check_rate(user_id) {
            return Ok(Some(
                Violation::new(
                    "rate_limiter",
                    ViolationSeverity::Medium,
                    format!("Rate limit exceeded for user: {}", user_id).as_str(),
                    ViolationAction::Block,
                )
            ));
        }

        Ok(None)
    }

    async fn check_output(&self, _context: &OutputContext) -> Result<Option<Violation>> {
        // Rate limiter only checks inputs
        Ok(None)
    }

    async fn check_tool_call(&self, _context: &ToolCallContext) -> Result<Option<Violation>> {
        // Rate limiter only checks inputs
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(10));

        let context = InputContext {
            message: "test".to_string(),
            user_id: Some("user1".to_string()),
            session_id: None,
        };

        // First two requests should pass
        let v1 = limiter.check_input(&context).await.unwrap();
        assert!(v1.is_none());

        let v2 = limiter.check_input(&context).await.unwrap();
        assert!(v2.is_none());
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_when_exceeded() {
        let limiter = RateLimiter::new(1, Duration::from_secs(10));

        let context = InputContext {
            message: "test".to_string(),
            user_id: Some("user1".to_string()),
            session_id: None,
        };

        // First request passes
        let v1 = limiter.check_input(&context).await.unwrap();
        assert!(v1.is_none());

        // Second request blocked
        let v2 = limiter.check_input(&context).await.unwrap();
        assert!(v2.is_some());
        assert!(v2.unwrap().should_block());
    }

    #[tokio::test]
    async fn test_rate_limiter_refills_over_time() {
        let limiter = RateLimiter::new(1, Duration::from_millis(100));

        let context = InputContext {
            message: "test".to_string(),
            user_id: Some("user1".to_string()),
            session_id: None,
        };

        // Use up token
        limiter.check_input(&context).await.unwrap();

        // Wait for refill
        sleep(Duration::from_millis(150)).await;

        // Should have refilled
        let v = limiter.check_input(&context).await.unwrap();
        assert!(v.is_none());
    }

    #[tokio::test]
    async fn test_different_users_independent() {
        let limiter = RateLimiter::new(1, Duration::from_secs(10));

        let ctx1 = InputContext {
            message: "test".to_string(),
            user_id: Some("user1".to_string()),
            session_id: None,
        };

        let ctx2 = InputContext {
            message: "test".to_string(),
            user_id: Some("user2".to_string()),
            session_id: None,
        };

        // Both users should get their own quota
        assert!(limiter.check_input(&ctx1).await.unwrap().is_none());
        assert!(limiter.check_input(&ctx2).await.unwrap().is_none());
    }
}

