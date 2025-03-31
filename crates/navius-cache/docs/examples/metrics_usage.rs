//! Metrics example for navius-cache
//!
//! This example demonstrates how to use the metrics functionality.
//! To run:
//! ```bash
//! cargo run --example metrics_usage --features redis,metrics
//! ```

use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Navius Cache: Metrics Example");
    println!("=============================");

    // Initialize metrics with Prometheus exporter
    println!("Initializing metrics...");
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder
        .install()
        .expect("Failed to install Prometheus recorder");

    // Set up metrics debugging to print to stdout
    println!("Setting up metrics debugging...");
    metrics_util::debugging::print_stdout();

    // Create cache configuration with metrics enabled
    let config = CacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "metrics-example:".to_string(),
        Duration::from_secs(300),
    )
    .with_metrics(true) // Enable metrics
    .with_trace(true); // Enable tracing

    println!("Connecting to Redis...");

    // Connect to Redis
    let cache = match CacheConnectionManager::new_redis(config.clone()).await {
        Ok(cache) => {
            println!("Successfully connected to Redis");
            cache
        }
        Err(e) => {
            println!("Failed to connect to Redis: {}", e);
            println!("This example requires a running Redis instance.");
            return Ok(());
        }
    };

    // Clear any existing data
    println!("Clearing any previous data...");
    cache.clear().await?;

    println!("\nGenerating cache activity to produce metrics...");

    // Generate hits and misses
    for i in 0..5 {
        let key = format!("key-{}", i);

        // Set a value
        println!("Setting key: {}", key);
        cache.set(&key, &format!("value-{}", i), None).await?;

        // Get the value (should be a hit)
        println!("Getting key (hit): {}", key);
        let _: Option<String> = cache.get(&key).await?;

        // Get a non-existent key (should be a miss)
        let missing_key = format!("{}-missing", key);
        println!("Getting missing key (miss): {}", missing_key);
        let _: Option<String> = cache.get(&missing_key).await?;

        // Delete the key
        println!("Deleting key: {}", key);
        cache.delete(&key).await?;
    }

    // Generate some errors (trying to increment a string)
    println!("\nGenerating some error metrics...");

    // First set a string value
    cache.set("string-key", "not-a-number", None).await?;

    // Try to increment it (this will cause an error in Redis)
    println!("Attempting to increment a string value (will generate an error metric)");
    let result = cache.increment("string-key", 1).await;
    println!("Result as expected: {:?}", result);

    // Health check
    println!("\nPerforming health check (generates metrics)");
    cache.health_check().await?;

    // Wait a moment for metrics to be recorded
    println!("\nWaiting for metrics to be processed...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // In a real application, metrics would be:
    // 1. Scraped by Prometheus
    // 2. Visualized in Grafana
    // 3. Used for alerting

    println!("\nMetrics have been recorded. In a real application:");
    println!("- They would be exposed via a metrics endpoint (e.g., /metrics)");
    println!("- Prometheus would scrape the metrics");
    println!("- Dashboards would display cache hit/miss ratios, operation latency, etc.");

    println!("\nThe following metrics should have been recorded:");
    println!("- navius_cache_operations_total{operation=\"get\",backend=\"redis\",result=\"hit\"}");
    println!(
        "- navius_cache_operations_total{operation=\"get\",backend=\"redis\",result=\"miss\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"set\",backend=\"redis\",result=\"success\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"delete\",backend=\"redis\",result=\"success\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"increment\",backend=\"redis\",result=\"error\"}"
    );
    println!("- navius_cache_operation_duration_seconds{operation=\"get\",backend=\"redis\"}");
    println!("- navius_cache_operation_duration_seconds{operation=\"set\",backend=\"redis\"}");

    // Cleanup
    println!("\nCleaning up...");
    cache.clear().await?;

    println!("\nExample completed successfully!");
    Ok(())
}
