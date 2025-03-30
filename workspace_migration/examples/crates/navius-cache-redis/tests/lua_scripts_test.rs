use std::sync::Arc;
use std::time::Duration;

use navius_cache::lua::RedisLuaScripting;
use navius_cache::{Cache, CacheError};
use navius_cache_redis::{RedisCache, RedisCacheError, RedisConnectionManager};
use serde::{Deserialize, Serialize};
use tokio::test;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestUser {
    id: u64,
    name: String,
    score: i32,
}

async fn setup_redis() -> Result<RedisCache, RedisCacheError> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());

    // Initialize the connection manager
    let connection_manager = Arc::new(RedisConnectionManager::new(&redis_url)?);

    // Create a cache with Lua scripting enabled
    let cache = RedisCache::new(connection_manager.clone()).with_lua_scripting();

    // Initialize common scripts
    cache.initialize_common_scripts().await?;

    Ok(cache)
}

#[test]
async fn test_atomic_set_nx() -> Result<(), CacheError> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let cache = setup_redis().await?;

    let key = "test_atomic_set_nx_key";
    let value = "test_value";

    // Clean up from previous test runs
    cache.delete(key).await?;

    // First set should succeed
    let result = cache
        .set_if_not_exists(key, value, Some(Duration::from_secs(60)))
        .await?;
    assert!(result, "First set_if_not_exists should return true");

    // Second set should fail
    let result = cache.set_if_not_exists(key, "another_value", None).await?;
    assert!(!result, "Second set_if_not_exists should return false");

    // Check that the value is still the original one
    let stored_value: String = cache.get(key).await?;
    assert_eq!(stored_value, value, "Value should be the original one");

    // Clean up
    cache.delete(key).await?;

    Ok(())
}

#[test]
async fn test_check_and_increment() -> Result<(), CacheError> {
    let cache = setup_redis().await?;

    let key = "test_check_and_increment_key";

    // Clean up from previous test runs
    cache.delete(key).await?;

    // Set initial value to 0
    cache.set(key, "0", None).await?;

    // Increment should succeed when below max
    let result = cache
        .check_and_increment_counter(key, 5, Some(Duration::from_secs(60)))
        .await?;
    assert!(result, "First increment should succeed");

    // Value should be 1
    let stored_value: String = cache.get(key).await?;
    assert_eq!(stored_value, "1", "Value should be 1 after first increment");

    // Increment several more times
    for i in 2..=5 {
        let result = cache
            .check_and_increment_counter(key, 5, Some(Duration::from_secs(60)))
            .await?;
        assert!(result, "Increment {} should succeed", i);

        let stored_value: String = cache.get(key).await?;
        assert_eq!(stored_value, i.to_string(), "Value should be {}", i);
    }

    // Next increment should fail as we reached the max
    let result = cache
        .check_and_increment_counter(key, 5, Some(Duration::from_secs(60)))
        .await?;
    assert!(!result, "Increment beyond max should fail");

    // Value should still be 5
    let stored_value: String = cache.get(key).await?;
    assert_eq!(stored_value, "5", "Value should remain at 5");

    // Clean up
    cache.delete(key).await?;

    Ok(())
}

#[test]
async fn test_custom_lua_script() -> Result<(), CacheError> {
    let cache = setup_redis().await?;

    let key = "test_custom_script_key";
    let user = TestUser {
        id: 1,
        name: "Alice".to_string(),
        score: 100,
    };

    // Clean up from previous test runs
    cache.delete(key).await?;

    // First, set the initial value
    cache.set(key, &user, None).await?;

    // Register a custom script to increment the score field in our JSON
    let script_name = "increment_score";
    let script_content = r#"
    local json = redis.call('GET', KEYS[1])
    if not json then
        return nil
    end
    
    local increment = tonumber(ARGV[1])
    if not increment then
        return {err = "Invalid increment value"}
    end
    
    -- Parse JSON (simplified approach for this test)
    local user = cjson.decode(json)
    user.score = user.score + increment
    
    -- Store updated JSON
    local new_json = cjson.encode(user)
    redis.call('SET', KEYS[1], new_json)
    
    return new_json
    "#;

    // Register the script
    cache.register_script(script_name, script_content).await?;

    // Execute the script to increment the score by 50
    let result: String = cache
        .execute_script(script_name, vec![key.to_string()], vec!["50".to_string()])
        .await?;
    info!("Script result: {}", result);

    // Get the updated user
    let updated_user: TestUser = cache.get(key).await?;

    // Check that the score was incremented
    assert_eq!(
        updated_user.score, 150,
        "Score should be incremented to 150"
    );
    assert_eq!(updated_user.name, "Alice", "Name should remain unchanged");
    assert_eq!(updated_user.id, 1, "ID should remain unchanged");

    // Clean up
    cache.delete(key).await?;

    Ok(())
}

#[test]
async fn test_atomic_update() -> Result<(), CacheError> {
    let cache = setup_redis().await?;

    let key = "test_atomic_update_key";
    let user = TestUser {
        id: 1,
        name: "Bob".to_string(),
        score: 200,
    };

    // Clean up from previous test runs
    cache.delete(key).await?;

    // First, set the initial value
    cache.set(key, &user, None).await?;

    // Use atomic_update to modify the user
    let updated_user = cache
        .atomic_update::<TestUser, _, _>(
            key,
            |existing_user| {
                let mut user = existing_user.unwrap_or_else(|| TestUser {
                    id: 0,
                    name: "Default".to_string(),
                    score: 0,
                });

                user.score += 75;
                user.name = format!("{}+", user.name);

                Ok(user)
            },
            None,
        )
        .await?;

    // Verify the update worked
    assert_eq!(updated_user.score, 275, "Score should be 275");
    assert_eq!(updated_user.name, "Bob+", "Name should be updated");
    assert_eq!(updated_user.id, 1, "ID should remain unchanged");

    // Double-check by getting the value
    let stored_user: TestUser = cache.get(key).await?;
    assert_eq!(
        stored_user, updated_user,
        "Stored user should match the returned user"
    );

    // Clean up
    cache.delete(key).await?;

    Ok(())
}

#[test]
async fn test_atomic_increment() -> Result<(), CacheError> {
    let cache = setup_redis().await?;

    let key = "test_atomic_increment_key";

    // Clean up from previous test runs
    cache.delete(key).await?;

    // Set initial value
    cache.set(key, "10", None).await?;

    // Increment atomically
    let new_value: i64 = cache.atomic_increment(key, 5, None).await?;
    assert_eq!(new_value, 15, "Value should be incremented to 15");

    // Double-check by getting the value
    let stored_value: String = cache.get(key).await?;
    assert_eq!(stored_value, "15", "Stored value should be 15");

    // Increment with negative value
    let new_value: i64 = cache.atomic_increment(key, -3, None).await?;
    assert_eq!(new_value, 12, "Value should be decremented to 12");

    // Clean up
    cache.delete(key).await?;

    Ok(())
}
