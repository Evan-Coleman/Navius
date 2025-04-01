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
async fn test_zset_add_remove() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let zset_key = "test_zset_add_remove";
    
    // Test adding items with scores
    let items = vec![
        (1.0, "item1"),
        (2.0, "item2"),
        (3.0, "item3"),
    ];
    
    let added = cache.zset_add(zset_key, items).await?;
    assert_eq!(added, 3);
    
    // Test adding with same score but different member
    let added = cache.zset_add(zset_key, vec![(2.0, "item4")]).await?;
    assert_eq!(added, 1);
    
    // Test updating score of existing member (should return 0 for newly added)
    let added = cache.zset_add(zset_key, vec![(5.0, "item2")]).await?;
    assert_eq!(added, 0);
    
    // Verify the score was updated
    let score = cache.zset_score(zset_key, &"item2").await?;
    assert_eq!(score, Some(5.0));
    
    // Test removing members
    let removed = cache.zset_remove(zset_key, vec!["item1", "item3"]).await?;
    assert_eq!(removed, 2);
    
    // Test removing non-existent members
    let removed = cache.zset_remove(zset_key, vec!["nonexistent"]).await?;
    assert_eq!(removed, 0);
    
    // Verify length after operations
    let count = cache.zset_length(zset_key).await?;
    assert_eq!(count, 2); // item2 and item4 should remain
    
    Ok(())
}

#[tokio::test]
async fn test_zset_score_and_increment() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let zset_key = "test_zset_score";
    
    // Add items with scores
    cache.zset_add(zset_key, vec![
        (10.5, "item1"),
        (20.5, "item2"),
    ]).await?;
    
    // Test getting score
    let score = cache.zset_score(zset_key, &"item1").await?;
    assert_eq!(score, Some(10.5));
    
    // Test getting score of non-existent item
    let score = cache.zset_score(zset_key, &"nonexistent").await?;
    assert_eq!(score, None);
    
    // Test incrementing score
    let new_score = cache.zset_increment_score(zset_key, &"item1", 5.0).await?;
    assert_eq!(new_score, 15.5);
    
    // Verify the incremented score
    let score = cache.zset_score(zset_key, &"item1").await?;
    assert_eq!(score, Some(15.5));
    
    // Test incrementing score of non-existent item (should add it)
    let new_score = cache.zset_increment_score(zset_key, &"new_item", 7.5).await?;
    assert_eq!(new_score, 7.5);
    
    // Verify the new item was added
    let score = cache.zset_score(zset_key, &"new_item").await?;
    assert_eq!(score, Some(7.5));
    
    // Test negative increment (decrement)
    let new_score = cache.zset_increment_score(zset_key, &"item2", -5.5).await?;
    assert_eq!(new_score, 15.0);
    
    Ok(())
}

#[tokio::test]
async fn test_zset_range() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let zset_key = "test_zset_range";
    
    // Add items with scores in specific order
    cache.zset_add(zset_key, vec![
        (10.0, "item1"),
        (20.0, "item2"),
        (30.0, "item3"),
        (40.0, "item4"),
        (50.0, "item5"),
    ]).await?;
    
    // Test range by index (start and stop inclusive)
    let members: Vec<String> = cache.zset_range(zset_key, 1, 3).await?;
    assert_eq!(members, vec!["item2", "item3", "item4"]);
    
    // Test range with negative indices (from the end)
    let members: Vec<String> = cache.zset_range(zset_key, -3, -1).await?;
    assert_eq!(members, vec!["item3", "item4", "item5"]);
    
    // Test range with scores
    let members_with_scores: Vec<(String, f64)> = cache.zset_range_with_scores(zset_key, 1, 3).await?;
    assert_eq!(members_with_scores, vec![
        ("item2".to_string(), 20.0),
        ("item3".to_string(), 30.0),
        ("item4".to_string(), 40.0),
    ]);
    
    // Test range by score
    let members: Vec<String> = cache.zset_range_by_score(zset_key, 15.0, 35.0).await?;
    assert_eq!(members, vec!["item2", "item3"]);
    
    // Test range by score with scores
    let members_with_scores: Vec<(String, f64)> = cache.zset_range_by_score_with_scores(zset_key, 15.0, 35.0).await?;
    assert_eq!(members_with_scores, vec![
        ("item2".to_string(), 20.0),
        ("item3".to_string(), 30.0),
    ]);
    
    // Test inclusive/exclusive score boundaries
    let members: Vec<String> = cache.zset_range_by_score(zset_key, 20.0, 30.0).await?;
    assert_eq!(members, vec!["item2", "item3"]);
    
    // Test count in score range
    let count = cache.zset_count(zset_key, 20.0, 40.0).await?;
    assert_eq!(count, 3); // item2, item3, item4
    
    Ok(())
}

