use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::error::{MessagingError, MessagingResult};

/// Trait for message serializers
pub trait MessageSerializer: Send + Sync {
    /// Serialize a value to bytes
    fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> MessagingResult<Vec<u8>>;

    /// Deserialize bytes to a value
    fn deserialize<'a, T: Deserialize<'a>>(&self, bytes: &'a [u8]) -> MessagingResult<T>;

    /// Get the content type of serialized data
    fn content_type(&self) -> &'static str;
}

/// JSON serializer using serde_json
pub struct JsonSerializer;

impl MessageSerializer for JsonSerializer {
    fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to JSON: {}", e))
        })
    }

    fn deserialize<'a, T: Deserialize<'a>>(&self, bytes: &'a [u8]) -> MessagingResult<T> {
        serde_json::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!("Failed to deserialize from JSON: {}", e))
        })
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }
}

/// CBOR serializer using serde_cbor
pub struct CborSerializer;

impl MessageSerializer for CborSerializer {
    fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        serde_cbor::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to CBOR: {}", e))
        })
    }

    fn deserialize<'a, T: Deserialize<'a>>(&self, bytes: &'a [u8]) -> MessagingResult<T> {
        serde_cbor::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!("Failed to deserialize from CBOR: {}", e))
        })
    }

    fn content_type(&self) -> &'static str {
        "application/cbor"
    }
}

/// MessagePack serializer using rmp_serde
pub struct MessagePackSerializer;

impl MessageSerializer for MessagePackSerializer {
    fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to MessagePack: {}", e))
        })
    }

    fn deserialize<'a, T: Deserialize<'a>>(&self, bytes: &'a [u8]) -> MessagingResult<T> {
        rmp_serde::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to deserialize from MessagePack: {}",
                e
            ))
        })
    }

    fn content_type(&self) -> &'static str {
        "application/msgpack"
    }
}

/// Binary serializer using bincode
pub struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> MessagingResult<Vec<u8>> {
        bincode::serialize(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize with bincode: {}", e))
        })
    }

    fn deserialize<'a, T: Deserialize<'a>>(&self, bytes: &'a [u8]) -> MessagingResult<T> {
        bincode::deserialize(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to deserialize with bincode: {}",
                e
            ))
        })
    }

    fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }
}

/// Serializer that can serialize to string
pub trait StringSerializer {
    /// Serialize a value to a string
    fn serialize_to_string<T: Serialize>(&self, value: &T) -> MessagingResult<String>;

    /// Deserialize a value from a string
    fn deserialize_from_string<T: for<'de> Deserialize<'de>>(
        &self,
        value: &str,
    ) -> MessagingResult<T>;
}

impl StringSerializer for JsonSerializer {
    fn serialize_to_string<T: Serialize>(&self, value: &T) -> MessagingResult<String> {
        serde_json::to_string(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to JSON string: {}", e))
        })
    }

    fn deserialize_from_string<T: for<'de> Deserialize<'de>>(
        &self,
        value: &str,
    ) -> MessagingResult<T> {
        serde_json::from_str(value).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to deserialize from JSON string: {}",
                e
            ))
        })
    }
}

/// Create a serializer based on the content type
pub fn create_serializer(content_type: &str) -> MessagingResult<Box<dyn MessageSerializer>> {
    match content_type {
        "application/json" => Ok(Box::new(JsonSerializer)),
        "application/cbor" => Ok(Box::new(CborSerializer)),
        "application/msgpack" => Ok(Box::new(MessagePackSerializer)),
        "application/octet-stream" => Ok(Box::new(BincodeSerializer)),
        _ => Err(MessagingError::SerializationError(format!(
            "Unsupported content type: {}",
            content_type
        ))),
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
        serializer: &dyn MessageSerializer,
    ) -> MessagingResult<Self> {
        let data = serializer.serialize(value)?;
        let content_type = serializer.content_type().to_string();

        Ok(Self { data, content_type })
    }

    /// Deserialize the binary data to a value using the content type
    pub fn to_value<T: for<'de> Deserialize<'de>>(&self) -> MessagingResult<T> {
        let serializer = create_serializer(&self.content_type)?;
        serializer.deserialize(&self.data)
    }

    /// Deserialize the binary data using a specific serializer
    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        &self,
        serializer: &dyn MessageSerializer,
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
    /// Create a new message from serializable data
    pub fn new<T: Serialize>(
        payload: &T,
        topic: impl Into<String>,
        serializer: Box<dyn MessageSerializer>,
    ) -> MessagingResult<Self> {
        let data = BinaryData::from_value(payload, serializer.as_ref())?;

        Ok(Self {
            data,
            topic: topic.into(),
            headers: std::collections::HashMap::new(),
            serializer,
        })
    }

    /// Deserialize the message to a specific type
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

/// A wrapper that can deserialize to different message types at runtime
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

    /// Deserialize bytes to the target type
    pub fn deserialize(&self, data: &[u8]) -> MessagingResult<T> {
        self.serializer.deserialize(data)
    }

    /// Get the serializer content type
    pub fn content_type(&self) -> &'static str {
        self.serializer.content_type()
    }
}
