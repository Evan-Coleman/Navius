use navius_cache::{
    config::CacheConfig,
    error::CacheResult,
    operations::Cache,
    serialization::{BinarySerializer, CompositeSerializer, JsonSerializer},
};
use navius_cache_redis::{
    config::RedisCacheConfig, connection::RedisConnectionManager, operations::RedisCache,
    pipeline::RedisPipeline,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

// Example data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

impl User {
    fn new(id: u64, name: &str, email: &str, active: bool) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
            active,
        }
    }
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Configure Redis cache
    let config = RedisCacheConfig {
        url: "redis://127.0.0.1:6379".to_string(),
        key_prefix: "navius:example:pipeline".to_string(),
        default_ttl: Duration::from_secs(300),
        max_connections: 10,
        database: 0,
        password: None,
        use_tls: false,
        connection_timeout_seconds: 5,
        command_timeout_seconds: 2,
        retry_commands: true,
        max_retries: 3,
    };

    // Create connection manager
    let conn_manager = RedisConnectionManager::new(config)
        .await
        .expect("Failed to create Redis connection manager");

    // Create cache with composite serializer
    let serializer = CompositeSerializer::new(Arc::new(JsonSerializer), Arc::new(BinarySerializer));
    let cache = RedisCache::with_serializer(conn_manager, serializer)
        .expect("Failed to create Redis cache");

    // Generate test data
    let users: Vec<(String, User)> = (1..=100)
        .map(|i| {
            (
                format!("user:{}", i),
                User::new(
                    i,
                    &format!("User {}", i),
                    &format!("user{}@example.com", i),
                    i % 2 == 0,
                ),
            )
        })
        .collect();

    println!("== Redis Pipelining Example ==");

    // Demonstrate individual operations vs pipelined operations
    println!("\n1. Individual SET operations:");
    let start = Instant::now();
    for (key, user) in &users[0..10] {
        cache.set(key, user, Some(Duration::from_secs(60))).await?;
    }
    let individual_set_duration = start.elapsed();
    println!("Time taken: {:?}", individual_set_duration);

    println!("\n2. Pipelined SET operations:");
    let start = Instant::now();
    let entries: Vec<(&str, &User)> = users[10..20].iter().map(|(k, v)| (k.as_str(), v)).collect();

    cache
        .set_many(&entries, Some(Duration::from_secs(60)))
        .await?;
    let pipelined_set_duration = start.elapsed();
    println!("Time taken: {:?}", pipelined_set_duration);
    println!(
        "Speedup factor: {:.2}x",
        individual_set_duration.as_micros() as f64 / pipelined_set_duration.as_micros() as f64
    );

    // Demonstrate get_many vs individual gets
    let keys_to_get: Vec<&str> = users[0..10].iter().map(|(k, _)| k.as_str()).collect();

    println!("\n3. Individual GET operations:");
    let start = Instant::now();
    for key in &keys_to_get {
        let _: Option<User> = cache.get(key).await?;
    }
    let individual_get_duration = start.elapsed();
    println!("Time taken: {:?}", individual_get_duration);

    println!("\n4. Pipelined GET operations:");
    let start = Instant::now();
    let results: Vec<Option<User>> = cache.get_many(&keys_to_get).await?;
    let pipelined_get_duration = start.elapsed();
    println!("Time taken: {:?}", pipelined_get_duration);
    println!(
        "Speedup factor: {:.2}x",
        individual_get_duration.as_micros() as f64 / pipelined_get_duration.as_micros() as f64
    );

    println!("\n5. Pipelined DELETE operations:");
    let start = Instant::now();
    let delete_results = cache.delete_many(&keys_to_get).await?;
    println!("Time taken: {:?}", start.elapsed());
    println!("Delete results: {:?}", delete_results);

    println!("\n6. Custom pipeline execution:");
    let start = Instant::now();

    // Create a custom pipeline that does multiple operations
    let results: Vec<redis::Value> = cache
        .execute_pipeline(|p| {
            p.set("custom:a", b"value-a")
                .set("custom:b", b"value-b")
                .get("custom:a")
                .get("custom:b")
                .exists("custom:c")
        })
        .await?;

    println!("Time taken: {:?}", start.elapsed());
    println!("Custom pipeline results: {:?}", results);

    // Clean up
    for i in 1..=100 {
        let key = format!("user:{}", i);
        let _ = cache.delete(&key).await;
    }
    let _ = cache.delete("custom:a").await;
    let _ = cache.delete("custom:b").await;

    println!("\nPipelined operations completed successfully!");
    Ok(())
}
