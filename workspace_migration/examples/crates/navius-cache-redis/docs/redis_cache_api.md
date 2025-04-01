# Redis Cache API Reference

This document provides a comprehensive guide to the Redis Cache API in the `navius-cache-redis` crate, including all available operations, their parameters, return types, and usage examples.

## Table of Contents

1. [Basic Cache Operations](#basic-cache-operations)
2. [Set Operations](#set-operations)
3. [Sorted Set Operations](#sorted-set-operations)
4. [Pipeline Commands](#pipeline-commands)
5. [Lua Scripting](#lua-scripting)
6. [Connection Management](#connection-management)
7. [Error Handling](#error-handling)
8. [Metrics](#metrics)
9. [Configuration](#configuration)

## Basic Cache Operations

These operations provide fundamental key-value functionality.

### Setting Values

```rust
// Set a value with optional expiration
async fn set<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> CacheResult<()>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;

// Set a value only if the key doesn't exist
async fn set_if_not_exists<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> CacheResult<bool>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;
```

### Getting Values

```rust
// Get a value 
async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Check if a key exists
async fn exists<K>(&self, key: K) -> CacheResult<bool>
where
    K: CacheKey + 'static;
```

### Deleting Values

```rust
// Delete a key
async fn delete<K>(&self, key: K) -> CacheResult<()>
where
    K: CacheKey + 'static;

// Delete multiple keys
async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<()>
where
    K: CacheKey + 'static;
```

### Expiration

```rust
// Set expiration on an existing key
async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
where
    K: CacheKey + 'static;

// Remove expiration from a key
async fn persist<K>(&self, key: K) -> CacheResult<bool>
where
    K: CacheKey + 'static;

// Get the remaining TTL of a key
async fn ttl<K>(&self, key: K) -> CacheResult<Option<Duration>>
where
    K: CacheKey + 'static;
```

### Numeric Operations

```rust
// Increment a numeric value
async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
where
    K: CacheKey + 'static;

// Decrement a numeric value
async fn decrement<K>(&self, key: K, amount: i64) -> CacheResult<i64>
where
    K: CacheKey + 'static;
```

### Key Operations

```rust
// Get keys matching a pattern
async fn get_keys_by_pattern(&self, pattern: &str) -> CacheResult<Vec<String>>;

// Rename a key
async fn rename<K1, K2>(&self, key: K1, new_key: K2) -> CacheResult<()>
where
    K1: CacheKey + 'static,
    K2: CacheKey + 'static;
```

## Set Operations

Sets are unordered collections of unique items.

### Adding Items

```rust
// Add items to a set
async fn set_add<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;
```

### Removing Items

```rust
// Remove items from a set
async fn set_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;
```

### Membership Operations

```rust
// Check if an item is a member of a set
async fn set_contains<K, V>(&self, key: K, member: &V) -> CacheResult<bool>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;

// Get the number of items in a set
async fn set_length<K>(&self, key: K) -> CacheResult<usize>
where
    K: CacheKey + 'static;

// Get all members of a set
async fn set_members<V, K>(&self, key: K) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Get random members from a set
async fn set_random_members<V, K>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;
```

### Set Operations

```rust
// Get the intersection of multiple sets
async fn set_intersection<V, K>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Store the intersection of multiple sets
async fn set_intersection_store<K1, K2>(&self, destination: K1, keys: Vec<K2>) -> CacheResult<usize>
where
    K1: CacheKey + 'static,
    K2: CacheKey + 'static;

// Get the union of multiple sets
async fn set_union<V, K>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Store the union of multiple sets
async fn set_union_store<K1, K2>(&self, destination: K1, keys: Vec<K2>) -> CacheResult<usize>
where
    K1: CacheKey + 'static,
    K2: CacheKey + 'static;

// Get the difference between sets
async fn set_difference<V, K>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Store the difference between sets
async fn set_difference_store<K1, K2>(&self, destination: K1, keys: Vec<K2>) -> CacheResult<usize>
where
    K1: CacheKey + 'static,
    K2: CacheKey + 'static;
```

## Sorted Set Operations

Sorted sets are sets where each member has an associated score for ordering.

### Adding Items

```rust
// Add items with scores to a sorted set
async fn zset_add<K, V>(&self, key: K, members: HashMap<V, f64>) -> CacheResult<usize>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;
```

### Removing Items

```rust
// Remove items from a sorted set
async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;

// Remove items by score range
async fn zset_remove_by_score<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
where
    K: CacheKey + 'static;

// Remove items by rank range
async fn zset_remove_by_rank<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<usize>
where
    K: CacheKey + 'static;
```

### Score Operations

```rust
// Get the score of a member
async fn zset_score<K, V>(&self, key: K, member: V) -> CacheResult<Option<f64>>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;

// Increment the score of a member
async fn zset_increment_score<K, V>(&self, key: K, member: V, increment: f64) -> CacheResult<f64>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;
```

### Range Queries

```rust
// Get members by rank range (ascending)
async fn zset_range<V, K>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Get members by rank range with scores (ascending)
async fn zset_range_with_scores<V, K>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<(V, f64)>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Get members by rank range (descending)
async fn zset_rev_range<V, K>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Get members by rank range with scores (descending)
async fn zset_rev_range_with_scores<V, K>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<(V, f64)>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Get members by score range
async fn zset_range_by_score<V, K>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;

// Get members by score range with scores
async fn zset_range_by_score_with_scores<V, K>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<(V, f64)>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static;
```

### Rank Operations

```rust
// Get the rank of a member (ascending order, 0-based)
async fn zset_rank<K, V>(&self, key: K, member: V) -> CacheResult<Option<usize>>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;

// Get the rank of a member (descending order, 0-based)
async fn zset_rev_rank<K, V>(&self, key: K, member: V) -> CacheResult<Option<usize>>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;
```

### Other Operations

```rust
// Get the size of a sorted set
async fn zset_size<K>(&self, key: K) -> CacheResult<usize>
where
    K: CacheKey + 'static;

// Count members within a score range
async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
where
    K: CacheKey + 'static;
```

### Set Operations

```rust
// Store the union of sorted sets
async fn zset_union_store<K1, K2>(
    &self,
    destination: K1,
    keys: Vec<K2>,
    weights: &[f64],
    aggregate: SortedSetOperations,
) -> CacheResult<usize>
where
    K1: CacheKey + 'static,
    K2: CacheKey + 'static;

// Store the intersection of sorted sets
async fn zset_intersection_store<K1, K2>(
    &self,
    destination: K1,
    keys: Vec<K2>,
    weights: &[f64],
    aggregate: SortedSetOperations,
) -> CacheResult<usize>
where
    K1: CacheKey + 'static,
    K2: CacheKey + 'static;
```

## Pipeline Commands

Pipeline commands allow executing multiple Redis operations in a single network roundtrip, significantly improving performance for batch operations.

### Executing Pipeline Commands

```rust
// Execute multiple commands in a pipeline
async fn execute_pipeline_command<F, T>(&self, f: F) -> CacheResult<T>
where
    F: FnOnce(&mut redis::aio::Connection) -> Result<Pipeline, RedisCacheError>,
    T: FromRedisValue;
```

### Example Usage

```rust
// Example: Set multiple keys in a pipeline
let results = connection_manager.execute_pipeline_command(|connection| {
    let mut pipeline = Pipeline::new();
    
    // Add operations to the pipeline
    for i in 0..100 {
        let key = format!("key:{}", i);
        let value = format!("value:{}", i);
        pipeline.set(key, value);
    }
    
    Ok(pipeline)
}).await?;
```

## Lua Scripting

Lua scripting provides a way to execute atomic operations in Redis.

### Script Management

```rust
// Register a Lua script
async fn register_script(&self, name: &str, script: &str) -> CacheResult<()>;

// Execute a registered script
async fn execute_script<T>(
    &self,
    name: &str,
    keys: Vec<String>,
    args: Vec<String>,
) -> CacheResult<T>
where
    T: FromRedisValue;
```

### Built-in Script Operations

```rust
// Atomic increment with optional expiration
async fn atomic_increment<K>(
    &self,
    key: K,
    amount: i64,
    ttl: Option<Duration>,
) -> CacheResult<i64>
where
    K: CacheKey + 'static;

// Set with expiration, only if key doesn't exist
async fn set_if_not_exists<K, V>(
    &self,
    key: K,
    value: &V,
    ttl: Option<Duration>,
) -> CacheResult<bool>
where
    K: CacheKey + 'static,
    V: Serialize + 'static;

// Check and increment for rate limiting
async fn check_and_increment_counter<K>(
    &self,
    key: K,
    max_count: i64,
    ttl: Option<Duration>,
) -> CacheResult<bool>
where
    K: CacheKey + 'static;
```

## Connection Management

Connection management handles the pooling and lifecycle of Redis connections.

### Connection Pool

```rust
// Create a new connection manager
async fn new(config: RedisCacheConfig) -> Result<Self, RedisCacheError>;

// Get the connection pool statistics
async fn get_stats(&self) -> PoolStats;

// Get the Redis URL
fn get_url(&self) -> &str;

// Get the key prefix
fn get_prefix(&self) -> &str;
```

### Executing Commands

```rust
// Execute a command on a Redis connection
async fn execute_command<F, T>(&self, f: F) -> CacheResult<T>
where
    F: FnOnce(&mut redis::aio::Connection) -> Result<T, RedisCacheError>;
```

## Error Handling

The Redis cache provides detailed error types for proper error handling.

### Error Types

- `RedisCacheError::ConnectionError`: Errors related to Redis connections
- `RedisCacheError::OperationError`: Errors during Redis operations
- `RedisCacheError::ConfigurationError`: Configuration-related errors
- `RedisCacheError::SerializationError`: Serialization/deserialization errors
- `RedisCacheError::Timeout`: Timeout errors
- `RedisCacheError::Unavailable`: Service unavailability errors

### Handling Errors

```rust
// Example of proper error handling
match result {
    Ok(value) => {
        // Handle success
    },
    Err(RedisCacheError::ConnectionError(msg)) => {
        // Handle connection errors
    },
    Err(RedisCacheError::Timeout(msg)) => {
        // Handle timeout errors
    },
    Err(e) => {
        // Handle other errors
    }
}
```

## Metrics

The Redis cache provides comprehensive metrics for monitoring cache performance.

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

### Recording Metrics

Metrics are automatically recorded for all operations. Custom metrics can be recorded using:

```rust
let timer = TimedOperation::new(metrics::names::CUSTOM_OPERATION);
// Perform operation
timer.record(&result);
```

## Configuration

Configuration options for the Redis cache.

### Basic Configuration

```rust
// Create a Redis cache configuration
let config = RedisCacheConfig::builder()
    .with_url("redis://127.0.0.1:6379")
    .with_key_prefix("navius")
    .with_default_ttl(Duration::from_secs(3600))
    .with_pool_size(10)
    .with_connection_timeout(Duration::from_secs(1))
    .with_operation_timeout(Duration::from_secs(2))
    .build();
```

### Advanced Configuration

```rust
let config = RedisCacheConfig::builder()
    // Basic configuration
    .with_url("redis://127.0.0.1:6379")
    .with_key_prefix("navius")
    
    // Connection pool configuration
    .with_pool_size(20)  // 20 connections in the pool
    .with_min_idle(5)    // Keep at least 5 idle connections
    .with_max_lifetime(Duration::from_secs(1800))  // Max connection lifetime
    .with_idle_timeout(Duration::from_secs(300))   // Idle connection timeout
    
    // Timeout configuration
    .with_connection_timeout(Duration::from_millis(500))
    .with_operation_timeout(Duration::from_millis(1000))
    
    // Retry configuration
    .with_retry_attempts(3)
    .with_retry_delay(Duration::from_millis(50))
    
    // TLS configuration (if needed)
    .with_tls(true)
    .with_tls_cert_path("/path/to/cert.pem")
    
    // Authentication (if needed)
    .with_username("redis_user")
    .with_password("redis_password")
    
    .build();
```

## Examples

For complete working examples, see the `examples` directory in the crate:

- `examples/basic_usage.rs`: Basic cache operations
- `examples/set_operations.rs`: Set operations
- `examples/sorted_set_operations.rs`: Sorted set operations
- `examples/pipeline_commands.rs`: Pipeline commands
- `examples/lua_scripting.rs`: Lua scripting
- `examples/connection_pooling.rs`: Connection pooling
- `examples/metrics_example.rs`: Metrics collection 