use metrics::{counter, gauge, histogram};
use metrics_util::MetricKindMask;
use navius_cache_redis::{
    ConnectionHealth, RedisCache, RedisCacheConfig, RedisCacheError, RedisLuaManager,
    TimedOperation, initialize_common_scripts,
};
use std::{sync::Arc, time::Duration};
use tokio::time;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct TestUser {
    id: i32,
    name: String,
    email: String,
}

// Custom recorder for testing metrics
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

    fn print_records(&self) {
        let records = self.records.lock().unwrap();
        for record in records.iter() {
            println!("Recorded: {}", record);
        }
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
    let mut conn = cache.connection().await?;
    let _: () = redis::cmd("FLUSHDB")
        .query_async(&mut conn)
        .await
        .map_err(|e| RedisCacheError::CommandError(format!("Failed to flush database: {}", e)))?;

    Ok(cache)
}

#[tokio::test]
async fn test_basic_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Set and get a string value
    cache.set("test_key", "test_value", None).await?;
    let value: String = cache.get("test_key").await?;
    assert_eq!(value, "test_value");

    // Set and get with expiration
    cache
        .set(
            "expiring_key",
            "expiring_value",
            Some(Duration::from_secs(1)),
        )
        .await?;
    let value: String = cache.get("expiring_key").await?;
    assert_eq!(value, "expiring_value");

    // Wait for expiration
    time::sleep(Duration::from_secs(2)).await;
    let result = cache.get::<String, _>("expiring_key").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(RedisCacheError::KeyNotFound(_))));

    // Delete a key
    cache.set("to_delete", "value", None).await?;
    let exists = cache.exists("to_delete").await?;
    assert!(exists);

    cache.delete("to_delete").await?;
    let exists = cache.exists("to_delete").await?;
    assert!(!exists);

    // Check non-existent key
    let exists = cache.exists("non_existent_key").await?;
    assert!(!exists);

    Ok(())
}

#[tokio::test]
async fn test_object_serialization() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Create a test user
    let user = TestUser {
        id: 1,
        name: String::from("Test User"),
        email: String::from("test@example.com"),
    };

    // Store and retrieve the user
    cache.set("user:1", &user, None).await?;
    let retrieved_user: TestUser = cache.get("user:1").await?;

    assert_eq!(user, retrieved_user);

    // Test with optional values
    let opt_user = Some(user.clone());
    cache.set("opt_user:1", &opt_user, None).await?;
    let retrieved_opt_user: Option<TestUser> = cache.get("opt_user:1").await?;

    assert_eq!(opt_user, retrieved_opt_user);

    Ok(())
}

#[tokio::test]
async fn test_list_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let list_key = "test_list";

    // Push items to the list
    cache.lpush(list_key, "item1").await?;
    cache.lpush(list_key, "item2").await?;
    cache.rpush(list_key, "item3").await?;

    // Get list length
    let len = cache.llen(list_key).await?;
    assert_eq!(len, 3);

    // Get range of items
    let items: Vec<String> = cache.lrange(list_key, 0, -1).await?;
    assert_eq!(items, vec!["item2", "item1", "item3"]);

    // Pop items
    let left_item: String = cache.lpop(list_key).await?;
    assert_eq!(left_item, "item2");

    let right_item: String = cache.rpop(list_key).await?;
    assert_eq!(right_item, "item3");

    // Check remaining item
    let items: Vec<String> = cache.lrange(list_key, 0, -1).await?;
    assert_eq!(items, vec!["item1"]);

    Ok(())
}

#[tokio::test]
async fn test_hash_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let hash_key = "test_hash";

    // Set hash fields
    cache.hset(hash_key, "field1", "value1").await?;
    cache.hset(hash_key, "field2", "value2").await?;

    // Get a specific field
    let value: String = cache.hget(hash_key, "field1").await?;
    assert_eq!(value, "value1");

    // Check if a field exists
    let exists = cache.hexists(hash_key, "field1").await?;
    assert!(exists);

    let exists = cache.hexists(hash_key, "nonexistent").await?;
    assert!(!exists);

    // Get all fields and values
    let hash_map: std::collections::HashMap<String, String> = cache.hgetall(hash_key).await?;
    assert_eq!(hash_map.len(), 2);
    assert_eq!(hash_map.get("field1"), Some(&"value1".to_string()));
    assert_eq!(hash_map.get("field2"), Some(&"value2".to_string()));

    // Delete a field
    cache.hdel(hash_key, "field1").await?;
    let exists = cache.hexists(hash_key, "field1").await?;
    assert!(!exists);

    Ok(())
}

