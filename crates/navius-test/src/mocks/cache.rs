use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// Define the CacheConnection trait
pub trait CacheConnection: Send + Sync {
    fn get<K, V>(&self, key: K) -> Result<Option<V>, Box<dyn std::error::Error + Send + Sync>>
    where
        K: Send + Sync + Debug + 'static,
        V: DeserializeOwned + Send + Sync + Debug + 'static;

    fn set<K, V>(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        K: Send + Sync + Debug + 'static,
        V: Serialize + Send + Sync + Debug + 'static;

    fn delete<K>(&self, key: K) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
    where
        K: Send + Sync + Debug + 'static;

    fn exists<K>(&self, key: K) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
    where
        K: Send + Sync + Debug + 'static;
}

pub trait CacheConnectionManager: Send + Sync {
    fn get_connection(
        &self,
    ) -> Result<Arc<dyn MockableCacheConnection>, Box<dyn std::error::Error + Send + Sync>>;
    fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// Implement a simple in-memory cache for testing
pub struct InMemoryCache {
    data: Arc<RwLock<HashMap<String, (Box<dyn std::any::Any + Send + Sync>, Option<Instant>)>>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_expiry(&self, key: &str) -> bool {
        let data = self.data.read().await;
        if let Some((_, expiry)) = data.get(key) {
            if let Some(expiry_time) = expiry {
                return Instant::now() < *expiry_time;
            }
        }
        true
    }
}

#[async_trait]
pub trait AsyncCache {
    async fn get<T: 'static + Send + Sync + Clone>(
        &self,
        key: &str,
    ) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set<T: 'static + Send + Sync + Clone>(
        &self,
        key: &str,
        value: T,
        ttl: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn clear(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_many<T: 'static + Send + Sync + Clone>(
        &self,
        keys: Vec<&str>,
    ) -> Result<Vec<Option<T>>, Box<dyn std::error::Error + Send + Sync>>;
    async fn increment(
        &self,
        key: &str,
        amount: i64,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;
    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl AsyncCache for InMemoryCache {
    async fn get<T: 'static + Send + Sync + Clone>(
        &self,
        key: &str,
    ) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await;

        if let Some((value, expiry)) = data.get(key) {
            // Check if the value has expired
            if let Some(expiry_time) = expiry {
                if Instant::now() > *expiry_time {
                    return Ok(None);
                }
            }

            // Downcast to the requested type
            if let Some(typed_value) = value.downcast_ref::<T>() {
                return Ok(Some(typed_value.clone()));
            }
        }

        Ok(None)
    }

    async fn set<T: 'static + Send + Sync + Clone>(
        &self,
        key: &str,
        value: T,
        ttl: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;

        // Calculate expiry time if TTL is provided
        let expiry = ttl.map(|duration| Instant::now() + duration);

