//! Basic usage example for navius-cache
//!
//! This example demonstrates common cache operations using the in-memory backend.
//! To run:
//! ```bash
//! cargo run --example basic_usage
//! ```

use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Navius Cache: Basic Usage Example");
    println!("==================================");

    // Create cache configuration
    let config = CacheConfig::new(
        "memory://".to_string(),
        "example:".to_string(),
        Duration::from_secs(300), // 5 minutes default TTL
    );

    println!("Creating in-memory cache...");

    // Create in-memory cache
    let cache = CacheConnectionManager::new_memory(config);
    println!("Successfully created in-memory cache");

    // Clear any existing data from previous runs
    println!("Clearing any previous data...");
    cache.clear().await?;

    // Storing a simple string value
    println!("\nStoring a simple string value");
    cache.set("greeting", "Hello, Cache World!", None).await?;

    // Retrieving the string value
    let greeting: Option<String> = cache.get("greeting").await?;
    println!("Retrieved value: {:?}", greeting);

    // Storing a complex object
    println!("\nStoring a complex object (User)");
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    cache.set("user:42", &user, None).await?;

    // Retrieving the complex object
    let retrieved_user: Option<User> = cache.get("user:42").await?;
    println!("Retrieved user: {:?}", retrieved_user);

    // Using custom TTL
    println!("\nStoring a value with custom TTL (10 seconds)");
    let options = CacheOptions::new().ttl(Duration::from_secs(10));
    cache
        .set("short-lived", "I will expire soon", Some(options))
        .await?;

    println!("Value exists now: {}", cache.exists("short-lived").await?);

    println!("Waiting for expiration (11 seconds)...");
    tokio::time::sleep(Duration::from_secs(11)).await;

    println!(
        "Value exists after TTL: {}",
        cache.exists("short-lived").await?
    );

    // Batch operations
    println!("\nPerforming batch operations");
    let entries = vec![
        ("batch:1", "First value"),
        ("batch:2", "Second value"),
        ("batch:3", "Third value"),
    ];

    println!("Setting multiple values at once");
    cache.set_many(entries, None).await?;

    println!("Getting multiple values at once");
    let keys = vec!["batch:1", "batch:2", "batch:3", "batch:missing"];
    let values: Vec<Option<String>> = cache.get_many(keys).await?;

    for (i, value) in values.iter().enumerate() {
        println!("Value {}: {:?}", i + 1, value);
    }

    // Increment operations
    println!("\nPerforming increment operations");
    println!("Initial value: {}", cache.increment("counter", 0).await?);
    println!("Increment by 5: {}", cache.increment("counter", 5).await?);
    println!("Increment by 10: {}", cache.increment("counter", 10).await?);

    // Cleanup
    println!("\nCleaning up...");
    cache.clear().await?;
    println!("Cache cleared");

    // Health check
    println!("\nPerforming health check");
    match cache.health_check().await {
        Ok(_) => println!("Cache is healthy!"),
        Err(e) => println!("Cache health check failed: {}", e),
    }

    println!("\nExample completed successfully!");
    Ok(())
}
