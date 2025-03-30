use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Properties associated with a message.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageProperties {
    /// Custom headers for the message
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Content type of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Content encoding of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    /// Message delivery mode (1 = non-persistent, 2 = persistent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_mode: Option<u8>,

    /// Message priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    /// Message correlation ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Message reply-to address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,

    /// Message expiration time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,

    /// Message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Message timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,

    /// Message type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,

    /// Message user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Message application ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
}

impl MessageProperties {
    /// Create new message properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a header to the message properties.
    ///
    /// # Arguments
    /// * `key` - The header key
    /// * `value` - The header value
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the content type.
    ///
    /// # Arguments
    /// * `content_type` - The content type
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_content_type<S: Into<String>>(mut self, content_type: S) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the content encoding.
    ///
    /// # Arguments
    /// * `content_encoding` - The content encoding
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_content_encoding<S: Into<String>>(mut self, content_encoding: S) -> Self {
        self.content_encoding = Some(content_encoding.into());
        self
    }

    /// Set the delivery mode.
    ///
    /// # Arguments
    /// * `persistent` - Whether the message should be persistent
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_persistent(mut self, persistent: bool) -> Self {
        self.delivery_mode = Some(if persistent { 2 } else { 1 });
        self
    }

    /// Set the priority.
    ///
    /// # Arguments
    /// * `priority` - The priority
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set the correlation ID.
    ///
    /// # Arguments
    /// * `correlation_id` - The correlation ID
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_correlation_id<S: Into<String>>(mut self, correlation_id: S) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Set the reply-to address.
    ///
    /// # Arguments
    /// * `reply_to` - The reply-to address
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_reply_to<S: Into<String>>(mut self, reply_to: S) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Set the expiration time.
    ///
    /// # Arguments
    /// * `expiration` - The expiration time in milliseconds
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_expiration(mut self, expiration: u64) -> Self {
        self.expiration = Some(expiration.to_string());
        self
    }

    /// Set the message ID.
    ///
    /// # Arguments
    /// * `message_id` - The message ID
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_message_id<S: Into<String>>(mut self, message_id: S) -> Self {
        self.message_id = Some(message_id.into());
        self
    }

    /// Set the timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the message type.
    ///
    /// # Arguments
    /// * `message_type` - The message type
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_message_type<S: Into<String>>(mut self, message_type: S) -> Self {
        self.message_type = Some(message_type.into());
        self
    }

    /// Set the user ID.
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_user_id<S: Into<String>>(mut self, user_id: S) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the application ID.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_app_id<S: Into<String>>(mut self, app_id: S) -> Self {
        self.app_id = Some(app_id.into());
        self
    }
}

/// A message envelope that wraps a payload with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// The message ID
    pub id: String,

    /// The message payload
    pub payload: Vec<u8>,

    /// The message properties
    #[serde(default)]
    pub properties: MessageProperties,

    /// The timestamp when the message was created
    pub created_at: u64,

    /// The sequence number of the message (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
}

impl MessageEnvelope {
    /// Create a new message envelope with the given payload.
    ///
    /// # Arguments
    /// * `payload` - The message payload
    ///
    /// # Returns
    /// A new message envelope
    pub fn new<P: Into<Vec<u8>>>(payload: P) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            payload: payload.into(),
            properties: MessageProperties::default(),
            created_at: now,
            sequence: None,
        }
    }

    /// Create a new message envelope with the given payload and properties.
    ///
    /// # Arguments
    /// * `payload` - The message payload
    /// * `properties` - The message properties
    ///
    /// # Returns
    /// A new message envelope
    pub fn with_properties<P: Into<Vec<u8>>>(payload: P, properties: MessageProperties) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            payload: payload.into(),
            properties,
            created_at: now,
            sequence: None,
        }
    }

    /// Set the sequence number for this message.
    ///
    /// # Arguments
    /// * `sequence` - The sequence number
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    /// Set the message ID.
    ///
    /// # Arguments
    /// * `id` - The message ID
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = id.into();
        self
    }
}
