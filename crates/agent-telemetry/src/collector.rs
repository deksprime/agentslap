//! Telemetry collector

use tokio::sync::broadcast;
use std::sync::Arc;

use crate::TelemetryEvent;

/// Telemetry collector for agent events
///
/// Collects and broadcasts events to subscribers.
/// Used for monitoring, logging, and debugging.
#[derive(Clone)]
pub struct TelemetryCollector {
    sender: Arc<broadcast::Sender<TelemetryEvent>>,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    ///
    /// # Arguments
    /// * `capacity` - Channel capacity (default: 1000)
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Create with default capacity
    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Emit a telemetry event
    ///
    /// Events are broadcast to all subscribers.
    /// If no subscribers, events are dropped.
    pub fn emit(&self, event: TelemetryEvent) {
        tracing::trace!("Telemetry event: {} from {}", 
            serde_json::to_string(&event).unwrap_or_default(),
            event.agent_id()
        );
        
        // Send to all subscribers (ignore if no receivers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to telemetry events
    ///
    /// Returns a receiver that will receive all future events.
    pub fn subscribe(&self) -> broadcast::Receiver<TelemetryEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collector_creation() {
        let collector = TelemetryCollector::new(100);
        assert_eq!(collector.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_emit_and_receive() {
        let collector = TelemetryCollector::new(100);
        let mut sub = collector.subscribe();

        let event = TelemetryEvent::agent_started("agent-1", "test");
        collector.emit(event.clone());

        let received = sub.recv().await.unwrap();
        assert_eq!(received.agent_id(), "agent-1");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let collector = TelemetryCollector::new(100);
        let mut sub1 = collector.subscribe();
        let mut sub2 = collector.subscribe();

        assert_eq!(collector.subscriber_count(), 2);

        let event = TelemetryEvent::llm_request("agent-1", "gpt-4", 5);
        collector.emit(event);

        // Both receive
        let recv1 = sub1.recv().await.unwrap();
        let recv2 = sub2.recv().await.unwrap();

        assert_eq!(recv1.agent_id(), "agent-1");
        assert_eq!(recv2.agent_id(), "agent-1");
    }

    #[tokio::test]
    async fn test_no_subscribers_no_error() {
        let collector = TelemetryCollector::new(100);
        
        // Emit without subscribers (should not panic)
        collector.emit(TelemetryEvent::agent_started("a", "r"));
        collector.emit(TelemetryEvent::llm_request("a", "m", 1));
        
        // No panic = success
    }

    #[test]
    fn test_subscriber_count() {
        let collector = TelemetryCollector::new(100);
        assert_eq!(collector.subscriber_count(), 0);

        let _sub1 = collector.subscribe();
        assert_eq!(collector.subscriber_count(), 1);

        let _sub2 = collector.subscribe();
        assert_eq!(collector.subscriber_count(), 2);
    }
}

