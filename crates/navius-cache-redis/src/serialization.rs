use crate::error::{RedisCacheError, RedisCacheResult};
use async_trait::async_trait;
use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::{instrument, warn};

/// Serialization format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// JSON format (more readable, less efficient)
    Json,
    /// MessagePack format (more efficient, binary)
    MsgPack,
}

impl Default for SerializationFormat {
    fn default() -> Self {
        Self::Json
    }
}

/// Enum for serializer implementations, to avoid object safety issues
#[derive(Debug, Clone)]
pub enum SerializerImpl {
    /// JSON serializer
    Json(JsonSerializer),
    /// MessagePack serializer
    MsgPack(MsgPackSerializer),
}

impl SerializerImpl {
    /// Create a new serializer implementation based on format
    pub fn new(format: SerializationFormat) -> Self {
        match format {
            SerializationFormat::Json => Self::Json(JsonSerializer::default()),
            SerializationFormat::MsgPack => Self::MsgPack(MsgPackSerializer::default()),
        }
    }

    /// Serialize a value to bytes
    #[instrument(skip(self, value), level = "debug")]
    pub async fn serialize<T>(&self, value: &T) -> RedisCacheResult<Vec<u8>>
    where
        T: Serialize + Send + Sync + 'static,
    {
        match self {
            Self::Json(serializer) => serializer.serialize(value).await,
            Self::MsgPack(serializer) => serializer.serialize(value).await,
        }
    }

    /// Deserialize bytes to a value
    #[instrument(skip(self, data), level = "debug")]
    pub async fn deserialize<T>(&self, data: &[u8]) -> RedisCacheResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        match self {
            Self::Json(serializer) => serializer.deserialize(data).await,
            Self::MsgPack(serializer) => serializer.deserialize(data).await,
        }
    }

    /// Get the serialization format
    pub fn format(&self) -> SerializationFormat {
        match self {
            Self::Json(_) => SerializationFormat::Json,
            Self::MsgPack(_) => SerializationFormat::MsgPack,
        }
    }
}

/// Trait for serializing and deserializing cache values
#[async_trait]
pub trait Serializer: Send + Sync + 'static {
    /// Serialize a value to bytes
    async fn serialize<T>(&self, value: &T) -> RedisCacheResult<Vec<u8>>
    where
        T: Serialize + Send + Sync + 'static;

    /// Deserialize bytes to a value
    async fn deserialize<T>(&self, data: &[u8]) -> RedisCacheResult<T>
    where
        T: DeserializeOwned + Send + 'static;

    /// Get the serialization format
    fn format(&self) -> SerializationFormat;
}

/// JSON serializer for Redis cache values
#[derive(Debug, Clone, Default)]
pub struct JsonSerializer;

#[async_trait]
impl Serializer for JsonSerializer {
    #[instrument(skip(self, value), level = "debug")]
    async fn serialize<T>(&self, value: &T) -> RedisCacheResult<Vec<u8>>
    where
        T: Serialize + Send + Sync + 'static,
    {
        serde_json::to_vec(value).map_err(|e| {
            tracing::error!(?e, "Failed to serialize to JSON");
            RedisCacheError::Serialization(e.to_string())
        })
    }

    #[instrument(skip(self, data), level = "debug")]
    async fn deserialize<T>(&self, data: &[u8]) -> RedisCacheResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        serde_json::from_slice(data).map_err(|e| {
            warn!(error = %e, "JSON deserialization failed");
            RedisCacheError::Deserialization(format!("JSON deserialization failed: {}", e))
        })
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::Json
    }
}

/// MessagePack serializer for Redis cache values (typically more efficient than JSON)
#[derive(Debug, Clone, Default)]
pub struct MsgPackSerializer;

#[async_trait]
impl Serializer for MsgPackSerializer {
    #[instrument(skip(self, value), level = "debug")]
    async fn serialize<T>(&self, value: &T) -> RedisCacheResult<Vec<u8>>
    where
        T: Serialize + Send + Sync + 'static,
    {
        rmp_serde::to_vec(value).map_err(|e| {
            tracing::error!(?e, "Failed to serialize to MsgPack");
            RedisCacheError::Serialization(e.to_string())
        })
    }

    #[instrument(skip(self, data), level = "debug")]
    async fn deserialize<T>(&self, data: &[u8]) -> RedisCacheResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        rmp_serde::from_slice(data).map_err(|e| {
            warn!(error = %e, "MsgPack deserialization failed");
            RedisCacheError::Deserialization(format!("MsgPack deserialization failed: {}", e))
        })
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::MsgPack
    }
}

/// Create a serializer based on the specified format
pub fn create_serializer(format: SerializationFormat) -> SerializerImpl {
    SerializerImpl::new(format)
}

/// Serializes data for Redis storage
///
/// Converts any Serialize type to a Redis-compatible string
/// Uses JSON serialization by default
pub fn serialize<T: Serialize + Debug>(value: &T) -> RedisCacheResult<String> {
    serde_json::to_string(value)
        .map_err(|e| RedisCacheError::Serialization(format!("Failed to serialize value: {:?}", e)))
}

/// Deserializes data from Redis storage
///
/// Converts a Redis-compatible string to any Deserialize type
/// Uses JSON deserialization by default
pub fn deserialize<T: DeserializeOwned>(value: &str) -> RedisCacheResult<T> {
    serde_json::from_str(value).map_err(|e| {
        RedisCacheError::Deserialization(format!("Failed to deserialize value: {:?}", e))
    })
}

/// Adapter trait to convert from Serialize to ToRedisArgs
pub trait SerializeToRedisArgs: Serialize + Debug {
    /// Convert to Redis arguments
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        match serde_json::to_string(self) {
            Ok(s) => vec![s.into_bytes()],
            Err(e) => {
                // Log the error and return an empty vec
                tracing::error!(
                    "Failed to serialize value to Redis args: {:?}, error: {:?}",
                    self,
                    e
                );
                vec![]
            }
        }
    }
}

/// Implement SerializeToRedisArgs for all types that implement Serialize
impl<T: Serialize + Debug> SerializeToRedisArgs for T {}

// We'll need to implement ToRedisArgs for specific types instead of a blanket impl
// Define a wrapper struct to implement ToRedisArgs for
pub struct SerializeWrapper<T: SerializeToRedisArgs>(pub T);

impl<T: SerializeToRedisArgs> ToRedisArgs for SerializeWrapper<T> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let args = self.0.to_redis_args();
        for arg in args {
            out.write_arg(&arg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestData {
        id: u32,
        name: String,
        tags: Vec<String>,
    }

    #[tokio::test]
    async fn test_json_serializer() {
        let serializer = JsonSerializer::default();
        let data = TestData {
            id: 42,
            name: "Test".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();

        assert_eq!(data, deserialized);
    }

    #[tokio::test]
    async fn test_msgpack_serializer() {
        let serializer = MsgPackSerializer::default();
        let data = TestData {
            id: 42,
            name: "Test".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();

        assert_eq!(data, deserialized);
    }

    #[tokio::test]
    async fn test_serializer_impl() {
        let data = TestData {
            id: 42,
            name: "Test".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        // Test JSON serializer impl
        let serializer = SerializerImpl::new(SerializationFormat::Json);
        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();
        assert_eq!(data, deserialized);
        assert_eq!(serializer.format(), SerializationFormat::Json);

        // Test MsgPack serializer impl
        let serializer = SerializerImpl::new(SerializationFormat::MsgPack);
        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();
        assert_eq!(data, deserialized);
        assert_eq!(serializer.format(), SerializationFormat::MsgPack);
    }
}
