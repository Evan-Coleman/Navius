use navius_cache_redis::{CacheOptions, RedisCache, RedisError, RedisHash, RedisList, RedisSet};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestData {
    id: i64,
    name: String,
    values: Vec<i32>,
}

// Helper function to measure and log execution time
async fn measure<F, T>(operation_name: &str, threshold_ms: u128, f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = f.await;
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    info!("{} took {}ms", operation_name, elapsed_ms);
    
    if elapsed_ms > threshold_ms {
        warn!(
            "🚨 PERFORMANCE REGRESSION: {} exceeded threshold of {}ms (actual: {}ms)",
            operation_name, threshold_ms, elapsed_ms
        );
    }
    
    result
}

#[tokio::test]
async fn test_string_operations_performance() -> Result<(), RedisError> {
    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis string operations performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Test data
    let key = "perf:test:string";
    let test_data = TestData {
        id: 1,
        name: "Performance Test".to_string(),
        values: vec![1, 2, 3, 4, 5],
    };
    
    // Clean up before test
    let _ = cache.delete(key).await;
    
    // Test SET operation
    measure("SET", 5, cache.set(key, &test_data, None)).await?;
    
    // Test GET operation
    let _: TestData = measure("GET", 5, cache.get(key)).await?.unwrap();
    
    // Test EXISTS operation
    let exists = measure("EXISTS", 5, cache.exists(key)).await?;
    assert!(exists);
    
    // Test DEL operation
    measure("DEL", 5, cache.delete(key)).await?;
    
    // Test EXPIRE operation
    cache.set(key, &test_data, None).await?;
    measure("EXPIRE", 5, cache.expire(key, Duration::from_secs(10))).await?;
    
    // Clean up
    cache.delete(key).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_list_operations_performance() -> Result<(), RedisError> {
    // Setup
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis list operations performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Test data
    let key = "perf:test:list";
    let test_data = vec![
        TestData { id: 1, name: "Item 1".to_string(), values: vec![1, 2] },
        TestData { id: 2, name: "Item 2".to_string(), values: vec![3, 4] },
        TestData { id: 3, name: "Item 3".to_string(), values: vec![5, 6] },
    ];
    
    // Clean up before test
    let _ = cache.delete(key).await;
    
    // Test LPUSH operation (add multiple items at once)
    measure("LPUSH (multiple)", 10, cache.lpush_multiple(key, &test_data)).await?;
    
    // Test LRANGE operation
    let items: Vec<TestData> = measure("LRANGE", 10, cache.lrange(key, 0, -1)).await?;
    assert_eq!(items.len(), 3);
    
    // Test LLEN operation
    let len = measure("LLEN", 5, cache.llen(key)).await?;
    assert_eq!(len, 3);
    
    // Test LPUSH single item
    let new_item = TestData { id: 4, name: "Item 4".to_string(), values: vec![7, 8] };
    measure("LPUSH (single)", 5, cache.lpush(key, &new_item)).await?;
    
    // Test RPOP operation
    let _: TestData = measure("RPOP", 5, cache.rpop(key)).await?.unwrap();
    
    // Clean up
    cache.delete(key).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_hash_operations_performance() -> Result<(), RedisError> {
    // Setup
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis hash operations performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Test data
    let key = "perf:test:hash";
    let field_values = vec![
        ("field1", "value1"),
        ("field2", "value2"),
        ("field3", "value3"),
        ("field4", "value4"),
        ("field5", "value5"),
    ];
    
    // Clean up before test
    let _ = cache.delete(key).await;
    
    // Test HSET multiple operation
    let fields_map: Vec<(&str, &str)> = field_values.clone();
    measure("HSET_MULTIPLE", 10, cache.hset_multiple(key, fields_map)).await?;
    
    // Test HGETALL operation
    let items: std::collections::HashMap<String, String> = measure("HGETALL", 10, cache.hgetall(key)).await?;
    assert_eq!(items.len(), 5);
    
    // Test HGET operation
    let value: String = measure("HGET", 5, cache.hget(key, "field1")).await?;
    assert_eq!(value, "value1");
    
    // Test HEXISTS operation
    let exists = measure("HEXISTS", 5, cache.hexists(key, "field1")).await?;
    assert!(exists);
    
    // Test HDEL operation
    measure("HDEL", 5, cache.hdel(key, "field1")).await?;
    
    // Test HLEN operation
    let len = measure("HLEN", 5, cache.hlen(key)).await?;
    assert_eq!(len, 4);
    
    // Clean up
    cache.delete(key).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_set_operations_performance() -> Result<(), RedisError> {
    // Setup
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis set operations performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Test data
    let key = "perf:test:set";
    let members = vec!["member1", "member2", "member3", "member4", "member5"];
    
    // Clean up before test
    let _ = cache.delete(key).await;
    
    // Test SADD multiple operation
    measure("SADD", 10, cache.sadd_multiple(key, &members)).await?;
    
    // Test SMEMBERS operation
    let items: Vec<String> = measure("SMEMBERS", 10, cache.smembers(key)).await?;
    assert_eq!(items.len(), 5);
    
    // Test SISMEMBER operation
    let is_member = measure("SISMEMBER", 5, cache.sismember(key, "member1")).await?;
    assert!(is_member);
    
    // Test SCARD operation
    let len = measure("SCARD", 5, cache.scard(key)).await?;
    assert_eq!(len, 5);
    
    // Test SREM operation
    measure("SREM", 5, cache.srem(key, "member1")).await?;
    
    // Clean up
    cache.delete(key).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_batch_operations_performance() -> Result<(), RedisError> {
    // Setup
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis batch operations performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Prepare test data - 1000 keys
    let keys: Vec<String> = (0..1000)
        .map(|i| format!("perf:test:batch:{}", i))
        .collect();
    
    // Clean up before test
    for key in &keys {
        let _ = cache.delete(key).await;
    }
    
    // Measure time to set 1000 keys in sequence
    let start = Instant::now();
    for i in 0..1000 {
        let key = &keys[i];
        let value = format!("value-{}", i);
        cache.set(key, &value, None).await?;
    }
    let sequential_time = start.elapsed();
    info!("Sequential SET of 1000 keys took {}ms", sequential_time.as_millis());
    
    // Use pipeline for batch operations
    let start = Instant::now();
    let mut pipeline = cache.pipeline();
    for i in 0..1000 {
        let key = &keys[i];
        let value = format!("value-{}", i);
        pipeline.set(key, &value, None);
    }
    pipeline.execute().await?;
    let pipeline_time = start.elapsed();
    info!("Pipeline SET of 1000 keys took {}ms", pipeline_time.as_millis());
    
    // Compare and assert performance improvement
    let improvement_factor = sequential_time.as_millis() as f64 / pipeline_time.as_millis() as f64;
    info!("Pipeline provides {}x speedup", improvement_factor);
    
    if improvement_factor < 5.0 {
        warn!(
            "🚨 PERFORMANCE REGRESSION: Pipeline speedup factor of {} is below threshold of 5.0",
            improvement_factor
        );
    }
    
    // Clean up after test
    let mut pipeline = cache.pipeline();
    for key in &keys {
        pipeline.delete(key);
    }
    pipeline.execute().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_cache_expiration_performance() -> Result<(), RedisError> {
    // Setup
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis cache expiration performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Test data
    let key = "perf:test:expiration";
    let value = "test-value";
    
    // Clean up before test
    let _ = cache.delete(key).await;
    
    // Test setting value with expiration
    let ttl = Duration::from_millis(200);
    measure(
        "SET with TTL",
        10,
        cache.set(key, &value, Some(CacheOptions::new().with_ttl(ttl)))
    ).await?;
    
    // Verify key exists
    let exists = cache.exists(key).await?;
    assert!(exists);
    
    // Test TTL command
    let remaining_ttl = measure("TTL", 5, cache.ttl(key)).await?;
    info!("Remaining TTL: {:?}ms", remaining_ttl.as_millis());
    
    // Wait for expiration
    info!("Waiting for key to expire...");
    sleep(Duration::from_millis(220)).await;
    
    // Verify key has expired
    let exists_after = cache.exists(key).await?;
    assert!(!exists_after, "Key should have expired");
    
    // Measure performance of accessing expired key
    let start = Instant::now();
    let result: Option<String> = cache.get(key).await?;
    let get_expired_time = start.elapsed();
    
    info!("GET on expired key took {}ms", get_expired_time.as_millis());
    assert!(result.is_none());
    
    if get_expired_time.as_millis() > 5 {
        warn!(
            "🚨 PERFORMANCE REGRESSION: GET on expired key took {}ms, exceeding threshold of 5ms",
            get_expired_time.as_millis()
        );
    }
    
    Ok(())
}

#[tokio::test]
async fn test_large_data_performance() -> Result<(), RedisError> {
    // Setup
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Redis large data performance test");
    
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379/0").await?;
    
    // Test data
    let key = "perf:test:large_data";
    
    // Generate a large JSON object (approximately 1MB)
    let large_object = generate_large_test_data(1000);
    
    // Clean up before test
    let _ = cache.delete(key).await;
    
    // Measure SET performance with large data
    measure("SET large data (1MB)", 50, cache.set(key, &large_object, None)).await?;
    
    // Measure GET performance with large data
    let _: Vec<TestData> = measure("GET large data (1MB)", 50, cache.get(key)).await?.unwrap();
    
    // Clean up
    cache.delete(key).await?;
    
    Ok(())
}

// Helper function to generate large test data
fn generate_large_test_data(count: usize) -> Vec<TestData> {
    let mut data = Vec::with_capacity(count);
    
    for i in 0..count {
        let values = (0..100).collect(); // 100 integers per item
        data.push(TestData {
            id: i as i64,
            name: format!("Large Item {}", i),
            values,
        });
    }
    
    data
} 