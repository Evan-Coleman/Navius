use metrics::{describe_counter, describe_gauge, describe_histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use navius_cache::operations::Cache;
use navius_cache_redis::{RedisCache, RedisCacheConfig};
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedItem {
    id: String,
    value: String,
    created_at: u64,
}

// Set up Prometheus metrics exporter
fn setup_metrics_exporter() -> PrometheusHandle {
    // Create a metrics registry with a Prometheus exporter
    let builder = PrometheusBuilder::new();

    let handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // Define and describe metrics
    describe_counter!("cache_hits_total", "Total number of cache hit operations");

    describe_counter!(
        "cache_misses_total",
        "Total number of cache miss operations"
    );

    describe_counter!(
        "cache_operations_total",
        "Total number of cache operations by type"
    );

    describe_histogram!(
        "cache_operation_duration_seconds",
        "Duration of cache operations in seconds"
    );

    describe_gauge!(
        "cache_entries_count",
        "Number of items currently in the cache"
    );

    handle
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up metrics
    let metrics_handle = setup_metrics_exporter();

    println!("Metrics server initialized");
    println!("Metrics available at: http://localhost:9000/metrics");

    // Start metrics endpoint
    tokio::spawn(async move {
        let server = hyper::Server::bind(&([127, 0, 0, 1], 9000).into()).serve(
            hyper::service::make_service_fn(|_| async {
                Ok::<_, hyper::Error>(hyper::service::service_fn(move |_| async move {
                    let metrics = metrics_handle.render();
                    Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from(metrics)))
                }))
            }),
        );

        if let Err(e) = server.await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    println!("Setting up Redis cache with metrics enabled...");

    // Create a Redis cache configuration
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "metrics_example:".to_string(),
        Duration::from_secs(300),
    );

    // Initialize the Redis cache
    let cache = match RedisCache::new(config).await {
        Ok(cache) => {
            println!("Successfully connected to Redis");
            cache
        }
        Err(err) => {
            println!("Failed to connect to Redis: {}", err);
            println!("This example requires a running Redis server on localhost:6379");
            std::process::exit(1);
        }
    };

    // Clear any existing data
    cache.clear().await?;
    println!("Cache cleared");

    // Simulate different cache operations to generate metrics
    println!("\nPerforming cache operations to generate metrics...");

    // 1. Cache misses
    println!("Generating cache misses...");
    for i in 1..5 {
        let _: Option<String> = cache.get(&format!("missing_key_{}", i)).await?;
    }

    // 2. Cache hits
    println!("Generating cache hits...");
    for i in 1..10 {
        let item = CachedItem {
            id: format!("item_{}", i),
            value: format!("value_{}", i),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Store the item
        cache.set(&format!("item:{}", i), &item, None).await?;

        // Retrieve it multiple times to generate hits
        for _ in 0..i {
            let _: Option<CachedItem> = cache.get(&format!("item:{}", i)).await?;
        }
    }

    // 3. Expirations
    println!("Setting up items with expiration...");
    for i in 1..4 {
        let item = CachedItem {
            id: format!("temp_item_{}", i),
            value: format!("temp_value_{}", i),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Store with short TTL
        cache
            .set(
                &format!("temp_item:{}", i),
                &item,
                Some(Duration::from_secs(2)),
            )
            .await?;
    }

    // 4. Deletions
    println!("Performing deletions...");
    for i in 1..4 {
        cache.delete(&format!("item:{}", i)).await?;
    }

    // 5. Increment/Decrement operations
    println!("Performing counter operations...");
    cache.set("counter", &0, None).await?;

    for i in 1..10 {
        cache.increment("counter", i).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    for i in 1..5 {
        cache.decrement("counter", i).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Wait for TTLs to expire
    println!("\nWaiting for TTL expirations (3 seconds)...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check if keys with TTL still exist
    for i in 1..4 {
        let exists = cache.exists(&format!("temp_item:{}", i)).await?;
        println!("temp_item:{} exists after TTL: {}", i, exists);
    }

    // Keep the server running
    println!("\nExample running. Metrics available at http://localhost:9000/metrics");
    println!("Press Ctrl+C to exit");

    // Sleep to keep the application running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
