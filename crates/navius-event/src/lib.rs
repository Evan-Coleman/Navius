//! Navius Event System
//!
//! This crate provides event handling and notification interfaces for the Navius framework.
//! It includes a flexible event broker API, in-memory implementation, and utilities
//! for publishing and subscribing to events.
//!
//! The event system allows components to communicate asynchronously through a
//! publish-subscribe pattern, with support for typed events, filtering, and
//! prioritization.

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

// Import EventBroker trait for use in methods
use broker::EventBroker;

// Re-export primary types from broker
pub use broker::{BrokerInfo, EventBrokerConfig, EventBrokerFactory, EventStream, TopicInfo};

// Re-export primary types from error
pub use error::{DeliveryStatus, EventError, EventPriority, EventResult};

// Re-export primary types from event
pub use event::{Event, EventEnvelope, EventFilterConfig, SubscriptionInfo, SubscriptionOptions};

// Re-export memory broker
pub use memory::{InMemoryEventBroker, InMemoryEventBrokerFactory};

use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

/// A wrapper type for EventBroker that doesn't have object safety issues
pub struct EventBrokerHandle {
    /// The underlying broker implementation
    inner: Arc<InMemoryEventBroker>,
}

impl EventBrokerHandle {
    /// Create a new event broker handle from an InMemoryEventBroker
    pub fn new(broker: InMemoryEventBroker) -> Self {
        Self {
            inner: Arc::new(broker),
        }
    }

    /// Create a new in-memory event broker with default configuration
    pub async fn create_memory_broker() -> EventResult<EventBrokerHandle> {
        let factory = InMemoryEventBrokerFactory::new();
        let broker = factory
            .create_memory_broker(EventBrokerConfig::default())
            .await?;
        Ok(EventBrokerHandle { inner: broker })
    }

    /// Create a new in-memory event broker with custom configuration
    pub async fn create_configured_memory_broker(
        config: EventBrokerConfig,
    ) -> EventResult<EventBrokerHandle> {
        let factory = InMemoryEventBrokerFactory::new();
        let broker = factory.create_memory_broker(config).await?;
        Ok(EventBrokerHandle { inner: broker })
    }

    /// Get the underlying broker implementation
    pub fn inner(&self) -> Arc<InMemoryEventBroker> {
        self.inner.clone()
    }

    /// Publish a typed event
    pub async fn publish<T>(&self, event: Event<T>) -> EventResult<DeliveryStatus>
    where
        T: Serialize + Send + Sync + 'static,
    {
        EventBroker::publish(&*self.inner, event).await
    }

    /// Subscribe to events on a topic with options
    pub async fn subscribe<T>(
        &self,
        topic: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(String, EventStream<T>)>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        EventBroker::subscribe(&*self.inner, topic, options).await
    }
}

/// Create a new in-memory event broker with default configuration
pub async fn create_memory_broker() -> EventResult<EventBrokerHandle> {
    EventBrokerHandle::create_memory_broker().await
}

/// Create a new in-memory event broker with custom configuration
pub async fn create_configured_memory_broker(
    config: EventBrokerConfig,
) -> EventResult<EventBrokerHandle> {
    EventBrokerHandle::create_configured_memory_broker(config).await
}
