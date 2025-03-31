use navius_cache::{
    operations::Cache,
    serialization::{BinarySerializer, JsonSerializer, default_serializer},
};
use navius_cache_redis::{
    config::RedisCacheConfig, connection::RedisConnectionManager, operations::RedisCache,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
    is_active: bool,
    roles: Vec<String>,
    metadata: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Redis configuration
    let config = RedisCacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "serialization-example:".to_string(),
        Duration::from_secs(300),
    );

    // Create connection manager
    let connection_manager = RedisConnectionManager::new(config)
        .await
        .expect("Failed to create Redis connection manager");

    println!("Cache Serialization Example");
    println!("===========================");

    // Create test data
    let user = User {
        id: 42,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        is_active: true,
        roles: vec!["user".to_string(), "editor".to_string()],
        metadata: serde_json::json!({
            "preferences": {
                "theme": "dark",
                "notifications": true
            },
            "login_count": 27,
            "last_login": "2025-03-28T15:42:10Z"
        }),
    };

    // Example 1: Default JSON serializer
    {
        println!("\n1. Using default JSON serializer");
        let cache = RedisCache::new(connection_manager.clone())?;

        // Store user with JSON serialization
        cache.set("user:json", &user, None).await?;
        println!("Stored user with JSON serialization");

        // Retrieve user
        let retrieved_user: User = cache.get("user:json").await?.unwrap();
        println!(
            "Retrieved user: {} ({})",
            retrieved_user.name, retrieved_user.email
        );

        assert_eq!(user, retrieved_user);
    }

    // Example 2: Using binary serializer
    {
        println!("\n2. Using binary serializer");
        let binary_serializer = BinarySerializer::new();
        let cache = RedisCache::with_serializer(connection_manager.clone(), binary_serializer)?;

        // Store user with binary serialization
        cache.set("user:binary", &user, None).await?;
        println!("Stored user with binary serialization");

        // Retrieve user
        let retrieved_user: User = cache.get("user:binary").await?.unwrap();
        println!(
            "Retrieved user: {} ({})",
            retrieved_user.name, retrieved_user.email
        );

        assert_eq!(user, retrieved_user);
    }

    // Example 3: Performance comparison
    {
        println!("\n3. Performance comparison");
        let json_cache = RedisCache::with_serializer(connection_manager.clone(), JsonSerializer)?;
        let binary_cache =
            RedisCache::with_serializer(connection_manager.clone(), BinarySerializer::new())?;

        // Generate large test data
        let mut large_data = Vec::with_capacity(1000);
        for i in 0..1000 {
            large_data.push(User {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
                is_active: i % 2 == 0,
                roles: vec!["user".to_string()],
                metadata: serde_json::json!({
                    "preferences": {
                        "theme": i % 3 == 0 ? "dark" : "light",
                        "notifications": i % 5 == 0
                    },
                    "login_count": i % 100,
                    "last_login": "2025-03-29T10:00:00Z"
                }),
            });
        }

        // Measure JSON serialization time
        let start = std::time::Instant::now();
        for (i, item) in large_data.iter().enumerate().take(100) {
            json_cache
                .set(&format!("perf:json:{}", i), item, None)
                .await?;
        }
        let json_time = start.elapsed();
        println!("JSON serialization time (100 items): {:?}", json_time);

        // Measure binary serialization time
        let start = std::time::Instant::now();
        for (i, item) in large_data.iter().enumerate().take(100) {
            binary_cache
                .set(&format!("perf:binary:{}", i), item, None)
                .await?;
        }
        let binary_time = start.elapsed();
        println!("Binary serialization time (100 items): {:?}", binary_time);

        // Calculate improvement
        let improvement = if binary_time < json_time {
            let percentage = (json_time.as_micros() as f64 - binary_time.as_micros() as f64)
                / json_time.as_micros() as f64
                * 100.0;
            format!("Binary format is {:.2}% faster", percentage)
        } else {
            let percentage = (binary_time.as_micros() as f64 - json_time.as_micros() as f64)
                / binary_time.as_micros() as f64
                * 100.0;
            format!("JSON format is {:.2}% faster", percentage)
        };

        println!("Performance comparison: {}", improvement);
    }

    // Clean up
    let default_cache = RedisCache::new(connection_manager.clone())?;
    default_cache.delete("user:json").await?;
    default_cache.delete("user:binary").await?;

    // Clean up performance test keys
    for i in 0..100 {
        default_cache.delete(&format!("perf:json:{}", i)).await?;
        default_cache.delete(&format!("perf:binary:{}", i)).await?;
    }

    println!("\nCleanup complete");

    Ok(())
}
