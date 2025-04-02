use serde_json::Value;

use crate::error::{MessagingError, MessagingResult};

/// Bincode serializer implementation
#[derive(Debug, Clone)]
pub struct BincodeSerializer;

impl BincodeSerializer {
    pub fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }

    pub fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        bincode::serialize(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize Bincode: {}", e))
        })
    }

    pub fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        bincode::deserialize(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!("Failed to deserialize Bincode: {}", e))
        })
    }
}
