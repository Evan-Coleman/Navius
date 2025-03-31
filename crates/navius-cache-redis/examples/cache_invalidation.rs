use navius_cache::{
    error::CacheResult,
    invalidation::{CacheInvalidator, CacheTtlManager},
    operations::Cache,
};
use navius_cache_redis::{
    config::RedisCacheConfig, connection::RedisConnectionManager, invalidation::RedisInvalidator,
    operations::RedisCache,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create Redis configuration (adjust connection string as needed)
    let config = RedisCacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "example:".to_string(),
        Duration::from_secs(300),
    );

    // Create connection manager and cache operations
    let connection_manager = RedisConnectionManager::new(config)
        .await
        .expect("Failed to create Redis connection manager");

    let cache = RedisCache::new(connection_manager.clone()).expect("Failed to create Redis cache");

    // Create invalidator
    let invalidator = RedisInvalidator::new(connection_manager);

    println!("Cache Invalidation Example");
    println!("==========================");

    // Clear any existing data
    invalidator.invalidate_all().await?;
    println!("Cleared all cache data");

    // Store some user objects
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
        },
    ];

    // Store users with tags
    for user in &users {
        let key = format!("user:{}", user.id);

        // Store in cache
        cache.set(&key, user, None).await?;
        println!("Stored user: {}", user.name);

        // Tag the cache entry
        invalidator.tag_key(&key, &["user", "entity"]).await?;
        println!("Tagged key '{}' with tags: user, entity", key);

        // Set TTL
        invalidator.set_ttl(&key, Duration::from_secs(600)).await?;
    }

    // Retrieve TTL for a key
    let key = "user:1";
    if let Some(ttl) = invalidator.get_ttl(key).await? {
        println!("TTL for '{}': {} seconds", key, ttl.as_secs());
    }

    // Get tags for a key
    let tags = invalidator.get_key_tags(key).await?;
    println!("Tags for '{}': {:?}", key, tags);

    // Retrieve a user
    if let Some(user) = cache.get::<User>(key).await? {
        println!("Retrieved user: {} ({})", user.name, user.email);
    }

    // Invalidate a specific key
    invalidator.invalidate_key(key).await?;
    println!("Invalidated key: {}", key);

    // Verify it's gone
    let result = cache.get::<User>(key).await?;
    println!(
        "After invalidation, key '{}' exists: {}",
        key,
        result.is_some()
    );

    // Store more data
    for i in 1..=5 {
        let key = format!("product:{}", i);
        cache.set(&key, &format!("Product {}", i), None).await?;
        invalidator.tag_key(&key, &["product", "entity"]).await?;
    }

    // Invalidate by pattern
    let count = invalidator.invalidate_by_pattern("product:").await?;
    println!("Invalidated {} keys matching pattern 'product:'", count);

    // Invalidate by tag
    let count = invalidator.invalidate_by_tag("user").await?;
    println!("Invalidated {} keys with tag 'user'", count);

    // Final cleanup
    invalidator.invalidate_all().await?;
    println!("Final cleanup: removed all keys");

    Ok(())
}