#[tokio::test]
async fn test_set_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let set_key = "test_set";

    // Add members to the set
    cache.sadd(set_key, "member1").await?;
    cache.sadd(set_key, "member2").await?;
    cache.sadd(set_key, "member3").await?;

    // Check if members exist
    let is_member = cache.sismember(set_key, "member1").await?;
    assert!(is_member);

    let is_member = cache.sismember(set_key, "nonexistent").await?;
    assert!(!is_member);

    // Get all members
    let members: Vec<String> = cache.smembers(set_key).await?;
    assert_eq!(members.len(), 3);
    assert!(members.contains(&"member1".to_string()));
    assert!(members.contains(&"member2".to_string()));
    assert!(members.contains(&"member3".to_string()));

    // Remove a member
    cache.srem(set_key, "member1").await?;
    let is_member = cache.sismember(set_key, "member1").await?;
    assert!(!is_member);

    // Test set operations with another set
    let other_set_key = "other_set";
    cache.sadd(other_set_key, "member2").await?;
    cache.sadd(other_set_key, "member4").await?;

    // Intersection
    let intersection: Vec<String> = cache.sinter(&[set_key, other_set_key]).await?;
    assert_eq!(intersection, vec!["member2"]);

    // Union
    let mut union: Vec<String> = cache.sunion(&[set_key, other_set_key]).await?;
    union.sort(); // Sort for deterministic comparison
    assert_eq!(union, vec!["member2", "member3", "member4"]);

    // Difference
    let difference: Vec<String> = cache.sdiff(&[set_key, other_set_key]).await?;
    assert_eq!(difference, vec!["member3"]);

    Ok(())
}

#[tokio::test]
async fn test_sorted_set_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let zset_key = "test_zset";

    // Add members with scores
    cache.zadd(zset_key, "member1", 1.0).await?;
    cache.zadd(zset_key, "member2", 2.0).await?;
    cache.zadd(zset_key, "member3", 3.0).await?;

    // Get range by score
    let members: Vec<String> = cache.zrangebyscore(zset_key, 1.0, 2.0).await?;
    assert_eq!(members, vec!["member1", "member2"]);

    // Get range by rank
    let members: Vec<String> = cache.zrange(zset_key, 0, 1).await?;
    assert_eq!(members, vec!["member1", "member2"]);

    // Get score
    let score = cache.zscore(zset_key, "member2").await?;
    assert_eq!(score, 2.0);

    // Get rank
    let rank = cache.zrank(zset_key, "member2").await?;
    assert_eq!(rank, 1);

    // Remove member
    cache.zrem(zset_key, "member2").await?;
    let exists = cache.zscore(zset_key, "member2").await.is_ok();
    assert!(!exists);

    Ok(())
}

#[tokio::test]
async fn test_lua_scripting() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Initialize common scripts
    let lua_manager = RedisLuaManager::new(cache.clone_inner()).await?;
    initialize_common_scripts(&lua_manager).await?;

    // Use a Lua script to set and get a value atomically
    let script = r#"
    redis.call('SET', KEYS[1], ARGV[1])
    return redis.call('GET', KEYS[1])
    "#;

    let script_hash = lua_manager.load_script(script).await?;
    let result: String = lua_manager
        .execute_script(&script_hash, &["lua_test_key"], &["lua_test_value"])
        .await?;

    assert_eq!(result, "lua_test_value");

    // Verify the key was actually set
    let value: String = cache.get("lua_test_key").await?;
    assert_eq!(value, "lua_test_value");

    // Test the increment_and_expire common script
    let key = "counter_key";
    let value: i64 = lua_manager
        .execute_script(
            "increment_and_expire",
            &[key],
            &["1", "5"], // Increment by 1, expire in 5 seconds
        )
        .await?;

    assert_eq!(value, 1);

    // Increment again
    let value: i64 = lua_manager
        .execute_script(
            "increment_and_expire",
            &[key],
            &["2", "5"], // Increment by 2, expire in 5 seconds
        )
        .await?;

    assert_eq!(value, 3);

    // Wait and check expiration
    time::sleep(Duration::from_secs(6)).await;
    let exists = cache.exists(key).await?;
    assert!(!exists);

    Ok(())
}