#[tokio::test]
async fn test_zset_rank() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let zset_key = "test_zset_rank";
    
    // Add items with scores in specific order
    cache.zset_add(zset_key, vec![
        (10.0, "item1"),
        (20.0, "item2"),
        (30.0, "item3"),
        (40.0, "item4"),
        (50.0, "item5"),
    ]).await?;
    
    // Test getting rank (0-based index)
    let rank = cache.zset_rank(zset_key, &"item3").await?;
    assert_eq!(rank, Some(2)); // 0-based, so third item is index 2
    
    // Test getting rank of non-existent item
    let rank = cache.zset_rank(zset_key, &"nonexistent").await?;
    assert_eq!(rank, None);
    
    // Test getting reverse rank (highest score first)
    let rank = cache.zset_reverse_rank(zset_key, &"item3").await?;
    assert_eq!(rank, Some(2)); // 0-based, reversed, so third from end is index 2
    
    // Test getting reverse rank of first item (should be last in reverse order)
    let rank = cache.zset_reverse_rank(zset_key, &"item1").await?;
    assert_eq!(rank, Some(4)); // 0-based, last item in reverse order
    
    Ok(())
}

#[tokio::test]
async fn test_zset_store_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Create test sorted sets
    let zset1 = "test_zset_store_1";
    let zset2 = "test_zset_store_2";
    
    cache.zset_add(zset1, vec![
        (1.0, "a"),
        (2.0, "b"),
        (3.0, "c"),
    ]).await?;
    
    cache.zset_add(zset2, vec![
        (4.0, "c"),
        (5.0, "d"),
        (6.0, "e"),
    ]).await?;
    
    // Test intersection store (only common member is "c")
    let dest_key = "test_zset_inter_store";
    let count = cache.zset_intersection_store(
        dest_key,
        vec![zset1, zset2],
        None, // default weights
        None, // default SUM aggregate
    ).await?;
    
    assert_eq!(count, 1); // only "c" should be in the result
    
    // Verify the result with its score (should be 3.0 + 4.0 = 7.0)
    let members: Vec<(String, f64)> = cache.zset_range_with_scores(dest_key, 0, -1).await?;
    assert_eq!(members, vec![("c".to_string(), 7.0)]);
    
    // Test union store
    let dest_key = "test_zset_union_store";
    let count = cache.zset_union_store(
        dest_key,
        vec![zset1, zset2],
        None, // default weights
        None, // default SUM aggregate
    ).await?;
    
    assert_eq!(count, 5); // a, b, c, d, e should be in the result
    
    // Verify the result with scores
    let members: Vec<(String, f64)> = cache.zset_range_with_scores(dest_key, 0, -1).await?;
    
    // Expected: a(1.0), b(2.0), c(7.0), d(5.0), e(6.0)
    // Sorted by score: a, b, d, e, c
    assert_eq!(members.len(), 5);
    
    // Find member "c" which should have score 7.0 (3.0 + 4.0)
    let c_entry = members.iter().find(|(member, _)| member == "c").unwrap();
    assert_eq!(c_entry.1, 7.0);
    
    // Test with custom weights
    let dest_key = "test_zset_weighted_store";
    let count = cache.zset_union_store(
        dest_key,
        vec![zset1, zset2],
        Some(vec![2.0, 3.0]), // weight zset1 by 2, zset2 by 3
        None, // default SUM aggregate
    ).await?;
    
    assert_eq!(count, 5);
    
    // Verify weighted scores - "c" should have (3.0 * 2) + (4.0 * 3) = 6 + 12 = 18
    let members: Vec<(String, f64)> = cache.zset_range_with_scores(dest_key, 0, -1).await?;
    let c_entry = members.iter().find(|(member, _)| member == "c").unwrap();
    assert_eq!(c_entry.1, 18.0);
    
    // Test with custom aggregate function (MIN)
    let dest_key = "test_zset_min_store";
    let count = cache.zset_intersection_store(
        dest_key,
        vec![zset1, zset2],
        None, // default weights
        Some("MIN".to_string()), // use MIN aggregate
    ).await?;
    
    assert_eq!(count, 1);
    
    // Verify MIN aggregate - "c" should have min(3.0, 4.0) = 3.0
    let members: Vec<(String, f64)> = cache.zset_range_with_scores(dest_key, 0, -1).await?;
    assert_eq!(members, vec![("c".to_string(), 3.0)]);
    
    // Test with custom aggregate function (MAX)
    let dest_key = "test_zset_max_store";
    let count = cache.zset_intersection_store(
        dest_key,
        vec![zset1, zset2],
        None, // default weights
        Some("MAX".to_string()), // use MAX aggregate
    ).await?;
    
    assert_eq!(count, 1);
    
    // Verify MAX aggregate - "c" should have max(3.0, 4.0) = 4.0
    let members: Vec<(String, f64)> = cache.zset_range_with_scores(dest_key, 0, -1).await?;
    assert_eq!(members, vec![("c".to_string(), 4.0)]);
    
    Ok(())
}

