use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Represents message metadata with standardized properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageProperties {
    /// Unique identifier for the message
    pub message_id: String,

    /// Correlation ID for message tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Reply-to address for response messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,

    /// Message content type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Message content encoding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    /// Message expiration as ISO 8601 datetime
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,

    /// Message priority (0-9, where 0 is lowest)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    /// Message timestamp in milliseconds since epoch
    pub timestamp: u64,

    /// Application-specific headers
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

impl Default for MessageProperties {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        Self {
            message_id: Uuid::new_v4().to_string(),
            correlation_id: None,
            reply_to: None,
            content_type: Some("application/json".to_string()),
            content_encoding: Some("utf-8".to_string()),
            expiration: None,
            priority: None,
            timestamp: now,
            headers: HashMap::new(),
        }
    }
}

impl MessageProperties {
    /// Create new message properties with a generated UUID
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new message properties with the specified message ID
    pub fn with_id(message_id: impl Into<String>) -> Self {
        let mut props = Self::default();
        props.message_id = message_id.into();
        props
    }

    /// Add a header to the message properties
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Set the reply-to address
    pub fn with_reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Set the content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the content encoding
    pub fn with_content_encoding(mut self, content_encoding: impl Into<String>) -> Self {
        self.content_encoding = Some(content_encoding.into());
        self
    }

    /// Set the expiration time as an ISO 8601 datetime string
    pub fn with_expiration(mut self, expiration: impl Into<String>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    /// Set the expiration as a duration from now
    pub fn with_ttl(self, ttl: Duration) -> Self {
        // Convert the TTL to an ISO 8601 duration format
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        let expiration = now + ttl.as_millis() as u64;

        // ISO 8601 timestamp
        let expiration_str = format!("{}", expiration);
        self.with_expiration(expiration_str)
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.min(9)); // Ensure priority is 0-9
        self
    }
}

/// Trait for message payload objects.
///
/// Implementors of this trait can be serialized to and deserialized from bytes
/// for transmission via messaging systems.
pub trait Message: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// Get the type name of this message.
    /// This is used for message type routing and serialization.
    fn type_name() -> &'static str;

    /// Get the content type of this message.
    /// Default is application/json.
    fn content_type() -> &'static str {
        "application/json"
    }

    /// Get the routing key for this message.
    /// Default is the message type name.
    fn routing_key() -> &'static str {
        Self::type_name()
    }
}

/// Envelope that wraps a message with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T: Message> {
    /// Message properties and metadata
    pub properties: MessageProperties,

    /// The actual message payload
    pub payload: T,
}

impl<T: Message> MessageEnvelope<T> {
    /// Create a new message envelope with default properties
    pub fn new(payload: T) -> Self {
        Self {
            properties: MessageProperties::default(),
            payload,
        }
    }

    /// Create a new message envelope with specific properties
    pub fn with_properties(payload: T, properties: MessageProperties) -> Self {
        Self {
            properties,
            payload,
        }
    }
}

/// Represents a raw message that hasn't been deserialized yet.
#[derive(Debug, Clone)]
pub struct RawMessage {
    /// Message properties and metadata
    pub properties: MessageProperties,

    /// The raw message body as bytes
    pub body: Vec<u8>,
}

impl RawMessage {
    /// Create a new raw message
    pub fn new(properties: MessageProperties, body: Vec<u8>) -> Self {
        Self { properties, body }
    }

    /// Try to deserialize this raw message into a specific message type
    pub fn try_into<T: Message>(&self) -> Result<MessageEnvelope<T>, serde_json::Error> {
        let payload = serde_json::from_slice(&self.body)?;
        Ok(MessageEnvelope {
            properties: self.properties.clone(),
            payload,
        })
    }

    /// Get the size of the message body in bytes
    pub fn size(&self) -> usize {
        self.body.len()
    }
}
