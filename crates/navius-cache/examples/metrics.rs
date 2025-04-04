//! Metrics example for navius-cache
//!
//! This example demonstrates metrics integration for cache operations
//!
//! Run with:
//! cargo run --example metrics --features metrics

use navius_cache::{CacheConfig, CacheConnectionManager, CacheOperations, CacheOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Navius Cache: Metrics Example");
    println!("=============================");

    // Set up metrics using Prometheus
    setup_metrics();

    // Create cache configuration with metrics enabled
    let config = CacheConfig::new(
        "memory://".to_string(),
        "metrics-example:".to_string(),
        Duration::from_secs(300),
    )
    .with_metrics(true);

    // Create the cache instance
    let cache = CacheConnectionManager::new_memory(config);

    // Clear any existing data
    cache.clear().await?;

    println!("\nPerforming cache operations...");

    // Set a value
    cache.set("key1", &"value1".to_string(), None).await?;
    println!("Set key1 = value1");

    // Get the value (hit)
    let _: Option<String> = cache.get("key1").await?;
    println!("Get key1 (hit)");

    // Get a non-existent key (miss)
    let _: Option<String> = cache.get("key2").await?;
    println!("Get key2 (miss)");

    // Set with options
    let options = CacheOptions::new().ttl(Duration::from_secs(10));
    cache
        .set("key3", &"value3".to_string(), Some(options))
        .await?;
    println!("Set key3 with 10s TTL");

    // Delete a key
    cache.delete("key1").await?;
    println!("Delete key1");

    // Wait a moment to allow metrics to be collected
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Print metrics information
    println!("\nFollowing metrics have been recorded:");
    println!(
        "- navius_cache_operations_total{{operation=\"get\",backend=\"memory\",result=\"hit\"}}"
    );
    println!(
        "- navius_cache_operations_total{{operation=\"get\",backend=\"memory\",result=\"miss\"}}"
    );
    println!(
        "- navius_cache_operations_total{{operation=\"set\",backend=\"memory\",result=\"success\"}}"
    );
    println!(
        "- navius_cache_operations_total{{operation=\"delete\",backend=\"memory\",result=\"success\"}}"
    );
    println!("- navius_cache_operation_duration_seconds{{operation=\"get\",backend=\"memory\"}}");
    println!("- navius_cache_operation_duration_seconds{{operation=\"set\",backend=\"memory\"}}");

    println!("\nExample completed successfully!");
    println!("In a real application, metrics would be:");
    println!("1. Exposed on an HTTP endpoint (e.g., /metrics)");
    println!("2. Scraped by Prometheus");
    println!("3. Visualized in Grafana dashboards");

    Ok(())
}

// Set up metrics using the Prometheus exporter
fn setup_metrics() {
    // Create a Prometheus handle for metrics
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder
        .install()
        .expect("Failed to install Prometheus recorder");
}
