# Navius Cache Provider Guide

**Version:** 1.0  
**Last Updated:** March 29, 2025

This guide explains how to implement a cache provider for the Navius caching system. It follows the provider pattern established throughout the Navius framework, enabling modular, extensible caching solutions.

## Overview

The Navius caching system uses a provider-based architecture that separates:

1. **Interface definitions** in the `navius-cache` crate, which define the contract that all cache implementations must fulfill
2. **Provider implementations** in separate crates (like `navius-cache-redis`), which implement these interfaces for specific caching technologies

This separation offers several benefits:

- Applications can swap cache providers without code changes
- Dependency trees are minimized for applications using specific providers
- Testing is simplified through mock implementations
- New providers can be added without modifying core interfaces

## Core Interfaces

A complete cache provider needs to implement the following interfaces:

### 1. Cache Operations Interface

The `Cache` trait provides standard key-value operations:

```rust
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>>;
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<()>;
    async fn delete(&self, key: &str) -> CacheResult<bool>;
    async fn exists(&self, key: &str) -> CacheResult<bool>;
    async fn increment(&self, key: &str, amount: i64) -> CacheResult<i64>;
    async fn decrement(&self, key: &str, amount: i64) -> CacheResult<i64>;
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool>;
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>>;
    async fn clear(&self) -> CacheResult<()>;
    async fn health_check(&self) -> CacheResult<bool>;
}
```

### 2. Cache Invalidation Interface

The `CacheInvalidator` trait provides mechanisms for tag-based cache invalidation:

```rust
#[async_trait]
pub trait CacheInvalidator: Send + Sync {
    async fn tag(&self, key: &str, tags: &[&str]) -> CacheInvalidatorResult<()>;
    async fn untag(&self, key: &str, tags: &[&str]) -> CacheInvalidatorResult<()>;
    async fn invalidate_tags(&self, tags: &[&str]) -> CacheInvalidatorResult<u64>;
    async fn get_tags(&self, key: &str) -> CacheInvalidatorResult<Vec<String>>;
    async fn get_keys_by_tag(&self, tag: &str) -> CacheInvalidatorResult<Vec<String>>;
    async fn invalidate_pattern(&self, pattern: &str) -> CacheInvalidatorResult<u64>;
    async fn clear(&self) -> CacheInvalidatorResult<()>;
    async fn health_check(&self) -> CacheInvalidatorResult<bool>;
}
```

### 3. Cache Provider Interface

The `CacheProvider` trait represents the main entry point for cache provider registration:

```rust
pub trait CacheProvider: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}
```

## Implementation Steps

To create a new cache provider, follow these steps:

### 1. Create a New Crate

Start by creating a new crate with the naming pattern `navius-cache-{provider}`:

```bash
cargo new --lib navius-cache-memcached
```

### 2. Define Dependencies

Add the required dependencies to your Cargo.toml:

```toml
[dependencies]
navius-core = { version = "0.1.0", path = "../navius-core" }
navius-cache = { version = "0.1.0", path = "../navius-cache" }
async-trait = "0.1.68"
thiserror = "1.0.40"
serde = { version = "1.0.163", features = ["derive"] }
serde_json = "1.0.96"
tracing = "0.1.37"
futures = "0.3.28"
tokio = { version = "1.28.1", features = ["full"] }
```

Add provider-specific dependencies (for example, for Redis):

```toml
redis = { version = "0.23.0", features = ["tokio-comp", "connection-manager"] }
```

### 3. Implement Configuration

Create a configuration struct that implements conversions to/from the generic `CacheConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCacheConfig {
    pub url: String,
    pub key_prefix: String,
    pub default_ttl: Duration,
    // ... provider-specific configuration
}

impl From<ProviderCacheConfig> for CacheConfig {
    fn from(config: ProviderCacheConfig) -> Self {
        CacheConfig {
            provider: "provider_name".to_string(),
            connection_string: config.url.clone(),
            key_prefix: config.key_prefix.clone(),
            default_ttl: config.default_ttl,
        }
    }
}
```

### 4. Implement Error Handling

Create provider-specific error types that implement conversions to the standard `CacheError`:

```rust
#[derive(Error, Debug)]
pub enum ProviderCacheError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Operation error: {0}")]
    Operation(String),
    
    // ... other provider-specific errors
}

impl From<ProviderCacheError> for CacheError {
    fn from(err: ProviderCacheError) -> Self {
        match err {
            ProviderCacheError::Connection(msg) => CacheError::Unavailable(msg),
            ProviderCacheError::Operation(msg) => CacheError::Provider(msg),
            // ... other conversions
        }
    }
}
```

