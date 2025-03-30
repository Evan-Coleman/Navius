//! Error types for the event system

use std::fmt::{Debug, Display};
use thiserror::Error;

/// Errors that can occur during event operations.
#[derive(Error, Debug)]
pub enum EventError {
    /// Error when publishing an event.
    #[error("Failed to publish event: {message}")]
    PublishError {
        /// Error message.
        message: String,
    },

    /// Error when subscribing to events.
    #[error("Failed to subscribe to event: {message}")]
    SubscriptionError {
        /// Error message.
        message: String,
    },

    /// Error when handling an event.
    #[error("Error handling event: {message}")]
    HandlingError {
        /// Error message.
        message: String,
    },

    /// Error when serializing an event.
    #[error("Failed to serialize event: {message}")]
    SerializationError {
        /// Error message.
        message: String,
    },

    /// Error when deserializing an event.
    #[error("Failed to deserialize event: {message}")]
    DeserializationError {
        /// Error message.
        message: String,
    },

    /// Timeout waiting for an event.
    #[error("Timeout waiting for event")]
    TimeoutError,

    /// The event bus has been shut down.
    #[error("Event bus has been shut down")]
    BusShutdown,

    /// Generic event error.
    #[error("{message}")]
    GenericError {
        /// Error message.
        message: String,
    },

    /// Error when trying to publish an event to a closed channel.
    #[error("Event channel closed")]
    ChannelClosed,

    /// Error during bus shutdown.
    #[error("Bus shutdown error: {message}")]
    BusShutdownError {
        /// Error message.
        message: String,
    },

    /// Error when a handler could not be found.
    #[error("Event handler not found")]
    HandlerNotFound,
}

/// Result type for event operations.
pub type EventResult<T> = Result<T, EventError>;

/// Trait for converting errors to EventError.
pub trait IntoEventError<T> {
    /// Converts this error into an EventError.
    fn into_event_error<D: Display>(self, msg: D) -> EventResult<T>;
}

impl<T, E: Debug> IntoEventError<T> for Result<T, E> {
    fn into_event_error<D: Display>(self, msg: D) -> EventResult<T> {
        self.map_err(|e| EventError::GenericError {
            message: format!("{}: {:?}", msg, e),
        })
    }
}

// Implement conversion to plugin error
impl From<EventError> for navius_plugin::error::PluginError {
    fn from(error: EventError) -> Self {
        navius_plugin::error::PluginError::GenericError {
            message: error.to_string(),
        }
    }
}

// Helper trait to convert event errors to plugin errors
pub trait IntoPluginError {
    fn into_plugin_error<D: Display>(self, msg: D) -> navius_plugin::error::PluginError;
}

impl IntoPluginError for EventError {
    fn into_plugin_error<D: Display>(self, msg: D) -> navius_plugin::error::PluginError {
        navius_plugin::error::PluginError::GenericError {
            message: format!("{}: {}", msg, self),
        }
    }
}

impl<T> IntoPluginError for Result<T, EventError> {
    fn into_plugin_error<D: Display>(self, msg: D) -> navius_plugin::error::PluginError {
        match self {
            Ok(_) => panic!("Cannot convert Ok result to an error"),
            Err(e) => e.into_plugin_error(msg),
        }
    }
}
