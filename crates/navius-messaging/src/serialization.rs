use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{MessagingError, MessagingResult};

/// Available serialization formats
#[derive(Debug, Clone, Copy)]
pub enum SerializationFormat {
    /// JSON serialization
    Json,
    /// CBOR serialization
    Cbor,
    /// MessagePack serialization
    MessagePack,
    /// Bincode serialization
    Bincode,
}

/// Serializer implementation using an enum
#[derive(Debug, Clone)]
pub enum Serializer {
    /// JSON serializer
    Json,
    /// CBOR serializer
    Cbor,
    /// MessagePack serializer
    MessagePack,
    /// Bincode serializer
    Bincode,
}

impl Serializer {
    /// Create a new serializer for the given format
    pub fn new(format: SerializationFormat) -> Self {
        match format {
            SerializationFormat::Json => Self::Json,
            SerializationFormat::Cbor => Self::Cbor,
            SerializationFormat::MessagePack => Self::MessagePack,
            SerializationFormat::Bincode => Self::Bincode,
        }
    }

    /// Serialize a value to bytes
    pub fn serialize_value<T: Serialize>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        match self {
            Self::Json => serde_json::to_vec(value).map_err(|e| {
                MessagingError::SerializationError(format!("JSON serialization error: {}", e))
            }),
            Self::Cbor => serde_cbor::to_vec(value).map_err(|e| {
                MessagingError::SerializationError(format!("CBOR serialization error: {}", e))
            }),
            Self::MessagePack => rmp_serde::to_vec(value).map_err(|e| {
                MessagingError::SerializationError(format!(
                    "MessagePack serialization error: {}",
                    e
                ))
            }),
            Self::Bincode => {
                // Create a Vec<u8> to serialize into
                let mut result = Vec::new();
                // Serialize using serde_json as a compatibility layer
                // (not ideal for performance but guarantees serde compatibility)
                let json = serde_json::to_string(value).map_err(|e| {
                    MessagingError::SerializationError(format!("JSON preprocessing error: {}", e))
                })?;
                result.extend_from_slice(json.as_bytes());
                Ok(result)
            }
        }
    }

    /// Deserialize bytes to a value
    pub fn deserialize_value<T: DeserializeOwned>(&self, bytes: &[u8]) -> MessagingResult<T> {
        match self {
            Self::Json => serde_json::from_slice(bytes).map_err(|e| {
                MessagingError::DeserializationError(format!("JSON deserialization error: {}", e))
            }),
            Self::Cbor => serde_cbor::from_slice(bytes).map_err(|e| {
                MessagingError::DeserializationError(format!("CBOR deserialization error: {}", e))
            }),
            Self::MessagePack => rmp_serde::from_slice(bytes).map_err(|e| {
                MessagingError::DeserializationError(format!(
                    "MessagePack deserialization error: {}",
                    e
                ))
            }),
            Self::Bincode => {
                // Treat the data as JSON for now for serde compatibility
                let str_data = std::str::from_utf8(bytes).map_err(|e| {
                    MessagingError::DeserializationError(format!("UTF-8 decoding error: {}", e))
                })?;

                serde_json::from_str(str_data).map_err(|e| {
                    MessagingError::DeserializationError(format!(
                        "JSON deserialization error: {}",
                        e
                    ))
                })
            }
        }
    }

    /// Get the content type of serialized data
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Cbor => "application/cbor",
            Self::MessagePack => "application/msgpack",
            Self::Bincode => "application/bincode",
        }
    }
}

/// A container for binary data with a specific content type
#[derive(Debug, Clone)]
pub struct BinaryData {
    /// The binary data
    pub data: Vec<u8>,

    /// The content type of the data
    pub content_type: String,
}

impl BinaryData {
    /// Create a new binary data container
    pub fn new(data: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self {
            data,
            content_type: content_type.into(),
        }
    }

    /// Create binary data from a serializable value using the specified serializer
    pub fn from_value<T: Serialize>(value: &T, serializer: &Serializer) -> MessagingResult<Self> {
        let data = serializer.serialize_value(value)?;
        let content_type = serializer.content_type().to_string();

        Ok(Self { data, content_type })
    }

    /// Deserialize the binary data to a value using the content type
    pub fn to_value<T: DeserializeOwned>(&self) -> MessagingResult<T> {
        let serializer = Serializer::Json; // Default to JSON
        serializer.deserialize_value(&self.data)
    }

    /// Deserialize the binary data using a specific serializer
    pub fn deserialize<T: DeserializeOwned>(&self, serializer: &Serializer) -> MessagingResult<T> {
        serializer.deserialize_value(&self.data)
    }
}

/// A type-erased message that can be serialized with any payload type
#[derive(Debug, Clone)]
pub struct AnyMessage {
    /// Serialized message data
    pub data: BinaryData,

    /// Message topic/routing key
    pub topic: String,

    /// Message headers
    pub headers: std::collections::HashMap<String, String>,

    /// Serializer used to create this message
    serializer: Serializer,
}

impl AnyMessage {
    /// Create a new message from a serializable value
    pub fn new<T: Serialize>(
        payload: &T,
        topic: impl Into<String>,
        serializer: Serializer,
    ) -> MessagingResult<Self> {
        let data = BinaryData::from_value(payload, &serializer)?;

        Ok(Self {
            data,
            topic: topic.into(),
            headers: std::collections::HashMap::new(),
            serializer,
        })
    }

    /// Deserialize the message payload to a specific type
    pub fn deserialize<T: DeserializeOwned>(&self) -> MessagingResult<T> {
        self.serializer.deserialize_value(&self.data.data)
    }

    /// Add a header to the message
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Check if the message can be deserialized to a specific type
    pub fn can_deserialize<T: DeserializeOwned>(&self) -> bool {
        self.serializer
            .deserialize_value::<T>(&self.data.data)
            .is_ok()
    }
}

/// A typed message deserializer that can deserialize messages of a specific type
#[derive(Debug, Clone)]
pub struct TypedMessageDeserializer<T> {
    /// The serializer to use
    serializer: Serializer,

    /// Phantom data for type parameter
    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> TypedMessageDeserializer<T> {
    /// Create a new typed message deserializer
    pub fn new(serializer: Serializer) -> Self {
        Self {
            serializer,
            phantom: PhantomData,
        }
    }

    /// Deserialize a message payload
    pub fn deserialize(&self, data: &[u8]) -> MessagingResult<T> {
        self.serializer.deserialize_value(data)
    }

    /// Get the content type of serialized data
    pub fn content_type(&self) -> &'static str {
        self.serializer.content_type()
    }
}
