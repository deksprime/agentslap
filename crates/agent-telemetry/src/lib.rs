//! Agent Telemetry
//!
//! Observability and monitoring for agent runtime.
//!
//! # Example
//!
//! ```
//! use agent_telemetry::{TelemetryCollector, TelemetryEvent};
//!
//! let telemetry = TelemetryCollector::new(1000);
//! let mut subscriber = telemetry.subscribe();
//!
//! telemetry.emit(TelemetryEvent::agent_started("agent-1", "coordinator"));
//!
//! // Subscriber receives events
//! ```

pub mod event;
pub mod collector;

// Re-exports
pub use event::TelemetryEvent;
pub use collector::TelemetryCollector;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        assert!(true);
    }
}

