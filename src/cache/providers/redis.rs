use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use super::CacheProvider;
use crate::utils::api_resource::ApiResource;

/// Redis cache provider configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<i64>,
    pub ttl_seconds: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            username: None,
            password: None,
            database: Some(0),
            ttl_seconds: 3600,
        }
    }
}

/// Redis cache provider - skeleton for users to implement
///
/// # Example Implementation
/// ```rust
/// // To implement this Redis provider, you'll need to:
/// // 1. Add redis dependency to Cargo.toml
/// // 2. Implement the required CacheProvider trait methods
/// // 3. Create a Redis client connection
/// ```
pub struct RedisCacheProvider {
    config: RedisConfig,
    // client: Option<redis::Client>, // Uncomment when implementing
}

impl RedisCacheProvider {
    /// Create a new Redis cache provider
    pub fn new(config: RedisConfig) -> Self {
        Self {
            config,
            // client: None, // Uncomment when implementing
        }
    }
}

#[async_trait]
impl CacheProvider for RedisCacheProvider {
    fn init(&self) -> Result<(), String> {
        // TODO: Implement Redis client initialization
        // Example:
        // let client = redis::Client::open(self.config.url.as_str())
        //     .map_err(|e| format!("Failed to create Redis client: {}", e))?;
        // self.client = Some(client);

        Err(
            "Redis provider not implemented. See implementation instructions in the code."
                .to_string(),
        )
    }

    async fn set<T: ApiResource>(
        &self,
        _key: &str,
        _value: T,
        _ttl_seconds: u64,
    ) -> Result<(), String> {
        // TODO: Implement Redis set operation
        // Example:
        // let serialized = serde_json::to_string(&value)
        //     .map_err(|e| format!("Failed to serialize value: {}", e))?;
        //
        // let mut conn = self.client.as_ref().ok_or("Redis client not initialized")?.get_async_connection().await
        //     .map_err(|e| format!("Failed to get Redis connection: {}", e))?;
        //
        // redis::cmd("SET")
        //     .arg(key)
        //     .arg(serialized)
        //     .arg("EX")
        //     .arg(ttl_seconds)
        //     .query_async(&mut conn).await
        //     .map_err(|e| format!("Failed to set value in Redis: {}", e))?;

        Err(
            "Redis provider not implemented. See implementation instructions in the code."
                .to_string(),
        )
    }

    async fn get<T: ApiResource>(&self, _key: &str) -> Result<Option<T>, String> {
        // TODO: Implement Redis get operation
        // Example:
        // let mut conn = match self.client.as_ref() {
        //     Some(client) => client.get_async_connection().await
        //         .map_err(|e| format!("Failed to get Redis connection: {}", e))?,
        //     None => return Err("Redis client not initialized".to_string()),
        // };
        //
        // let result: Option<String> = redis::cmd("GET")
        //     .arg(key)
        //     .query_async(&mut conn).await
        //     .map_err(|e| format!("Failed to get value from Redis: {}", e))?;
        //
        // match result {
        //     Some(data) => {
        //         let value: T = serde_json::from_str(&data)
        //             .map_err(|e| format!("Failed to deserialize value: {}", e))?;
        //         Ok(Some(value))
        //     },
        //     None => Ok(None),
        // }

        Err(
            "Redis provider not implemented. See implementation instructions in the code."
                .to_string(),
        )
    }

    async fn delete(&self, _key: &str) -> Result<(), String> {
        // TODO: Implement Redis delete operation

        Err(
            "Redis provider not implemented. See implementation instructions in the code."
                .to_string(),
        )
    }

    async fn clear(&self) -> Result<(), String> {
        // TODO: Implement Redis clear operation
        // Note: This would typically use the FLUSHDB command, which is destructive

        Err(
            "Redis provider not implemented. See implementation instructions in the code."
                .to_string(),
        )
    }

