# Navius Cache - Redis Provider

A Redis-based implementation of the Navius caching system, following the provider pattern established in the workspace migration project.

## Features

- Complete implementation of the `Cache` trait using Redis as the backend
- Efficient connection pooling for improved performance
- Comprehensive error handling with specific error types
- Cache invalidation support with tag-based and pattern-based strategies
- Configurable TTL (Time To Live) for cache entries
- Metrics support (optional)
- Comprehensive test coverage

## Usage

### Basic Example

```rust
use navius_cache_redis::{RedisCache, RedisCacheConfig};
use navius_cache::operations::Cache;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "myapp:".to_string(),
        Duration::from_secs(3600),
    );
    
    // Initialize the cache
    let cache = RedisCache::new(config).await?;
    
    // Store a value
    cache.set("user:123", &"John Doe", None).await?;
    
    // Retrieve a value
    let user: Option<String> = cache.get("user:123").await?;
    
    println!("User: {:?}", user);
    
    // Check if a key exists
    let exists = cache.exists("user:123").await?;
    println!("Key exists: {}", exists);
    
    // Delete a key
    let deleted = cache.delete("user:123").await?;
    println!("Key deleted: {}", deleted);
    
    Ok(())
}
```

### Using the InvalidationCache

```rust
use navius_cache_redis::{RedisInvalidator, RedisCacheConfig};
use navius_cache::invalidation::CacheInvalidator;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "myapp:".to_string(),
        Duration::from_secs(3600),
    );
    
    // Initialize the invalidator
    let invalidator = RedisInvalidator::new(config).await?;
    
    // Get the underlying cache
    let cache = invalidator.cache();
    
    // Store values
    cache.set("user:1", &"John Doe", None).await?;
    cache.set("user:2", &"Jane Smith", None).await?;
    cache.set("product:1", &"Laptop", None).await?;
    
    // Tag keys
    invalidator.tag("user:1", &["user", "admin"]).await?;
    invalidator.tag("user:2", &["user"]).await?;
    invalidator.tag("product:1", &["product"]).await?;
    
    // Invalidate by tag
    let count = invalidator.invalidate_tags(&["user"]).await?;
    println!("Invalidated {} keys with 'user' tag", count);
    
    // Invalidate by pattern
    let count = invalidator.invalidate_pattern("product:*").await?;
    println!("Invalidated {} keys matching 'product:*'", count);
    
    Ok(())
}
```

## Configuration

The `RedisCacheConfig` struct provides various configuration options:

| Option | Description | Default |
|--------|-------------|---------|
| `url` | Redis connection URL | Required |
| `key_prefix` | Prefix for all cache keys | Required |
| `default_ttl` | Default time-to-live for cache entries | Required |
| `max_connections` | Maximum number of connections in the pool | 10 |
| `database` | Redis database index (0-15) | 0 |
| `password` | Redis password | None |
| `use_tls` | Whether to use TLS | false |
| `connection_timeout_seconds` | Timeout for establishing connections | 5 |
| `command_timeout_seconds` | Timeout for Redis commands | 2 |
| `retry_commands` | Whether to retry failed commands | true |
| `max_retries` | Maximum number of retries for failed commands | 3 |

## Implementation Details

### Connection Pooling

The crate implements an efficient connection pooling mechanism to reuse Redis connections and improve performance. The pool automatically grows and shrinks based on demand, up to the configured `max_connections` limit.

### Error Handling

The crate provides specific error types through the `RedisCacheError` enum, which maps to the generic `CacheError` types defined in the core `navius-cache` crate. This enables detailed error information while maintaining a consistent error handling approach across different cache providers.

### Serialization

Cache values are serialized to JSON using `serde_json` before storing in Redis, allowing any Rust type that implements `Serialize` and `Deserialize` to be stored in the cache.

### Cache Invalidation

The crate supports two invalidation strategies:

1. **Tag-based invalidation**: Keys can be tagged with one or more tags, and all keys with a specific tag can be invalidated at once.
2. **Pattern-based invalidation**: Keys matching a specific pattern can be invalidated together.

### Health Checks

The crate provides a `health_check` method that verifies the Redis connection is healthy, which can be used for monitoring and reliability purposes.

## Testing

The crate includes comprehensive tests that can be run with:

```bash
cargo test --package navius-cache-redis
```

Tests that require a Redis server will be skipped if Redis is not available, making it safe to run the test suite in any environment.

## Metrics (Optional)

To enable metrics collection, add the `metrics` feature to your dependency:

```toml
navius-cache-redis = { version = "0.1.0", features = ["metrics"] }
```

This will expose various metrics such as cache hits, misses, and operation latencies.

## License

This crate is part of the Navius project and is licensed under the same terms as the main project. 