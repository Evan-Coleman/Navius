use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::error::{MessagingError, MessagingResult};

/// Trait for type-specific message serializers
pub trait TypedMessageSerializer<T>: Send + Sync {
    /// Serialize a value to bytes
    fn serialize(&self, value: &T) -> MessagingResult<Vec<u8>>;

    /// Deserialize bytes to a value
    fn deserialize(&self, bytes: &[u8]) -> MessagingResult<T>;

    /// Get the content type of serialized data
    fn content_type(&self) -> &'static str;
}

/// Trait for message serialization
pub trait MessageSerializer: Send + Sync {
    /// Serialize a value to bytes
    fn serialize<T: Serialize>(&self, value: &T) -> MessagingResult<Vec<u8>>;

    /// Deserialize bytes to a value
    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> MessagingResult<T>;
}

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

/// JSON serializer implementation
pub struct JsonSerializer;

impl MessageSerializer for JsonSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to JSON: {}", e))
        })
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> MessagingResult<T> {
        serde_json::from_slice(bytes).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to deserialize from JSON: {}", e))
        })
    }
}

/// CBOR serializer implementation
pub struct CborSerializer;

impl MessageSerializer for CborSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        serde_cbor::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to CBOR: {}", e))
        })
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> MessagingResult<T> {
        serde_cbor::from_slice(bytes).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to deserialize from CBOR: {}", e))
        })
    }
}

/// MessagePack serializer implementation
pub struct MessagePackSerializer;

impl MessageSerializer for MessagePackSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to MessagePack: {}", e))
        })
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> MessagingResult<T> {
        rmp_serde::from_slice(bytes).map_err(|e| {
            MessagingError::SerializationError(format!(
                "Failed to deserialize from MessagePack: {}",
                e
            ))
        })
    }
}

/// Bincode serializer implementation
pub struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        bincode::serialize(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to Bincode: {}", e))
        })
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> MessagingResult<T> {
        bincode::deserialize(bytes).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to deserialize from Bincode: {}", e))
        })
    }
}

/// Create a serializer for the given format
pub fn create_serializer(format: SerializationFormat) -> Box<dyn MessageSerializer> {
    match format {
        SerializationFormat::Json => Box::new(JsonSerializer),
        SerializationFormat::Cbor => Box::new(CborSerializer),
        SerializationFormat::MessagePack => Box::new(MessagePackSerializer),
        SerializationFormat::Bincode => Box::new(BincodeSerializer),
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
    pub fn from_value<T: Serialize>(
        value: &T,
        serializer: &Box<dyn MessageSerializer>,
    ) -> MessagingResult<Self> {
        let data = serializer.serialize(value)?;
        let content_type = serializer.content_type().to_string();

        Ok(Self { data, content_type })
    }

    /// Deserialize the binary data to a value using the content type
    pub fn to_value<T: for<'de> Deserialize<'de>>(&self) -> MessagingResult<T> {
        let serializer = create_serializer(SerializationFormat::Json);
        serializer.deserialize(&self.data)
    }

    /// Deserialize the binary data using a specific serializer
    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        &self,
        serializer: &Box<dyn MessageSerializer>,
    ) -> MessagingResult<T> {
        serializer.deserialize(&self.data)
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
    serializer: Box<dyn MessageSerializer>,
}

impl AnyMessage {
    /// Create a new message from a serializable value
    pub fn new<T: Serialize>(
        payload: &T,
        topic: impl Into<String>,
        serializer: Box<dyn MessageSerializer>,
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
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> MessagingResult<T> {
        self.serializer.deserialize(&self.data.data)
    }

    /// Add a header to the message
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Check if the message can be deserialized to a specific type
    pub fn can_deserialize<T: for<'de> Deserialize<'de>>(&self) -> bool {
        self.serializer.deserialize::<T>(&self.data.data).is_ok()
    }
}

/// A typed message deserializer that can deserialize messages of a specific type
#[derive(Debug, Clone)]
pub struct TypedMessageDeserializer<T> {
    /// The serializer to use
    serializer: Box<dyn MessageSerializer>,

    /// Phantom data for type parameter
    phantom: PhantomData<T>,
}

impl<T: for<'de> Deserialize<'de>> TypedMessageDeserializer<T> {
    /// Create a new typed message deserializer
    pub fn new(serializer: Box<dyn MessageSerializer>) -> Self {
        Self {
            serializer,
            phantom: PhantomData,
        }
    }

    /// Deserialize a message payload
    pub fn deserialize(&self, data: &[u8]) -> MessagingResult<T> {
        self.serializer.deserialize(data)
    }

    /// Get the content type of serialized data
    pub fn content_type(&self) -> &'static str {
        self.serializer.content_type()
    }
}
