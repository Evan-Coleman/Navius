# Navius Redis Cache

This crate provides a Redis-based cache implementation for the Navius platform, including robust connection management, comprehensive metrics, and optimized performance.

## Features

- Robust connection pooling with timeout and retry handling
- Comprehensive metrics for monitoring cache operations and performance
- Support for all standard Redis operations, including:
  - Basic key-value operations (GET, SET, DEL, etc.)
  - List operations (LPUSH, RPUSH, LRANGE, etc.)
  - Hash operations (HGET, HSET, HGETALL, etc.)
  - Set operations (SADD, SMEMBERS, SINTER, etc.)
  - Sorted Set operations (ZADD, ZRANGE, ZRANK, etc.)
- Pipeline support for batch operations with optimized performance
- Lua scripting support for atomic operations

## Usage

### Basic Usage

```rust
use navius_cache::{operations::{Cache, CacheOperations}, error::CacheResult};
use navius_cache_redis::{connection::{RedisConnectionManager, RedisCacheConfig}, operations::RedisCache};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    active: bool,
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create Redis cache config
    let config = RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        key_prefix: "navius:".to_string(),
        connection_timeout_seconds: 5,
        connection_retries: 3,
        pool_size: 10,
    };

    // Create connection manager
    let connection_manager = RedisConnectionManager::new(config)?;
    
    // Create Redis cache
    let cache = RedisCache::new(connection_manager.into());
    
    // Store a user
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        active: true,
    };
    
    cache.set("user:1", &user, None).await?;
    
    // Retrieve the user
    let retrieved_user: Option<User> = cache.get("user:1").await?;
    
    if let Some(user) = retrieved_user {
        println!("Retrieved user: {}", user.name);
    }
    
    Ok(())
}
```

### Set Operations

```rust
use navius_cache::{operations::{Cache, CacheOperations}, error::CacheResult};
use navius_cache_redis::{connection::{RedisConnectionManager, RedisCacheConfig}, operations::RedisCache};

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Setup cache (as shown in basic example)
    let cache = setup_cache()?;
    
    // Add items to a set
    let set_key = "users:active";
    cache.set_add(set_key, vec![1, 2, 3, 4, 5]).await?;
    
    // Check membership
    let is_member = cache.set_contains(set_key, &3).await?;
    println!("Is user 3 active? {}", is_member); // true
    
    // Get all members
    let members: Vec<u32> = cache.set_members(set_key).await?;
    println!("Active users: {:?}", members);
    
    // Remove items
    cache.set_remove(set_key, vec![4, 5]).await?;
    
    // Set operations
    cache.set_add("users:premium", vec![2, 3, 7, 8]).await?;
    
    // Intersection (users who are both active and premium)
    let intersection: Vec<u32> = cache.set_intersection(vec![
        "users:active", 
        "users:premium"
    ]).await?;
    
    println!("Active premium users: {:?}", intersection); // [2, 3]
    
    // Store intersection in a new set
    cache.set_intersection_store(
        "users:active_premium",
        vec!["users:active", "users:premium"]
    ).await?;
    
    Ok(())
}
```

### Sorted Set Operations

```rust
use navius_cache::{operations::{Cache, CacheOperations}, error::CacheResult};
use navius_cache_redis::{connection::{RedisConnectionManager, RedisCacheConfig}, operations::RedisCache};

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Setup cache (as shown in basic example)
    let cache = setup_cache()?;
    
    // Add items to a sorted set (user scores)
    let zset_key = "users:scores";
    let scores = vec![
        (95.5, 1),  // (score, user_id)
        (80.0, 2),
        (65.5, 3),
        (90.0, 4),
        (75.5, 5),
    ];
    
    cache.zset_add(zset_key, scores).await?;
    
    // Get top 3 users
    let top_users: Vec<u32> = cache.zset_range(zset_key, -3, -1).await?;
    println!("Top 3 users: {:?}", top_users); // [5, 4, 1] (highest scores)
    
    // Get users with their scores
    let users_with_scores: Vec<(u32, f64)> = 
        cache.zset_range_with_scores(zset_key, -3, -1).await?;
    
    println!("Top users with scores:");
    for (user_id, score) in users_with_scores {
        println!("User {}: {}", user_id, score);
    }
    
    // Get users with scores between 70 and 90
    let mid_range: Vec<u32> = 
        cache.zset_range_by_score(zset_key, 70.0, 90.0).await?;
    
    println!("Users with scores between 70-90: {:?}", mid_range);
    
    // Get user rank (0-based, lowest to highest)
    let rank = cache.zset_rank(zset_key, &3).await?;
    println!("Rank of user 3: {:?}", rank);
    
    // Increment a user's score
    let new_score = cache.zset_increment_score(zset_key, &3, 15.0).await?;
    println!("User 3's new score: {}", new_score); // 80.5
    
    Ok(())
}
```

### Pipeline Operations

Using pipelines can significantly improve performance when executing multiple operations by reducing network round trips:

