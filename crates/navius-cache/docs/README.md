# Navius Cache

A flexible, type-safe caching library for the Navius framework.

## Features

- **Type-safe API**: Work with strongly-typed cache values using generics
- **Redis support**: Full Redis implementation with connection pooling
- **Multiple invalidation strategies**: TTL-based, pattern-based, and entity-based
- **Comprehensive metrics**: Track cache hits, misses, errors, and performance
- **Flexible configuration**: Customize cache behavior for different use cases

## Installation

Add navius-cache to your Cargo.toml:

```toml
[dependencies]
navius-cache = { path = "../navius-cache", features = ["redis", "metrics"] }
```

Available features:
- `redis` - Redis backend support (enabled by default)
- `metrics` - Metrics and telemetry integration

## Quick Start

```rust
use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions};
use std::time::Duration;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration
    let config = CacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "my-app:".to_string(),
        Duration::from_secs(300), // 5 minutes default TTL
    );

    // Connect to Redis
    let cache = CacheConnectionManager::new_redis(config).await?;

    // Store a value
    let user = User { id: 123, name: "Alice".to_string() };
    cache.set("user:123", &user, None).await?;

    // Retrieve a value
    let retrieved: Option<User> = cache.get("user:123").await?;
    assert_eq!(retrieved.unwrap().name, "Alice");

    // Store with custom TTL
    let options = CacheOptions::new().ttl(Duration::from_secs(60)); // 1 minute TTL
    cache.set("short-lived:key", "value", Some(options)).await?;

    // Delete a value
    cache.delete("user:123").await?;

    Ok(())
}
```

## Configuration

### Basic Configuration

```rust
// Minimal configuration
let config = CacheConfig::new(
    "redis://127.0.0.1:6379".to_string(), // Cache URL
    "my-app:".to_string(),                // Key prefix
    Duration::from_secs(3600),            // Default TTL (1 hour)
);

// Advanced configuration
let config = CacheConfig::new(
    "redis://127.0.0.1:6379".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
)
.with_max_connections(20)       // Set connection pool size
.with_metrics(true)             // Enable metrics tracking
.with_trace(true)               // Enable tracing
.with_serialization_format("json"); // Set serialization format
```

### Multiple Caches

You can create multiple cache instances with different configurations:

```rust
// Short-lived cache for temporary data
let temp_cache = CacheConnectionManager::new_redis(
    CacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "temp:",
        Duration::from_secs(60),
    )
).await?;

// Long-lived cache for reference data
let ref_cache = CacheConnectionManager::new_redis(
    CacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "ref:",
        Duration::from_secs(86400), // 24 hours
    )
).await?;
```

## Working with Cache Keys

Any type that implements `CacheKey` can be used as a key:

```rust
// String keys
cache.set("simple-key", "value", None).await?;
cache.set(&String::from("string-key"), "value", None).await?;

// Integer keys
cache.set(42, "value", None).await?;

// Custom struct keys
#[derive(Debug)]
struct UserKey {
    id: u64,
    tenant_id: u64,
}

impl CacheKey for UserKey {
    fn to_string(&self) -> String {
        format!("user:{}:{}", self.tenant_id, self.id)
    }
}

let key = UserKey { id: 123, tenant_id: 456 };
cache.set(key, "value", None).await?;
```

## Invalidation Strategies

The library provides multiple invalidation strategies:

```rust
use navius_cache::{CacheInvalidation, InvalidationStrategy};

// TTL-based invalidation (automatic)
let options = CacheOptions::new().ttl(Duration::from_secs(30));
cache.set("key", "value", Some(options)).await?;

// Immediate invalidation
cache.delete("key").await?;

// Pattern-based invalidation
let invalidator = CacheInvalidation::new(InvalidationStrategy::PatternBased("user:*".to_string()));
invalidator.invalidate(&cache).await?;

// Entity-based invalidation
let invalidator = CacheInvalidation::new(InvalidationStrategy::EntityBased);
invalidator.invalidate_entity(&cache, "user", "123").await?;
```

## Batch Operations

Perform operations on multiple keys efficiently:

```rust
// Get multiple values
let keys = vec!["key1", "key2", "key3"];
let values: Vec<Option<String>> = cache.get_many(keys).await?;

// Set multiple values
let entries = vec![
    ("key1", "value1"),
    ("key2", "value2"),
    ("key3", "value3"),
];
cache.set_many(entries, None).await?;

// Delete multiple keys
let keys = vec!["key1", "key2", "key3"];
let deleted_count = cache.delete_many(keys).await?;
```

## Metrics Integration

When the `metrics` feature is enabled, the crate automatically records cache metrics:

```rust
use navius_cache::{CacheConfig, CacheConnectionManager};

// Enable metrics in configuration
let config = CacheConfig::new(
    "redis://127.0.0.1:6379".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
).with_metrics(true);

// All operations will now record metrics
let cache = CacheConnectionManager::new_redis(config).await?;

// Perform operations - metrics are recorded automatically
let _: Option<String> = cache.get("key").await?;
```

### Available Metrics

The following metrics are recorded:

- `navius_cache_operations_total` - Counter for cache operations
  - Labels: `operation`, `backend`, `result`
- `navius_cache_operation_duration_seconds` - Histogram for operation durations
  - Labels: `operation`, `backend`

### Custom Metrics Usage