### 5. Implement Connection Management

Create a connection manager or client wrapper:

```rust
pub struct ProviderConnectionManager {
    client: Client,
    config: ProviderCacheConfig,
}

impl ProviderConnectionManager {
    pub async fn new(config: ProviderCacheConfig) -> Result<Self, ProviderCacheError> {
        // Initialize connection to cache provider
        // Handle connection errors
        // Return initialized manager
    }
    
    // Helper methods for connection management
}
```

### 6. Implement Cache Operations

Implement the `Cache` trait for your provider:

```rust
pub struct ProviderCache {
    connection_manager: ProviderConnectionManager,
}

#[async_trait]
impl Cache for ProviderCache {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        // Implementation specific to this provider
    }
    
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<()> {
        // Implementation specific to this provider
    }
    
    // Implement other Cache trait methods...
}
```

### 7. Implement Invalidation

Implement the `CacheInvalidator` trait:

```rust
pub struct ProviderInvalidator {
    cache: ProviderCache,
}

#[async_trait]
impl CacheInvalidator for ProviderInvalidator {
    async fn tag(&self, key: &str, tags: &[&str]) -> CacheInvalidatorResult<()> {
        // Implementation specific to this provider
    }
    
    // Implement other CacheInvalidator trait methods...
}
```

### 8. Implement Provider Registration

Create a provider struct and implement the `CacheProvider` trait:

```rust
pub struct Provider {
    name: String,
    version: String,
}

impl CacheProvider for Provider {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
}
```

### 9. Write Examples and Tests

Create examples showing how to use your provider:

```rust
// examples/basic_usage.rs
use navius_cache::operations::Cache;
use navius_cache_provider::{ProviderCacheConfig, ProviderCache};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProviderCacheConfig::new(/* parameters */);
    let cache = ProviderCache::new(config).await?;
    
    // Example cache operations
    
    Ok(())
}
```

Write tests that verify your implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_operations() {
        // Test implementation
    }
}
```

## Best Practices

When implementing a cache provider, follow these best practices:

### 1. Error Handling

- Convert provider-specific errors to the standard `CacheError` types
- Provide detailed error messages that include context
- Handle network failures and temporary errors appropriately

### 2. Connection Management

- Implement efficient connection pooling
- Handle connection failures gracefully
- Provide reconnection logic for lost connections
- Include timeout handling

### 3. Key Management

- Always prefix keys with the configured prefix
- Handle key encoding/escaping according to provider requirements
- Consider key length limitations

### 4. Serialization

- Use consistent serialization across all operations
- Handle serialization errors gracefully
- Consider binary vs. string serialization tradeoffs

### 5. Performance

- Batch operations when possible
- Minimize network round-trips
- Consider pipeline operations if supported
- Use bulk gets/sets for related items

### 6. Configuration

- Provide sensible defaults
- Validate configuration values
- Document configuration options thoroughly

## Example Implementation: Redis

For reference, the `navius-cache-redis` crate provides a complete implementation of a Redis cache provider:

```rust
// Main provider implementation
pub struct RedisProvider {
    name: String,
    version: String,
}

impl CacheProvider for RedisProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
}

// Helper function to initialize Redis
pub async fn initialize_redis(config: RedisCacheConfig) -> Result<RedisCache, RedisCacheError> {
    // Implementation details...
}
```

The Redis implementation also includes:

- Connection pooling with configurable pool size
- Comprehensive error mapping from Redis errors to CacheError
- Support for Redis clusters and sentinels
- TTL management
- Tag-based invalidation using Redis sets
- Pattern-based key invalidation

## Testing Your Provider

Test your provider with various scenarios:

1. **Basic operations**: get, set, delete
2. **TTL handling**: expiration and TTL queries
3. **Counters**: increment, decrement
4. **Invalidation**: tag-based, pattern-based
5. **Error handling**: connection failures, timeout handling
6. **Performance**: under load, with various payload sizes
7. **Concurrency**: multiple operations in parallel

## Documentation

Provide thorough documentation for your provider:

1. **README.md**: Overview, usage examples, configuration
2. **API documentation**: Document public types and methods
3. **Examples**: Include runnable examples
4. **Configuration guide**: Detail all configuration options

## Conclusion

Implementing a cache provider for the Navius caching system requires following the established provider pattern. This pattern ensures consistency across different cache technologies while allowing for provider-specific optimizations.

By separating the interface from the implementation, the Navius cache system gains flexibility, testability, and maintainability benefits that significantly enhance the developer experience. 