//! Approval audit trail

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{ApprovalRequest, ApprovalResponse};

/// Audit record for an approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// The original request
    pub request: ApprovalRequest,
    
    /// The response
    pub response: ApprovalResponse,
    
    /// When the response was given
    pub responded_at: DateTime<Utc>,
    
    /// Duration from request to response
    pub duration_ms: u64,
}

/// Approval audit trail
///
/// Tracks all approval requests and responses for compliance and debugging.
#[derive(Clone)]
pub struct ApprovalAudit {
    records: Arc<DashMap<String, AuditRecord>>,
}

impl ApprovalAudit {
    /// Create a new approval audit
    pub fn new() -> Self {
        Self {
            records: Arc::new(DashMap::new()),
        }
    }

    /// Record an approval decision
    pub fn record(
        &self,
        request: ApprovalRequest,
        response: ApprovalResponse,
        duration_ms: u64,
    ) {
        let record = AuditRecord {
            request: request.clone(),
            response,
            responded_at: Utc::now(),
            duration_ms,
        };

        self.records.insert(request.id.clone(), record);
        tracing::info!("Recorded approval decision for request: {}", request.id);
    }

    /// Get an audit record by request ID
    pub fn get(&self, request_id: &str) -> Option<AuditRecord> {
        self.records.get(request_id).map(|r| r.clone())
    }

    /// Get all audit records
    pub fn get_all(&self) -> Vec<AuditRecord> {
        self.records.iter().map(|r| r.clone()).collect()
    }

    /// Get count of records
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// Clear all records
    pub fn clear(&self) {
        self.records.clear();
    }
}

impl Default for ApprovalAudit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RiskLevel;
    use std::time::Duration;

    #[test]
    fn test_audit_record() {
        let audit = ApprovalAudit::new();
        
        let request = ApprovalRequest::new("Test", RiskLevel::Low)
            .with_timeout(Duration::from_secs(10));
        let request_id = request.id.clone();
        
        audit.record(request, ApprovalResponse::Approved, 100);
        
        assert_eq!(audit.count(), 1);
        
        let record = audit.get(&request_id).unwrap();
        assert_eq!(record.duration_ms, 100);
        assert!(record.response.is_approved());
    }

    #[test]
    fn test_get_all() {
        let audit = ApprovalAudit::new();
        
        for i in 0..3 {
            let req = ApprovalRequest::new(format!("Action {}", i), RiskLevel::Medium)
                .with_timeout(Duration::from_secs(10));
            audit.record(req, ApprovalResponse::Approved, 100);
        }
        
        assert_eq!(audit.get_all().len(), 3);
    }

    #[test]
    fn test_clear() {
        let audit = ApprovalAudit::new();
        
        let req = ApprovalRequest::new("Test", RiskLevel::Low)
            .with_timeout(Duration::from_secs(10));
        audit.record(req, ApprovalResponse::Approved, 100);
        
        assert_eq!(audit.count(), 1);
        audit.clear();
        assert_eq!(audit.count(), 0);
    }
}

