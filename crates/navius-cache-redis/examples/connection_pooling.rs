use navius_cache::error::CacheResult;
use navius_cache_redis::{RedisCache, RedisCacheConfig, RedisConnectionManager};
use rand::Rng;
use std::time::Duration;
use tokio::time::Instant;

// Example showing Redis connection pooling features
#[tokio::main]
async fn main() -> CacheResult<()> {
    println!("===== Redis Connection Pooling Example =====");

    // Configure Redis with optimized connection pooling
    let config = RedisCacheConfig {
        url: "redis://127.0.0.1:6379".to_string(),
        key_prefix: "example:pool:".to_string(),
        default_ttl: Duration::from_secs(60),
        max_connections: 15,
        min_connections: 5,
        database: 0,
        password: None,
        use_tls: false,
        connection_timeout_seconds: 2,
        command_timeout_seconds: 1,
        idle_timeout_seconds: 30,
        max_lifetime_seconds: 120,
        retry_commands: true,
        max_retries: 3,
        health_check_interval_seconds: 15,
        circuit_breaker_threshold: 5,
        circuit_reset_timeout_seconds: 5,
        enable_metrics: true,
    };

    // Create connection manager
    let connection_manager = match RedisConnectionManager::new(config.clone()).await {
        Ok(manager) => {
            println!("✅ Successfully connected to Redis");
            manager
        }
        Err(e) => {
            println!("❌ Failed to connect to Redis: {}", e);
            println!("Is Redis running on localhost:6379?");
            return Ok(());
        }
    };

    // Get initial stats
    let initial_stats = connection_manager.get_stats().await;
    println!("\n----- Initial Pool Stats -----");
    print_stats(&initial_stats);

    // Cache client
    let redis_cache = match RedisCache::new(connection_manager.clone())? {
        cache => {
            println!("✅ Successfully created Redis cache");
            cache
        }
    };

    // Example 1: Basic Connection Usage
    println!("\n----- Example 1: Single Connection Usage -----");
    let start = Instant::now();
    for i in 0..10 {
        let key = format!("test:key:{}", i);
        let value = format!("value-{}", i);

        match redis_cache.set(&key, &value, None).await {
            Ok(_) => println!("Set key: {}", key),
            Err(e) => println!("Failed to set key {}: {}", key, e),
        }
    }
    println!(
        "Time to execute 10 sequential operations: {:?}",
        start.elapsed()
    );

    // Check stats after basic usage
    let stats_after_basic = connection_manager.get_stats().await;
    println!("\n----- Stats After Basic Usage -----");
    print_stats(&stats_after_basic);

    // Example 2: Concurrent Connection Usage
    println!("\n----- Example 2: Concurrent Connection Usage -----");

    let start = Instant::now();
    let mut handles = vec![];

    // Create 20 concurrent tasks
    for i in 0..20 {
        let cache = redis_cache.clone();
        let handle = tokio::spawn(async move {
            let key = format!("test:concurrent:{}", i);
            let value = format!("concurrent-value-{}", i);

            // Add some random sleep to simulate real-world workload
            tokio::time::sleep(Duration::from_millis(rand::thread_rng().gen_range(10..50))).await;

            match cache.set(&key, &value, None).await {
                Ok(_) => (true, key),
                Err(_) => (false, key),
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    let mut failure_count = 0;

    for handle in handles {
        match handle.await {
            Ok((success, key)) => {
                if success {
                    success_count += 1;
                } else {
                    failure_count += 1;
                    println!("Failed to set key: {}", key);
                }
            }
            Err(e) => {
                println!("Task join error: {}", e);
                failure_count += 1;
            }
        }
    }

    println!(
        "Time to execute 20 concurrent operations: {:?}",
        start.elapsed()
    );
    println!("Successful operations: {}", success_count);
    println!("Failed operations: {}", failure_count);

    // Check stats after concurrent usage
    let stats_after_concurrent = connection_manager.get_stats().await;
    println!("\n----- Stats After Concurrent Usage -----");
    print_stats(&stats_after_concurrent);

    // Example 3: Connection Reuse
    println!("\n----- Example 3: Connection Reuse -----");

    // Wait for a moment to allow connections to be returned to the pool
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check pool status after allowing time for connections to be returned
    let stats_before_reuse = connection_manager.get_stats().await;
    println!("Stats before connection reuse:");
    print_stats(&stats_before_reuse);

    // Now use connections from the pool
    let start = Instant::now();

    for i in 0..10 {
        let key = format!("test:reuse:{}", i);
        let value = format!("reuse-value-{}", i);

        match redis_cache.set(&key, &value, None).await {
            Ok(_) => println!("Set key using pooled connection: {}", key),
            Err(e) => println!("Failed to set key {}: {}", key, e),
        }
    }

    println!(
        "Time to execute 10 operations with connection reuse: {:?}",
        start.elapsed()
    );

    // Final stats
    let final_stats = connection_manager.get_stats().await;
    println!("\n----- Final Pool Stats -----");
    print_stats(&final_stats);

    // Example 4: Ping test to verify connection health
    println!("\n----- Example 4: Connection Health Check -----");

    match connection_manager.ping().await {
        Ok(_) => println!("✅ Redis ping successful - connection is healthy"),
        Err(e) => println!("❌ Redis ping failed: {}", e),
    }

    // Clean up keys
    println!("\n----- Cleaning Up -----");

    for i in 0..20 {
        let _ = redis_cache.delete(&format!("test:key:{}", i)).await;
        let _ = redis_cache.delete(&format!("test:concurrent:{}", i)).await;
        let _ = redis_cache.delete(&format!("test:reuse:{}", i)).await;
    }

    println!("✅ Cleanup completed");
    println!("\nExample completed successfully!");

    Ok(())
}

// Helper function to print connection pool statistics
fn print_stats(stats: &navius_cache_redis::connection::PoolStats) {
    println!(
        "  Total connections created: {}",
        stats.total_connections_created
    );
    println!(
        "  Total connections closed: {}",
        stats.total_connections_closed
    );
    println!("  Total acquires: {}", stats.total_acquires);
    println!("  Total acquire failures: {}", stats.total_acquire_failures);
    println!("  Total acquire timeouts: {}", stats.total_acquire_timeouts);
    println!(
        "  Current active connections: {}",
        stats.current_active_connections
    );
    println!(
        "  Current idle connections: {}",
        stats.current_idle_connections
    );
}
