//! Approval strategy trait

use async_trait::async_trait;

use crate::{ApprovalRequest, ApprovalResponse, Result};

/// Trait for approval strategies
///
/// Implementations define how approval requests are handled.
/// This could be automatic, console-based, webhook-based, etc.
///
/// All strategies must be:
/// - Send + Sync (thread-safe)
/// - Testable (including in automated tests)
/// - Async (non-blocking)
#[async_trait]
pub trait ApprovalStrategy: Send + Sync {
    /// Request approval for an action
    ///
    /// # Arguments
    /// * `request` - The approval request
    ///
    /// # Returns
    /// The approval response (approved, denied, modified, timeout)
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalResponse>;

    /// Get the strategy name (for logging/debugging)
    fn name(&self) -> &str;

    /// Check if this strategy supports async approval
    ///
    /// If true, the strategy may return immediately and approval happens later.
    /// If false, approval is synchronous (blocks until decision).
    fn is_async(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RiskLevel;
    use std::time::Duration;

    struct TestStrategy;

    #[async_trait]
    impl ApprovalStrategy for TestStrategy {
        async fn request_approval(&self, _request: ApprovalRequest) -> Result<ApprovalResponse> {
            Ok(ApprovalResponse::Approved)
        }

        fn name(&self) -> &str {
            "test"
        }
    }

    #[tokio::test]
    async fn test_strategy_trait() {
        let strategy = TestStrategy;
        assert_eq!(strategy.name(), "test");
        assert!(!strategy.is_async());

        let req = ApprovalRequest::new("Test action", RiskLevel::Low)
            .with_timeout(Duration::from_secs(10));

        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_approved());
    }
}

