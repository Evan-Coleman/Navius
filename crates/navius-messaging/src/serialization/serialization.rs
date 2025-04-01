mod bincode;
mod cbor;
mod json;
mod msgpack;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::marker::PhantomData;

use crate::error::{MessagingError, MessagingResult};

pub use self::{
    bincode::BincodeSerializer, cbor::CborSerializer, json::JsonSerializer,
    msgpack::MessagePackSerializer,
};

/// Trait for message serializers
pub trait MessageSerializer: Send + Sync {
    /// Get the content type of serialized data
    fn content_type(&self) -> &'static str;

    /// Serialize a value to bytes
    fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>>;

    /// Deserialize bytes to a value
    fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value>;
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
        let json_value = serde_json::to_value(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to convert to JSON value: {}", e))
        })?;
        let data = serializer.serialize_value(&json_value)?;
        let content_type = serializer.content_type().to_string();

        Ok(Self { data, content_type })
    }

    /// Deserialize the binary data to a value using the content type
    pub fn to_value<T: for<'de> Deserialize<'de>>(&self) -> MessagingResult<T> {
        let serializer = create_serializer(&self.content_type)?;
        let json_value = serializer.deserialize_value(&self.data)?;
        serde_json::from_value(json_value).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to convert from JSON value: {}",
                e
            ))
        })
    }

    /// Deserialize the binary data using a specific serializer
    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        &self,
        serializer: &dyn MessageSerializer,
    ) -> MessagingResult<T> {
        let json_value = serializer.deserialize_value(&self.data)?;
        serde_json::from_value(json_value).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to convert from JSON value: {}",
                e
            ))
        })
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
        let json_value = self.serializer.deserialize_value(&self.data.data)?;
        serde_json::from_value(json_value).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to convert from JSON value: {}",
                e
            ))
        })
    }

    /// Add a header to the message
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Check if the message can be deserialized to a specific type
    pub fn can_deserialize<T: for<'de> Deserialize<'de>>(&self) -> bool {
        if let Ok(json_value) = self.serializer.deserialize_value(&self.data.data) {
            serde_json::from_value::<T>(json_value).is_ok()
        } else {
            false
        }
    }
}

/// A wrapper that can deserialize to different message types at runtime
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

    /// Deserialize a message
    pub fn deserialize(&self, data: &[u8]) -> MessagingResult<T> {
        let json_value = self.serializer.deserialize_value(data)?;
        serde_json::from_value(json_value).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to convert from JSON value: {}",
                e
            ))
        })
    }

    /// Get the content type
    pub fn content_type(&self) -> &'static str {
        self.serializer.content_type()
    }
}
