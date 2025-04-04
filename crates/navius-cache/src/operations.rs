use std::{collections::HashMap, fmt::Display, hash::Hash, time::Duration};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::{CacheError, CacheResult};

/// Cache key trait for converting types to cache keys
pub trait CacheKey: Display + Send + Sync {
    /// Convert the value to a string representation
    fn to_string(&self) -> String;
}

// Implement CacheKey for String
impl CacheKey for String {
    fn to_string(&self) -> String {
        self.clone()
    }
}

// Implement CacheKey for &str
impl CacheKey for &str {
    fn to_string(&self) -> String {
        (*self).to_string()
    }
}

/// Cache options for controlling cache behavior
#[derive(Debug, Clone)]
pub struct CacheOptions {
    /// Time to live for cache entries
    pub ttl: Option<Duration>,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self { ttl: None }
    }
}

impl CacheOptions {
    /// Create new cache options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the TTL
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

/// Cache operations trait for interacting with the cache
#[async_trait]
pub trait CacheOperations: Send + Sync + 'static {
    /// Get a value from the cache by key
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static;

    /// Get multiple values from the cache by keys
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static;

    /// Set a value in the cache
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Set multiple key-value pairs in the cache
    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + Eq + Hash + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Delete a key from the cache
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Delete multiple keys from the cache
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Check if a key exists in the cache
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Increment a counter in the cache
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Set an expiration time for a key
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Clear the entire cache
    async fn clear(&self) -> CacheResult<()>;

    /// Get the health status of the cache
    async fn health_check(&self) -> CacheResult<()>;

    // List Operations

    /// Push a value to the right of a list
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Push multiple values to the right of a list
    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Push a value to the left of a list
    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Push multiple values to the left of a list
    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Pop a value from the right of a list
    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Pop a value from the left of a list
    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get a range of values from a list
    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get the length of a list
    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Remove occurrences of a value from a list
    async fn list_remove<K, V>(&self, key: K, count: i32, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Trim a list to the specified range
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Set a value at the specified index in a list
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    // Hash Map Operations

    /// Get a field from a hash map
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Set a field in a hash map
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Get multiple fields from a hash map
    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Eq + Hash + Clone + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Set multiple fields in a hash map
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Eq + Hash + Clone + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Check if a field exists in a hash map
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static;

    /// Delete fields from a hash map
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static;

    /// Get all field-value pairs from a hash
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get all field names from a hash
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Get all values from a hash
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Increment a field in a hash by the given amount
    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static;

    /// Get the number of fields in a hash
    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    // Set Operations

    /// Add values to a set
    async fn set_add<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "set_add is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Remove values from a set
    async fn set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "set_remove is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Check if a value is in a set
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Get all members of a set
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get the number of members in a set
    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Get the intersection of multiple sets
    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Store the intersection of multiple sets into a destination key
    async fn set_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static;

    /// Get the union of multiple sets
    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Store the union of multiple sets in a destination key
    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static;

    /// Get the difference between multiple sets
    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "set_difference is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Store the difference between multiple sets into a destination key
    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static;

    /// Get random members from a set
    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    // Sorted Set Operations

    /// Add one or more members to a sorted set, or update its score if it already exists
    async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "zset_add is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Remove one or more members from a sorted set
    async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "zset_remove is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Get the score associated with the given member in a sorted set
    async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "zset_score is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Increment the score of a member in a sorted set
    async fn zset_increment<K, V>(&self, key: K, member: &V, increment: f64) -> CacheResult<f64>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "zset_increment is not implemented for MemoryCache".to_string(),
        ))
    }

    /// Get members by rank range (ordered by score)
    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get members with scores by rank range
    async fn zset_range_with_scores<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get members by score range (ordered by score)
    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get members with scores by score range
    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        min: f64,
        max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get the rank of a member in a sorted set (0-based, ascending order)
    async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Get the rank of a member in a sorted set (0-based, descending order)
    async fn zset_rev_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Get the number of members in a sorted set
    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Get the number of members in a sorted set within a score range
    async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Get members by score range (ordered by descending score)
    async fn zset_rev_range_by_score<K, V>(
        &self,
        key: K,
        max: f64,
        min: f64,
    ) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Get members with scores by score range (ordered by descending score)
    async fn zset_rev_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        max: f64,
        min: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Remove members from a sorted set within a rank range
    async fn zset_remove_range_by_rank<K>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Remove members from a sorted set within a score range
    async fn zset_remove_range_by_score<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Intersect sorted sets and store the resulting sorted set in a destination
    async fn zset_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static;

    /// Union sorted sets and store the resulting sorted set in a destination
    async fn zset_union_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static;
}

