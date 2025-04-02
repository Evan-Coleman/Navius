//! Mocking utilities for Navius applications.

// Re-export all mock modules
pub mod auth;
pub mod cache;
pub mod config;
pub mod database;
pub mod events;
pub mod filesystem;
pub mod http;
pub mod logger;
pub mod messaging;
pub mod metrics;

// Re-export essential types for ease of use
pub use events::{EventEnvelope, EventPriority, MockEvent, MockEventBroker};
pub use logger::{LogEntry, LogLevel, MockLogger};
pub use messaging::{MockMessageBroker, MockMessagingError, MockQueue};
pub use metrics::MockMetrics;

// Import messaging types from navius_messaging
pub use navius_messaging::{
    Message,
    broker::MessageBroker,
    error::{MessagingError, MessagingResult},
    publisher::PublishOptions,
};

// Import event types from navius_event
use navius_event::broker::EventBroker;

// Import auth types
use auth::AuthProvider as Auth;

// For registry setup utilities
use metrics::{MetricsCollector, MetricsExporter};
use std::sync::Arc;

use crate::error::TestResult;
use crate::mock::MockRegistry;

/// Register all mock components with the registry.
pub fn register_all_mocks(registry: &MockRegistry) -> TestResult<()> {
    // Register mocks from individual modules

    let event_broker = Arc::new(MockEventBroker::new());
    // Use concrete type since EventBroker trait is not object-safe
    registry.register_mock::<MockEventBroker>(event_broker);

    let message_broker = Arc::new(MockMessageBroker::new());
    registry.register::<dyn navius_messaging::broker::MessageBroker, _>(message_broker.clone())?;

    let metrics_collector = Arc::new(MockMetrics::new());
    registry.register_mock::<dyn MetricsCollector>(metrics_collector.clone());
    registry.register_mock::<dyn MetricsExporter>(metrics_collector);

    Ok(())
}

/// Register only message broker mocks.
pub fn register_messaging_mocks(registry: &MockRegistry) -> TestResult<()> {
    let message_broker = Arc::new(MockMessageBroker::new());
    registry.register::<dyn navius_messaging::broker::MessageBroker, _>(message_broker)?;
    Ok(())
}

/// Register core mocks for standard tests.
pub fn register_core_mocks(registry: &MockRegistry) -> TestResult<()> {
    let event_broker = Arc::new(MockEventBroker::new());
    // Use concrete type since EventBroker trait is not object-safe
    registry.register_mock::<MockEventBroker>(event_broker);

    let message_broker = Arc::new(MockMessageBroker::new());
    registry.register::<dyn navius_messaging::broker::MessageBroker, _>(message_broker)?;

    let metrics_collector = Arc::new(MockMetrics::new());
    registry.register_mock::<dyn MetricsCollector>(metrics_collector);

    Ok(())
}

pub mod registry_setup {
    use crate::mock::MockRegistry;
    use crate::mocks::metrics::{MetricsCollector, MetricsExporter, MockMetrics};
    use std::sync::Arc;

    /// Setup basic mocks in a registry
    pub fn setup_basic_mocks(registry: &MockRegistry) {
        let metrics = Arc::new(MockMetrics::new());
        registry.register_mock::<dyn MetricsCollector>(metrics.clone());
        registry.register_mock::<dyn MetricsExporter>(metrics);
    }

    /// Setup all mocks in a registry
    pub fn setup_all_mocks(registry: &MockRegistry) {
        // Setup metrics
        let metrics = Arc::new(MockMetrics::new());
        registry.register_mock::<dyn MetricsCollector>(metrics);
    }
}

// Common test suite registration
pub fn register_mock_suite(registry: &MockRegistry) -> TestResult<()> {
    // Create auth provider
    let auth_provider = Arc::new(auth::MockAuthProvider::default());
    registry.register::<dyn Auth, _>(auth_provider)?;

    // Create event broker
    let event_broker = Arc::new(MockEventBroker::new());

    // Not using EventBroker trait here due to object safety issues
    registry.register_mock::<MockEventBroker>(event_broker.clone());

    // Create metrics collector
    let metrics_collector = Arc::new(MockMetrics::new());
    registry.register_mock::<dyn MetricsCollector>(metrics_collector.clone());
    registry.register_mock::<dyn MetricsExporter>(metrics_collector);

    // Create message broker
    let message_broker = Arc::new(MockMessageBroker::new());
    registry.register::<dyn MessageBroker, _>(message_broker)?;

    Ok(())
}

// Extended test suite registration
pub fn register_extended_mock_suite(registry: &MockRegistry) -> TestResult<()> {
    // Create auth provider
    let auth_provider = Arc::new(auth::MockAuthProvider::default());
    registry.register::<dyn Auth, _>(auth_provider)?;

    // Create event broker
    let event_broker = Arc::new(MockEventBroker::new());

    // Not using EventBroker trait here due to object safety issues
    registry.register_mock::<MockEventBroker>(event_broker.clone());

    // Create metrics collector
    let metrics_collector = Arc::new(MockMetrics::new());
    registry.register_mock::<dyn MetricsCollector>(metrics_collector);

    // Add more mock registrations as needed

    Ok(())
}
