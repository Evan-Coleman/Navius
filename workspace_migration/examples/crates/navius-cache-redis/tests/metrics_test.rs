use std::sync::Arc;
use std::time::Duration;

use metrics::{counter, gauge, histogram};
use metrics_util::{MetricKindMask, Quantile, Recorder, Summary};
use navius_cache::Cache;
use navius_cache_redis::{RedisCache, RedisConnectionManager};
use serde::{Deserialize, Serialize};
use tokio::test;

struct TestMetricsRecorder {
    counter_names: Vec<String>,
    gauge_names: Vec<String>,
    histogram_names: Vec<String>,
}

impl TestMetricsRecorder {
    fn new() -> Self {
        Self {
            counter_names: Vec::new(),
            gauge_names: Vec::new(),
            histogram_names: Vec::new(),
        }
    }

    fn contains_metric(&self, name: &str) -> bool {
        self.counter_names.contains(&name.to_string())
            || self.gauge_names.contains(&name.to_string())
            || self.histogram_names.contains(&name.to_string())
    }

    fn print_recorded_metrics(&self) {
        println!("Recorded counters:");
        for name in &self.counter_names {
            println!("  - {}", name);
        }

        println!("Recorded gauges:");
        for name in &self.gauge_names {
            println!("  - {}", name);
        }

        println!("Recorded histograms:");
        for name in &self.histogram_names {
            println!("  - {}", name);
        }
    }
}

impl Recorder for TestMetricsRecorder {
    fn register_counter(
        &self,
        key: &metrics::KeyName,
        _: &metrics::Unit,
        _: &metrics::Metadata,
    ) -> metrics::Counter {
        let mut recorder = self as *const Self as *mut Self;
        unsafe {
            (*recorder).counter_names.push(key.to_string());
        }
        metrics::Counter::noop()
    }

    fn register_gauge(
        &self,
        key: &metrics::KeyName,
        _: &metrics::Unit,
        _: &metrics::Metadata,
    ) -> metrics::Gauge {
        let mut recorder = self as *const Self as *mut Self;
        unsafe {
            (*recorder).gauge_names.push(key.to_string());
        }
        metrics::Gauge::noop()
    }

    fn register_histogram(
        &self,
        key: &metrics::KeyName,
        _: &metrics::Unit,
        _: &metrics::Metadata,
    ) -> metrics::Histogram {
        let mut recorder = self as *const Self as *mut Self;
        unsafe {
            (*recorder).histogram_names.push(key.to_string());
        }
        metrics::Histogram::noop()
    }

    fn describe_counter(&self, _: &metrics::KeyName, _: &metrics::Unit, _: &metrics::Metadata) {}
    fn describe_gauge(&self, _: &metrics::KeyName, _: &metrics::Unit, _: &metrics::Metadata) {}
    fn describe_histogram(&self, _: &metrics::KeyName, _: &metrics::Unit, _: &metrics::Metadata) {}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestUser {
    id: u64,
    name: String,
    score: i32,
}

async fn setup_redis() -> RedisCache {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());

    // Initialize the connection manager
    let connection_manager = Arc::new(
        RedisConnectionManager::new(&redis_url).expect("Failed to create Redis connection manager"),
    );

    // Create a cache with Lua scripting enabled
    RedisCache::new(connection_manager.clone()).with_lua_scripting()
}

