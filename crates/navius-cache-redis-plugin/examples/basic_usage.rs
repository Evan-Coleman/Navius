use navius_cache::{Cache, CacheKey, CacheOperations, CacheOptions};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig};
use std::error::Error;
use std::time::Duration;

/// This example demonstrates basic Redis cache operations and configuration
///
/// To run this example:
/// ```bash
/// # Start a Redis server if not already running
/// docker run --name redis-test -p 6379:6379 -d redis
///
/// # Run the example
/// cargo run --example basic_usage
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Redis Cache Plugin - Basic Usage Example");
    println!("----------------------------------------");

    // Create a Redis cache configuration with default settings
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "example:".to_string(),   // Key prefix for namespace isolation
        Duration::from_secs(300), // Default TTL (5 minutes)
    );

    println!("Connecting to Redis...");
    // Create the Redis cache instance
    let cache = RedisCache::new(config).await?;
    println!("Connected successfully!");

    println!("\n1. Basic Set and Get Operations");
    println!("-----------------------------");

    // Set a simple string value
    cache.set("greeting", &"Hello, Redis!", None).await?;
    println!("Set 'greeting' key with value 'Hello, Redis!'");

    // Get the value back
    let greeting: Option<String> = cache.get("greeting").await?;
    println!("Retrieved 'greeting' value: {:?}", greeting);

    // Check if the key exists
    let exists = cache.exists("greeting").await?;
    println!("Does 'greeting' exist? {}", exists);

    println!("\n2. Working with TTL (Time To Live)");
    println!("------------------------------");

    // Get the current TTL for a key
    let ttl = cache.ttl("greeting").await?;
    println!("TTL for 'greeting': {:?} seconds", ttl);

    // Set a custom TTL
    cache.expire("greeting", Duration::from_secs(60)).await?;
    println!("Updated TTL for 'greeting' to 60 seconds");

    // Get the updated TTL
    let new_ttl = cache.ttl("greeting").await?;
    println!("New TTL for 'greeting': {:?} seconds", new_ttl);

    // Set with custom TTL
    let options = CacheOptions {
        ttl: Some(Duration::from_secs(10)), // 10 seconds
    };
    cache
        .set("short_lived", &"I'll expire soon!", Some(options))
        .await?;
    println!("Set 'short_lived' key with 10-second TTL");

    println!("\n3. Counter Operations");
    println!("------------------");

    // Initialize a counter
    cache.set("visitor_count", &0, None).await?;
    println!("Initialized 'visitor_count' counter to 0");

    // Increment the counter
    let count1 = cache.increment("visitor_count", 1).await?;
    println!("Incremented by 1: {}", count1);

    let count2 = cache.increment("visitor_count", 5).await?;
    println!("Incremented by 5: {}", count2);

    // Get the final count
    let final_count: Option<i64> = cache.get("visitor_count").await?;
    println!("Final visitor count: {:?}", final_count);

    println!("\n4. Batch Operations");
    println!("-----------------");

    // Set multiple keys at once
    let entries = vec![
        ("user:1", "Alice"),
        ("user:2", "Bob"),
        ("user:3", "Charlie"),
    ];

    cache.set_many(entries, None).await?;
    println!("Set multiple user records at once");

    // Get multiple keys at once
    let keys = vec!["user:1", "user:2", "user:3", "user:4"]; // including a missing key
    let results: Vec<Option<String>> = cache.get_many(keys).await?;

    println!("Retrieved multiple users:");
    for (i, result) in results.iter().enumerate() {
        println!("  user:{} - {:?}", i + 1, result);
    }

    // Delete multiple keys at once
    let delete_keys = vec!["user:1", "user:2", "user:3"];
    let deleted_count = cache.delete_many(delete_keys).await?;
    println!("Deleted {} keys", deleted_count);

    println!("\n5. List Operations");
    println!("---------------");

    // Working with lists
    let list_key = "recent_logins";

    // Push elements to the list
    cache.list_push_right(list_key, &"user_session_1").await?;
    cache.list_push_right(list_key, &"user_session_2").await?;
    cache.list_push_right(list_key, &"user_session_3").await?;
    println!("Added 3 sessions to the '{}' list", list_key);

    // Get the list length
    let list_len = cache.list_len(list_key).await?;
    println!("List length: {}", list_len);

    // Get list elements
    let sessions: Vec<String> = cache.list_range(list_key, 0, -1).await?;
    println!("Recent login sessions: {:?}", sessions);

    // Pop an element
    let oldest_session: Option<String> = cache.list_pop_left(list_key).await?;
    println!("Removed oldest session: {:?}", oldest_session);

    // Get updated list
    let updated_sessions: Vec<String> = cache.list_range(list_key, 0, -1).await?;
    println!("Updated sessions: {:?}", updated_sessions);

    println!("\n6. Health Check");
    println!("------------");

    // Perform a health check
    match cache.health_check().await {
        Ok(_) => println!("Redis server is healthy"),
        Err(e) => println!("Health check failed: {}", e),
    }

    println!("\n7. Cleanup");
    println!("--------");

    // Clean up all the keys we created
    let cleanup_keys = vec![
        "example:greeting",
        "example:short_lived",
        "example:visitor_count",
        "example:recent_logins",
    ];

    for key in cleanup_keys {
        match cache.delete(&key[8..]).await {
            Ok(true) => println!("Cleaned up key: {}", key),
            Ok(false) => println!("Key not found: {}", key),
            Err(e) => println!("Error cleaning up {}: {}", key, e),
        }
    }

    println!("\nExample completed successfully!");
    Ok(())
}
