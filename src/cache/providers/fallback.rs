use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::interval;
use tracing::{error, info, warn};

use super::{
    CacheProvider,
    memory::MemoryCacheProvider,
    redis::{RedisCacheProvider, RedisConfig},
};
use crate::config::{self, AppConfig};
use crate::utils::api_resource::ApiResource;

/// Configuration for the fallback cache provider
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Redis configuration
    pub redis_config: RedisConfig,
    /// In-memory cache configuration
    pub memory_max_capacity: u64,
    pub memory_ttl_seconds: u64,
    /// How often to try to reconnect to Redis (in seconds)
    pub reconnect_interval_seconds: u64,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        // Get the app config to use its cache settings
        let app_config = match config::get_config() {
            config => config,
        };

        Self {
            redis_config: RedisConfig::default(),
            memory_max_capacity: app_config.cache.max_capacity,
            memory_ttl_seconds: app_config.cache.ttl_seconds,
            reconnect_interval_seconds: app_config.cache.reconnect_interval_seconds,
        }
    }
}

// Create a new constructor for FallbackConfig from AppConfig
impl FallbackConfig {
    pub fn from_app_config(app_config: &AppConfig) -> Self {
        Self {
            redis_config: RedisConfig::default(),
            memory_max_capacity: app_config.cache.max_capacity,
            memory_ttl_seconds: app_config.cache.ttl_seconds,
            reconnect_interval_seconds: app_config.cache.reconnect_interval_seconds,
        }
    }
}

/// Simple string value that implements ApiResource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringValue(pub String);

impl ApiResource for StringValue {
    type Id = String;

    fn resource_type() -> &'static str {
        "string_value"
    }

    fn api_name() -> &'static str {
        "CacheTest"
    }
}

/// Fallback cache provider that tries Redis first, then falls back to memory
pub struct FallbackCacheProvider {
    redis: Arc<RedisCacheProvider>,
    memory: Arc<MemoryCacheProvider>,
    using_fallback: AtomicBool,
    last_redis_failure: Mutex<Option<Instant>>,
    reconnect_lock: AsyncMutex<()>,
    config: FallbackConfig,
}

impl FallbackCacheProvider {
    /// Create a new fallback cache provider
    pub fn new(config: FallbackConfig) -> Self {
        Self {
            redis: Arc::new(RedisCacheProvider::new(config.redis_config.clone())),
            memory: Arc::new(MemoryCacheProvider::new(
                config.memory_max_capacity,
                config.memory_ttl_seconds,
            )),
            using_fallback: AtomicBool::new(false),
            last_redis_failure: Mutex::new(None),
            reconnect_lock: AsyncMutex::new(()),
            config,
        }
    }

    /// Start the reconnection task
    pub async fn start_reconnect_task(&self) {
        let provider = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(
                provider.config.reconnect_interval_seconds,
            ));

            loop {
                interval.tick().await;
                provider.try_reconnect().await;
            }
        });
    }

    /// Try to reconnect to Redis if we're in fallback mode
    async fn try_reconnect(&self) {
        // Skip if we're not in fallback mode
        if !self.using_fallback.load(Ordering::Relaxed) {
            return;
        }

        // Get the lock to ensure only one reconnection attempt happens at a time
        let _guard = self.reconnect_lock.lock().await;

        info!("🔄 Attempting to reconnect to Redis...");

        // Try to initialize Redis
        match self.redis.init() {
            Ok(_) => {
                // Test Redis with a simple operation
                let test_key = "reconnect_test";
                let test_value = StringValue("test".to_string());

                match self.redis.set(test_key, test_value, 5).await {
                    Ok(_) => {
                        // Successfully reconnected
                        self.using_fallback.store(false, Ordering::SeqCst);
                        *self.last_redis_failure.lock().unwrap() = None;
                        info!(
                            "✅ Successfully reconnected to Redis, switching back from fallback mode"
                        );
                    }
                    Err(e) => {
                        // Still can't connect
                        warn!("❌ Redis reconnection test failed: {}", e);
                    }
                }
            }
            Err(e) => {
                // Still can't initialize
                warn!("❌ Failed to reconnect to Redis: {}", e);
            }
        }
    }

    /// Handle Redis failure by switching to fallback mode
    fn handle_redis_failure(&self, error: &str) {
        // Only log the first failure
        let mut last_failure = self.last_redis_failure.lock().unwrap();
        let now = Instant::now();

        if !self.using_fallback.load(Ordering::Relaxed) {
            // First failure, log it
            error!(
                "❌ Redis operation failed, switching to memory fallback: {}",
                error
            );
            self.using_fallback.store(true, Ordering::SeqCst);
            *last_failure = Some(now);
        } else if let Some(last_time) = *last_failure {
            // Only log if it's been a while since the last failure
            if now.duration_since(last_time) > Duration::from_secs(60) {
                warn!("⚠️ Redis still unavailable after 60s, continuing with memory fallback");
                *last_failure = Some(now);
            }
        }
    }
}