```rust
use navius_cache::{operations::{Cache, CacheOperations}, error::CacheResult};
use navius_cache_redis::{
    connection::{RedisConnectionManager, RedisCacheConfig}, 
    operations::RedisCache,
    RedisCommandPipeline,
};

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Setup cache (as shown in basic example)
    let cache = setup_cache()?;
    
    // Create a pipeline
    let mut pipeline = cache.pipeline();
    
    // Add multiple operations
    pipeline = pipeline
        .set("key1", &"value1")
        .set("key2", &"value2")
        .set("key3", &"value3")
        .get("key1")
        .delete("old_key");
    
    // Execute the pipeline
    cache.execute_pipeline(pipeline).await?;
    
    // Performance comparison example
    let users = generate_test_users(100);
    
    // Without pipeline
    let start = std::time::Instant::now();
    for user in &users {
        cache.set(format!("user:{}", user.id), user, None).await?;
    }
    let individual_duration = start.elapsed();
    
    // With pipeline
    let start = std::time::Instant::now();
    let mut pipeline = cache.pipeline();
    for user in &users {
        pipeline = pipeline.set(format!("user:{}", user.id), user);
    }
    cache.execute_pipeline(pipeline).await?;
    let pipeline_duration = start.elapsed();
    
    println!("Individual operations: {:?}", individual_duration);
    println!("Pipeline operations: {:?}", pipeline_duration);
    println!("Speed improvement: {:.2}x", 
        individual_duration.as_secs_f64() / pipeline_duration.as_secs_f64());
    
    Ok(())
}
```

### Lua Scripting

```rust
use navius_cache::{operations::{Cache, CacheOperations}, error::CacheResult};
use navius_cache_redis::{connection::{RedisConnectionManager, RedisCacheConfig}, operations::RedisCache};

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create Redis cache with Lua scripting enabled
    let config = RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        key_prefix: "navius:".to_string(),
        connection_timeout_seconds: 5,
        connection_retries: 3,
        pool_size: 10,
    };

    let connection_manager = RedisConnectionManager::new(config)?;
    let cache = RedisCache::new(connection_manager.into()).with_lua_scripting();
    
    // Register a Lua script for atomic operations
    let script = r#"
    local current = redis.call('GET', KEYS[1])
    if current and tonumber(current) < tonumber(ARGV[1]) then
        redis.call('SET', KEYS[1], ARGV[1])
        return 1
    else
        return 0
    end
    "#;
    
    cache.register_script("set_if_greater", script).await?;
    
    // Use the script
    let result: i64 = cache.execute_script(
        "set_if_greater",
        &["test_key"],
        &["100"]
    ).await?;
    
    println!("Script result: {}", result);
    
    Ok(())
}
```

## Metrics

The Redis cache implementation includes comprehensive metrics for monitoring cache operations and performance. All metrics are prefixed with `navius_` and include:

### Operation Metrics

- `{operation}_duration_ms` - Histogram of operation durations in milliseconds
- `{operation}_success` - Counter of successful operations
- `{operation}_error` - Counter of operation errors
- `{operation}_error_{type}` - Counter of specific error types

Operations include: `get`, `set`, `delete`, `exists`, `expire`, etc.

### Connection Metrics

- `connection_acquire` - Histogram of connection acquisition times
- `connection_acquire_error` - Counter of connection acquisition errors
- `connection_pool_size` - Gauge of the total connection pool size
- `connection_pool_idle` - Gauge of idle connections in the pool
- `connection_pool_used` - Gauge of currently used connections

### Health Check Metrics

- `health_check_healthy` - Counter of successful health checks
- `health_check_degraded` - Counter of degraded health checks
- `health_check_unhealthy` - Counter of failed health checks
- `health_check_status` - Gauge of current health status (1.0 = healthy, 0.5 = degraded, 0.0 = unhealthy)

## Viewing Metrics

Metrics are exposed via the standard metrics interface. You can view them using the metrics endpoint or by using the `metrics_example.rs` example:

```bash
cargo run --example metrics_example
```

## Testing Metrics

You can test the metrics using the `metrics_test.rs` example:

```bash
cargo run --example metrics_test
```

## Performance Benchmarking

The crate includes benchmarks for measuring performance under various conditions. To run the benchmarks:

```bash
cargo run --example benchmark
```

This will run a series of benchmarks and output the results. You can also use the benchmark visualizer to generate charts:

```bash
cargo run --example benchmark_visualizer
```

## Performance Optimization Guidelines

Based on our benchmark findings, we recommend the following best practices:

1. **Use Pipelining**: For batch operations, always use pipelining to reduce network round trips. Our benchmarks show up to 50x speedup for large batch operations.

2. **Optimize Connection Pool Size**: The optimal connection pool size depends on your workload:
   - For read-heavy workloads: pool_size = num_cores * 2
   - For write-heavy workloads: pool_size = num_cores * 4
   - For mixed workloads: pool_size = num_cores * 3

3. **Use Lua Scripts for Atomic Operations**: When you need to perform multiple operations atomically, use Lua scripts instead of transactions for better performance.

4. **Avoid Large Objects**: Redis performs best with smaller objects. Consider splitting large objects into smaller parts if possible.

5. **Use Appropriate TTLs**: Set reasonable TTLs for cache entries to avoid memory pressure.

6. **Monitor Metrics**: Regularly monitor the metrics to identify performance bottlenecks or issues.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details. 