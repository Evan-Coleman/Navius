use metrics::{counter, gauge, histogram};
use metrics_util::MetricKindMask;
use navius_cache::key::CacheKey;
use navius_cache_redis::{
    RedisCache, RedisCacheConfig, RedisCacheError,
    TimedOperation,
};
use std::{sync::Arc, time::Duration};
use tokio::time;

// Helper struct for serialization tests
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct TestItem {
    id: i32,
    name: String,
}

// Custom metrics recorder for testing
#[derive(Debug, Default)]
struct TestMetricsRecorder {
    records: std::sync::Mutex<Vec<String>>,
}

impl TestMetricsRecorder {
    fn new() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn has_recorded(&self, pattern: &str) -> bool {
        let records = self.records.lock().unwrap();
        records.iter().any(|r| r.contains(pattern))
    }
}

impl metrics::Recorder for TestMetricsRecorder {
    fn register_counter(&self, key: &metrics::Key) -> metrics::Counter {
        let name = format!("{}", key);
        self.records
            .lock()
            .unwrap()
            .push(format!("register_counter: {}", name));
        metrics::Counter::noop()
    }

    fn register_gauge(&self, key: &metrics::Key) -> metrics::Gauge {
        let name = format!("{}", key);
        self.records
            .lock()
            .unwrap()
            .push(format!("register_gauge: {}", name));
        metrics::Gauge::noop()
    }

    fn register_histogram(&self, key: &metrics::Key) -> metrics::Histogram {
        let name = format!("{}", key);
        self.records
            .lock()
            .unwrap()
            .push(format!("register_histogram: {}", name));
        metrics::Histogram::noop()
    }

    fn increment_counter(&self, key: &metrics::Key, value: u64) {
        let name = format!("{}", key);
        self.records
            .lock()
            .unwrap()
            .push(format!("increment_counter: {} += {}", name, value));
    }

    fn update_gauge(&self, key: &metrics::Key, value: f64) {
        let name = format!("{}", key);
        self.records
            .lock()
            .unwrap()
            .push(format!("update_gauge: {} = {}", name, value));
    }

    fn record_histogram(&self, key: &metrics::Key, value: f64) {
        let name = format!("{}", key);
        self.records
            .lock()
            .unwrap()
            .push(format!("record_histogram: {} = {}", name, value));
    }

    fn described_as(&self, _key: &metrics::Key, _description: &'static str) {}

    fn metrics_are_enabled(&self, _mask: MetricKindMask) -> bool {
        true
    }
}

async fn setup_redis() -> Result<RedisCache, RedisCacheError> {
    // Set up metrics recorder for tests
    let recorder = Box::new(TestMetricsRecorder::new());
    let recorder = Arc::new(recorder);
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    // Configure Redis connection
    let config = RedisCacheConfig::builder()
        .with_url("redis://localhost:6379")
        .with_connection_timeout(Duration::from_secs(5))
        .with_pool_size(5)
        .build();

    let cache = RedisCache::new(config).await?;

    // Flush the database before starting tests
    let mut conn = cache.connection_manager().get_connection().await?;
    let _: () = redis::cmd("FLUSHDB")
        .query_async(&mut conn)
        .await
        .map_err(|e| RedisCacheError::CommandError(format!("Failed to flush database: {}", e)))?;

    Ok(cache)
}

#[tokio::test]
async fn test_set_add_remove() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let set_key = "test_set_add_remove";
    
    // Test adding single items
    let added = cache.set_add(set_key, vec!["item1"]).await?;
    assert_eq!(added, 1);
    
    // Test adding multiple items at once
    let added = cache.set_add(set_key, vec!["item2", "item3", "item4"]).await?;
    assert_eq!(added, 3);
    
    // Test adding duplicate items (should return 0)
    let added = cache.set_add(set_key, vec!["item1", "item2"]).await?;
    assert_eq!(added, 0);
    
    // Test set length
    let count = cache.set_length(set_key).await?;
    assert_eq!(count, 4);
    
    // Test removing items
    let removed = cache.set_remove(set_key, vec!["item1", "item3"]).await?;
    assert_eq!(removed, 2);
    
    // Test removing non-existent items
    let removed = cache.set_remove(set_key, vec!["nonexistent"]).await?;
    assert_eq!(removed, 0);
    