// Allow cloning by implementing Clone manually
impl Clone for FallbackCacheProvider {
    fn clone(&self) -> Self {
        Self {
            redis: self.redis.clone(),
            memory: self.memory.clone(),
            using_fallback: AtomicBool::new(self.using_fallback.load(Ordering::Relaxed)),
            last_redis_failure: Mutex::new(*self.last_redis_failure.lock().unwrap()),
            reconnect_lock: AsyncMutex::new(()),
            config: self.config.clone(),
        }
    }
}

#[async_trait]
impl CacheProvider for FallbackCacheProvider {
    fn init(&self) -> Result<(), String> {
        // Initialize both providers
        match self.redis.init() {
            Ok(_) => {
                // Redis initialized successfully, use it as primary
                self.using_fallback.store(false, Ordering::SeqCst);
            }
            Err(e) => {
                // Redis failed to initialize, use memory as fallback
                warn!(
                    "❌ Redis initialization failed, using memory fallback: {}",
                    e
                );
                self.using_fallback.store(true, Ordering::SeqCst);
                *self.last_redis_failure.lock().unwrap() = Some(Instant::now());
            }
        }

        // Always initialize memory cache as fallback
        self.memory.init()?;

        Ok(())
    }

    async fn set<T: ApiResource>(
        &self,
        key: &str,
        value: T,
        ttl_seconds: u64,
    ) -> Result<(), String> {
        // Try Redis first if not in fallback mode
        if !self.using_fallback.load(Ordering::Relaxed) {
            match self.redis.set(key, value.clone(), ttl_seconds).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    self.handle_redis_failure(&e);
                    // Fall through to memory cache
                }
            }
        }

        // Use memory cache as fallback
        self.memory.set(key, value, ttl_seconds).await
    }

    async fn get<T: ApiResource>(&self, key: &str) -> Result<Option<T>, String> {
        // Try Redis first if not in fallback mode
        if !self.using_fallback.load(Ordering::Relaxed) {
            match self.redis.get::<T>(key).await {
                Ok(value) => return Ok(value),
                Err(e) => {
                    self.handle_redis_failure(&e);
                    // Fall through to memory cache
                }
            }
        }

        // Use memory cache as fallback
        self.memory.get(key).await
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        // Try Redis first if not in fallback mode
        if !self.using_fallback.load(Ordering::Relaxed) {
            match self.redis.delete(key).await {
                Ok(_) => {}
                Err(e) => {
                    self.handle_redis_failure(&e);
                    // Continue with memory cache regardless
                }
            }
        }

        // Always try to delete from memory cache too
        // Ignore errors from memory cache since it might not support direct key deletion
        let _ = self.memory.delete(key).await;

        Ok(())
    }

    async fn clear(&self) -> Result<(), String> {
        // Try Redis first if not in fallback mode
        if !self.using_fallback.load(Ordering::Relaxed) {
            match self.redis.clear().await {
                Ok(_) => {}
                Err(e) => {
                    self.handle_redis_failure(&e);
                    // Continue with memory cache regardless
                }
            }
        }

        // Always try to clear memory cache too
        // Ignore errors from memory cache since it might not support clear
        let _ = self.memory.clear().await;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, String> {
        // Try Redis first if not in fallback mode
        if !self.using_fallback.load(Ordering::Relaxed) {
            match self.redis.exists(key).await {
                Ok(exists) => return Ok(exists),
                Err(e) => {
                    self.handle_redis_failure(&e);
                    // Fall through to memory cache
                }
            }
        }

        // Use memory cache as fallback
        // Memory provider might not support exists directly
        match self.memory.exists(key).await {
            Ok(exists) => Ok(exists),
            Err(_) => {
                // If memory provider doesn't support exists, try to get the key
                match self.memory.get::<StringValue>(key).await {
                    Ok(Some(_)) => Ok(true),
                    Ok(None) => Ok(false),
                    Err(e) => Err(e),
                }
            }
        }
    }

    async fn get_stats(&self) -> Result<serde_json::Value, String> {
        let is_fallback = self.using_fallback.load(Ordering::Relaxed);

        // Get stats from both providers
        let redis_stats = match self.redis.get_stats().await {
            Ok(stats) => stats,
            Err(e) => json!({ "error": e }),
        };

        let memory_stats = match self.memory.get_stats().await {
            Ok(stats) => stats,
            Err(e) => json!({ "error": e }),
        };

        let result = json!({
            "provider": "fallback",
            "using_fallback": is_fallback,
            "primary": {
                "type": "redis",
                "stats": redis_stats,
            },
            "fallback": {
                "type": "memory",
                "stats": memory_stats,
            }
        });

        Ok(result)
    }
}