        // Store the value with its expiry time
        data.insert(key.to_string(), (Box::new(value), expiry));

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;
        Ok(data.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await;

        if let Some((_, expiry)) = data.get(key) {
            // Check if the value has expired
            if let Some(expiry_time) = expiry {
                if Instant::now() > *expiry_time {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    async fn clear(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }

    async fn get_many<T: 'static + Send + Sync + Clone>(
        &self,
        keys: Vec<&str>,
    ) -> Result<Vec<Option<T>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::with_capacity(keys.len());

        for key in keys {
            results.push(self.get::<T>(key).await?);
        }

        Ok(results)
    }

    async fn increment(
        &self,
        key: &str,
        amount: i64,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;

        let new_value = if let Some((value, expiry)) = data.get(key) {
            // Check if the value has expired
            if let Some(expiry_time) = expiry {
                if Instant::now() > *expiry_time {
                    amount
                } else if let Some(int_value) = value.downcast_ref::<i64>() {
                    int_value + amount
                } else {
                    return Err("Value is not an integer".into());
                }
            } else if let Some(int_value) = value.downcast_ref::<i64>() {
                int_value + amount
            } else {
                return Err("Value is not an integer".into());
            }
        } else {
            amount
        };

        // Store updated value
        data.insert(key.to_string(), (Box::new(new_value), None));

        Ok(new_value)
    }

    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // InMemoryCache is always healthy
        Ok(())
    }
}

// This trait provides non-generic methods that will be used to implement mocks
pub trait MockableCacheConnection: Send + Sync + Debug {
    // Get a string value
    fn get_string(
        &self,
        key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;

    // Set a string value
    fn set_string(
        &self,
        key: &str,
        value: &str,
        ttl: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    // Delete a key
    fn delete_key(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    // Check if a key exists
    fn key_exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

// Mock implementation of MockableCacheConnection
#[derive(Debug, Default)]
pub struct MockCacheConnection {
    get_string_results:
        std::sync::Mutex<Vec<Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>>>,
    set_string_results: std::sync::Mutex<Vec<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
    delete_key_results:
        std::sync::Mutex<Vec<Result<bool, Box<dyn std::error::Error + Send + Sync>>>>,
    key_exists_results:
        std::sync::Mutex<Vec<Result<bool, Box<dyn std::error::Error + Send + Sync>>>>,
}

impl MockCacheConnection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect_get_string(
        &self,
        _key: &str,
        result: Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>,
    ) {
        self.get_string_results.lock().unwrap().push(result);
    }

    pub fn expect_set_string(
        &self,
        _key: &str,
        _value: &str,
        _ttl: Option<Duration>,
        result: Result<(), Box<dyn std::error::Error + Send + Sync>>,
    ) {
        self.set_string_results.lock().unwrap().push(result);
    }

    pub fn expect_delete_key(
        &self,
        _key: &str,
        result: Result<bool, Box<dyn std::error::Error + Send + Sync>>,
    ) {
        self.delete_key_results.lock().unwrap().push(result);
    }

    pub fn expect_key_exists(
        &self,
        _key: &str,
        result: Result<bool, Box<dyn std::error::Error + Send + Sync>>,
    ) {
        self.key_exists_results.lock().unwrap().push(result);
    }
}

impl MockableCacheConnection for MockCacheConnection {
    fn get_string(
        &self,
        _key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(result) = self.get_string_results.lock().unwrap().pop() {
            result
        } else {
            Err("Not implemented".into())
        }
    }

    fn set_string(
        &self,
        _key: &str,
        _value: &str,
        _ttl: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(result) = self.set_string_results.lock().unwrap().pop() {
            result
        } else {
            Err("Not implemented".into())
        }
    }

    fn delete_key(&self, _key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(result) = self.delete_key_results.lock().unwrap().pop() {
            result
        } else {
            Err("Not implemented".into())
        }
    }

    fn key_exists(&self, _key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(result) = self.key_exists_results.lock().unwrap().pop() {
            result
        } else {
            Err("Not implemented".into())
        }
    }
}

// Mock implementation of CacheConnectionManager
#[derive(Debug, Default)]
pub struct MockCacheConnectionManager {
    connection_results: std::sync::Mutex<
        Vec<Result<Arc<dyn MockableCacheConnection>, Box<dyn std::error::Error + Send + Sync>>>,
    >,
    health_check_results:
        std::sync::Mutex<Vec<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
}

impl MockCacheConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect_get_connection(
        &self,
        result: Result<Arc<dyn MockableCacheConnection>, Box<dyn std::error::Error + Send + Sync>>,
    ) {
        self.connection_results.lock().unwrap().push(result);
    }

    pub fn expect_health_check(
        &self,
        result: Result<(), Box<dyn std::error::Error + Send + Sync>>,
    ) {
        self.health_check_results.lock().unwrap().push(result);
    }
}

impl CacheConnectionManager for MockCacheConnectionManager {
    fn get_connection(
        &self,
    ) -> Result<Arc<dyn MockableCacheConnection>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(result) = self.connection_results.lock().unwrap().pop() {
            result
        } else {
            Err("Not implemented".into())
        }
    }

    fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(result) = self.health_check_results.lock().unwrap().pop() {
            result
        } else {
            Err("Not implemented".into())
        }
    }
}

pub fn set_up_mock_cache(_cache_connection: &MockCacheConnection) {
    // Implementation would go here
}
