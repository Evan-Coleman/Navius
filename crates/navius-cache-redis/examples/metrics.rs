use metrics_exporter_prometheus::PrometheusBuilder;
use navius_cache_redis::{RedisCache, RedisCacheConfig, RedisCacheError, RedisConnectionManager};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Prometheus metrics recorder
    let prometheus = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    println!("Starting Redis cache metrics example");

    // Configure Redis cache
    let config = RedisCacheConfig::builder()
        .url("localhost:6379")
        .max_connections(10)
        .connection_timeout_seconds(5)
        .command_timeout_seconds(2)
        .key_prefix("metrics-example")
        .build();

    // Create connection manager and cache
    println!("Connecting to Redis...");
    let connection_manager = Arc::new(RedisConnectionManager::new(config).await?);
    let cache = RedisCache::new(connection_manager.clone());

    // Run a variety of operations to generate metrics
    println!("Running operations to generate metrics...");

    // Basic operations
    for i in 0..100 {
        let key = format!("test-key-{}", i % 10);
        let value = format!("test-value-{}", i).into_bytes();

        // Mix of operations to generate different metrics
        match i % 5 {
            0 => {
                let _ = cache.set_raw(&key, value).await;
            }
            1 => {
                let _ = cache.get_raw(&key).await;
            }
            2 => {
                let _ = cache.exists(&key).await;
                let _ = cache.expire(&key, 60).await;
            }
            3 => {
                // List operations
                let list_key = format!("list-{}", i % 5);
                let _ = cache.list_push(&list_key, value).await;
                let _ = cache.list_len(&list_key).await;
                let _ = cache.list_range(&list_key, 0, 10).await;
            }
            _ => {
                // Hash operations
                let hash_key = format!("hash-{}", i % 5);
                let field = format!("field-{}", i % 3);
                let _ = cache.hash_set(&hash_key, &field, value).await;
                let _ = cache.hash_get(&hash_key, &field).await;
            }
        }

        // Introduce some artificial errors
        if i % 20 == 0 {
            let _ = cache.get_raw("non-existent-key").await;
        }

        // Add small delay to spread operations over time
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Get and display the current connection pool statistics
    let stats = connection_manager.get_stats().await;
    println!("\nConnection Pool Statistics:");
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

    // Export and display Prometheus metrics
    let metrics_output = prometheus.render();
    println!("\nPrometheus Metrics:");
    println!("{}", metrics_output);

    // Clean up
    println!("\nCleaning up test keys...");
    for i in 0..10 {
        let _ = cache.delete(&format!("test-key-{}", i)).await;
    }
    for i in 0..5 {
        let _ = cache.delete(&format!("list-{}", i)).await;
        let _ = cache.delete(&format!("hash-{}", i)).await;
    }

    println!("Example completed");
    Ok(())
}
