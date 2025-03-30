use navius_cache::operations::Cache;
use navius_cache_redis::{RedisCache, RedisCacheConfig};
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
struct User {
    id: i32,
    name: String,
    email: String,
    active: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create a Redis cache configuration
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "example:".to_string(),
        Duration::from_secs(3600),
    );

    println!("Connecting to Redis...");

    // Step 2: Initialize the Redis cache (this will fail if Redis is not available)
    let cache = match RedisCache::new(config).await {
        Ok(cache) => {
            println!("Successfully connected to Redis");
            cache
        }
        Err(err) => {
            println!("Failed to connect to Redis: {}", err);
            println!("This example requires a running Redis server on localhost:6379");
            std::process::exit(1);
        }
    };

    // Step 3: Perform a health check
    let health = cache.health_check().await?;
    println!(
        "Redis health check: {}",
        if health { "OK" } else { "FAILED" }
    );

    // Step 4: Clear any existing data (for clean example)
    cache.clear().await?;
    println!("Cleared all existing data");

    // Step 5: Store some data
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        active: true,
    };

    println!("\nStoring user: {:?}", user);
    cache.set("user:1", &user, None).await?;
    println!("User stored successfully");

    // Step 6: Store some data with TTL
    let temp_user = User {
        id: 2,
        name: "Jane Smith".to_string(),
        email: "jane@example.com".to_string(),
        active: true,
    };

    println!("\nStoring temporary user with 10s TTL: {:?}", temp_user);
    cache
        .set("user:2", &temp_user, Some(Duration::from_secs(10)))
        .await?;
    println!("Temporary user stored successfully");

    // Step 7: Retrieve data
    println!("\nRetrieving user:1");
    let retrieved_user: Option<User> = cache.get("user:1").await?;
    println!("Retrieved user: {:?}", retrieved_user);

    // Step 8: Check if keys exist
    println!("\nChecking if keys exist:");
    println!("user:1 exists: {}", cache.exists("user:1").await?);
    println!("user:999 exists: {}", cache.exists("user:999").await?);

    // Step 9: Use increment/decrement
    println!("\nPerforming counter operations:");

    // Set an initial counter
    cache.set("counter", &10, None).await?;
    println!("Initial counter value: 10");

    // Increment it
    let counter = cache.increment("counter", 5).await?;
    println!("After increment by 5: {}", counter);

    // Decrement it
    let counter = cache.decrement("counter", 3).await?;
    println!("After decrement by 3: {}", counter);

    // Step 10: Check TTL on a key
    println!("\nChecking TTL on keys:");

    if let Some(ttl) = cache.ttl("user:2").await? {
        println!("TTL for user:2: {} seconds", ttl.as_secs());
    } else {
        println!("No TTL set for user:2 or key doesn't exist");
    }

    if let Some(ttl) = cache.ttl("user:1").await? {
        println!("TTL for user:1: {} seconds", ttl.as_secs());
    } else {
        println!("No TTL set for user:1 or key doesn't exist");
    }

    // Step 11: Delete a key
    println!("\nDeleting user:1");
    let deleted = cache.delete("user:1").await?;
    println!("Deleted: {}", deleted);

    // Verify it's gone
    let check: Option<User> = cache.get("user:1").await?;
    println!("After deletion, user:1 exists: {}", check.is_some());

    // Step 12: Wait for TTL expiration
    println!("\nWaiting for user:2 TTL to expire (5 seconds)...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check if still exists
    let still_exists = cache.exists("user:2").await?;
    let remaining_ttl = cache.ttl("user:2").await?;

    println!("After 5 seconds:");
    println!("user:2 exists: {}", still_exists);
    println!(
        "Remaining TTL: {:?} seconds",
        remaining_ttl.map(|d| d.as_secs())
    );

    println!("\nWaiting another 6 seconds...");
    tokio::time::sleep(Duration::from_secs(6)).await;

    // Check again
    let still_exists = cache.exists("user:2").await?;
    println!("After full TTL expiration:");
    println!("user:2 exists: {}", still_exists);

    println!("\nExample completed successfully!");

    Ok(())
}
