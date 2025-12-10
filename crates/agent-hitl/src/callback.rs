//! Callback-based approval strategy for async/remote approval

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::timeout;

use crate::{ApprovalRequest, ApprovalResponse, ApprovalStrategy, Result, HitlError};

/// Callback approval strategy
///
/// For async approval workflows:
/// - Web apps (approval via HTTP endpoint)
/// - Slack/Discord bots (approval via message button)
/// - Queue systems (approval via message queue)
///
/// Works by:
/// 1. Store pending request
/// 2. Return request ID to caller
/// 3. Caller notifies human (webhook, message, etc.)
/// 4. Human responds via callback
/// 5. Response delivered to waiting agent
pub struct CallbackApproval {
    /// Pending approval requests
    pending: Arc<DashMap<String, oneshot::Sender<ApprovalResponse>>>,
}

impl CallbackApproval {
    /// Create a new callback approval strategy
    pub fn new() -> Self {
        Self {
            pending: Arc::new(DashMap::new()),
        }
    }

    /// Submit an approval response (called by external system)
    ///
    /// This is called when a human makes a decision via your UI/webhook/bot.
    ///
    /// # Arguments
    /// * `request_id` - The approval request ID
    /// * `response` - The human's decision
    ///
    /// # Returns
    /// Ok if the request was found and response delivered
    pub fn submit_response(&self, request_id: &str, response: ApprovalResponse) -> Result<()> {
        if let Some((_, sender)) = self.pending.remove(request_id) {
            let _ = sender.send(response);  // Ignore send error (receiver might be gone)
            Ok(())
        } else {
            Err(HitlError::strategy(format!(
                "No pending request with ID: {}",
                request_id
            )))
        }
    }

    /// Get count of pending requests
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Cancel a pending request
    pub fn cancel(&self, request_id: &str) -> Result<()> {
        if let Some((_, sender)) = self.pending.remove(request_id) {
            let _ = sender.send(ApprovalResponse::Denied {
                reason: Some("Request cancelled".to_string()),
            });
            Ok(())
        } else {
            Err(HitlError::strategy("Request not found"))
        }
    }
}

impl Default for CallbackApproval {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CallbackApproval {
    fn clone(&self) -> Self {
        Self {
            pending: Arc::clone(&self.pending),
        }
    }
}

#[async_trait]
impl ApprovalStrategy for CallbackApproval {
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalResponse> {
        let request_id = request.id.clone();
        let timeout_duration = request.timeout;

        // Create channel for response
        let (tx, rx) = oneshot::channel();
        
        // Store pending request
        self.pending.insert(request_id.clone(), tx);

        tracing::info!(
            "Waiting for approval: {} (request_id: {}, timeout: {:?})",
            request.action,
            request_id,
            timeout_duration
        );

        // Wait for response with timeout
        match timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => {
                tracing::info!("Received approval response for: {}", request_id);
                Ok(response)
            }
            Ok(Err(_)) => {
                // Channel closed without response
                self.pending.remove(&request_id);
                Ok(ApprovalResponse::Denied {
                    reason: Some("Approval channel closed".to_string()),
                })
            }
            Err(_) => {
                // Timeout
                self.pending.remove(&request_id);
                tracing::warn!("Approval timeout for: {}", request_id);
                Ok(ApprovalResponse::Timeout)
            }
        }
    }

    fn name(&self) -> &str {
        "callback"
    }

    fn is_async(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RiskLevel;
    use std::time::Duration;

    #[tokio::test]
    async fn test_callback_approval() {
        let strategy = CallbackApproval::new();
        let strategy_clone = strategy.clone();

        let req = ApprovalRequest::new("Test action", RiskLevel::Medium)
            .with_timeout(Duration::from_secs(5));
        
        let request_id = req.id.clone();

        // Spawn task to approve after delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            strategy_clone.submit_response(&request_id, ApprovalResponse::Approved).unwrap();
        });

        // Wait for approval
        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_approved());
    }

    #[tokio::test]
    async fn test_callback_timeout() {
        let strategy = CallbackApproval::new();

        let req = ApprovalRequest::new("Test action", RiskLevel::High)
            .with_timeout(Duration::from_millis(100));  // Short timeout

        // Don't submit response - should timeout
        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_timeout());
    }

    #[tokio::test]
    async fn test_callback_deny() {
        let strategy = CallbackApproval::new();
        let strategy_clone = strategy.clone();

        let req = ApprovalRequest::new("Test action", RiskLevel::Medium)
            .with_timeout(Duration::from_secs(5));
        
        let request_id = req.id.clone();

        // Spawn task to deny
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            strategy_clone.submit_response(&request_id, ApprovalResponse::Denied {
                reason: Some("Test denial".to_string()),
            }).unwrap();
        });

        let response = strategy.request_approval(req).await.unwrap();
        assert!(response.is_denied());
    }

    #[test]
    fn test_pending_count() {
        let strategy = CallbackApproval::new();
        assert_eq!(strategy.pending_count(), 0);
        
        let (tx, _rx) = oneshot::channel();
        strategy.pending.insert("test".to_string(), tx);
        assert_eq!(strategy.pending_count(), 1);
    }

    #[test]
    fn test_cancel_request() {
        let strategy = CallbackApproval::new();
        let (tx, _rx) = oneshot::channel();
        strategy.pending.insert("test".to_string(), tx);
        
        strategy.cancel("test").unwrap();
        assert_eq!(strategy.pending_count(), 0);
    }
}

