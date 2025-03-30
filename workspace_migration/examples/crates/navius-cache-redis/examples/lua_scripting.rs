use navius_cache::error::CacheResult;
use navius_cache::operations::CacheOperations;
use navius_cache_redis::{RedisCache, RedisCacheConfig, RedisConnectionManager, RedisLuaScripting};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: u8,
}

// Example showing how to use Redis Lua scripting for atomic operations
#[tokio::main]
async fn main() -> CacheResult<()> {
    // Configure Redis
    let config = RedisCacheConfig {
        url: "redis://127.0.0.1:6379".to_string(),
        key_prefix: "example:lua:".to_string(),
        default_ttl: Duration::from_secs(60),
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
    let connection_manager = match RedisConnectionManager::new(config).await {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to connect to Redis: {}", e);
            eprintln!("Is Redis running on localhost:6379?");
            return Ok(());
        }
    };

    // Create cache
    let redis_cache = RedisCache::new(connection_manager)?;

    println!("===== Redis Lua Scripting Example =====");

    // Clean up keys from previous runs
    redis_cache.delete("counter").await?;
    redis_cache.delete("rate_limit").await?;
    redis_cache.delete("unique_user").await?;
    redis_cache.delete("hash_example").await?;

    // Example 1: Using execute_raw_script to evaluate a simple expression
    println!("\n----- Example 1: Simple Script Execution -----");
    let result: i64 = redis_cache
        .execute_raw_script(
            "return tonumber(ARGV[1]) * tonumber(ARGV[2])",
            &[],
            &["5", "7"],
        )
        .await?;
    println!("Simple script result (5 * 7): {}", result);

    // Example 2: Rate limiting with check_and_increment_counter
    println!("\n----- Example 2: Rate Limiting -----");
    println!("Simulating 10 API requests with a rate limit of 5 per minute:");

    let rate_limit_key = "rate_limit";
    let max_requests = 5;
    let ttl = Some(Duration::from_secs(60));

    for i in 1..=10 {
        let allowed = redis_cache
            .check_and_increment_counter(rate_limit_key, max_requests, ttl)
            .await?;

        println!(
            "Request {}: {}",
            i,
            if allowed { "Allowed" } else { "Rate Limited" }
        );
    }

    // Current counter value
    let counter_value: Option<i64> = redis_cache.get(rate_limit_key).await?;
    println!("Current rate limit counter: {:?}", counter_value);

    // Example 3: Atomic set-if-not-exists for unique resources
    println!("\n----- Example 3: Set If Not Exists (Atomic) -----");

    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
    };

    // First attempt should succeed
    let result1 = redis_cache
        .set_if_not_exists("unique_user", &user, Some(Duration::from_secs(300)))
        .await?;

    // Second attempt should fail
    let result2 = redis_cache
        .set_if_not_exists("unique_user", &user, Some(Duration::from_secs(300)))
        .await?;

    println!("First attempt to create unique user: {}", result1);
    println!("Second attempt to create unique user: {}", result2);

    // Example 4: Atomic hash field update with conditional check
    println!("\n----- Example 4: Conditional Hash Update -----");

    // First set the hash field
    redis_cache
        .set("hash_example:name", "initial_value")
        .await?;

    // Update if current value matches
    let update1 = redis_cache
        .update_hash_if_equals("hash_example", "name", "initial_value", &"new_value")
        .await?;

    // This should fail because the value is now different
    let update2 = redis_cache
        .update_hash_if_equals("hash_example", "name", "initial_value", &"another_value")
        .await?;

    println!("First update (expecting success): {}", update1);
    println!("Second update (expecting failure): {}", update2);

    // Get current value
    let current_value: Option<String> = redis_cache.get("hash_example:name").await?;
    println!("Current value: {:?}", current_value);

    // Example 5: Atomic increment and expire
    println!("\n----- Example 5: Atomic Increment and Expire -----");

    let key = "counter";
    let ttl = Duration::from_secs(60);

    let count1 = redis_cache.increment_and_expire(key, 5, ttl).await?;
    let count2 = redis_cache.increment_and_expire(key, 10, ttl).await?;

    println!("First increment by 5: {}", count1);
    println!("Second increment by 10: {}", count2);

    // Check TTL
    let remaining_ttl = redis_cache.ttl(key).await?;
    println!("Remaining TTL: {:?} seconds", remaining_ttl);

    // Example 6: Performance comparison - regular operations vs Lua script
    println!("\n----- Example 6: Performance Comparison -----");

    // Clean up
    for i in 0..1000 {
        redis_cache.delete(&format!("perf_test:{}", i)).await?;
    }

    // Standard approach - separate increment and expire calls
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("perf_test:{}", i);
        redis_cache.increment(&key, 1).await?;
        redis_cache.expire(&key, Duration::from_secs(60)).await?;
    }
    let standard_duration = start.elapsed();

    // Clean up
    for i in 0..1000 {
        redis_cache.delete(&format!("perf_test:{}", i)).await?;
    }

    // Lua script approach - combined atomic operation
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("perf_test:{}", i);
        redis_cache
            .increment_and_expire(&key, 1, Duration::from_secs(60))
            .await?;
    }
    let lua_duration = start.elapsed();

    println!(
        "Standard approach (separate calls): {:?}",
        standard_duration
    );
    println!("Lua script approach (atomic): {:?}", lua_duration);
    println!(
        "Performance improvement: {:.2}x",
        standard_duration.as_secs_f64() / lua_duration.as_secs_f64()
    );

    // Clean up all keys
    redis_cache.delete("counter").await?;
    redis_cache.delete("rate_limit").await?;
    redis_cache.delete("unique_user").await?;
    redis_cache.delete("hash_example").await?;
    for i in 0..1000 {
        redis_cache.delete(&format!("perf_test:{}", i)).await?;
    }

    println!("\nAll examples completed successfully!");

    Ok(())
}
