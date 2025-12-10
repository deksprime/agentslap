//! Auto-approve strategy for trusted environments

use async_trait::async_trait;

use crate::{ApprovalRequest, ApprovalResponse, ApprovalStrategy, Result};

/// Auto-approve strategy
///
/// Automatically approves all requests.
/// Use only in trusted environments or for testing.
pub struct AutoApprove;

impl AutoApprove {
    /// Create a new auto-approve strategy
    pub fn new() -> Self {
        Self
    }
}

impl Default for AutoApprove {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApprovalStrategy for AutoApprove {
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalResponse> {
        tracing::debug!("Auto-approved: {} (risk: {:?})", request.action, request.risk_level);
        Ok(ApprovalResponse::Approved)
    }

    fn name(&self) -> &str {
        "auto_approve"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RiskLevel;
    use std::time::Duration;

    #[tokio::test]
    async fn test_auto_approve() {
        let strategy = AutoApprove::new();
        
        let req = ApprovalRequest::new("Dangerous action", RiskLevel::Critical)
            .with_timeout(Duration::from_secs(10));

        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_approved());
    }
}

