//! Example demonstrating cache metrics integration
//!
//! This example shows how to use the navius-cache crate with metrics enabled.
//! Run with:
//! ```
//! cargo run --example metrics --features metrics
//! ```

use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions, RedisCache};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the metrics registry
    // In a real application, you'd use a metrics exporter like metrics-exporter-prometheus
    let _ = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()
        .expect("Failed to install Prometheus recorder");

    // Set up custom metrics recorder to print metrics to the console
    metrics_util::debugging::print_stdout();

    // Create a cache configuration with metrics enabled
    let config = CacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "metrics-example:".to_string(),
        Duration::from_secs(60),
    )
    .with_metrics(true)
    .with_trace(true);

    println!("Connecting to Redis...");

    // Create a cache connection
    let cache = match CacheConnectionManager::new_redis(config.clone()).await {
        Ok(cache) => {
            println!("Successfully connected to Redis");
            cache
        }
        Err(e) => {
            println!(
                "Failed to connect to Redis: {}. Using mock implementation.",
                e
            );
            return Ok(());
        }
    };

    // Generate some cache activity
    println!("Generating cache activity...");

    // Perform a series of operations to generate metrics
    for i in 0..10 {
        let key = format!("test-key-{}", i);
        let value = format!("test-value-{}", i);

        // Set a value
        cache
            .set(
                &key,
                &value,
                Some(CacheOptions::new().ttl(Duration::from_secs(30))),
            )
            .await?;

        // Get the value we just set (cache hit)
        let _: Option<String> = cache.get(&key).await?;

        // Get a non-existent key (cache miss)
        let _: Option<String> = cache.get(format!("{}-nonexistent", key)).await?;

        // Delete the key
        cache.delete(&key).await?;
    }

    // Perform health check
    cache.health_check().await?;

    // Wait a moment to ensure all metrics are recorded
    println!("Cache operations completed. Metrics have been recorded.");
    println!("In a real application, these metrics would be exposed via Prometheus.");

    // In a real application, you would expose the metrics endpoint
    // For this example, we'll just wait a moment and then exit
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
