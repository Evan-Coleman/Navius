use metrics::{counter, gauge, histogram};
use metrics_util::MetricKindMask;
use navius_cache::key::CacheKey;
use navius_cache_redis::{
    RedisCache, RedisCacheConfig, RedisCacheError,
    TimedOperation,
};
use redis::{pipe, Pipeline, Value as RedisValue};
use std::{sync::Arc, time::Duration};
use tokio::time;

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
async fn test_basic_pipeline() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Use pipeline to set multiple keys at once
    let result = cache.connection_manager().execute_pipeline_command("batch_set", |mut conn| {
        let mut pipe = pipe();
        pipe.set("pipeline_key1", "value1")
            .set("pipeline_key2", "value2")
            .set("pipeline_key3", "value3");
        pipe.query_async(&mut conn)
    }).await?;
    
    // Verify keys were set properly
    let value1: String = cache.get("pipeline_key1").await?;
    let value2: String = cache.get("pipeline_key2").await?;
    let value3: String = cache.get("pipeline_key3").await?;
    
    assert_eq!(value1, "value1");
    assert_eq!(value2, "value2");
    assert_eq!(value3, "value3");
    
    Ok(())
}

#[tokio::test]
async fn test_pipeline_with_results() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Set some keys first
    cache.set("pipeline_res1", "value1", None).await?;
    cache.set("pipeline_res2", "value2", None).await?;
    
    // Use pipeline to get multiple keys at once and manipulate values
    let results: Vec<RedisValue> = cache.connection_manager().execute_pipeline_command("batch_get", |mut conn| {
        let mut pipe = pipe();
        pipe.get("pipeline_res1")
            .get("pipeline_res2")
            .exists("pipeline_nonexistent");
        pipe.query_async(&mut conn)
    }).await?;
    
    // Verify the results
    assert_eq!(results.len(), 3);
    
    // Convert RedisValue to expected types
    let value1: String = redis::from_redis_value(&results[0])?;
    let value2: String = redis::from_redis_value(&results[1])?;
    let exists: bool = redis::from_redis_value(&results[2])?;
    
    assert_eq!(value1, "value1");
    assert_eq!(value2, "value2");
    assert_eq!(exists, false);
    
    Ok(())
}

#[tokio::test]
async fn test_large_pipeline() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Create a pipeline with many operations
    let num_operations = 1000;
    
    let operation_start = std::time::Instant::now();
    
    // Benchmark individual operations
    for i in 0..100 {
        let key = format!("single_key_{}", i);
        let value = format!("value_{}", i);
        cache.set(&key, &value, None).await?;
    }
    
    let single_ops_time = operation_start.elapsed();
    
    // Now benchmark pipeline operations
    let pipeline_start = std::time::Instant::now();
    
    // Execute a large pipeline of SET operations
    cache.connection_manager().execute_pipeline_command("large_batch_set", |mut conn| {
        let mut pipe = pipe();
        for i in 0..num_operations {
            let key = format!("pipeline_large_key_{}", i);
            let value = format!("value_{}", i);
            pipe.set(key, value);
        }
        pipe.query_async(&mut conn)
    }).await?;
    
    let pipeline_time = pipeline_start.elapsed();
    
    // Verify a sample of keys were set
    let value_500: String = cache.get("pipeline_large_key_500").await?;
    let value_999: String = cache.get("pipeline_large_key_999").await?;
    
    assert_eq!(value_500, "value_500");
    assert_eq!(value_999, "value_999");
    
    // Pipeline should be significantly faster than individual operations
    // Print performance comparison for debugging
    println!("Single operations (100): {:?}", single_ops_time);
    println!("Pipeline operations ({}): {:?}", num_operations, pipeline_time);
    println!("Performance improvement factor: {:.2}x", 
        (single_ops_time.as_micros() as f64 / 100.0) / 
        (pipeline_time.as_micros() as f64 / num_operations as f64));
    
    Ok(())
}