You can also use the metrics utilities directly:

```rust
#[cfg(feature = "metrics")]
use navius_cache::metrics::{CacheOperation, CacheTimer, MetricsResult};

#[cfg(feature = "metrics")]
fn with_metrics() {
    // Create a timer for an operation
    let timer = CacheTimer::new(CacheOperation::Get, "redis");
    
    // Record the result
    timer.hit(); // Cache hit
    
    // Or for other operations
    let timer = CacheTimer::new(CacheOperation::Set, "redis");
    timer.success(); // Successful operation
    
    // For failures
    let timer = CacheTimer::new(CacheOperation::Delete, "redis");
    timer.error(); // Failed operation
}
```

## Best Practices

### Key Naming

- Use consistent key naming patterns
- Include entity type in keys (e.g., `user:123`)
- Consider using namespaces for different parts of your application
- Keep keys relatively short to minimize storage overhead

### TTL Strategies

- Set default TTLs conservatively
- Use shorter TTLs for frequently changing data
- Use longer TTLs for reference data
- Consider infinite TTL only for critical reference data

### Performance Optimization

- Use batch operations (`get_many`, `set_many`) when possible
- Consider key compression for very large datasets
- Monitor hit/miss ratios and adjust caching strategy accordingly
- Use pattern-based invalidation carefully as it can be expensive

### Metrics Monitoring

- Set up alerts for sudden changes in hit/miss ratios
- Monitor operation latency for early detection of Redis issues
- Track cache size growth over time
- Configure dashboards to visualize cache performance

### Error Handling

- Implement graceful degradation when cache is unavailable
- Consider using circuit breakers for cache operations
- Log cache errors but don't necessarily fail application operations
- Have a strategy for handling deserialization errors

## Integration with Other Navius Components

### With navius-db

```rust
use navius_db::Repository;
use navius_cache::{CacheConnectionManager, CacheOptions};

struct UserRepository {
    db: PgPool,
    cache: CacheConnectionManager,
}

impl UserRepository {
    async fn get_user(&self, id: i64) -> Result<Option<User>, Error> {
        // Try cache first
        let cache_key = format!("user:{}", id);
        if let Some(user) = self.cache.get(&cache_key).await? {
            return Ok(Some(user));
        }
        
        // If not in cache, get from database
        let user = self.find_by_id(id).await?;
        
        // Store in cache if found
        if let Some(ref user) = user {
            self.cache.set(&cache_key, user, None).await?;
        }
        
        Ok(user)
    }
}
```

### With navius-http

```rust
use axum::{Router, routing::get, extract::State};
use navius_cache::CacheConnectionManager;
use navius_http::AppState;

// Define application state with cache
struct AppState {
    cache: CacheConnectionManager,
}

// Cache-aware handler
async fn get_data(State(state): State<AppState>) -> impl IntoResponse {
    let data: Option<Data> = state.cache.get("data-key").await.unwrap_or(None);
    match data {
        Some(data) => Json(data).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// Create router with cached responses
fn create_router(cache: CacheConnectionManager) -> Router {
    Router::new()
        .route("/data", get(get_data))
        .with_state(AppState { cache })
}
```

## Testing with navius-cache

The crate provides utilities for testing:

```rust
#[cfg(test)]
mod tests {
    use navius_cache::{CacheConfig, CacheConnectionManager, MockCache};
    
    #[tokio::test]
    async fn test_with_mock_cache() {
        // Create a mock cache for testing
        let mut mock = MockCache::new();
        
        // Set up expectations
        mock.expect_get()
            .with(eq("test-key"))
            .returning(|_| Ok(Some("test-value".to_string())));
            
        // Test your code with the mock
        let result: Option<String> = mock.get("test-key").await.unwrap();
        assert_eq!(result, Some("test-value".to_string()));
    }
    
    #[tokio::test]
    async fn test_with_real_redis() {
        // Only run this test if REDIS_TEST env var is set
        if std::env::var("REDIS_TEST").is_err() {
            return;
        }
        
        // Create a real Redis connection for integration testing
        let config = CacheConfig::new(
            "redis://127.0.0.1:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(10),
        );
        
        let cache = CacheConnectionManager::new_redis(config).await.unwrap();
        
        // Clear test data before test
        cache.clear().await.unwrap();
        
        // Perform test operations
        cache.set("test-key", "test-value", None).await.unwrap();
        let result: Option<String> = cache.get("test-key").await.unwrap();
        assert_eq!(result, Some("test-value".to_string()));
        
        // Clean up
        cache.clear().await.unwrap();
    }
}
```

## Troubleshooting

### Common Issues

1. **Connection Failures**
   - Check that Redis is running and accessible
   - Verify connection URL format is correct
   - Ensure network settings allow connections

2. **Serialization Errors**
   - Ensure types implement `Serialize` and `Deserialize`
   - Check for schema changes between serialization and deserialization
   - Use schema versioning for evolving data structures

3. **Cache Misses**
   - Verify keys are being constructed consistently
   - Check TTL settings aren't too short
   - Look for invalidation operations that might be clearing cache

4. **Performance Issues**
   - Check Redis server metrics
   - Monitor operation latency
   - Reduce key/value sizes if possible
   - Optimize invalidation strategies

## API Reference

For complete API documentation, see the rustdoc documentation:

```bash
cargo doc --open
``` 