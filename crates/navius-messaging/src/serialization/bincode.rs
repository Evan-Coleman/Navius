use serde_json::Value;

use super::MessageSerializer;
use crate::error::{MessagingError, MessagingResult};

/// Bincode serializer implementation
#[derive(Debug, Clone, Default)]
pub struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }

    fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        bincode::serialize(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to Bincode: {}", e))
        })
    }

    fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        bincode::deserialize(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!(
                "Failed to deserialize from Bincode: {}",
                e
            ))
        })
    }
}
