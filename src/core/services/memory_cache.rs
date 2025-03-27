use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bincode::config::standard;
use bincode::{Decode, Encode};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::debug;

use crate::core::services::cache_provider::{
    CacheConfig, CacheError, CacheFactory, CacheOperations, CacheProvider, CacheStats,
    DynCacheOperations, EvictionPolicy, TypedCache, TypedCacheFactory,
};

/// Cache entry with metadata
struct CacheEntry {
    /// Serialized value
    value: Vec<u8>,
    /// Creation time
    created_at: Instant,
    /// Time to live
    ttl: Option<Duration>,
    /// Last access time
    last_accessed: Instant,
    /// Hit count
    hit_count: u64,
}

impl CacheEntry {
    /// Create a new entry
    fn new(value: Vec<u8>, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            ttl,
            last_accessed: now,
            hit_count: 0,
        }
    }

    /// Check if the entry is expired
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    /// Access the entry and update last_accessed
    fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.hit_count += 1;
    }
}

/// In-memory cache implementation
pub struct InMemoryCache {
    name: String,
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
    _cleanup_task: Mutex<Option<JoinHandle<()>>>,
}

/// Typed cache for in-memory implementation
pub struct InMemoryTypedCache<T> {
    cache: Arc<InMemoryCache>,
    _marker: PhantomData<T>,
}

