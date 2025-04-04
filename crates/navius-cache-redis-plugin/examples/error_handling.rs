use navius_cache::{Cache, CacheError, CacheKey, CacheOperations};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig, RedisError, RedisResult};
use std::error::Error;
use std::time::Duration;

/// This example demonstrates error handling with the Redis cache plugin
///
/// To run this example:
/// ```bash
/// # Start a Redis server if not already running
/// docker run --name redis-test -p 6379:6379 -d redis
///
/// # Run the example
/// cargo run --example error_handling
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Redis Cache Plugin - Error Handling Example");
    println!("-----------------------------------------");

    // 1. Handling connection errors
    println!("\n1. Handling Connection Errors");
    println!("---------------------------");

    // Create a config with a non-existent Redis server
    let invalid_config = RedisCacheConfig::new(
        "redis://nonexistent-host:6379".to_string(),
        "invalid:".to_string(),
        Duration::from_secs(300),
    );

    println!("Attempting to connect to non-existent Redis server...");
    match RedisCache::new(invalid_config).await {
        Ok(_) => println!("Unexpectedly connected successfully"),
        Err(e) => {
            println!("✓ Expected connection error received: {}", e);
            println!("Error type: {:?}", e);

            // Show how to match on error types
            match e {
                RedisError::Connection(msg) => println!("  → Connection error: {}", msg),
                _ => println!("  → Other error type: {:?}", e),
            }
        }
    }

    // Create a valid configuration for the next tests
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "error:".to_string(),
        Duration::from_secs(300),
    );

    let cache = match RedisCache::new(config).await {
        Ok(cache) => {
            println!("\nConnected to Redis server successfully for the remaining examples");
            cache
        }
        Err(e) => {
            println!("\nFailed to connect to Redis server: {}", e);
            println!("Please make sure Redis is running at localhost:6379");
            println!("Skipping the remaining examples");
            return Ok(());
        }
    };

    // 2. Handling timeout errors (simulated)
    println!("\n2. Handling Timeout Errors");
    println!("------------------------");
    println!("Note: This section simulates timeout errors - no actual timeouts are triggered.");

    // Example of catching a timeout error
    let example_timeout_error = RedisError::Timeout("Command timed out after 2000ms".to_string());
    println!("Example timeout error: {}", example_timeout_error);

    // Show how timeout errors map to CacheError
    let cache_error: CacheError = example_timeout_error.clone().into();
    println!("Converted to CacheError: {}", cache_error);

    // 3. Handling serialization errors
    println!("\n3. Handling Serialization Errors");
    println!("------------------------------");

    // Create a complex structure that will be serialized
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct User {
        id: u32,
        name: String,
        email: String,
    }

    // Valid serialization
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    println!("Storing serialized user object...");
    match cache.set("user:1", &user, None).await {
        Ok(_) => println!("✓ Successfully serialized and stored user"),
        Err(e) => println!("Error serializing user: {}", e),
    }

    // Retrieving with correct type
    println!("Retrieving and deserializing user object...");
    match cache.get::<_, User>("user:1").await {
        Ok(Some(retrieved_user)) => println!("✓ Successfully retrieved user: {:?}", retrieved_user),
        Ok(None) => println!("User not found"),
        Err(e) => println!("Error deserializing user: {}", e),
    }

    // Retrieving with incorrect type to force deserialization error
    println!("Forcing a deserialization error (retrieving as wrong type)...");
    #[derive(serde::Deserialize, Debug)]
    struct DifferentStruct {
        uuid: String,
        active: bool,
    }

    match cache.get::<_, DifferentStruct>("user:1").await {
        Ok(Some(_)) => println!("Unexpectedly succeeded deserializing as wrong type"),
        Ok(None) => println!("User not found"),
        Err(e) => {
            println!("✓ Expected deserialization error: {}", e);

            // Show how to match on error types
            match e {
                RedisError::Serialization(msg) => println!("  → Serialization error: {}", msg),
                _ => println!("  → Unexpected error type: {:?}", e),
            }
        }
    }

    // 4. Handling non-existent keys
    println!("\n4. Handling Non-existent Keys");
    println!("---------------------------");

    // Try to get a key that doesn't exist
    println!("Attempting to retrieve non-existent key...");
    match cache.get::<_, String>("non-existent-key").await {
        Ok(Some(value)) => println!("Unexpectedly found value: {}", value),
        Ok(None) => println!("✓ Correctly returned None for non-existent key"),
        Err(e) => println!("Unexpected error: {}", e),
    }

    // Try to delete a key that doesn't exist
    println!("Attempting to delete non-existent key...");
    match cache.delete("non-existent-key").await {
        Ok(true) => println!("Unexpectedly deleted a non-existent key"),
        Ok(false) => println!("✓ Correctly returned false for non-existent key deletion"),
        Err(e) => println!("Unexpected error: {}", e),
    }

    // 5. Handling command errors
    println!("\n5. Handling Command Errors");
    println!("------------------------");

    // Try an invalid command (this example simulates it)
    let example_command_error = RedisError::Command("ERR unknown command 'INVALID'".to_string());
    println!("Example command error: {}", example_command_error);

    // Show how command errors map to CacheError
    let command_cache_error: CacheError = example_command_error.clone().into();
    println!("Converted to CacheError: {}", command_cache_error);

    // 6. Handling errors in transactions
    println!("\n6. Custom Result Type");
    println!("-------------------");

    // Example function that returns RedisResult
    async fn example_operation(cache: &RedisCache) -> RedisResult<String> {
        let result = cache.get::<_, String>("example").await?;
        match result {
            Some(value) => Ok(value),
            None => Err(RedisError::Other("Key not found".to_string())),
        }
    }

    // Set a value for our example
    cache.set("example", &"Example value", None).await?;

    // Show how RedisResult helps with error handling
    println!("Using RedisResult for error handling:");
    match example_operation(&cache).await {
        Ok(value) => println!("✓ Operation succeeded: {}", value),
        Err(e) => println!("Operation failed: {}", e),
    }

    // Delete the key to trigger the error case
    cache.delete("example").await?;

    match example_operation(&cache).await {
        Ok(value) => println!("Operation unexpectedly succeeded: {}", value),
        Err(e) => println!("✓ Expected error received: {}", e),
    }

    // 7. Error Conversion
    println!("\n7. Error Conversion");
    println!("-----------------");

    // Function that demonstrates converting between CacheError and RedisError
    fn demonstrate_error_conversion() {
        // Create a RedisError
        let redis_err = RedisError::Connection("Connection refused".to_string());
        println!("Original RedisError: {}", redis_err);

        // Convert RedisError to CacheError
        let cache_err: CacheError = redis_err.into();
        println!("Converted to CacheError: {}", cache_err);

        // Convert CacheError back to RedisError using the to_redis_result helper
        let result: RedisResult<()> = navius_cache_redis_plugin::to_redis_result(Err(cache_err));
        match result {
            Ok(_) => println!("Unexpected success"),
            Err(e) => println!("Converted back to RedisError: {}", e),
        }
    }

    demonstrate_error_conversion();

    // 8. Health Check Errors
    println!("\n8. Health Check Errors");
    println!("-------------------");

    // Health checks are important for monitoring the connection status
    println!("Performing health check...");
    match cache.health_check().await {
        Ok(_) => println!("✓ Health check passed - Redis is healthy"),
        Err(e) => {
            println!("Health check failed: {}", e);
            match e {
                RedisError::HealthCheck(msg) => println!("  → Health check error: {}", msg),
                _ => println!("  → Unexpected error type: {:?}", e),
            }
        }
    }

    // 9. Error handling best practices
    println!("\n9. Error Handling Best Practices");
    println!("------------------------------");
    println!("- Always match on specific error types for targeted handling");
    println!("- Use the ? operator with RedisResult for clean error propagation");
    println!("- Consider retrying operations on connection errors");
    println!("- Implement graceful degradation for cache failures");
    println!("- Log errors with appropriate severity levels");
    println!("- Use health checks to proactively monitor Redis health");

    // Cleanup
    println!("\nCleaning up test keys...");
    let _ = cache.delete("user:1").await?;

    println!("\nExample completed successfully!");
    Ok(())
}
