//! Metrics example for navius-cache
//!
//! This example demonstrates how to use the metrics functionality.
//! To run:
//! ```bash
//! cargo run --example metrics_usage --features metrics
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
        "memory://".to_string(),
        "metrics-example:".to_string(),
        Duration::from_secs(300),
    )
    .with_metrics(true) // Enable metrics
    .with_trace(true); // Enable tracing

    println!("Creating in-memory cache...");

    // Create in-memory cache
    let cache = CacheConnectionManager::new_memory(config);
    println!("Successfully created in-memory cache");

    // Clear any existing data
    println!("Clearing any previous data...");
    cache.clear().await?;

    println!("\nGenerating cache activity to produce metrics...");

    // Generate hits and misses
    for i in 0..5 {
        // Set some values
        cache
            .set(&format!("key:{}", i), &format!("value:{}", i), None)
            .await?;
        println!("Set key:{} -> value:{}", i, i);

        // Generate some hits
        let _: Option<String> = cache.get(&format!("key:{}", i)).await?;
        println!("Hit: Retrieved key:{}", i);

        // Generate some misses
        let _: Option<String> = cache.get(&format!("missing:{}", i)).await?;
        println!("Miss: Attempted to retrieve missing:{}", i);
    }

    // Generate some deletes
    for i in 0..3 {
        cache.delete(&format!("key:{}", i)).await?;
        println!("Delete: Removed key:{}", i);
    }

    // Generate some batch operations
    let entries = vec![
        ("batch:1", "value:1"),
        ("batch:2", "value:2"),
        ("batch:3", "value:3"),
    ];
    cache.set_many(entries, None).await?;
    println!("Set many: batch:1, batch:2, batch:3");

    let keys = vec!["batch:1", "batch:2", "batch:3", "batch:missing"];
    let _: Vec<Option<String>> = cache.get_many(keys).await?;
    println!("Get many: batch:1, batch:2, batch:3, batch:missing");

    // Generate some expirations
    let options = CacheOptions::new().ttl(Duration::from_millis(10));
    cache
        .set("expiring", "This will expire quickly", Some(options))
        .await?;
    println!("Set expiring key with 10ms TTL");

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(20)).await;
    let _: Option<String> = cache.get("expiring").await?;
    println!("Miss: Attempted to retrieve expired key");

    // Print metric values
    println!("\nMetrics were recorded for all operations above.");
    println!("Check the console output for metrics debugging information.");
    println!("\nYou should see metrics like:");
    println!(
        "  - navius_cache_operations_total{{operation=\"get\",backend=\"memory\",result=\"hit\"}}"
    );
    println!(
        "  - navius_cache_operations_total{{operation=\"get\",backend=\"memory\",result=\"miss\"}}"
    );
    println!(
        "  - navius_cache_operations_total{{operation=\"set\",backend=\"memory\",result=\"success\"}}"
    );
    println!("  - navius_cache_operation_duration_seconds{{operation=\"get\",backend=\"memory\"}}");

    // Clean up
    cache.clear().await?;

    Ok(())
}
