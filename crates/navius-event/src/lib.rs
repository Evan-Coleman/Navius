/*!
Navius Event - Event handling and notification system for the Navius framework

This crate provides a comprehensive event system for the Navius framework, enabling
publish-subscribe patterns, event dispatching, and integration with the plugin system.

## Features

- **Event Dispatching**: Type-safe event publishing and subscribing
- **Async Event Handling**: Process events asynchronously with Tokio
- **Topic-Based Routing**: Subscribe to specific event topics
- **Plugin Integration**: Register event handlers through plugins
- **Filtering**: Filter events based on custom predicates
- **Error Handling**: Robust error handling for event processing

## Main Components

- `Event`: Base trait for all events
- `EventBus`: Central hub for publishing and subscribing to events
- `EventHandler`: Trait for implementing event handlers
- `EventPlugin`: Plugin for registering event handlers
*/

pub mod bus;
pub mod error;
pub mod event;
pub mod handler;
pub mod plugin;
#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use bus::EventBus;
pub use error::{EventError, EventResult};
pub use event::{Event, EventEnvelope, EventMetadata, EventPriority};
pub use handler::{EventHandler, EventSubscription};
pub use plugin::EventPlugin;