    async fn exists(&self, _key: &str) -> Result<bool, String> {
        // TODO: Implement Redis exists operation

        Err(
            "Redis provider not implemented. See implementation instructions in the code."
                .to_string(),
        )
    }

    async fn get_stats(&self) -> Result<serde_json::Value, String> {
        // TODO: Implement Redis stats operation using INFO command

        let result = json!({
            "provider": "redis",
            "status": "not implemented",
            "config": {
                "url": self.config.url,
                "database": self.config.database,
                "ttl_seconds": self.config.ttl_seconds
            }
        });

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::api_resource::ApiResource;
    use serde::{Deserialize, Serialize};
    use std::sync::{Arc, Mutex, RwLock};

    // Test data structure that implements ApiResource
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResource {
        id: String,
        name: String,
        value: i32,
    }

    impl ApiResource for TestResource {
        type Id = String;

        fn resource_type() -> &'static str {
            "test_resource"
        }

        fn api_name() -> &'static str {
            "TestService"
        }
    }

    // Simple mocking of Redis functions
    struct MockRedisClient {
        init_called: RwLock<bool>,
        set_keys: RwLock<Vec<String>>,
        set_values: RwLock<Vec<String>>,
        set_ttls: RwLock<Vec<u64>>,
        get_keys: RwLock<Vec<String>>,
        get_return_values: RwLock<Vec<Option<String>>>,
        delete_keys: RwLock<Vec<String>>,
        clear_called: RwLock<bool>,
        exists_keys: RwLock<Vec<String>>,
        exists_return_values: RwLock<Vec<bool>>,
        should_fail: RwLock<bool>,
    }

    impl Default for MockRedisClient {
        fn default() -> Self {
            Self {
                init_called: RwLock::new(false),
                set_keys: RwLock::new(Vec::new()),
                set_values: RwLock::new(Vec::new()),
                set_ttls: RwLock::new(Vec::new()),
                get_keys: RwLock::new(Vec::new()),
                get_return_values: RwLock::new(Vec::new()),
                delete_keys: RwLock::new(Vec::new()),
                clear_called: RwLock::new(false),
                exists_keys: RwLock::new(Vec::new()),
                exists_return_values: RwLock::new(Vec::new()),
                should_fail: RwLock::new(false),
            }
        }
    }

    // Wrapper provider that uses our mock client
    struct MockRedisCacheProvider {
        config: RedisConfig,
        client: Arc<MockRedisClient>,
    }

    impl MockRedisCacheProvider {
        fn new(config: RedisConfig, client: Arc<MockRedisClient>) -> Self {
            Self { config, client }
        }
    }

    #[async_trait]
    impl CacheProvider for MockRedisCacheProvider {
        fn init(&self) -> Result<(), String> {
            *self.client.init_called.write().unwrap() = true;

            if *self.client.should_fail.read().unwrap() {
                return Err("Mock Redis initialization failed".to_string());
            }

            Ok(())
        }

        async fn set<T: ApiResource>(
            &self,
            key: &str,
            value: T,
            ttl_seconds: u64,
        ) -> Result<(), String> {
            // Store the data in our mocks
            let serialized = serde_json::to_string(&value)
                .map_err(|e| format!("Failed to serialize value: {}", e))?;

            if *self.client.should_fail.read().unwrap() {
                return Err("Mock Redis write error".to_string());
            }

            self.client.set_keys.write().unwrap().push(key.to_string());
            self.client.set_values.write().unwrap().push(serialized);
            self.client.set_ttls.write().unwrap().push(ttl_seconds);

            Ok(())
        }

        async fn get<T: ApiResource>(&self, key: &str) -> Result<Option<T>, String> {
            self.client.get_keys.write().unwrap().push(key.to_string());

            if *self.client.should_fail.read().unwrap() {
                return Err("Mock Redis connection error".to_string());
            }

            // Check if we have predetermined return values
            let return_values = self.client.get_return_values.read().unwrap();
            if !return_values.is_empty() {
                // Get the first return value and process it
                if let Some(data) = return_values.get(0).cloned().unwrap_or(None) {
                    // Deserialize and return
                    let value: T = serde_json::from_str(&data)
                        .map_err(|e| format!("Failed to deserialize value: {}", e))?;
                    return Ok(Some(value));
                }

                // Return None if we had a None in our predetermined values
                return Ok(None);
            }

            // Otherwise, look up in our set data
            let keys = self.client.set_keys.read().unwrap();
            let values = self.client.set_values.read().unwrap();

            for (i, k) in keys.iter().enumerate() {
                if k == key {
                    if let Some(data) = values.get(i) {
                        let value: T = serde_json::from_str(data)
                            .map_err(|e| format!("Failed to deserialize value: {}", e))?;
                        return Ok(Some(value));
                    }
                }
            }

            // Key not found
            Ok(None)
        }

        async fn delete(&self, key: &str) -> Result<(), String> {
            self.client
                .delete_keys
                .write()
                .unwrap()
                .push(key.to_string());

            if *self.client.should_fail.read().unwrap() {
                return Err("Mock Redis delete error".to_string());
            }

            Ok(())
        }

        async fn clear(&self) -> Result<(), String> {
            *self.client.clear_called.write().unwrap() = true;

            if *self.client.should_fail.read().unwrap() {
                return Err("Mock Redis clear error".to_string());
            }

            Ok(())
        }

        async fn exists(&self, key: &str) -> Result<bool, String> {
            self.client
                .exists_keys
                .write()
                .unwrap()
                .push(key.to_string());

            if *self.client.should_fail.read().unwrap() {
                return Err("Mock Redis exists error".to_string());
            }

            // Check if we have predetermined return values
            let return_values = self.client.exists_return_values.read().unwrap();
            if !return_values.is_empty() {
                return Ok(return_values[0]);
            }

            // Otherwise, check if the key exists in our set data
            let keys = self.client.set_keys.read().unwrap();
            Ok(keys.contains(&key.to_string()))
        }

        async fn get_stats(&self) -> Result<serde_json::Value, String> {
            let result = json!({
                "provider": "redis",
                "status": "mocked",
                "config": {
                    "url": self.config.url,
                    "database": self.config.database,
                    "ttl_seconds": self.config.ttl_seconds
                }
            });

            Ok(result)
        }
    }

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        use tokio::runtime::Builder;

        /// A wrapper to run async functions in proptest
        fn run_async<F: std::future::Future>(future: F) -> F::Output {
            // Use a basic single-threaded runtime to avoid thread pool issues
            let rt = Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(future)
        }

        proptest! {
            /// Test that mock Redis cache behaves correctly with different inputs
            #[test]
            fn mock_redis_cache_operations(
                id in "[a-z0-9]{1,10}",
                name in "[a-zA-Z ]{1,20}",
                value in 0..1000,
                ttl in 1..3600u64
            ) {
                // Create mock Redis client and provider
                let config = RedisConfig::default();
                let mock_client = Arc::new(MockRedisClient::default());
                let provider = MockRedisCacheProvider::new(config, mock_client.clone());

                // Initialize the provider
                let init_result = provider.init();
                prop_assert!(init_result.is_ok());

                let resource = TestResource {
                    id: id.clone(),
                    name,
                    value,
                };

                // Set the resource
                let set_result = run_async(provider.set(&id, resource.clone(), ttl));
                prop_assert!(set_result.is_ok());

                // Get the resource
                let get_result = run_async(provider.get::<TestResource>(&id));
                prop_assert!(get_result.is_ok());

                let retrieved = get_result.unwrap();
                prop_assert!(retrieved.is_some());

                let retrieved_resource = retrieved.unwrap();
                prop_assert_eq!(retrieved_resource.id, resource.id);
                prop_assert_eq!(retrieved_resource.name, resource.name);
                prop_assert_eq!(retrieved_resource.value, resource.value);

                // Test exists
                let exists_result = run_async(provider.exists(&id));
                prop_assert!(exists_result.is_ok());
                prop_assert!(exists_result.unwrap());

                // Test delete
                let delete_result = run_async(provider.delete(&id));
                prop_assert!(delete_result.is_ok());

                // Verify key doesn't exist after delete
                // This requires checking our mock's internal state since we've mocked exists
                let mock_keys = mock_client.delete_keys.read().unwrap();
                prop_assert!(mock_keys.contains(&id));
            }

            /// Test that mock Redis cache handling of errors works correctly
            #[test]
            fn mock_redis_error_handling(
                id in "[a-z0-9]{1,10}",
                name in "[a-zA-Z ]{1,20}",
                value in 0..1000
            ) {
                // Create mock Redis client and provider
                let config = RedisConfig::default();
                let mock_client = Arc::new(MockRedisClient::default());

                // Set mock to fail
                *mock_client.should_fail.write().unwrap() = true;

                let provider = MockRedisCacheProvider::new(config, mock_client.clone());

                let resource = TestResource {
                    id: id.clone(),
                    name,
                    value,
                };

                // Set should return error
                let set_result = run_async(provider.set(&id, resource.clone(), 3600));
                prop_assert!(set_result.is_err());
                prop_assert_eq!(set_result.unwrap_err(), "Mock Redis write error");

                // Get should return error
                let get_result = run_async(provider.get::<TestResource>(&id));
                prop_assert!(get_result.is_err());
                prop_assert_eq!(get_result.unwrap_err(), "Mock Redis connection error");

                // Delete should return error
                let delete_result = run_async(provider.delete(&id));
                prop_assert!(delete_result.is_err());
                prop_assert_eq!(delete_result.unwrap_err(), "Mock Redis delete error");

                // Clear should return error
                let clear_result = run_async(provider.clear());
                prop_assert!(clear_result.is_err());
                prop_assert_eq!(clear_result.unwrap_err(), "Mock Redis clear error");

                // Exists should return error
                let exists_result = run_async(provider.exists(&id));
                prop_assert!(exists_result.is_err());
                prop_assert_eq!(exists_result.unwrap_err(), "Mock Redis exists error");
            }
        }
    }

    #[tokio::test]
    async fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.url, "redis://127.0.0.1:6379");
        assert_eq!(config.username, None);
        assert_eq!(config.password, None);
        assert_eq!(config.database, Some(0));
        assert_eq!(config.ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_redis_cache_provider_creation() {
        let config = RedisConfig::default();
        let provider = RedisCacheProvider::new(config.clone());

        // Verify config was stored correctly
        assert_eq!(provider.config.url, config.url);
        assert_eq!(provider.config.username, config.username);
        assert_eq!(provider.config.password, config.password);
        assert_eq!(provider.config.database, config.database);
        assert_eq!(provider.config.ttl_seconds, config.ttl_seconds);
    }

    #[tokio::test]
    async fn test_redis_cache_get_stats() {
        let config = RedisConfig::default();
        let provider = RedisCacheProvider::new(config);

        // Get stats
        let stats_result = provider.get_stats().await;
        assert!(stats_result.is_ok());

        let stats = stats_result.unwrap();
        assert_eq!(stats["provider"], "redis");
        assert_eq!(stats["status"], "not implemented");
        assert!(stats["config"].is_object());
        assert_eq!(stats["config"]["url"], "redis://127.0.0.1:6379");
    }

    #[tokio::test]
    async fn test_mocked_redis_init() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());
        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Call init
        let result = provider.init();
        assert!(result.is_ok());

        // Verify mock was called
        assert!(*mock_client.init_called.read().unwrap());
    }

    #[tokio::test]
    async fn test_mocked_redis_set_and_get() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());
        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Create a test resource
        let resource = TestResource {
            id: "test-1".to_string(),
            name: "Test Resource".to_string(),
            value: 42,
        };

        // Set the resource
        let set_result = provider.set("test-1", resource.clone(), 3600).await;
        assert!(set_result.is_ok());

        // Verify set operation was recorded
        assert_eq!(mock_client.set_keys.read().unwrap()[0], "test-1");
        assert_eq!(mock_client.set_ttls.read().unwrap()[0], 3600);

        // Get the resource
        let get_result = provider.get::<TestResource>("test-1").await;
        assert!(get_result.is_ok());

        let retrieved = get_result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), resource);

        // Verify get operation was recorded
        assert_eq!(mock_client.get_keys.read().unwrap()[0], "test-1");
    }

    #[tokio::test]
    async fn test_mocked_redis_get_nonexistent() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());
        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Get nonexistent resource
        let get_result = provider.get::<TestResource>("nonexistent").await;
        assert!(get_result.is_ok());

        let retrieved = get_result.unwrap();
        assert!(retrieved.is_none());

        // Verify get operation was recorded
        assert_eq!(mock_client.get_keys.read().unwrap()[0], "nonexistent");
    }

    #[tokio::test]
    async fn test_mocked_redis_delete() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());
        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Delete the key
        let delete_result = provider.delete("test-key").await;
        assert!(delete_result.is_ok());

        // Verify delete operation was recorded
        assert_eq!(mock_client.delete_keys.read().unwrap()[0], "test-key");
    }

    #[tokio::test]
    async fn test_mocked_redis_clear() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());
        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Clear the cache
        let clear_result = provider.clear().await;
        assert!(clear_result.is_ok());

        // Verify clear operation was recorded
        assert!(*mock_client.clear_called.read().unwrap());
    }

    #[tokio::test]
    async fn test_mocked_redis_exists() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());

        // Set up mock to return specific values
        mock_client.exists_return_values.write().unwrap().push(true);

        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Check existing key
        let exists_result = provider.exists("existing-key").await;
        assert!(exists_result.is_ok());
        assert!(exists_result.unwrap());

        // Verify exists operation was recorded
        assert_eq!(mock_client.exists_keys.read().unwrap()[0], "existing-key");

        // Now test a nonexistent key
        // Reset mocked return values
        *mock_client.exists_return_values.write().unwrap() = vec![false];

        // Check nonexistent key
        let exists_result = provider.exists("nonexistent-key").await;
        assert!(exists_result.is_ok());
        assert!(!exists_result.unwrap());

        // Verify exists operation was recorded
        assert_eq!(
            mock_client.exists_keys.read().unwrap()[1],
            "nonexistent-key"
        );
    }

    #[tokio::test]
    async fn test_mocked_redis_error_handling() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());

        // Set mock to fail
        *mock_client.should_fail.write().unwrap() = true;

        let provider = MockRedisCacheProvider::new(config, mock_client.clone());

        // Get should return error
        let get_result = provider.get::<TestResource>("test-key").await;
        assert!(get_result.is_err());
        assert_eq!(get_result.unwrap_err(), "Mock Redis connection error");

        // Create a test resource
        let resource = TestResource {
            id: "test-1".to_string(),
            name: "Test Resource".to_string(),
            value: 42,
        };

        // Set should return error
        let set_result = provider.set("test-key", resource, 3600).await;
        assert!(set_result.is_err());
        assert_eq!(set_result.unwrap_err(), "Mock Redis write error");
    }

    #[tokio::test]
    async fn test_mocked_redis_get_stats() {
        let config = RedisConfig::default();
        let mock_client = Arc::new(MockRedisClient::default());
        let provider = MockRedisCacheProvider::new(config, mock_client);

        // Get stats
        let stats_result = provider.get_stats().await;
        assert!(stats_result.is_ok());

        let stats = stats_result.unwrap();
        assert_eq!(stats["provider"], "redis");
        assert_eq!(stats["status"], "mocked");
        assert!(stats["config"].is_object());
    }
}