#[tokio::test]
async fn test_timed_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    // Test timed set operation
    {
        let _timer = TimedOperation::new("set");
        cache.set("timed_key", "timed_value", None).await?;
    }

    // Test timed get operation
    {
        let _timer = TimedOperation::new("get");
        let _: String = cache.get("timed_key").await?;
    }

    // Verify metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_set_duration_ms"));
    assert!(recorder.has_recorded("navius_redis_cache_get_duration_ms"));

    Ok(())
}

#[tokio::test]
async fn test_connection_health() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Test health check
    let health = cache.health_check().await?;
    assert_eq!(health, ConnectionHealth::Healthy);

    // Test with bad connection (this is a bit tricky in a test environment)
    // We'll simulate by manually recording health metrics
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    navius_cache_redis::record_connection_health(ConnectionHealth::Degraded);
    assert!(recorder.has_recorded("navius_redis_cache_connection_health = 1"));

    navius_cache_redis::record_connection_health(ConnectionHealth::Unhealthy);
    assert!(recorder.has_recorded("navius_redis_cache_connection_health = 2"));

    navius_cache_redis::record_connection_health(ConnectionHealth::Healthy);
    assert!(recorder.has_recorded("navius_redis_cache_connection_health = 0"));

    Ok(())
}

#[tokio::test]
async fn test_connection_pool() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    // Open multiple connections concurrently
    let mut handles = Vec::new();
    for i in 0..5 {
        let cache_clone = cache.clone();
        handles.push(tokio::spawn(async move {
            let mut conn = cache_clone.connection().await.unwrap();
            let _: () = redis::cmd("PING").query_async(&mut conn).await.unwrap();
            // Hold connection for a bit to ensure pool is used
            time::sleep(Duration::from_millis(100 * i)).await;
        }));
    }

    // Wait for all connections to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Record pool stats
    navius_cache_redis::record_connection_pool_stats(5, 3, 2);

    // Verify metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_connection_pool_size = 5"));
    assert!(recorder.has_recorded("navius_redis_cache_connection_pool_idle = 3"));
    assert!(recorder.has_recorded("navius_redis_cache_connection_pool_used = 2"));

    Ok(())
}

#[tokio::test]
async fn test_metrics_registration() -> Result<(), RedisCacheError> {
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    // Set up cache to trigger metrics registration
    let cache = setup_redis().await?;

    // Register some metrics explicitly
    counter!("navius_redis_cache_operations_total", 1);
    gauge!("navius_redis_cache_connection_pool_size", 10.0);
    histogram!("navius_redis_cache_operation_duration", 150.0);

    // Basic operation to ensure metrics are recorded
    cache.set("metrics_test", "value", None).await?;
    cache.get::<String, _>("metrics_test").await?;

    // Verify all metrics were registered
    assert!(recorder.has_recorded("register_counter: navius_redis_cache_operations_total"));
    assert!(recorder.has_recorded("register_gauge: navius_redis_cache_connection_pool_size"));
    assert!(recorder.has_recorded("register_histogram: navius_redis_cache_operation_duration"));
    assert!(recorder.has_recorded("register_histogram: navius_redis_cache_set_duration_ms"));
    assert!(recorder.has_recorded("register_histogram: navius_redis_cache_get_duration_ms"));

    Ok(())
}

