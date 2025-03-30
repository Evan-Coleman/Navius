# Navius Redis Cache

A high-performance Redis cache implementation with comprehensive metrics, connection pooling, and Lua scripting support.

## Features

- Robust connection pooling with health monitoring
- Comprehensive metrics for all operations
- Serialization/deserialization of complex objects
- Support for atomic operations via Lua scripting
- Pipeline support for batch operations
- Collection operations (lists, sets, hashes)
- Prometheus metrics integration

## Usage

### Basic Usage

```rust
use std::sync::Arc;
use std::time::Duration;
use navius_cache::{Cache, CacheError};
use navius_cache_redis::{RedisCache, RedisConnectionManager};

async fn example() -> Result<(), CacheError> {
    // Create a connection manager
    let redis_url = "redis://127.0.0.1:6379/";
    let connection_manager = Arc::new(RedisConnectionManager::new(redis_url)?);
    
    // Create a cache instance
    let cache = RedisCache::new(connection_manager);
    
    // Basic operations
    cache.set("my_key", "my_value", Some(Duration::from_secs(60))).await?;
    let value: String = cache.get("my_key").await?;
    
    // Delete a key
    cache.delete("my_key").await?;
    
    Ok(())
}
```

### Lua Scripting Support

Enable Lua scripting for atomic operations:

```rust
// Create a cache with Lua scripting enabled
let cache = RedisCache::new(connection_manager.clone())
    .with_lua_scripting();

// Initialize common scripts
cache.initialize_common_scripts().await?;

// Use atomic operations
let result = cache.set_if_not_exists("lock_key", "owner_id", Some(Duration::from_secs(10))).await?;
if result {
    // Lock acquired
    // ... do work ...
    
    // Release lock
    cache.delete("lock_key").await?;
}

// Increment a counter atomically
let new_value: i64 = cache.atomic_increment("counter", 1, None).await?;

// Check and increment (useful for rate limiting)
let can_proceed = cache.check_and_increment_counter(
    "rate_limit:user:123", 
    5,  // max allowed
    Some(Duration::from_secs(60))
).await?;

// Register and use custom scripts
let script_name = "my_custom_script";
let script_content = r#"
    return redis.call('SET', KEYS[1], ARGV[1])
"#;

cache.register_script(script_name, script_content).await?;
let result: String = cache.execute_script(
    script_name,
    vec!["key1".to_string()],
    vec!["value1".to_string()]
).await?;
```

## Metrics

The Navius Redis Cache provides comprehensive metrics for monitoring cache performance and health. These metrics are exposed via Prometheus and can be easily integrated with your monitoring infrastructure.

### Metrics Categories

1. **Operation Metrics**
   - `navius_redis_cache_operation_duration_seconds`: Histogram of operation durations
   - `navius_redis_cache_operation_errors_total`: Counter of operation errors
   - `navius_redis_cache_operation_total`: Counter of operations executed

2. **Connection Pool Metrics**
   - `navius_redis_cache_pool_size`: Current size of the connection pool
   - `navius_redis_cache_connection_acquisition_time_seconds`: Time to acquire a connection
   - `navius_redis_cache_connection_errors_total`: Counter of connection errors
   - `navius_redis_cache_connection_health`: Health status of the Redis connection (0-1)

3. **Lua Script Metrics**
   - `navius_redis_cache_script_execution_duration_seconds`: Histogram of script execution times
   - `navius_redis_cache_script_errors_total`: Counter of script execution errors

### Viewing Metrics

To view metrics in real-time, run the metrics example:

```bash
cargo run --example metrics_example
```

This will start a metrics server on port 9091. You can view the metrics at:
```
http://localhost:9091/metrics
```

### Grafana Dashboard

For visualization, import the provided Grafana dashboard JSON:

```
docs/dashboards/redis_cache_metrics.json
```

## Examples

- `examples/basic_usage.rs`: Demonstrates basic cache operations
- `examples/metrics_example.rs`: Shows metrics collection and visualization
- `examples/lua_scripting.rs`: Demonstrates Lua scripting capabilities

## Running Tests

```bash
# Install redis-server if not already installed
# On MacOS: brew install redis
# On Ubuntu: apt-get install redis-server

# Start Redis server
redis-server --port 6379

# Run tests
cargo test
```

## Performance Considerations

- Use connection pooling appropriately for your workload
- Consider using pipeline operations for batch processing
- Lua scripts provide atomic operations without network round-trips
- Monitor metrics to identify bottlenecks and optimize accordingly

## License

Apache 2.0 