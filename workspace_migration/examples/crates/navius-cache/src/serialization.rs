use crate::error::{CacheError, CacheResult};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Cache serialization interface
///
/// This trait provides methods for serializing and deserializing data
/// for storage in a cache. Implementations can decide how to encode/decode
/// the data, such as using JSON, bincode, or other formats.
#[async_trait]
pub trait CacheSerializer: Send + Sync + Debug {
    /// Serialize a value to bytes
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>>;

    /// Deserialize bytes to a value
    async fn deserialize<T: DeserializeOwned + Send + Sync>(&self, data: &[u8]) -> CacheResult<T>;

    /// Get the content type for the serialized data (e.g., "application/json")
    fn content_type(&self) -> &str;
}

/// JSON serializer implementation using serde_json
#[derive(Debug, Clone)]
pub struct JsonSerializer;

#[async_trait]
impl CacheSerializer for JsonSerializer {
    #[instrument(skip(self, value), level = "debug")]
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        debug!("Serializing value to JSON");
        serde_json::to_vec(value).map_err(|e| {
            CacheError::SerializationError(format!("Failed to serialize to JSON: {}", e))
        })
    }

    #[instrument(skip(self, data), level = "debug")]
    async fn deserialize<T: DeserializeOwned + Send + Sync>(&self, data: &[u8]) -> CacheResult<T> {
        debug!("Deserializing value from JSON");
        serde_json::from_slice(data).map_err(|e| {
            CacheError::SerializationError(format!("Failed to deserialize from JSON: {}", e))
        })
    }

    fn content_type(&self) -> &str {
        "application/json"
    }
}

/// Binary serializer implementation using bincode
#[derive(Debug, Clone)]
pub struct BinarySerializer {
    config: bincode::config::Configuration,
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self {
            config: bincode::config::standard(),
        }
    }
}

impl BinarySerializer {
    /// Create a new binary serializer with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new binary serializer with custom configuration
    pub fn with_config(config: bincode::config::Configuration) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CacheSerializer for BinarySerializer {
    #[instrument(skip(self, value), level = "debug")]
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        debug!("Serializing value to binary format");
        bincode::encode_to_vec(value, self.config).map_err(|e| {
            CacheError::SerializationError(format!("Failed to serialize to binary: {}", e))
        })
    }

    #[instrument(skip(self, data), level = "debug")]
    async fn deserialize<T: DeserializeOwned + Send + Sync>(&self, data: &[u8]) -> CacheResult<T> {
        debug!("Deserializing value from binary format");
        bincode::decode_from_slice(data, self.config)
            .map(|(val, _)| val)
            .map_err(|e| {
                CacheError::SerializationError(format!("Failed to deserialize from binary: {}", e))
            })
    }

    fn content_type(&self) -> &str {
        "application/octet-stream"
    }
}

/// Composite serializer that supports multiple formats
#[derive(Debug, Clone)]
pub struct CompositeSerializer {
    serializers: Vec<(String, Arc<dyn CacheSerializer>)>,
    default_format: String,
}

impl CompositeSerializer {
    /// Create a new composite serializer
    pub fn new() -> Self {
        Self {
            serializers: Vec::new(),
            default_format: "json".to_string(),
        }
    }

    /// Add a serializer for a specific format
    pub fn with_serializer<S: CacheSerializer + 'static>(
        mut self,
        format: &str,
        serializer: S,
    ) -> Self {
        self.serializers
            .push((format.to_string(), Arc::new(serializer)));
        self
    }

    /// Set the default format
    pub fn with_default_format(mut self, format: &str) -> Self {
        self.default_format = format.to_string();
        self
    }

    /// Get a serializer for a specific format
    pub fn get_serializer(&self, format: &str) -> Option<Arc<dyn CacheSerializer>> {
        self.serializers
            .iter()
            .find(|(f, _)| f == format)
            .map(|(_, s)| s.clone())
    }

    /// Get the default serializer
    pub fn default_serializer(&self) -> Option<Arc<dyn CacheSerializer>> {
        self.get_serializer(&self.default_format)
    }
}

#[async_trait]
impl CacheSerializer for CompositeSerializer {
    #[instrument(skip(self, value), level = "debug")]
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        if let Some(serializer) = self.default_serializer() {
            serializer.serialize(value).await
        } else {
            Err(CacheError::SerializationError(
                "No default serializer configured".to_string(),
            ))
        }
    }

    #[instrument(skip(self, data), level = "debug")]
    async fn deserialize<T: DeserializeOwned + Send + Sync>(&self, data: &[u8]) -> CacheResult<T> {
        if let Some(serializer) = self.default_serializer() {
            serializer.deserialize(data).await
        } else {
            Err(CacheError::SerializationError(
                "No default serializer configured".to_string(),
            ))
        }
    }

    fn content_type(&self) -> &str {
        self.default_serializer()
            .map(|s| s.content_type())
            .unwrap_or("application/octet-stream")
    }
}

/// Helper function to create a default JSON serializer
pub fn json_serializer() -> JsonSerializer {
    JsonSerializer
}

/// Helper function to create a default binary serializer
pub fn binary_serializer() -> BinarySerializer {
    BinarySerializer::new()
}

/// Helper function to create a composite serializer with JSON and binary formats
pub fn default_serializer() -> CompositeSerializer {
    CompositeSerializer::new()
        .with_serializer("json", JsonSerializer)
        .with_serializer("binary", BinarySerializer::new())
        .with_default_format("json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        tags: Vec<String>,
    }

    impl TestData {
        fn new(id: u32, name: &str, tags: Vec<&str>) -> Self {
            Self {
                id,
                name: name.to_string(),
                tags: tags.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    #[tokio::test]
    async fn test_json_serializer() {
        let serializer = JsonSerializer;
        let data = TestData::new(1, "Test", vec!["tag1", "tag2"]);

        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();

        assert_eq!(data, deserialized);
        assert_eq!(serializer.content_type(), "application/json");
    }

    #[tokio::test]
    async fn test_binary_serializer() {
        let serializer = BinarySerializer::new();
        let data = TestData::new(1, "Test", vec!["tag1", "tag2"]);

        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();

        assert_eq!(data, deserialized);
        assert_eq!(serializer.content_type(), "application/octet-stream");
    }

    #[tokio::test]
    async fn test_composite_serializer() {
        let serializer = default_serializer();
        let data = TestData::new(1, "Test", vec!["tag1", "tag2"]);

        let serialized = serializer.serialize(&data).await.unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();

        assert_eq!(data, deserialized);
    }

    #[tokio::test]
    async fn test_composite_serializer_formats() {
        let serializer = CompositeSerializer::new()
            .with_serializer("json", JsonSerializer)
            .with_serializer("binary", BinarySerializer::new())
            .with_default_format("binary");

        let data = TestData::new(1, "Test", vec!["tag1", "tag2"]);

        // Should use binary format as default
        let serialized = serializer.serialize(&data).await.unwrap();
        let binary_serializer = BinarySerializer::new();
        let binary_serialized = binary_serializer.serialize(&data).await.unwrap();

        // Should be able to deserialize with correct format
        let deserialized: TestData = serializer.deserialize(&serialized).await.unwrap();
        assert_eq!(data, deserialized);

        // Content type should match default serializer
        assert_eq!(serializer.content_type(), "application/octet-stream");
    }
}
