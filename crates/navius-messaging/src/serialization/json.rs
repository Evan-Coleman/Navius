use serde_json::Value;

use super::MessageSerializer;
use crate::error::{MessagingError, MessagingResult};

/// JSON serializer implementation
#[derive(Debug, Clone, Default)]
pub struct JsonSerializer;

impl MessageSerializer for JsonSerializer {
    fn content_type(&self) -> &'static str {
        "application/json"
    }

    fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to JSON: {}", e))
        })
    }

    fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        serde_json::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!("Failed to deserialize from JSON: {}", e))
        })
    }
}