    // Verify remaining contents
    let members: Vec<String> = cache.set_members(set_key).await?;
    assert_eq!(members.len(), 2);
    assert!(members.contains(&"item2".to_string()));
    assert!(members.contains(&"item4".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_set_contains() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let set_key = "test_set_contains";
    
    // Add items to the set
    cache.set_add(set_key, vec!["item1", "item2", "item3"]).await?;
    
    // Test contains for existing item
    let contains = cache.set_contains(set_key, &"item2").await?;
    assert!(contains);
    
    // Test contains for non-existing item
    let contains = cache.set_contains(set_key, &"nonexistent").await?;
    assert!(!contains);
    
    // Test with complex object
    let item = TestItem { id: 42, name: "Test Item".to_string() };
    let complex_key = "test_set_complex";
    
    cache.set_add(complex_key, vec![&item]).await?;
    
    let contains = cache.set_contains(complex_key, &item).await?;
    assert!(contains);
    
    // Slightly different item should not match
    let different_item = TestItem { id: 42, name: "Different Test Item".to_string() };
    let contains = cache.set_contains(complex_key, &different_item).await?;
    assert!(!contains);
    
    Ok(())
}

#[tokio::test]
async fn test_set_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Create two sets with overlapping items
    let set1 = "test_set_ops_1";
    let set2 = "test_set_ops_2";
    let set3 = "test_set_ops_3";
    
    cache.set_add(set1, vec!["a", "b", "c", "d"]).await?;
    cache.set_add(set2, vec!["c", "d", "e", "f"]).await?;
    cache.set_add(set3, vec!["a", "c", "e", "g"]).await?;
    
    // Test intersection of two sets
    let intersection: Vec<String> = cache.set_intersection(vec![set1, set2]).await?;
    // Sort for deterministic comparison
    let mut intersection_sorted = intersection;
    intersection_sorted.sort();
    assert_eq!(intersection_sorted, vec!["c", "d"]);
    
    // Test intersection of three sets
    let intersection: Vec<String> = cache.set_intersection(vec![set1, set2, set3]).await?;
    assert_eq!(intersection, vec!["c"]);
    
    // Test storing intersection
    let dest_key = "test_intersection_store";
    let count = cache.set_intersection_store(dest_key, vec![set1, set2]).await?;
    assert_eq!(count, 2);
    
    let members: Vec<String> = cache.set_members(dest_key).await?;
    let mut members_sorted = members;
    members_sorted.sort();
    assert_eq!(members_sorted, vec!["c", "d"]);
    
    // Test union
    let union: Vec<String> = cache.set_union(vec![set1, set2]).await?;
    let mut union_sorted = union;
    union_sorted.sort();
    assert_eq!(union_sorted, vec!["a", "b", "c", "d", "e", "f"]);
    
    // Test storing union
    let dest_key = "test_union_store";
    let count = cache.set_union_store(dest_key, vec![set1, set3]).await?;
    assert_eq!(count, 6);
    
    let members: Vec<String> = cache.set_members(dest_key).await?;
    let mut members_sorted = members;
    members_sorted.sort();
    assert_eq!(members_sorted, vec!["a", "b", "c", "d", "e", "g"]);
    
    // Test difference
    let difference: Vec<String> = cache.set_difference(vec![set1, set2]).await?;
    let mut difference_sorted = difference;
    difference_sorted.sort();
    assert_eq!(difference_sorted, vec!["a", "b"]);
    
    // Test storing difference
    let dest_key = "test_difference_store";
    let count = cache.set_difference_store(dest_key, vec![set1, set2, set3]).await?;
    assert_eq!(count, 1);
    
    let members: Vec<String> = cache.set_members(dest_key).await?;
    assert_eq!(members, vec!["b"]);
    
    // Test with empty result
    let empty_diff: Vec<String> = cache.set_difference(vec![set1, "nonexistent_set"]).await?;
    assert_eq!(empty_diff, vec!["a", "b", "c", "d"]);
    
    Ok(())
}

#[tokio::test]
async fn test_set_random_members() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let set_key = "test_random_members";
    
    // Add items to the set
    let items = vec!["item1", "item2", "item3", "item4", "item5"];
    cache.set_add(set_key, items.clone()).await?;
    
    // Test getting one random member
    let random: Vec<String> = cache.set_random_members(set_key, 1).await?;
    assert_eq!(random.len(), 1);
    assert!(items.contains(&random[0].as_str()));
    
    // Test getting multiple random members
    let random: Vec<String> = cache.set_random_members(set_key, 3).await?;
    assert_eq!(random.len(), 3);
    for item in random {
        assert!(items.contains(&item.as_str()));
    }
    
    // Test getting more random members than exist in the set
    let random: Vec<String> = cache.set_random_members(set_key, 10).await?;
    assert_eq!(random.len(), 5); // Should return all 5 items
    
    // Test with empty set
    let empty_key = "empty_set";
    let random: Vec<String> = cache.set_random_members(empty_key, 1).await?;
    assert_eq!(random.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_set_metrics() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());
    
    let set_key = "metrics_test_set";
    
    // Test metrics for set_add
    {
        let _timer = TimedOperation::new("redis_cache_set_add");
        cache.set_add(set_key, vec!["item1", "item2"]).await?;
    }
    
    // Test metrics for set_contains
    {
        let _timer = TimedOperation::new("redis_cache_set_contains");
        cache.set_contains(set_key, &"item1").await?;
    }
    
    // Test metrics for set_members
    {
        let _timer = TimedOperation::new("redis_cache_set_members");
        let _: Vec<String> = cache.set_members(set_key).await?;
    }
    
    // Verify operation metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_set_add_duration_ms"));
    assert!(recorder.has_recorded("navius_redis_cache_set_contains_duration_ms"));
    assert!(recorder.has_recorded("navius_redis_cache_set_members_duration_ms"));
    
    Ok(())
} 