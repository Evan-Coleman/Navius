//! Metrics example for navius-cache
//!
//! This example demonstrates how to use the metrics functionality.
//! To run:
//! ```bash
//! cargo run --example metrics_usage --features metrics
//! ```

use navius_cache::{CacheConfig, CacheOptions, MemoryCache};
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
        "memory://".to_string(),
        "metrics-example:".to_string(),
        Duration::from_secs(300),
    )
    .with_metrics(true) // Enable metrics
    .with_trace(true); // Enable tracing

    println!("Creating memory cache...");

    // Create memory cache
    let cache = MemoryCache::new(config.key_prefix.unwrap_or_default(), config.default_ttl);
    println!("Successfully created memory cache");

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

    // Generate some errors
    println!("\nGenerating some error metrics...");

    // Try to use an unsupported operation
    println!("Attempting to use an unsupported operation (will generate an error metric)");
    let result = cache.hash_get::<_, _, String>("hash-key", "field").await;
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
    println!(
        "- navius_cache_operations_total{operation=\"get\",backend=\"memory\",result=\"hit\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"get\",backend=\"memory\",result=\"miss\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"set\",backend=\"memory\",result=\"success\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"delete\",backend=\"memory\",result=\"success\"}"
    );
    println!(
        "- navius_cache_operations_total{operation=\"hash_get\",backend=\"memory\",result=\"error\"}"
    );
    println!("- navius_cache_operation_duration_seconds{operation=\"get\",backend=\"memory\"}");
    println!("- navius_cache_operation_duration_seconds{operation=\"set\",backend=\"memory\"}");

    // Cleanup
    println!("\nCleaning up...");
    cache.clear().await?;

    println!("\nExample completed successfully!");
    Ok(())
}
