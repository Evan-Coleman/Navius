use std::sync::Arc;
use std::time::Duration;

use navius_cache::metrics::MetricsRegistry;
use navius_cache::{Cache, CacheError};
use navius_cache_redis::{RedisCache, RedisConnectionManager};
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct User {
    id: u64,
    name: String,
    last_login: u64,
}

#[tokio::main]
async fn main() -> Result<(), CacheError> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting metrics example application");

    // Initialize Redis connection
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());
    let connection_manager = Arc::new(
        RedisConnectionManager::new(&redis_url).expect("Failed to create Redis connection manager"),
    );

    // Create a cache with Lua scripting enabled
    let cache = RedisCache::new(connection_manager.clone()).with_lua_scripting();

    // Initialize common scripts
    cache
        .initialize_common_scripts()
        .await
        .expect("Failed to initialize common scripts");

    // Create a metrics HTTP server
    let metrics_port = 9091;
    tokio::spawn(start_metrics_server(metrics_port));
    info!("Metrics server started on port {}", metrics_port);
    info!(
        "You can view metrics at http://localhost:{}/metrics",
        metrics_port
    );

    // Run some example workloads to generate metrics
    let mut iteration = 0;
    loop {
        iteration += 1;
        info!("Running workload iteration {}", iteration);

        // Demonstrate different cache operations to generate various metrics
        run_basic_operations(&cache).await?;
        run_collection_operations(&cache).await?;
        run_atomic_operations(&cache).await?;

        // Introduce some failures to demonstrate error metrics
        if iteration % 3 == 0 {
            run_error_generating_operations(&cache).await;
        }

        // Every 5 iterations, print current metrics to console
        if iteration % 5 == 0 {
            print_current_metrics();
        }

        // Wait before next iteration
        time::sleep(Duration::from_secs(2)).await;
    }
}

async fn run_basic_operations(cache: &RedisCache) -> Result<(), CacheError> {
    // Set and get operations
    for i in 1..=10 {
        let key = format!("user:{}", i);
        let user = User {
            id: i,
            name: format!("User {}", i),
            last_login: chrono::Utc::now().timestamp() as u64,
        };

        // Set with TTL
        cache
            .set(&key, &user, Some(Duration::from_secs(300)))
            .await?;

        // Get immediately after (to simulate cache hit)
        let _: User = cache.get(&key).await?;
    }

    // Generate some cache misses
    for i in 100..=105 {
        let key = format!("user:{}", i);
        let result: Result<User, _> = cache.get(&key).await;
        if result.is_err() {
            // Expected cache miss, no need to handle
        }
    }

    // Set multiple items in a batch to test pipeline performance
    let mut pipeline_keys = Vec::new();
    for i in 20..=30 {
        let key = format!("pipeline_user:{}", i);
        pipeline_keys.push(key.clone());
        let user = User {
            id: i,
            name: format!("Pipeline User {}", i),
            last_login: chrono::Utc::now().timestamp() as u64,
        };
        cache
            .set(&key, &user, Some(Duration::from_secs(300)))
            .await?;
    }

    // Delete operations
    for key in pipeline_keys.iter().take(3) {
        cache.delete(key).await?;
    }

    Ok(())
}

async fn run_collection_operations(cache: &RedisCache) -> Result<(), CacheError> {
    // List operations
    let list_key = "example:list";

    // Clear existing list
    cache.delete(list_key).await?;

    // Add items to list
    for i in 1..=5 {
        cache
            .list_push_right(list_key, &format!("list_item_{}", i), None)
            .await?;
    }

    // Get list length
    let _: usize = cache.list_len(list_key).await?;

    // Get list items
    let _: Vec<String> = cache.list_range(list_key, 0, -1).await?;

    // Hash operations
    let hash_key = "example:hash";

    // Clear existing hash
    cache.delete(hash_key).await?;

    // Set hash fields
    for i in 1..=5 {
        cache
            .hash_set(
                hash_key,
                &format!("field_{}", i),
                &format!("value_{}", i),
                None,
            )
            .await?;
    }

    // Get hash values
    let _: Option<String> = cache.hash_get(hash_key, "field_1").await?;
    let _: std::collections::HashMap<String, String> = cache.hash_get_all(hash_key).await?;

    Ok(())
}

