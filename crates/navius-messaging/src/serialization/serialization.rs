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

/// Enum of available serializer types
#[derive(Debug, Clone)]
pub enum SerializerType {
    Json(JsonSerializer),
    Cbor(CborSerializer),
    MessagePack(MessagePackSerializer),
    Bincode(BincodeSerializer),
}

impl SerializerType {
    pub fn content_type(&self) -> &'static str {
        match self {
            SerializerType::Json(s) => s.content_type(),
            SerializerType::Cbor(s) => s.content_type(),
            SerializerType::MessagePack(s) => s.content_type(),
            SerializerType::Bincode(s) => s.content_type(),
        }
    }

    pub fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        match self {
            SerializerType::Json(s) => s.serialize_value(value),
            SerializerType::Cbor(s) => s.serialize_value(value),
            SerializerType::MessagePack(s) => s.serialize_value(value),
            SerializerType::Bincode(s) => s.serialize_value(value),
        }
    }

    pub fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        match self {
            SerializerType::Json(s) => s.deserialize_value(bytes),
            SerializerType::Cbor(s) => s.deserialize_value(bytes),
            SerializerType::MessagePack(s) => s.deserialize_value(bytes),
            SerializerType::Bincode(s) => s.deserialize_value(bytes),
        }
    }
}

/// Create a serializer based on the content type
pub fn create_serializer(content_type: &str) -> MessagingResult<SerializerType> {
    match content_type {
        "application/json" => Ok(SerializerType::Json(JsonSerializer)),
        "application/cbor" => Ok(SerializerType::Cbor(CborSerializer)),
        "application/msgpack" => Ok(SerializerType::MessagePack(MessagePackSerializer)),
        "application/octet-stream" => Ok(SerializerType::Bincode(BincodeSerializer)),
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
        serializer: &SerializerType,
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
    serializer: SerializerType,
}

impl AnyMessage {
    /// Create a new message from serializable data
    pub fn new<T: Serialize>(
        payload: &T,
        topic: impl Into<String>,
        serializer: SerializerType,
    ) -> MessagingResult<Self> {
        let data = BinaryData::from_value(payload, &serializer)?;

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

    /// Get the content type of the message
    pub fn content_type(&self) -> &str {
        self.serializer.content_type()
    }
}

/// A wrapper that can deserialize to different message types at runtime
#[derive(Debug, Clone)]
pub struct TypedMessageDeserializer<T> {
    /// The binary data to deserialize
    data: BinaryData,

    /// The serializer to use
    serializer: SerializerType,

    /// Phantom data for the target type
    _phantom: PhantomData<T>,
}

impl<T> TypedMessageDeserializer<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Create a new typed message deserializer
    pub fn new(data: BinaryData, serializer: SerializerType) -> Self {
        Self {
            data,
            serializer,
            _phantom: PhantomData,
        }
    }

    /// Deserialize the message to the target type
    pub fn deserialize(&self) -> MessagingResult<T> {
        let json_value = self.serializer.deserialize_value(&self.data.data)?;
        serde_json::from_value(json_value).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to convert from JSON value: {}",
                e
            ))
        })
    }

    /// Get the content type of the message
    pub fn content_type(&self) -> &str {
        self.serializer.content_type()
    }
}