impl<T> InMemoryTypedCache<T>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    fn new(cache: Arc<InMemoryCache>) -> Self {
        Self {
            cache,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<T> TypedCache<T> for InMemoryTypedCache<T>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    async fn get(&self, key: &str) -> Result<Option<T>, CacheError> {
        let entry_opt = {
            let mut entries = self.cache.entries.write().unwrap();
            let mut stats = self.cache.stats.write().unwrap();

            if let Some(entry) = entries.get_mut(key) {
                if entry.is_expired() {
                    entries.remove(key);
                    stats.evictions += 1;
                    stats.misses += 1;
                    None
                } else {
                    entry.access();
                    stats.hits += 1;
                    Some(entry.value.clone())
                }
            } else {
                stats.misses += 1;
                None
            }
        };

        if let Some(value) = entry_opt {
            match bincode::decode_from_slice::<T, _>(&value, standard()) {
                Ok((val, _)) => Ok(Some(val)),
                Err(e) => Err(CacheError::Deserialization(e.to_string())),
            }
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError> {
        // Serialize value
        let serialized = match bincode::encode_to_vec(&value, standard()) {
            Ok(val) => val,
            Err(e) => return Err(CacheError::Serialization(e.to_string())),
        };

        let mut entries = self.cache.entries.write().unwrap();

        // Check capacity
        if let Some(capacity) = self.cache.config.capacity {
            if entries.len() >= capacity && !entries.contains_key(key) {
                self.cache.ensure_capacity(&mut entries)?;
            }
        }

        // Set entry with effective TTL
        let effective_ttl = ttl.or(self.cache.config.default_ttl);
        entries.insert(key.to_string(), CacheEntry::new(serialized, effective_ttl));

        // Update stats
        self.cache.stats.write().unwrap().size = entries.len();

        Ok(())
    }

    async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, Option<T>>, CacheError> {
        let mut results = HashMap::with_capacity(keys.len());

        for key in keys {
            let val = self.get(key).await?;
            results.insert(key.to_string(), val);
        }

        Ok(results)
    }

    async fn set_many(
        &self,
        items: HashMap<String, T>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        for (key, value) in items {
            self.set(&key, value, ttl).await?;
        }

        Ok(())
    }
}

// Implementation of the TypedCacheFactory for InMemoryTypedCache
impl<T> TypedCacheFactory<T> for InMemoryTypedCache<T>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    fn create_typed_cache(&self) -> Box<dyn TypedCache<T>> {
        Box::new(Self::new(self.cache.clone()))
    }
}

impl InMemoryCache {
    /// Create a new in-memory cache
    pub fn new(config: CacheConfig) -> Self {
        let entries = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(RwLock::new(CacheStats {
            size: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            capacity: config.capacity,
            custom_metrics: HashMap::new(),
        }));

        let cache = Self {
            name: config.name.clone(),
            config,
            entries: Arc::clone(&entries),
            stats: Arc::clone(&stats),
            _cleanup_task: Mutex::new(None),
        };

        // Start cleanup task if TTL is configured
        if cache.config.default_ttl.is_some() {
            cache.start_cleanup_task();
        }

        cache
    }

    /// Start background cleanup of expired entries
    fn start_cleanup_task(&self) {
        let entries = Arc::clone(&self.entries);
        let stats = Arc::clone(&self.stats);
        let interval = Duration::from_secs(60); // Clean every minute

        let handle = tokio::spawn(async move {
            let mut interval_timer = time::interval(interval);

            loop {
                interval_timer.tick().await;

                let mut entries_guard = entries.write().unwrap();
                let mut stats_guard = stats.write().unwrap();

                let before_count = entries_guard.len();

                // Remove expired entries
                entries_guard.retain(|_, entry| {
                    let keep = !entry.is_expired();
                    if !keep {
                        stats_guard.evictions += 1;
                    }
                    keep
                });

                stats_guard.size = entries_guard.len();

                let removed = before_count - entries_guard.len();
                if removed > 0 {
                    debug!("Removed {} expired cache entries", removed);
                }
            }
        });

        let mut cleanup_task = self._cleanup_task.try_lock().unwrap();
        *cleanup_task = Some(handle);
    }

    /// Ensure capacity by evicting entries according to policy
    fn ensure_capacity(&self, entries: &mut HashMap<String, CacheEntry>) -> Result<(), CacheError> {
        let capacity = match self.config.capacity {
            Some(cap) => cap,
            None => return Ok(()),
        };

        if entries.len() < capacity {
            return Ok(());
        }

        // Choose entry to evict based on policy
        match self.config.eviction_policy {
            EvictionPolicy::None => {
                return Err(CacheError::Capacity(format!(
                    "Cache {} is full (capacity: {})",
                    self.name, capacity
                )));
            }
            EvictionPolicy::LRU => {
                // Find least recently used entry
                if let Some(key) = entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.last_accessed)
                    .map(|(k, _)| k.clone())
                {
                    entries.remove(&key);
                    self.stats.write().unwrap().evictions += 1;
                }
            }
            EvictionPolicy::LFU => {
                // Find least frequently used entry
                if let Some(key) = entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.hit_count)
                    .map(|(k, _)| k.clone())
                {
                    entries.remove(&key);
                    self.stats.write().unwrap().evictions += 1;
                }
            }
            EvictionPolicy::FIFO => {
                // Find oldest entry
                if let Some(key) = entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(k, _)| k.clone())
                {
                    entries.remove(&key);
                    self.stats.write().unwrap().evictions += 1;
                }
            }
            EvictionPolicy::TTL => {
                // Find entry closest to expiration or already expired
                let now = Instant::now();
                if let Some(key) = entries
                    .iter()
                    .filter_map(|(k, entry)| {
                        entry.ttl.map(|ttl| {
                            let remaining = if entry.created_at + ttl > now {
                                entry.created_at + ttl - now
                            } else {
                                Duration::from_secs(0)
                            };
                            (k.clone(), remaining)
                        })
                    })
                    .min_by_key(|(_, remaining)| *remaining)
                    .map(|(k, _)| k)
                {
                    entries.remove(&key);
                    self.stats.write().unwrap().evictions += 1;
                } else {
                    // If no TTL set, fall back to LRU
                    if let Some(key) = entries
                        .iter()
                        .min_by_key(|(_, entry)| entry.last_accessed)
                        .map(|(k, _)| k.clone())
                    {
                        entries.remove(&key);
                        self.stats.write().unwrap().evictions += 1;
                    }
                }
            }
            EvictionPolicy::Random => {
                // Just remove the first entry we find
                if let Some(key) = entries.keys().next().cloned() {
                    entries.remove(&key);
                    self.stats.write().unwrap().evictions += 1;
                }
            }
        }

        Ok(())
    }
}

impl CacheFactory for InMemoryCache {
    fn for_type<T>(&self) -> Box<dyn TypedCacheFactory<T>>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        Box::new(InMemoryTypedCache::new(Arc::new(self.clone())))
    }
}

#[async_trait]
impl DynCacheOperations for InMemoryCache {
    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let mut entries = self.entries.write().unwrap();
        let removed = entries.remove(key).is_some();

        if removed {
            let mut stats = self.stats.write().unwrap();
            stats.size = entries.len();
        }

        Ok(removed)
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut entries = self.entries.write().unwrap();
        entries.clear();

        let mut stats = self.stats.write().unwrap();
        stats.size = 0;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let entries = self.entries.read().unwrap();

