# Navius Redis Cache Plugin

A Redis implementation of the `navius-cache` interface using the widely-adopted `redis-rs` crate.

## Features

- Fully implements the `navius-cache` interface for Redis
- Async-first design with Tokio runtime support
- Connection pooling with automatic reconnection
- Key prefixing for namespace isolation
- Configurable TTL for cache entries
- Support for basic data types and collections
- Comprehensive error handling
- Robust reconnection and retry strategies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
navius-cache = { version = "0.1", features = ["plugin"] }
navius-cache-redis-plugin = { version = "0.1" }
```

## Optional Features

The Redis plugin has several optional features:

- `cluster` - Enables Redis Cluster support
- `tls` - Enables TLS encryption support
- `json` - Enables RedisJSON support
- `default` - Includes basic Redis functionality

Enable them in your `Cargo.toml`:

```toml
[dependencies]
navius-cache-redis-plugin = { version = "0.1", features = ["tls", "cluster"] }
```

## Basic Usage

```rust
use navius_cache::{Cache, CacheKey, CacheOperations, CacheOptions};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Redis cache configuration
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "my-app:".to_string(),         // Key prefix
        Duration::from_secs(3600),     // Default TTL (1 hour)
    );
    
    // Create the Redis cache instance
    let cache = RedisCache::new(config).await?;
    
    // Basic key-value operations
    cache.set("user:123", &"John Doe", None).await?;
    
    // Get with type inference
    let user: Option<String> = cache.get("user:123").await?;
    println!("User: {:?}", user);
    
    // Set with custom TTL
    let options = CacheOptions {
        ttl: Some(Duration::from_secs(30)), // 30 seconds
    };
    cache.set("session:abc", &"session-data", Some(options)).await?;
    
    // Check if key exists
    let exists = cache.exists("user:123").await?;
    println!("User exists: {}", exists);
    
    // Delete a key
    let deleted = cache.delete("user:123").await?;
    println!("User deleted: {}", deleted);
    
    // Using counter operations
    let count = cache.increment("visits", 1).await?;
    println!("Visit count: {}", count);
    
    // Working with lists
    cache.list_push_right("recent_users", &"user1").await?;
    cache.list_push_right("recent_users", &"user2").await?;
    cache.list_push_right("recent_users", &"user3").await?;
    
    let users: Vec<String> = cache.list_range("recent_users", 0, -1).await?;
    println!("Recent users: {:?}", users);
    
    // Batch operations
    let keys = vec!["key1", "key2", "key3"];
    let values = vec!["value1", "value2", "value3"];
    
    let entries: Vec<(&str, &str)> = keys.iter().zip(values.iter())
        .map(|(k, v)| (*k, *v))
        .collect();
        
    cache.set_many(entries, None).await?;
    
    let results: Vec<Option<String>> = cache.get_many(keys).await?;
    println!("Batch results: {:?}", results);
    
    // Health check
    cache.health_check().await?;
    
    Ok(())
}
```

## Plugin Registration

To use Redis cache with the plugin system:

```rust
use navius_cache::plugin::PluginRegistry;
use navius_plugin::PluginRegistration;
use navius_cache_redis_plugin::register_plugin;

fn main() {
    let mut registry = PluginRegistry::new();
    
    // Register the Redis cache plugin
    register_plugin(&mut registry).expect("Failed to register Redis cache plugin");
    
    // Now you can create Redis cache instances through the registry
    // ...
}
```

## Configuration Options

### Redis URL Format

The Redis URL supports the following formats:

- `redis://[:<password>@]<hostname>[:port][/<db>]` - Standard Redis
- `rediss://[:<password>@]<hostname>[:port][/<db>]` - Redis with TLS encryption

### Connection Pool Configuration

Control pool behavior with:

```rust
let pool_config = PoolConfig {
    min_connections: Some(5),
    max_connections: Some(20),
    connect_timeout: Some(Duration::from_secs(3)),
    command_timeout: Some(Duration::from_secs(2)),
    connect_retries: 3,
    enable_connection_recycling: true,
    health_check_interval: Duration::from_secs(30),
};

let config = RedisCacheConfig::new(
    "redis://localhost:6379".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
).with_pool(pool_config);
```

### TLS Configuration

Enable TLS with:

```rust
let tls_config = TlsConfig {
    enabled: true,
    server_name: Some("redis.example.com".to_string()),
    ca_cert_path: Some("/path/to/ca.crt".to_string()),
    client_cert_path: None,
    client_key_path: None,
};

let config = RedisCacheConfig::new(
    "rediss://redis.example.com:6379".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
).with_tls(tls_config);
```

### Cluster Configuration

Enable cluster mode with:

```rust
let cluster_config = ClusterConfig {
    enabled: true,
    retry_count: 3,
    read_from_replicas: true,
};

let config = RedisCacheConfig::new(
    "redis://redis-cluster:6379".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
).with_cluster(cluster_config);
```

## Error Handling

The plugin provides comprehensive error handling with detailed error types:

```rust
use navius_cache_redis_plugin::RedisError;

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(RedisError::Connection(msg)) => println!("Connection error: {}", msg),
    Err(RedisError::Timeout(msg)) => println!("Timeout error: {}", msg),
    Err(err) => println!("Other error: {}", err),
}
```

## Performance Considerations

- Use batch operations (`set_many`, `get_many`) when possible
- Consider the trade-offs between consistency and performance when setting TTLs
- For high-throughput scenarios, adjust the connection pool size

## License

This project is licensed under the MIT License - see the LICENSE file for details. 