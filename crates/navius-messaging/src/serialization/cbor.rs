use serde_json::Value;

use super::MessageSerializer;
use crate::error::{MessagingError, MessagingResult};

/// CBOR serializer implementation
#[derive(Debug, Clone, Default)]
pub struct CborSerializer;

impl MessageSerializer for CborSerializer {
    fn content_type(&self) -> &'static str {
        "application/cbor"
    }

    fn serialize_value(&self, value: &Value) -> MessagingResult<Vec<u8>> {
        serde_cbor::to_vec(value).map_err(|e| {
            MessagingError::SerializationError(format!("Failed to serialize to CBOR: {}", e))
        })
    }

    fn deserialize_value(&self, bytes: &[u8]) -> MessagingResult<Value> {
        serde_cbor::from_slice(bytes).map_err(|e| {
            MessagingError::DeserializationError(format!("Failed to deserialize from CBOR: {}", e))
        })
    }
}
