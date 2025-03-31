use std::fmt;
use std::io;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

/// Result type for messaging operations
pub type MessagingResult<T> = Result<T, MessagingError>;

/// Errors that can occur in messaging operations
#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    /// Error connecting to the broker
    #[error("Failed to connect to message broker: {0}")]
    ConnectionError(String),

    /// Error publishing a message
    #[error("Failed to publish message: {0}")]
    PublishError(String),

    /// Error consuming a message
    #[error("Failed to consume message: {0}")]
    ConsumeError(String),

    /// Error acknowledging a message
    #[error("Failed to acknowledge message: {0}")]
    AcknowledgmentError(String),

    /// Error declaring exchange
    #[error("Failed to declare exchange '{0}': {1}")]
    ExchangeError(String, String),

    /// Error declaring queue
    #[error("Failed to declare queue '{0}': {1}")]
    QueueError(String, String),

    /// Error binding queue to exchange
    #[error("Failed to bind queue '{0}' to exchange '{1}': {2}")]
    BindingError(String, String, String),

    /// Broker not found
    #[error("Message broker '{0}' not found")]
    BrokerNotFound(String),

    /// Channel closed
    #[error("Channel closed unexpectedly: {0}")]
    ChannelClosed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Timeout error
    #[error("Operation timed out after {0} ms")]
    TimeoutError(u64),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Topology recovery error
    #[error("Failed to recover topology: {0}")]
    TopologyRecoveryError(String),

    /// Message rejection
    #[error("Message rejected: {0}")]
    MessageRejectedError(String),

    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessageError(String),

    /// Consumer canceled
    #[error("Consumer was canceled by the broker: {0}")]
    ConsumerCanceledError(String),

    /// IO Error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Join error from task
    #[error("Task join error: {0}")]
    JoinError(#[from] JoinError),

    /// Channel send error
    #[error("Channel send error: {0}")]
    ChannelSendError(String),

    /// Event system error
    #[error("Event system error: {0}")]
    EventError(#[from] navius_event::EventError),

    /// Plugin system error
    #[error("Plugin system error: {0}")]
    PluginError(String),

    /// Other error
    #[error("Messaging error: {0}")]
    Other(String),
}

impl<T> From<SendError<T>> for MessagingError
where
    T: fmt::Debug,
{
    fn from(err: SendError<T>) -> Self {
        MessagingError::ChannelSendError(format!("{:?}", err))
    }
}

/// Delivery mode for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryMode {
    /// Non-persistent message
    NonPersistent = 1,

    /// Persistent message (survives broker restarts)
    Persistent = 2,
}

impl fmt::Display for DeliveryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryMode::NonPersistent => write!(f, "non-persistent"),
            DeliveryMode::Persistent => write!(f, "persistent"),
        }
    }
}

/// Message acknowledgment mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AcknowledgmentMode {
    /// Auto acknowledgment (auto-ack)
    Auto,

    /// Manual acknowledgment (client must explicitly acknowledge messages)
    Manual,
}

impl fmt::Display for AcknowledgmentMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AcknowledgmentMode::Auto => write!(f, "auto"),
            AcknowledgmentMode::Manual => write!(f, "manual"),
        }
    }
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionStatus {
    /// Connection is initializing
    Initializing,

    /// Connection is established
    Connected,

    /// Connection is disconnected
    Disconnected,

    /// Connection is reconnecting
    Reconnecting,

    /// Connection has permanently failed
    Failed,

    /// Connection is closed by application
    Closed,
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Initializing => write!(f, "initializing"),
            ConnectionStatus::Connected => write!(f, "connected"),
            ConnectionStatus::Disconnected => write!(f, "disconnected"),
            ConnectionStatus::Reconnecting => write!(f, "reconnecting"),
            ConnectionStatus::Failed => write!(f, "failed"),
            ConnectionStatus::Closed => write!(f, "closed"),
        }
    }
}
