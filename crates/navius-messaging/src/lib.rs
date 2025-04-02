//! Navius Messaging Framework
//!
//! This crate provides abstractions for working with message brokers like RabbitMQ, Kafka, and others.
//! It defines common interfaces for message publishing, consuming, and routing across different
//! messaging providers.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Messaging errors and result types
pub mod error;

/// Core messaging traits and interfaces
pub mod broker;

/// Message definition and routing
pub mod message;

/// Publishers for sending messages
pub mod publisher;

/// Consumers for receiving messages
pub mod consumer;

/// Serialization support for messages
pub mod serialization;

/// Exchange and queue abstractions
pub mod topology;

/// Telemetry and metrics
pub mod metrics;

/// Configuration for brokers and connections
pub mod config;

/// Utility functions and helpers
pub mod util;

// Re-export important types
pub use broker::{MessageBroker, TypedMessageBroker};
pub use consumer::ConsumerConfig;
pub use error::{
    AcknowledgmentMode, ConnectionStatus, DeliveryMode, MessagingError, MessagingResult,
};
pub use message::{Message, MessageHeaders};
pub use publisher::{PublishOptions, PublishResult};