#[tokio::test]
async fn test_zset_with_complex_objects() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let zset_key = "test_zset_objects";
    
    // Create test items
    let item1 = TestItem { id: 1, name: "Item One".to_string() };
    let item2 = TestItem { id: 2, name: "Item Two".to_string() };
    let item3 = TestItem { id: 3, name: "Item Three".to_string() };
    
    // Add items to the sorted set
    cache.zset_add(zset_key, vec![
        (10.5, &item1),
        (20.5, &item2),
        (30.5, &item3),
    ]).await?;
    
    // Test retrieving score
    let score = cache.zset_score(zset_key, &item2).await?;
    assert_eq!(score, Some(20.5));
    
    // Test retrieving range
    let items: Vec<TestItem> = cache.zset_range(zset_key, 0, -1).await?;
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], item1);
    assert_eq!(items[1], item2);
    assert_eq!(items[2], item3);
    
    // Test range with scores
    let items_with_scores: Vec<(TestItem, f64)> = cache.zset_range_with_scores(zset_key, 0, -1).await?;
    assert_eq!(items_with_scores.len(), 3);
    assert_eq!(items_with_scores[0].0, item1);
    assert_eq!(items_with_scores[0].1, 10.5);
    assert_eq!(items_with_scores[1].0, item2);
    assert_eq!(items_with_scores[1].1, 20.5);
    assert_eq!(items_with_scores[2].0, item3);
    assert_eq!(items_with_scores[2].1, 30.5);
    
    // Test removing item
    cache.zset_remove(zset_key, vec![&item2]).await?;
    
    // Verify item was removed
    let items: Vec<TestItem> = cache.zset_range(zset_key, 0, -1).await?;
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], item1);
    assert_eq!(items[1], item3);
    
    Ok(())
}

#[tokio::test]
async fn test_zset_metrics() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());
    
    let zset_key = "metrics_test_zset";
    
    // Test metrics for zset_add
    {
        let _timer = TimedOperation::new("redis_cache_zset_add");
        cache.zset_add(zset_key, vec![(1.0, "item1"), (2.0, "item2")]).await?;
    }
    
    // Test metrics for zset_range
    {
        let _timer = TimedOperation::new("redis_cache_zset_range");
        let _: Vec<String> = cache.zset_range(zset_key, 0, -1).await?;
    }
    
    // Test metrics for zset_score
    {
        let _timer = TimedOperation::new("redis_cache_zset_score");
        let _ = cache.zset_score(zset_key, &"item1").await?;
    }
    
    // Verify operation metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_zset_add_duration_ms"));
    assert!(recorder.has_recorded("navius_redis_cache_zset_range_duration_ms"));
    assert!(recorder.has_recorded("navius_redis_cache_zset_score_duration_ms"));
    
    Ok(())
} 