/// Cache trait combining cache operations and cloning
pub trait Cache: CacheOperations + Clone {}

// Add a new mock in-memory cache for examples and testing
pub mod memory {
    use super::*;
    use crate::error::{CacheError, CacheResult};
    use serde_json;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
        time::{Duration, Instant},
    };

    /// In-memory cache entry with expiration
    #[derive(Clone, Debug)]
    pub struct CacheEntry {
        /// The serialized value
        value: Vec<u8>,
        /// When the entry expires (None = no expiration)
        expires_at: Option<Instant>,
    }

    /// In-memory cache implementation for examples and testing
    #[derive(Debug, Clone)]
    pub struct MemoryCache {
        /// The underlying storage
        data: Arc<RwLock<HashMap<String, CacheEntry>>>,
        /// Default TTL for entries
        default_ttl: Option<Duration>,
        /// Key prefix
        prefix: String,
    }

    impl MemoryCache {
        /// Create a new memory cache
        pub fn new(prefix: String, default_ttl: Option<Duration>) -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
                default_ttl,
                prefix,
            }
        }

        /// Get the prefixed key
        fn prefixed_key<K: super::CacheKey>(&self, key: K) -> String {
            format!("{}{}", self.prefix, key.to_string())
        }

        /// Get TTL from options or default
        fn get_ttl(&self, options: Option<super::CacheOptions>) -> Option<Instant> {
            let ttl = options
                .and_then(|opts| opts.ttl)
                .or_else(|| self.default_ttl);

            ttl.map(|duration| Instant::now() + duration)
        }

        /// Check if an entry is expired
        fn is_expired(&self, entry: &CacheEntry) -> bool {
            if let Some(expires_at) = entry.expires_at {
                Instant::now() > expires_at
            } else {
                false
            }
        }

        /// Clean expired entries
        fn clean_expired(&self) {
            let mut data = self.data.write().unwrap();
            let now = Instant::now();
            data.retain(|_, entry| {
                if let Some(expires_at) = entry.expires_at {
                    expires_at > now
                } else {
                    true
                }
            });
        }
    }

    impl super::Cache for MemoryCache {}

    #[async_trait]
    impl super::CacheOperations for MemoryCache {
        async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let prefixed_key = self.prefixed_key(key);

            // Clean expired entries periodically
            self.clean_expired();

            let data = self.data.read().unwrap();
            if let Some(entry) = data.get(&prefixed_key) {
                if self.is_expired(entry) {
                    return Ok(None);
                }

                match serde_json::from_slice(&entry.value) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => Err(CacheError::SerializationError(format!(
                        "Failed to deserialize value: {}",
                        e
                    ))),
                }
            } else {
                Ok(None)
            }
        }

        async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: DeserializeOwned + Send + 'static,
        {
            let mut results = Vec::with_capacity(keys.len());

            for key in keys {
                results.push(self.get(key).await?);
            }

            Ok(results)
        }

        async fn set<K, V>(
            &self,
            key: K,
            value: &V,
            options: Option<super::CacheOptions>,
        ) -> CacheResult<()>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let prefixed_key = self.prefixed_key(key);
            let expires_at = self.get_ttl(options);

            let serialized = match serde_json::to_vec(value) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Err(CacheError::SerializationError(format!(
                        "Failed to serialize value: {}",
                        e
                    )));
                }
            };

            let entry = CacheEntry {
                value: serialized,
                expires_at,
            };

            let mut data = self.data.write().unwrap();
            data.insert(prefixed_key, entry);

            Ok(())
        }

        async fn set_many<K, V>(
            &self,
            entries: Vec<(K, V)>,
            options: Option<super::CacheOptions>,
        ) -> CacheResult<()>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            for (key, value) in entries {
                self.set(key, &value, options.clone()).await?;
            }

            Ok(())
        }

        async fn delete<K>(&self, key: K) -> CacheResult<bool>
        where
            K: super::CacheKey + 'static,
        {
            let prefixed_key = self.prefixed_key(key);
            let mut data = self.data.write().unwrap();
            Ok(data.remove(&prefixed_key).is_some())
        }

        async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
        {
            let mut count = 0;

            for key in keys {
                if self.delete(key).await? {
                    count += 1;
                }
            }

            Ok(count)
        }

        async fn exists<K>(&self, key: K) -> CacheResult<bool>
        where
            K: super::CacheKey + 'static,
        {
            let prefixed_key = self.prefixed_key(key);

            // Clean expired entries periodically
            self.clean_expired();

            let data = self.data.read().unwrap();
            if let Some(entry) = data.get(&prefixed_key) {
                if self.is_expired(entry) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }

        async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
        where
            K: super::CacheKey + 'static,
        {
            let prefixed_key = self.prefixed_key(key);
            let mut data = self.data.write().unwrap();

            let current_value: i64 = if let Some(entry) = data.get(&prefixed_key) {
                if self.is_expired(entry) {
                    0
                } else {
                    match serde_json::from_slice(&entry.value) {
                        Ok(v) => v,
                        Err(_) => 0,
                    }
                }
            } else {
                0
            };

            let new_value = current_value + amount;
            let serialized = serde_json::to_vec(&new_value).map_err(|e| {
                CacheError::SerializationError(format!("Failed to serialize value: {}", e))
            })?;

            let entry = CacheEntry {
                value: serialized,
                expires_at: data.get(&prefixed_key).and_then(|e| e.expires_at),
            };

            data.insert(prefixed_key, entry);

            Ok(new_value)
        }

        async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
        where
            K: super::CacheKey + 'static,
        {
            let prefixed_key = self.prefixed_key(key);
            let mut data = self.data.write().unwrap();

            if let Some(entry) = data.get_mut(&prefixed_key) {
                if self.is_expired(entry) {
                    return Ok(false);
                }

                entry.expires_at = Some(Instant::now() + ttl);
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn clear(&self) -> CacheResult<()> {
            let mut data = self.data.write().unwrap();
            data.clear();
            Ok(())
        }

        async fn health_check(&self) -> CacheResult<()> {
            // Memory cache is always healthy
            Ok(())
        }

        // List operations - stub implementations
        async fn list_push_right<K, V>(&self, _key: K, _value: &V) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_push_right is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_push_right_many<K, V>(&self, _key: K, _values: &[V]) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_push_right_many is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_push_left<K, V>(&self, _key: K, _value: &V) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_push_left is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_push_left_many<K, V>(&self, _key: K, _values: &[V]) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_push_left_many is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_pop_right<K, V>(&self, _key: K) -> CacheResult<Option<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_pop_right is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_pop_left<K, V>(&self, _key: K) -> CacheResult<Option<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_pop_left is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_range<K, V>(
            &self,
            _key: K,
            _start: isize,
            _stop: isize,
        ) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_range is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_length<K>(&self, _key: K) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_length is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_remove<K, V>(&self, _key: K, _count: i32, _value: &V) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_remove is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_trim<K>(&self, _key: K, _start: isize, _stop: isize) -> CacheResult<()>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_trim is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn list_set<K, V>(&self, _key: K, _index: isize, _value: &V) -> CacheResult<()>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "list_set is not implemented for MemoryCache".to_string(),
            ))
        }

        // Hash map operations - stub implementations
        async fn hash_get<K, F, V>(&self, _key: K, _field: F) -> CacheResult<Option<V>>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_get is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_set<K, F, V>(&self, _key: K, _field: F, _value: &V) -> CacheResult<bool>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_set is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_get_many<K, F, V>(
            &self,
            _key: K,
            _fields: Vec<F>,
        ) -> CacheResult<Vec<Option<V>>>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_get_many is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_set_many<K, F, V>(&self, _key: K, _entries: Vec<(F, V)>) -> CacheResult<()>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_set_many is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_exists<K, F>(&self, _key: K, _field: F) -> CacheResult<bool>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_exists is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_delete<K, F>(&self, _key: K, _fields: Vec<F>) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_delete is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_get_all<K, V>(&self, _key: K) -> CacheResult<Vec<(String, V)>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_get_all is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_keys<K>(&self, _key: K) -> CacheResult<Vec<String>>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_keys is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_values<K, V>(&self, _key: K) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_values is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_increment<K, F>(&self, _key: K, _field: F, _amount: i64) -> CacheResult<i64>
        where
            K: super::CacheKey + 'static,
            F: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_increment is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn hash_length<K>(&self, _key: K) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "hash_length is not implemented for MemoryCache".to_string(),
            ))
        }

        // Set operations - stub implementations
        async fn set_add<K, V>(&self, _key: K, _values: Vec<V>) -> CacheResult<usize>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_add is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_remove<K, V>(&self, _key: K, _values: Vec<V>) -> CacheResult<usize>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_remove is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_contains<K, V>(&self, _key: K, _value: &V) -> CacheResult<bool>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_contains is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_members<K, V>(&self, _key: K) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_members is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_length<K>(&self, _key: K) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_length is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_intersection<K, V>(&self, _keys: Vec<K>) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_intersection is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_intersection_store<K, D>(
            &self,
            _destination: D,
            _keys: Vec<K>,
        ) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            D: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_intersection_store is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_union<K, V>(&self, _keys: Vec<K>) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_union is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_union_store<K, D>(&self, _destination: D, _keys: Vec<K>) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            D: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_union_store is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_difference<K, V>(&self, _keys: Vec<K>) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: DeserializeOwned + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_difference is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_difference_store<K, D>(
            &self,
            _destination: D,
            _keys: Vec<K>,
        ) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            D: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_difference_store is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn set_random_members<K, V>(&self, _key: K, _count: usize) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "set_random_members is not implemented for MemoryCache".to_string(),
            ))
        }

        // Sorted set operations - stub implementations
        async fn zset_add<K, V>(&self, _key: K, _items: Vec<(f64, V)>) -> CacheResult<usize>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_add is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_remove<K, V>(&self, _key: K, _members: Vec<V>) -> CacheResult<usize>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_remove is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_score<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<f64>>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_score is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_increment<K, V>(
            &self,
            _key: K,
            _member: &V,
            _increment: f64,
        ) -> CacheResult<f64>
        where
            K: super::CacheKey + std::fmt::Debug + 'static,
            V: Serialize + Send + Sync + Clone + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_increment is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_range<K, V>(
            &self,
            _key: K,
            _start: isize,
            _stop: isize,
        ) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_range is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_range_with_scores<K, V>(
            &self,
            _key: K,
            _start: isize,
            _stop: isize,
        ) -> CacheResult<Vec<(V, f64)>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_range_with_scores is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_range_by_score<K, V>(
            &self,
            _key: K,
            _min: f64,
            _max: f64,
        ) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_range_by_score is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_range_by_score_with_scores<K, V>(
            &self,
            _key: K,
            _min: f64,
            _max: f64,
        ) -> CacheResult<Vec<(V, f64)>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_range_by_score_with_scores is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_rank<K, V>(&self, _key: K, _value: &V) -> CacheResult<Option<usize>>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_rank is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_rev_rank<K, V>(&self, _key: K, _value: &V) -> CacheResult<Option<usize>>
        where
            K: super::CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_rev_rank is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_length<K>(&self, _key: K) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_length is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_count<K>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_count is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_rev_range_by_score<K, V>(
            &self,
            _key: K,
            _max: f64,
            _min: f64,
        ) -> CacheResult<Vec<V>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_rev_range_by_score is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_rev_range_by_score_with_scores<K, V>(
            &self,
            _key: K,
            _max: f64,
            _min: f64,
        ) -> CacheResult<Vec<(V, f64)>>
        where
            K: super::CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_rev_range_by_score_with_scores is not implemented for MemoryCache"
                    .to_string(),
            ))
        }

        async fn zset_remove_range_by_rank<K>(
            &self,
            _key: K,
            _start: isize,
            _stop: isize,
        ) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_remove_range_by_rank is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_remove_range_by_score<K>(
            &self,
            _key: K,
            _min: f64,
            _max: f64,
        ) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_remove_range_by_score is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_intersection_store<K, D>(
            &self,
            _destination: D,
            _keys: Vec<K>,
            _weights: Option<Vec<f64>>,
            _aggregate: Option<String>,
        ) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            D: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_intersection_store is not implemented for MemoryCache".to_string(),
            ))
        }

        async fn zset_union_store<K, D>(
            &self,
            _destination: D,
            _keys: Vec<K>,
            _weights: Option<Vec<f64>>,
            _aggregate: Option<String>,
        ) -> CacheResult<usize>
        where
            K: super::CacheKey + 'static,
            D: super::CacheKey + 'static,
        {
            Err(CacheError::UnsupportedOperation(
                "zset_union_store is not implemented for MemoryCache".to_string(),
            ))
        }
    }
}
