use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Result type for messaging operations.
pub type MessagingResult<T> = Result<T, MessagingError>;

/// Error types for messaging operations.
#[derive(Debug, Error)]
pub enum MessagingError {
    /// Error with configuration
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Error connecting to broker
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Error with authentication
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Error publishing message
    #[error("Publish error: {0}")]
    PublishError(String),

    /// Error subscribing to messages
    #[error("Subscribe error: {0}")]
    SubscribeError(String),

    /// Error acknowledging message
    #[error("Acknowledge error: {0}")]
    AcknowledgeError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Channel error
    #[error("Channel error: {0}")]
    ChannelError(String),

    /// Error with provider
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Broker is closed
    #[error("Broker closed: {0}")]
    BrokerClosed(String),

    /// Unexpected error
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl MessagingError {
    /// Creates a new configuration error.
    pub fn configuration_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::ConfigurationError(msg.into())
    }

    /// Creates a new connection error.
    pub fn connection_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::ConnectionError(msg.into())
    }

    /// Creates a new authentication error.
    pub fn authentication_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::AuthenticationError(msg.into())
    }

    /// Creates a new publish error.
    pub fn publish_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::PublishError(msg.into())
    }

    /// Creates a new subscribe error.
    pub fn subscribe_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::SubscribeError(msg.into())
    }

    /// Creates a new acknowledge error.
    pub fn acknowledge_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::AcknowledgeError(msg.into())
    }

    /// Creates a new timeout error.
    pub fn timeout_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::TimeoutError(msg.into())
    }

    /// Creates a new serialization error.
    pub fn serialization_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::SerializationError(msg.into())
    }

    /// Creates a new deserialization error.
    pub fn deserialization_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::DeserializationError(msg.into())
    }

    /// Creates a new channel error.
    pub fn channel_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::ChannelError(msg.into())
    }

    /// Creates a new provider error.
    pub fn provider_error<T: Into<String>>(msg: T) -> Self {
        MessagingError::ProviderError(msg.into())
    }

    /// Creates a new broker closed error.
    pub fn broker_closed<T: Into<String>>(msg: T) -> Self {
        MessagingError::BrokerClosed(msg.into())
    }

    /// Creates a new unexpected error.
    pub fn unexpected<T: Into<String>>(msg: T) -> Self {
        MessagingError::UnexpectedError(msg.into())
    }

    /// Convert a dynamic error into a MessagingError.
    pub fn from_error<E: std::error::Error>(err: E) -> Self {
        MessagingError::UnexpectedError(err.to_string())
    }

    /// Get the error code for this error.
    pub fn error_code(&self) -> &'static str {
        match self {
            MessagingError::ConnectionError(_) => "MESSAGING_CONNECTION_ERROR",
            MessagingError::AuthenticationError(_) => "MESSAGING_AUTH_ERROR",
            MessagingError::PublishError(_) => "MESSAGING_PUBLISH_ERROR",
            MessagingError::SubscribeError(_) => "MESSAGING_SUBSCRIBE_ERROR",
            MessagingError::AcknowledgeError(_) => "MESSAGING_ACK_ERROR",
            MessagingError::DeserializationError(_) => "MESSAGING_DESER_ERROR",
            MessagingError::SerializationError(_) => "MESSAGING_SER_ERROR",
            MessagingError::ChannelError(_) => "MESSAGING_CHANNEL_ERROR",
            MessagingError::ProviderError(_) => "MESSAGING_PROVIDER_ERROR",
            MessagingError::TimeoutError(_) => "MESSAGING_TIMEOUT_ERROR",
            MessagingError::BrokerClosed(_) => "MESSAGING_BROKER_CLOSED",
            MessagingError::UnexpectedError(_) => "MESSAGING_UNEXPECTED_ERROR",
            MessagingError::ConfigurationError(_) => "MESSAGING_CONFIG_ERROR",
        }
    }

    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            MessagingError::ConnectionError(_) => 503,
            MessagingError::AuthenticationError(_) => 401,
            MessagingError::PublishError(_) => 500,
            MessagingError::SubscribeError(_) => 500,
            MessagingError::AcknowledgeError(_) => 500,
            MessagingError::DeserializationError(_) => 400,
            MessagingError::SerializationError(_) => 500,
            MessagingError::ChannelError(_) => 500,
            MessagingError::ProviderError(_) => 500,
            MessagingError::TimeoutError(_) => 504,
            MessagingError::BrokerClosed(_) => 503,
            MessagingError::UnexpectedError(_) => 500,
            MessagingError::ConfigurationError(_) => 500,
        }
    }

    /// Determine if this error is transient (can be retried).
    pub fn is_transient(&self) -> bool {
        match self {
            MessagingError::ConnectionError(_) => true,
            MessagingError::TimeoutError(_) => true,
            MessagingError::BrokerClosed(_) => true,
            MessagingError::UnexpectedError(_) => true,
            MessagingError::ConfigurationError(_) => true,
            _ => false,
        }
    }
}

// Implement From<serde_json::Error> for MessagingError
impl From<serde_json::Error> for MessagingError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() {
            MessagingError::DeserializationError(format!("JSON data error: {}", err))
        } else if err.is_syntax() {
            MessagingError::DeserializationError(format!("JSON syntax error: {}", err))
        } else if err.is_eof() {
            MessagingError::DeserializationError(format!("JSON unexpected EOF: {}", err))
        } else {
            MessagingError::SerializationError(format!("JSON error: {}", err))
        }
    }
}

// Implement From<std::io::Error> for MessagingError
impl From<std::io::Error> for MessagingError {
    fn from(err: std::io::Error) -> Self {
        let kind = err.kind();
        match kind {
            std::io::ErrorKind::TimedOut => {
                MessagingError::TimeoutError(format!("IO timeout: {}", err))
            }
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::NotConnected => {
                MessagingError::ConnectionError(format!("IO connection error: {}", err))
            }
            _ => MessagingError::ProviderError(format!("IO error: {}", err)),
        }
    }
}