        if let Some(entry) = entries.get(key) {
            if entry.is_expired() {
                // Key exists but is expired, consider it non-existent
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn delete_many(&self, keys: &[&str]) -> Result<usize, CacheError> {
        let mut entries = self.entries.write().unwrap();
        let mut deleted = 0;

        for key in keys {
            if entries.remove(*key).is_some() {
                deleted += 1;
            }
        }

        if deleted > 0 {
            let mut stats = self.stats.write().unwrap();
            stats.size = entries.len();
        }

        Ok(deleted)
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64, CacheError> {
        let mut entries = self.entries.write().unwrap();

        // First, check if we have the key
        let current_value = if let Some(entry) = entries.get(key) {
            if !entry.is_expired() {
                // Try to deserialize as i64
                match bincode::decode_from_slice::<i64, _>(&entry.value, standard()) {
                    Ok((value, _)) => Some((value, entry.ttl)),
                    Err(_) => {
                        return Err(CacheError::Type(format!(
                            "Value for key '{}' is not an integer",
                            key
                        )));
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Now handle the value
        match current_value {
            Some((value, ttl)) => {
                let new_value = value + delta;

                // Serialize and store
                let serialized = bincode::encode_to_vec(&new_value, standard())
                    .map_err(|e| CacheError::Serialization(e.to_string()))?;

                // Update entry
                entries.insert(key.to_string(), CacheEntry::new(serialized, ttl));

                Ok(new_value)
            }
            None => {
                // Key doesn't exist or is expired - initialize with delta
                let serialized = bincode::encode_to_vec(&delta, standard())
                    .map_err(|e| CacheError::Serialization(e.to_string()))?;

                // Create new entry with default TTL
                entries.insert(
                    key.to_string(),
                    CacheEntry::new(serialized, self.config.default_ttl),
                );

                // Update stats
                let mut stats = self.stats.write().unwrap();
                stats.size = entries.len();

                Ok(delta)
            }
        }
    }

    fn stats(&self) -> Result<CacheStats, CacheError> {
        Ok(self.stats.read().unwrap().clone())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Cast to Any for dynamic type conversion
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for InMemoryCache {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            config: self.config.clone(),
            entries: Arc::clone(&self.entries),
            stats: Arc::clone(&self.stats),
            _cleanup_task: Mutex::new(None),
        }
    }
}

/// In-memory cache provider
pub struct InMemoryCacheProvider;

impl InMemoryCacheProvider {
    /// Create a new provider instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CacheProvider for InMemoryCacheProvider {
    async fn create_cache(
        &self,
        config: CacheConfig,
    ) -> Result<Box<dyn DynCacheOperations>, CacheError> {
        Ok(Box::new(InMemoryCache::new(config)))
    }

    fn supports(&self, config: &CacheConfig) -> bool {
        config.provider == "memory" || config.provider.is_empty()
    }

    fn name(&self) -> &str {
        "memory"
    }

    fn capabilities(&self) -> HashMap<String, String> {
        let mut caps = HashMap::new();
        caps.insert("type".to_string(), "memory".to_string());
        caps.insert("persistent".to_string(), "false".to_string());
        caps.insert("distributed".to_string(), "false".to_string());
        caps.insert("thread_safe".to_string(), "true".to_string());
        caps
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// This is a trait extension to allow cloning Box<dyn DynCacheOperations>
pub trait ClonableDynCacheOperations: DynCacheOperations {
    fn clone_box(&self) -> Box<dyn DynCacheOperations>;
}

// Implement ClonableDynCacheOperations for any T that implements DynCacheOperations + Clone
impl<T: 'static + DynCacheOperations + Clone> ClonableDynCacheOperations for T {
    fn clone_box(&self) -> Box<dyn DynCacheOperations> {
        Box::new(self.clone())
    }
}

// Implement Clone for Box<dyn DynCacheOperations> using the ClonableDynCacheOperations trait
impl Clone for Box<dyn DynCacheOperations> {
    fn clone(&self) -> Self {
        if let Some(clonable) = self
            .as_any()
            .downcast_ref::<Box<dyn ClonableDynCacheOperations>>()
        {
            clonable.clone_box()
        } else if let Some(mem_cache) = self.as_any().downcast_ref::<InMemoryCache>() {
            Box::new(mem_cache.clone())
        } else {
            // Fallback to panic - in a real-world scenario, you'd want a better solution
            panic!("Unable to clone Box<dyn DynCacheOperations>");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_operations() {
        let config = CacheConfig {
            name: "test".to_string(),
            provider: "memory".to_string(),
            capacity: Some(100),
            default_ttl: Some(Duration::from_secs(3600)),
            eviction_policy: EvictionPolicy::LRU,
            provider_config: HashMap::new(),
        };

        let cache = InMemoryCache::new(config);
        let typed_cache = cache.for_type::<String>().create_typed_cache();

        // Set value
        typed_cache
            .set("key1", "value1".to_string(), None)
            .await
            .unwrap();

        // Get value
        let value = typed_cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Set with TTL
        typed_cache
            .set(
                "key2",
                "value2".to_string(),
                Some(Duration::from_millis(50)),
            )
            .await
            .unwrap();

        // Check exists
        assert!(cache.exists("key2").await.unwrap());

        // Wait for expiration
        sleep(Duration::from_millis(100)).await;

        // Should be expired
        assert!(!cache.exists("key2").await.unwrap());
        assert_eq!(typed_cache.get("key2").await.unwrap(), None);

        // Delete key
        assert!(cache.delete("key1").await.unwrap());
        assert!(!cache.exists("key1").await.unwrap());

        // Stats should be accurate
        let stats = cache.stats().unwrap();
        assert_eq!(stats.hits, 1); // One hit from get("key1")
        assert_eq!(stats.misses, 1); // One miss from get("key2") after expiration
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let config = CacheConfig {
            name: "test-ttl".to_string(),
            provider: "memory".to_string(),
            capacity: Some(10),
            default_ttl: None, // No default TTL
            eviction_policy: EvictionPolicy::LRU,
            provider_config: HashMap::new(),
        };

        let cache = InMemoryCache::new(config);
        let typed_cache = cache.for_type::<i32>().create_typed_cache();

        // Set value with short TTL
        typed_cache
            .set("key1", 123, Some(Duration::from_millis(50)))
            .await
            .unwrap();

        // Value should exist
        assert_eq!(typed_cache.get("key1").await.unwrap(), Some(123));

        // Wait for expiration
        sleep(Duration::from_millis(100)).await;

        // Value should be gone
        assert_eq!(typed_cache.get("key1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_capacity_and_eviction() {
        let config = CacheConfig {
            name: "test-eviction".to_string(),
            provider: "memory".to_string(),
            capacity: Some(3), // Small capacity for testing
            default_ttl: None,
            eviction_policy: EvictionPolicy::LRU,
            provider_config: HashMap::new(),
        };

        let cache = InMemoryCache::new(config);
        let typed_cache = cache.for_type::<i32>().create_typed_cache();

        // Fill cache
        typed_cache.set("key1", 1, None).await.unwrap();
        typed_cache.set("key2", 2, None).await.unwrap();
        typed_cache.set("key3", 3, None).await.unwrap();

        // Access key1 to make it most recently used
        typed_cache.get("key1").await.unwrap();

        // Add one more to trigger eviction (key2 should be evicted as LRU)
        typed_cache.set("key4", 4, None).await.unwrap();

        // key2 should be gone, others should exist
        assert_eq!(typed_cache.get("key1").await.unwrap(), Some(1));
        assert_eq!(typed_cache.get("key2").await.unwrap(), None);
        assert_eq!(typed_cache.get("key3").await.unwrap(), Some(3));
        assert_eq!(typed_cache.get("key4").await.unwrap(), Some(4));
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let config = CacheConfig::default();
        let cache = InMemoryCache::new(config);
        let typed_cache = cache.for_type::<String>().create_typed_cache();

        // Batch set
        let mut items = HashMap::new();
        items.insert("key1".to_string(), "value1".to_string());
        items.insert("key2".to_string(), "value2".to_string());
        items.insert("key3".to_string(), "value3".to_string());

        typed_cache.set_many(items, None).await.unwrap();

        // Batch get
        let results = typed_cache
            .get_many(&["key1", "key2", "key3", "key4"])
            .await
            .unwrap();

        assert_eq!(results.get("key1").unwrap(), &Some("value1".to_string()));
        assert_eq!(results.get("key2").unwrap(), &Some("value2".to_string()));
        assert_eq!(results.get("key3").unwrap(), &Some("value3".to_string()));
        assert_eq!(results.get("key4").unwrap(), &None);

        // Batch delete
        let deleted = cache.delete_many(&["key1", "key3"]).await.unwrap();
        assert_eq!(deleted, 2);

        // Verify keys are gone
        assert!(!cache.exists("key1").await.unwrap());
        assert!(cache.exists("key2").await.unwrap());
        assert!(!cache.exists("key3").await.unwrap());
    }

    #[tokio::test]
    async fn test_provider() {
        let provider = InMemoryCacheProvider::new();
        assert_eq!(provider.name(), "memory");

        let config = CacheConfig {
            name: "test-provider".to_string(),
            provider: "memory".to_string(),
            capacity: Some(10),
            default_ttl: Some(Duration::from_secs(60)),
            eviction_policy: EvictionPolicy::LRU,
            provider_config: HashMap::new(),
        };

        assert!(provider.supports(&config));

        let cache = provider.create_cache(config).await.unwrap();
        assert_eq!(cache.name(), "test-provider");

        // Verify stats
        let stats = cache.stats().unwrap();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.capacity, Some(10));
    }
}
