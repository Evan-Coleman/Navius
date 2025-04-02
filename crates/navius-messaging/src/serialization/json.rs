use serde_json::Value;

use crate::error::{MessagingError, MessagingResult};

/// JSON serializer implementation
#[derive(Debug, Clone)]
pub struct JsonSerializer;

impl JsonSerializer {
    pub fn content_type(&self) -> &'static str {
        "application/json"
    }

    pub fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize JSON: {}", e))
        })
    }

    pub fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        serde_json::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!("Failed to deserialize JSON: {}", e))
        })
    }
}