#[test]
async fn test_metrics_registration() {
    // Install a custom metrics recorder
    let recorder = Box::new(TestMetricsRecorder::new());
    let recorder_ref = recorder.as_ref();
    let _handle = metrics::set_boxed_recorder(recorder).unwrap();

    // Initialize Redis cache
    let cache = setup_redis().await;

    // Generate some metrics by performing operations
    let key = "test_metrics_key";
    let value = TestUser {
        id: 1,
        name: "Test User".to_string(),
        score: 100,
    };

    // Clean up from previous test runs
    let _ = cache.delete(key).await;

    // Perform some operations to generate metrics
    let _ = cache.set(key, &value, Some(Duration::from_secs(60))).await;
    let _: Option<TestUser> = cache.get(key).await.unwrap_or(None);
    let _ = cache.delete(key).await;

    // Test that important metrics were recorded
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_set_duration_ms"),
        "Missing set duration metric"
    );
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_get_duration_ms"),
        "Missing get duration metric"
    );
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_delete_duration_ms"),
        "Missing delete duration metric"
    );
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_connection_acquire"),
        "Missing connection acquisition metric"
    );

    // Print recorded metrics for debugging
    recorder_ref.print_recorded_metrics();
}

#[test]
async fn test_lua_script_metrics() {
    // Install a custom metrics recorder
    let recorder = Box::new(TestMetricsRecorder::new());
    let recorder_ref = recorder.as_ref();
    let _handle = metrics::set_boxed_recorder(recorder).unwrap();

    // Initialize Redis cache
    let cache = setup_redis().await;

    // Initialize common scripts
    cache
        .initialize_common_scripts()
        .await
        .expect("Failed to initialize common scripts");

    // Generate metrics by performing lua script operations
    let key = "test_lua_metrics";
    let _ = cache.delete(key).await;

    // Use check_and_increment_counter to generate script metrics
    let _ = cache
        .check_and_increment_counter(key, 5, Some(Duration::from_secs(60)))
        .await;

    // Test that script metrics were recorded
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_script_execution_duration_seconds"),
        "Missing script execution duration metric"
    );

    // Clean up
    let _ = cache.delete(key).await;
}

#[test]
async fn test_connection_pool_metrics() {
    // Install a custom metrics recorder
    let recorder = Box::new(TestMetricsRecorder::new());
    let recorder_ref = recorder.as_ref();
    let _handle = metrics::set_boxed_recorder(recorder).unwrap();

    // Initialize Redis cache
    let cache = setup_redis().await;

    // Get connection manager to access stats
    let connection_manager = cache.connection_manager().clone();

    // Perform some operations to generate connection metrics
    for i in 0..5 {
        let key = format!("connection_pool_test_{}", i);
        let _ = cache.set(&key, &i, None).await;
        let _ = cache.get::<i32>(&key).await;
        let _ = cache.delete(&key).await;
    }

    // Get pool stats
    let stats = connection_manager.get_stats().await;

    // Record metrics manually to ensure they're generated
    navius_cache_redis::metrics::record_connection_pool_stats(
        stats.current_active_connections + stats.current_idle_connections,
        stats.current_idle_connections,
        stats.current_active_connections,
    );

    // Test that connection pool metrics were recorded
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_connection_pool_size"),
        "Missing connection pool size metric"
    );
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_connection_pool_idle"),
        "Missing idle connections metric"
    );
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_connection_pool_used"),
        "Missing used connections metric"
    );
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_connection_acquisition_time_seconds"),
        "Missing connection acquisition time metric"
    );
}

#[test]
async fn test_error_metrics() {
    // Install a custom metrics recorder
    let recorder = Box::new(TestMetricsRecorder::new());
    let recorder_ref = recorder.as_ref();
    let _handle = metrics::set_boxed_recorder(recorder).unwrap();

    // Initialize Redis cache
    let cache = setup_redis().await;

    // Generate an error by trying to deserialize an invalid value
    let key = "test_error_metrics";

    // First, set a valid value
    let _ = cache.set(key, "not_a_number", None).await;

    // Try to get it as a number (will cause deserialization error)
    let result: Result<i32, _> = cache.get(key).await;
    assert!(result.is_err(), "Expected an error when deserializing");

    // Clean up
    let _ = cache.delete(key).await;

    // Test that error metrics were recorded
    assert!(
        recorder_ref.contains_metric("navius_redis_cache_get_error"),
        "Missing get error metric"
    );
}