async fn run_atomic_operations(cache: &RedisCache) -> Result<(), CacheError> {
    // Atomic increment
    let counter_key = "example:counter";
    cache.set(counter_key, "10", None).await?;
    let _: i64 = cache.atomic_increment(counter_key, 5, None).await?;

    // Check and increment (rate limiter example)
    let rate_limit_key = format!("rate_limit:user:{}", rand::random::<u16>());
    cache.delete(&rate_limit_key).await?;

    let max_requests = 5;
    for i in 1..=max_requests + 2 {
        let result = cache
            .check_and_increment_counter(
                &rate_limit_key,
                max_requests,
                Some(Duration::from_secs(60)),
            )
            .await?;
        if i <= max_requests {
            assert!(
                result,
                "Should allow {} requests before rate limiting",
                max_requests
            );
        } else {
            assert!(!result, "Should deny requests after reaching limit");
        }
    }

    // Set if not exists (distributed lock example)
    let lock_key = format!("lock:resource:{}", rand::random::<u16>());
    cache.delete(&lock_key).await?;

    // First acquisition should succeed
    let acquired = cache
        .set_if_not_exists(&lock_key, "lock_owner_1", Some(Duration::from_secs(10)))
        .await?;
    assert!(acquired, "First lock acquisition should succeed");

    // Second acquisition should fail
    let acquired = cache
        .set_if_not_exists(&lock_key, "lock_owner_2", Some(Duration::from_secs(10)))
        .await?;
    assert!(!acquired, "Second lock acquisition should fail");

    // Release lock
    cache.delete(&lock_key).await?;

    // Atomic update example
    let user_key = format!("atomic_user:{}", rand::random::<u16>());
    let user = User {
        id: 42,
        name: "Initial User".to_string(),
        last_login: chrono::Utc::now().timestamp() as u64 - 86400, // 1 day ago
    };

    cache.set(&user_key, &user, None).await?;

    // Atomically update the user's last login
    let _updated_user = cache
        .atomic_update::<User, _, _>(
            &user_key,
            |existing_user| {
                let mut user = existing_user.unwrap();
                user.last_login = chrono::Utc::now().timestamp() as u64;
                Ok(user)
            },
            None,
        )
        .await?;

    Ok(())
}

async fn run_error_generating_operations(cache: &RedisCache) {
    // Try to get with wrong type to generate deserialization errors
    let key = "error_example:wrong_type";
    cache.set(key, "not_a_valid_user_json", None).await.ok();
    let _result: Result<User, _> = cache.get(key).await;

    // Try to use a non-existent Lua script
    let _result = cache
        .execute_script(
            "non_existent_script",
            vec!["key1".to_string()],
            vec!["arg1".to_string()],
        )
        .await;

    // Try to increment a non-numeric value
    let key = "error_example:non_numeric";
    cache.set(key, "not_a_number", None).await.ok();
    let _result: Result<i64, _> = cache.atomic_increment(key, 1, None).await;
}

async fn start_metrics_server(port: u16) {
    // Create a simple HTTP server that exposes the Prometheus metrics
    let addr = ([0, 0, 0, 0], port).into();

    let make_service = hyper::service::make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(hyper::service::service_fn(|_req| async {
            let metrics = get_metrics_output();
            Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from(metrics)))
        }))
    });

    let server = hyper::Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("Metrics server error: {}", e);
    }
}

fn get_metrics_output() -> String {
    // Get the metrics registry and encode all metrics to the Prometheus text format
    let registry = MetricsRegistry::get().registry();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();

    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn print_current_metrics() {
    let metrics_output = get_metrics_output();
    let filtered_metrics: Vec<&str> = metrics_output
        .lines()
        .filter(|line| line.starts_with("navius_redis_cache_") && !line.starts_with("#"))
        .collect();

    info!("Current Redis cache metrics (sample):");
    for (i, metric) in filtered_metrics.iter().enumerate().take(15) {
        info!("  {}", metric);
        if i == 14 && filtered_metrics.len() > 15 {
            info!("  ... and {} more", filtered_metrics.len() - 15);
            break;
        }
    }
}