#[tokio::test]
async fn test_pipeline() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Create a pipeline
    let mut pipeline = cache.pipeline();

    // Add commands to the pipeline
    pipeline.set("pipeline_key1", "value1", None);
    pipeline.set("pipeline_key2", "value2", None);
    pipeline.set("pipeline_key3", "value3", None);

    // Execute the pipeline
    pipeline.execute().await?;

    // Verify all keys were set
    let value1: String = cache.get("pipeline_key1").await?;
    let value2: String = cache.get("pipeline_key2").await?;
    let value3: String = cache.get("pipeline_key3").await?;

    assert_eq!(value1, "value1");
    assert_eq!(value2, "value2");
    assert_eq!(value3, "value3");

    // Test pipeline with results
    let mut pipeline = cache.pipeline();
    pipeline.get::<String, _>("pipeline_key1");
    pipeline.get::<String, _>("pipeline_key2");
    pipeline.get::<String, _>("pipeline_key3");

    let results = pipeline.execute_with_results().await?;
    assert_eq!(results.len(), 3);

    // Parse results
    let result1: String = results[0].clone().try_into()?;
    let result2: String = results[1].clone().try_into()?;
    let result3: String = results[2].clone().try_into()?;

    assert_eq!(result1, "value1");
    assert_eq!(result2, "value2");
    assert_eq!(result3, "value3");

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    // Test key not found error
    let result = cache.get::<String, _>("nonexistent_key").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(RedisCacheError::KeyNotFound(_))));

    // Test type error by storing a string and trying to get it as an integer
    cache.set("string_key", "not_an_integer", None).await?;
    let result = cache.get::<i32, _>("string_key").await;
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(RedisCacheError::DeserializationError(_))
    ));

    // Test invalid operation (using wrong data type)
    cache.set("not_a_list", "string_value", None).await?;
    let result = cache.lpop::<String, _>("not_a_list").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(RedisCacheError::CommandError(_))));

    // Verify error metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_get_error"));
    assert!(recorder.has_recorded("navius_redis_cache_lpop_error"));

    Ok(())
}

#[tokio::test]
async fn test_script_error_metrics() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;
    let recorder = Arc::new(TestMetricsRecorder::new());
    let _handle = metrics::set_boxed_recorder(recorder.clone());

    // Initialize lua manager
    let lua_manager = RedisLuaManager::new(cache.clone_inner()).await?;

    // Create a script with an error
    let bad_script = r#"
    return redis.call('HGET', KEYS[1])  -- Missing second argument
    "#;

    let script_hash = lua_manager.load_script(bad_script).await?;

    // Execute the script which should cause an error
    let result: Result<String, _> = lua_manager
        .execute_script(&script_hash, &["some_key"], &[])
        .await;

    assert!(result.is_err());

    // Verify script error metrics were recorded
    assert!(recorder.has_recorded("navius_redis_cache_script_execution_error"));

    Ok(())
}

#[tokio::test]
async fn test_ttl_and_expiration() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Set a key with expiration
    cache
        .set("ttl_key", "value", Some(Duration::from_secs(10)))
        .await?;

    // Check TTL
    let ttl = cache.ttl("ttl_key").await?;
    assert!(ttl > 0 && ttl <= 10);

    // Set a key without expiration
    cache.set("no_ttl_key", "value", None).await?;

    // Check that it has no TTL
    let ttl = cache.ttl("no_ttl_key").await?;
    assert_eq!(ttl, -1); // -1 indicates no expiration

    // Manually expire a key
    cache.expire("no_ttl_key", Duration::from_secs(5)).await?;

    // Check TTL again
    let ttl = cache.ttl("no_ttl_key").await?;
    assert!(ttl > 0 && ttl <= 5);

    // Remove expiration
    cache.persist("ttl_key").await?;

    // Check TTL after persist
    let ttl = cache.ttl("ttl_key").await?;
    assert_eq!(ttl, -1); // -1 indicates no expiration

    Ok(())
}

#[tokio::test]
async fn test_multi_operations() -> Result<(), RedisCacheError> {
    let cache = setup_redis().await?;

    // Set multiple keys
    let mut keys_values = std::collections::HashMap::new();
    keys_values.insert("multi_key1".to_string(), "value1".to_string());
    keys_values.insert("multi_key2".to_string(), "value2".to_string());
    keys_values.insert("multi_key3".to_string(), "value3".to_string());

    cache.mset(keys_values).await?;

    // Get multiple keys
    let keys = vec!["multi_key1", "multi_key2", "multi_key3", "non_existent_key"];
    let values: Vec<Option<String>> = cache.mget(&keys).await?;

    assert_eq!(values.len(), 4);
    assert_eq!(values[0], Some("value1".to_string()));
    assert_eq!(values[1], Some("value2".to_string()));
    assert_eq!(values[2], Some("value3".to_string()));
    assert_eq!(values[3], None);

    // Delete multiple keys
    cache.del(&["multi_key1", "multi_key2"]).await?;

    // Verify deletion
    let exists1 = cache.exists("multi_key1").await?;
    let exists2 = cache.exists("multi_key2").await?;
    let exists3 = cache.exists("multi_key3").await?;

    assert!(!exists1);
    assert!(!exists2);
    assert!(exists3);

    Ok(())
}
