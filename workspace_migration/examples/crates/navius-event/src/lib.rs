// Navius Event System
//
// This crate provides event handling and notification interfaces for the Navius framework.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Event broker interfaces
pub mod broker;

/// Event system error types
pub mod error;

/// Core event structures
pub mod event;

/// In-memory event broker implementation
pub mod memory;

// Re-export primary types from broker
pub use broker::{
    BrokerInfo, EventBroker, EventBrokerConfig, EventBrokerFactory, EventStream, TopicInfo,
};

// Re-export primary types from error
pub use error::{DeliveryStatus, EventError, EventPriority, EventResult};

// Re-export primary types from event
pub use event::{Event, EventEnvelope, EventFilterConfig, SubscriptionInfo, SubscriptionOptions};

// Re-export memory broker
pub use memory::{InMemoryEventBroker, InMemoryEventBrokerFactory};

/// Create a new in-memory event broker with default configuration
pub async fn create_memory_broker() -> EventResult<std::sync::Arc<dyn EventBroker>> {
    let factory = InMemoryEventBrokerFactory::new();
    factory.create_broker(EventBrokerConfig::default()).await
}

/// Create a new in-memory event broker with custom configuration
pub async fn create_configured_memory_broker(
    config: EventBrokerConfig,
) -> EventResult<std::sync::Arc<dyn EventBroker>> {
    let factory = InMemoryEventBrokerFactory::new();
    factory.create_broker(config).await
}
