use crate::error::CacheResult;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use std::time::Duration;

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
        self.to_string()
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
    /// Get a value from the cache
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get multiple values from the cache
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Set a value in the cache
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Set multiple values in the cache
    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Delete a value from the cache
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static;

    /// Delete multiple values from the cache
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    /// Check if a key exists in the cache
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static;

    /// Increment a counter in the cache
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static;

    /// Expire a key in the cache
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static;

    /// Clear the entire cache
    async fn clear(&self) -> CacheResult<()>;

    /// Get the health status of the cache
    async fn health_check(&self) -> CacheResult<()>;

    // List Operations

    /// Push a value to the right of a list
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Push multiple values to the right of a list
    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Push a value to the left of a list
    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Push multiple values to the left of a list
    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Pop a value from the right of a list
    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Pop a value from the left of a list
    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get a range of values from a list
    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get the length of a list
    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    /// Remove occurrences of a value from a list
    async fn list_remove<K, V>(&self, key: K, count: isize, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Trim a list to the specified range
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static;

    /// Set a value at the specified index in a list
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    // Hash Map Operations

    /// Get a field from a hash map
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Set a field in a hash map
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Get multiple fields from a hash map
    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Set multiple fields in a hash map
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Check if a field exists in a hash map
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static;

    /// Delete fields from a hash map
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static;

    /// Get all fields and values from a hash map
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get all field names from a hash map
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static;

    /// Get all values from a hash map
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Increment a numeric field in a hash map
    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static;

    /// Get the number of fields in a hash map
    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    // Set Operations

    /// Add values to a set
    async fn set_add<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Remove values from a set
    async fn set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Check if a value exists in a set
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Get all members of a set
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get the number of members in a set
    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    /// Get the intersection of multiple sets
    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Store the intersection of multiple sets in a destination set
    async fn set_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static;

    /// Get the union of multiple sets
    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Store the union of multiple sets in a destination set
    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static;

    /// Get the difference between sets (first set minus all others)
    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Store the difference between sets in a destination set
    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static;

    /// Get random members from a set
    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    // Sorted Set Operations

    /// Add members with scores to a sorted set
    async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Remove members from a sorted set
    async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Get the score of a member in a sorted set
    async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Increment the score of a member in a sorted set
    async fn zset_increment_score<K, V>(
        &self,
        key: K,
        member: &V,
        increment: f64,
    ) -> CacheResult<f64>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Get members by rank range (ordered by score)
    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get members with scores by rank range
    async fn zset_range_with_scores<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get members by score range
    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get members with scores by score range
    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        min: f64,
        max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get the rank of a member in a sorted set (0-based, ordered from low to high)
    async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Get the rank of a member in a sorted set (0-based, ordered from high to low)
    async fn zset_reverse_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Get the number of members in a sorted set
    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    /// Get the number of members in a sorted set within a score range
    async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    /// Get the intersection of multiple sorted sets with optional weights and aggregate function
    async fn zset_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static;

    /// Get the union of multiple sorted sets with optional weights and aggregate function
    async fn zset_union_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static;
}

/// Cache trait combining cache operations and cloning
pub trait Cache: CacheOperations + Clone {}

// Export Redis module
#[cfg(feature = "redis")]
pub mod redis;
