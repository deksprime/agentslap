//! Mock approval strategy for testing

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{ApprovalRequest, ApprovalResponse, ApprovalStrategy, Result};

/// Mock approval strategy for automated testing
///
/// Allows configuring behavior for tests:
/// - Always approve
/// - Always deny
/// - Approve N times then deny
/// - Simulate timeout
/// - Return modified response
///
/// **Critical for end-to-end testing!**
pub struct MockApproval {
    /// Behavior mode
    mode: MockMode,
    /// Call counter
    call_count: Arc<AtomicUsize>,
}

/// Mock behavior modes
#[derive(Debug, Clone)]
pub enum MockMode {
    /// Always approve
    AlwaysApprove,
    
    /// Always deny with reason
    AlwaysDeny(String),
    
    /// Always timeout
    AlwaysTimeout,
    
    /// Approve first N calls, then deny
    ApproveNTimes(usize),
    
    /// Return modified response
    AlwaysModify(serde_json::Value),
}

impl MockApproval {
    /// Create mock that always approves
    pub fn always_approve() -> Self {
        Self {
            mode: MockMode::AlwaysApprove,
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create mock that always denies
    pub fn always_deny<S: Into<String>>(reason: S) -> Self {
        Self {
            mode: MockMode::AlwaysDeny(reason.into()),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create mock that always times out
    pub fn always_timeout() -> Self {
        Self {
            mode: MockMode::AlwaysTimeout,
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create mock that approves N times then denies
    pub fn approve_n_times(n: usize) -> Self {
        Self {
            mode: MockMode::ApproveNTimes(n),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create mock that modifies requests
    pub fn always_modify(modified: serde_json::Value) -> Self {
        Self {
            mode: MockMode::AlwaysModify(modified),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get number of times this mock was called
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    /// Reset call counter
    pub fn reset(&self) {
        self.call_count.store(0, Ordering::SeqCst);
    }
}

#[async_trait]
impl ApprovalStrategy for MockApproval {
    async fn request_approval(&self, _request: ApprovalRequest) -> Result<ApprovalResponse> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        match &self.mode {
            MockMode::AlwaysApprove => Ok(ApprovalResponse::Approved),
            
            MockMode::AlwaysDeny(reason) => Ok(ApprovalResponse::Denied {
                reason: Some(reason.clone()),
            }),
            
            MockMode::AlwaysTimeout => Ok(ApprovalResponse::Timeout),
            
            MockMode::ApproveNTimes(n) => {
                if count < *n {
                    Ok(ApprovalResponse::Approved)
                } else {
                    Ok(ApprovalResponse::Denied {
                        reason: Some(format!("Exceeded {} approvals", n)),
                    })
                }
            }
            
            MockMode::AlwaysModify(modified) => Ok(ApprovalResponse::Modified {
                modified_context: modified.clone(),
            }),
        }
    }

    fn name(&self) -> &str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RiskLevel;
    use std::time::Duration;

    #[tokio::test]
    async fn test_always_approve() {
        let strategy = MockApproval::always_approve();
        
        let req = ApprovalRequest::new("Test", RiskLevel::High)
            .with_timeout(Duration::from_secs(10));

        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_approved());
        assert_eq!(strategy.call_count(), 1);
    }

    #[tokio::test]
    async fn test_always_deny() {
        let strategy = MockApproval::always_deny("Test denial");
        
        let req = ApprovalRequest::new("Test", RiskLevel::High)
            .with_timeout(Duration::from_secs(10));

        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_denied());
    }

    #[tokio::test]
    async fn test_always_timeout() {
        let strategy = MockApproval::always_timeout();
        
        let req = ApprovalRequest::new("Test", RiskLevel::High)
            .with_timeout(Duration::from_secs(10));

        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_timeout());
    }

    #[tokio::test]
    async fn test_approve_n_times() {
        let strategy = MockApproval::approve_n_times(2);
        
        let req = ApprovalRequest::new("Test", RiskLevel::High)
            .with_timeout(Duration::from_secs(10));

        // First two should approve
        let r1 = strategy.request_approval(req.clone()).await.unwrap();
        assert!(r1.is_approved());
        
        let r2 = strategy.request_approval(req.clone()).await.unwrap();
        assert!(r2.is_approved());
        
        // Third should deny
        let r3 = strategy.request_approval(req).await.unwrap();
        assert!(r3.is_denied());
        
        assert_eq!(strategy.call_count(), 3);
    }

    #[tokio::test]
    async fn test_always_modify() {
        let modified = serde_json::json!({"modified": true});
        let strategy = MockApproval::always_modify(modified.clone());
        
        let req = ApprovalRequest::new("Test", RiskLevel::Medium)
            .with_timeout(Duration::from_secs(10));

        let response = strategy.request_approval(req).await.unwrap();
        match response {
            ApprovalResponse::Modified { modified_context } => {
                assert_eq!(modified_context, modified);
            }
            _ => panic!("Expected Modified response"),
        }
    }

    #[test]
    fn test_reset_counter() {
        let strategy = MockApproval::always_approve();
        strategy.call_count.store(5, Ordering::SeqCst);
        assert_eq!(strategy.call_count(), 5);
        
        strategy.reset();
        assert_eq!(strategy.call_count(), 0);
    }
}

