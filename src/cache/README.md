# Caching System

This directory contains the application's caching system interfaces and implementations. The caching system is designed to be flexible and extensible, allowing you to use different cache providers depending on your needs.

## Configuration

The caching system can be configured through the application configuration in `config/default.yaml`:

```yaml
cache:
  enabled: true                  # Whether caching is enabled
  ttl_seconds: 300               # Default TTL for cached items
  max_capacity: 1000             # Maximum number of items in the memory cache
  reconnect_interval_seconds: 30 # How often to try reconnecting to Redis when using fallback
```

## Directory Structure

- `providers/`: Contains cache provider implementations
  - `memory.rs`: In-memory cache provider implementation using Moka
  - `redis.rs`: Skeleton for Redis provider (to be implemented)
  - `fallback.rs`: Fallback provider that uses Redis with automatic fallback to memory cache
- `registry_stats.rs`: Functions for retrieving cache statistics

## Usage

### Basic Usage with Default Provider

The simplest way to use the cache is with the default memory provider:

```rust
use crate::cache::{DefaultCacheProvider, providers::CacheProvider};
use crate::utils::api_resource::ApiResource;

// Create a new memory cache provider
let cache = DefaultCacheProvider::new(10000, 3600);

// Initialize the cache
cache.init().unwrap();

// Store a value in the cache
cache.set("my-key", my_value, 3600).await.unwrap();

// Retrieve a value from the cache
let value: Option<MyType> = cache.get("my-key").await.unwrap();
```

### Using Redis Provider

To use Redis for caching, you'll need to implement the Redis provider. A skeleton is provided in `providers/redis.rs`:

```rust
use crate::cache::providers::{redis::RedisCacheProvider, redis::RedisConfig, CacheProvider};

// Create a Redis configuration
let config = RedisConfig {
    url: "redis://localhost:6379".to_string(),
    username: None, 
    password: None,
    database: Some(0),
    ttl_seconds: 3600,
};

// Create a Redis cache provider
let cache = RedisCacheProvider::new(config);

// Initialize the cache
cache.init().unwrap();

// Use the cache
// ... similar to the memory provider
```

### Using Fallback Provider (Redis + Memory)

The fallback provider will try Redis first and automatically fall back to in-memory caching if Redis is unavailable:

```rust
use crate::cache::{FallbackCacheProvider, providers::fallback::FallbackConfig, providers::CacheProvider};
use crate::config::app_config;

// Option 1: Use default configuration (reads from config/default.yaml)
let config = FallbackConfig::default();

// Option 2: Use the app configuration explicitly
let app_config = app_config::load_config().unwrap();
let config = FallbackConfig::from_app_config(&app_config);

// Option 3: Create a custom configuration
let config = FallbackConfig {
    redis_config: RedisConfig {
        url: "redis://localhost:6379".to_string(),
        username: None,
        password: None,
        database: Some(0),
        ttl_seconds: 3600,
    },
    memory_max_capacity: 10000,
    memory_ttl_seconds: 3600,
    reconnect_interval_seconds: 30,
};

// Create a fallback cache provider
let cache = FallbackCacheProvider::new(config);

// Initialize the cache
cache.init().unwrap();

// Start the reconnect task to periodically try to reconnect to Redis if it's down
cache.start_reconnect_task().await;

// Use the cache - it will automatically try Redis first and fall back to memory if Redis fails
let value: Option<MyType> = cache.get("my-key").await.unwrap();
```

### Creating Your Own Provider

You can implement your own cache provider by implementing the `CacheProvider` trait:

```rust
use async_trait::async_trait;
use crate::cache::providers::CacheProvider;
use crate::utils::api_resource::ApiResource;

pub struct MyCustomProvider {
    // Your provider implementation details
}

#[async_trait]
impl CacheProvider for MyCustomProvider {
    // Implement all required methods
    fn init(&self) -> Result<(), String> {
        // Implementation
    }
    
    async fn set<T: ApiResource>(&self, key: &str, value: T, ttl_seconds: u64) -> Result<(), String> {
        // Implementation
    }
    
    // ... other methods
}
```

## Core Implementation

The core caching implementation is located in the `src/core/cache` directory and provides the underlying functionality for the in-memory provider. This implementation is not intended to be used directly by application code.

For more details on the core implementation, see the [core cache README](../core/cache/README.md). 