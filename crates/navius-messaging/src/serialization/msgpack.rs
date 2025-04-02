use serde_json::Value;

use crate::error::{MessagingError, MessagingResult};

/// MessagePack serializer implementation
#[derive(Debug, Clone)]
pub struct MessagePackSerializer;

impl MessagePackSerializer {
    pub fn content_type(&self) -> &'static str {
        "application/msgpack"
    }

    pub fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize MessagePack: {}", e))
        })
    }

    pub fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        rmp_serde::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to deserialize MessagePack: {}",
                e
            ))
        })
    }
}