#[tokio::test]
async fn test_multi_command_pipeline() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Set a mix of command types in a single pipeline
    let results: Vec<RedisValue> = cache.connection_manager().execute_pipeline_command("mixed_commands", |mut conn| {
        let mut pipe = pipe();
        
        // String operations
        pipe.set("pipeline_multi_str", "string_value");
        
        // List operations
        pipe.lpush("pipeline_multi_list", "list_item1");
        pipe.lpush("pipeline_multi_list", "list_item2");
        pipe.llen("pipeline_multi_list");
        
        // Set operations
        pipe.sadd("pipeline_multi_set", "set_item1");
        pipe.sadd("pipeline_multi_set", "set_item2");
        pipe.sismember("pipeline_multi_set", "set_item1");
        
        // Hash operations
        pipe.hset("pipeline_multi_hash", "field1", "hash_value1");
        pipe.hset("pipeline_multi_hash", "field2", "hash_value2");
        pipe.hget("pipeline_multi_hash", "field1");
        
        pipe.query_async(&mut conn)
    }).await?;
    
    // First two commands are SET and LPUSH which return "OK" and 1
    // We're most interested in the query responses
    let list_len: i64 = redis::from_redis_value(&results[3])?;
    let set_contains: bool = redis::from_redis_value(&results[6])?;
    let hash_value: String = redis::from_redis_value(&results[9])?;
    
    assert_eq!(list_len, 2);
    assert_eq!(set_contains, true);
    assert_eq!(hash_value, "hash_value1");
    
    Ok(())
}

#[tokio::test]
async fn test_pipeline_transaction() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Test transactional pipeline (MULTI/EXEC)
    let results: Vec<RedisValue> = cache.connection_manager().execute_pipeline_command("transaction", |mut conn| {
        let mut pipe = pipe();
        
        // Create a transactional pipeline
        pipe.atomic();
        
        // Set operations within transaction
        pipe.set("tx_key1", "tx_value1");
        pipe.set("tx_key2", "tx_value2");
        pipe.get("tx_key1");
        pipe.get("tx_key2");
        
        pipe.query_async(&mut conn)
    }).await?;
    
    // Verify results - first two are OK from SET, next two are GET results
    let value1: String = redis::from_redis_value(&results[2])?;
    let value2: String = redis::from_redis_value(&results[3])?;
    
    assert_eq!(value1, "tx_value1");
    assert_eq!(value2, "tx_value2");
    
    Ok(())
}

#[tokio::test]
async fn test_pipeline_error_handling() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Test with an invalid command that should cause an error
    let result = cache.connection_manager().execute_pipeline_command("error_command", |mut conn| {
        let mut pipe = pipe();
        
        // Valid command
        pipe.set("error_key1", "value1");
        
        // Invalid command - using a string where a list is expected
        pipe.set("error_key2", "not_a_list");
        pipe.lpop::<_, String>("error_key2");
        
        pipe.query_async(&mut conn)
    });
    
    // The pipeline should fail due to the invalid command
    assert!(result.await.is_err());
    
    // Verify that the transaction was aborted - first key should not be set
    let exists = cache.exists("error_key1").await?;
    assert!(!exists);
    
    Ok(())
}

#[tokio::test]
async fn test_pipeline_timeout() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    
    // Create a pipeline with a lot of operations to potentially trigger timeout
    // Then attempt to execute it with a very short timeout
    let config = RedisCacheConfig::builder()
        .with_url("redis://localhost:6379")
        .with_connection_timeout(Duration::from_millis(1)) // Extremely short timeout
        .with_operation_timeout(Duration::from_millis(1)) // Extremely short timeout
        .with_pool_size(1)
        .build();
    
    let short_timeout_cache = RedisCache::new(config).await?;
    
    // This should likely time out
    let result = short_timeout_cache.connection_manager().execute_pipeline_command("timeout_test", |mut conn| {
        let mut pipe = pipe();
        
        // Add many operations to increase likelihood of timeout
        for i in 0..10000 {
            pipe.set(format!("timeout_key_{}", i), format!("value_{}", i));
        }
        
        pipe.query_async(&mut conn)
    }).await;
    
    // We either got a timeout error or it was successful (depends on machine speed)
    // Just make sure we didn't crash
    match result {
        Ok(_) => {
            // Operation succeeded, verify a sample key was set
            let value: Result<String, _> = short_timeout_cache.get("timeout_key_5000").await;
            if value.is_ok() {
                assert_eq!(value.unwrap(), "value_5000");
            }
        },
        Err(e) => {
            // Operation failed with error, but that's expected with the short timeout
            println!("Expected timeout error: {:?}", e);
            // Should be a timeout error or connection error
            assert!(matches!(e, 
                RedisCacheError::Timeout(_) | 
                RedisCacheError::ConnectionError(_) |
                RedisCacheError::OperationError(_)
            ));
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_pipeline_metrics() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());
    
    // Execute a pipeline with the metrics recorder active
    let _: Vec<RedisValue> = cache.connection_manager().execute_pipeline_command("metrics_test", |mut conn| {
        let mut pipe = pipe();
        pipe.set("metrics_key1", "value1")
            .set("metrics_key2", "value2")
            .get("metrics_key1");
        pipe.query_async(&mut conn)
    }).await?;
    
    // Verify metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_pipeline_execute_duration_ms"));
    
    Ok(())
} 