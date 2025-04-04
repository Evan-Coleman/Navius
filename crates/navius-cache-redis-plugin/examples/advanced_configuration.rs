use navius_cache::{Cache, CacheKey, CacheOperations};
use navius_cache_redis_plugin::{
    ClusterConfig, PoolConfig, RedisCache, RedisCacheConfig, TlsConfig,
};
use std::error::Error;
use std::time::Duration;

/// This example demonstrates advanced Redis configuration options
///
/// To run this example:
/// ```bash
/// # Start a Redis server if not already running
/// docker run --name redis-test -p 6379:6379 -d redis
///
/// # Run the example
/// cargo run --example advanced_configuration
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Redis Cache Plugin - Advanced Configuration Example");
    println!("--------------------------------------------------");

    // 1. Basic Configuration
    println!("\n1. Basic Configuration");
    println!("--------------------");

    let basic_config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "basic:".to_string(),
        Duration::from_secs(3600), // 1 hour TTL
    );

    println!("Basic configuration created with:");
    println!("  - URL: redis://localhost:6379");
    println!("  - Key prefix: basic:");
    println!("  - Default TTL: 3600 seconds (1 hour)");

    // Connect with basic config
    println!("Connecting with basic configuration...");
    let basic_cache = RedisCache::new(basic_config).await?;
    println!("Successfully connected with basic configuration");

    // Test connection
    basic_cache.set("test", &"Configuration test", None).await?;
    let test_value: Option<String> = basic_cache.get("test").await?;
    println!("Test value: {:?}", test_value);

    // 2. Connection Pool Configuration
    println!("\n2. Connection Pool Configuration");
    println!("------------------------------");

    let pool_config = PoolConfig {
        // Adjust these values based on your expected load
        min_connections: Some(2),                       // Minimum pool size
        max_connections: Some(10),                      // Maximum pool size
        connect_timeout: Some(Duration::from_secs(5)),  // Connection timeout
        command_timeout: Some(Duration::from_secs(2)),  // Command timeout
        connect_retries: 3,                             // Retry 3 times on connection failures
        enable_connection_recycling: true,              // Recycle connections
        health_check_interval: Duration::from_secs(30), // Health check every 30 seconds
    };

    let pool_redis_config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "pool:".to_string(),
        Duration::from_secs(3600),
    )
    .with_pool(pool_config);

    println!("Pool configuration created with:");
    println!("  - Min connections: 2");
    println!("  - Max connections: 10");
    println!("  - Connect timeout: 5 seconds");
    println!("  - Command timeout: 2 seconds");
    println!("  - Connect retries: 3");
    println!("  - Connection recycling: enabled");
    println!("  - Health check interval: 30 seconds");

    println!("Connecting with pool configuration...");
    let pool_cache = RedisCache::new(pool_redis_config).await?;
    println!("Successfully connected with pool configuration");

    pool_cache
        .set("test", &"Pool configuration test", None)
        .await?;
    let pool_test_value: Option<String> = pool_cache.get("test").await?;
    println!("Test value: {:?}", pool_test_value);

    // 3. TLS Configuration (Commented out as it requires TLS-enabled Redis server)
    println!("\n3. TLS Configuration");
    println!("------------------");
    println!("Note: This example is commented out as it requires a TLS-enabled Redis server.");

    /*
    // Example TLS configuration:
    let tls_config = TlsConfig {
        enabled: true,
        server_name: Some("redis.example.com".to_string()),
        ca_cert_path: Some("/path/to/ca.crt".to_string()),
        client_cert_path: None,
        client_key_path: None,
    };

    let tls_redis_config = RedisCacheConfig::new(
        "rediss://redis.example.com:6379".to_string(), // Note "rediss://" for TLS
        "tls:".to_string(),
        Duration::from_secs(3600),
    ).with_tls(tls_config);

    // Then connect:
    // let tls_cache = RedisCache::new(tls_redis_config).await?;
    */

    println!("To use TLS, you would need:");
    println!("  - Redis server with TLS enabled");
    println!("  - URL starting with 'rediss://'");
    println!("  - TLS configuration with certificates");

    // 4. Cluster Configuration (Commented out as it requires Redis Cluster)
    println!("\n4. Cluster Configuration");
    println!("----------------------");
    println!("Note: This example is commented out as it requires a Redis Cluster setup.");

    /*
    // Example Cluster configuration:
    let cluster_config = ClusterConfig {
        enabled: true,
        retry_count: 3,
        read_from_replicas: true,
    };

    let cluster_redis_config = RedisCacheConfig::new(
        "redis://redis-cluster:6379".to_string(),
        "cluster:".to_string(),
        Duration::from_secs(3600),
    ).with_cluster(cluster_config);

    // Then connect:
    // let cluster_cache = RedisCache::new(cluster_redis_config).await?;
    */

    println!("To use Redis Cluster, you would need:");
    println!("  - Redis Cluster setup with multiple nodes");
    println!("  - Cluster configuration enabled");
    println!("  - Connection to a cluster node");

    // 5. Combined Configuration
    println!("\n5. Combined Configuration");
    println!("----------------------");

    // Create a configuration with pool, and (optionally) TLS and cluster
    let combined_pool_config = PoolConfig {
        min_connections: Some(5),
        max_connections: Some(20),
        connect_timeout: Some(Duration::from_secs(3)),
        command_timeout: Some(Duration::from_secs(2)),
        connect_retries: 3,
        enable_connection_recycling: true,
        health_check_interval: Duration::from_secs(30),
    };

    let mut combined_config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "combined:".to_string(),
        Duration::from_secs(1800), // 30 minutes
    )
    .with_pool(combined_pool_config);

    // You could add TLS and cluster configurations as well:
    // combined_config = combined_config.with_tls(tls_config);
    // combined_config = combined_config.with_cluster(cluster_config);

    println!("Combined configuration created with:");
    println!("  - URL: redis://localhost:6379");
    println!("  - Key prefix: combined:");
    println!("  - Default TTL: 1800 seconds (30 minutes)");
    println!("  - Pool configuration: Custom");
    // println!("  - TLS configuration: Enabled");
    // println!("  - Cluster configuration: Enabled");

    println!("Connecting with combined configuration...");
    let combined_cache = RedisCache::new(combined_config).await?;
    println!("Successfully connected with combined configuration");

    combined_cache
        .set("test", &"Combined configuration test", None)
        .await?;
    let combined_test_value: Option<String> = combined_cache.get("test").await?;
    println!("Test value: {:?}", combined_test_value);

    // 6. Configuration from Environment Variables
    println!("\n6. Configuration from Environment");
    println!("-----------------------------");
    println!("Note: This example simulates loading from environment variables.");

    // Simulate environment variable loading
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let redis_prefix = std::env::var("REDIS_PREFIX").unwrap_or("env:".to_string());
    let redis_ttl = std::env::var("REDIS_TTL")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(3600);

    let env_config = RedisCacheConfig::new(redis_url, redis_prefix, Duration::from_secs(redis_ttl));

    println!("Environment configuration created with:");
    println!("  - URL: {}", env_config.redis_url);
    println!("  - Key prefix: {}", env_config.key_prefix);
    println!(
        "  - Default TTL: {} seconds",
        env_config.default_ttl.as_secs()
    );

    println!("Connecting with environment configuration...");
    let env_cache = RedisCache::new(env_config).await?;
    println!("Successfully connected with environment configuration");

    env_cache
        .set("test", &"Environment configuration test", None)
        .await?;
    let env_test_value: Option<String> = env_cache.get("test").await?;
    println!("Test value: {:?}", env_test_value);

    // Cleanup
    println!("\nCleaning up test keys...");
    let _ = basic_cache.delete("test").await?;
    let _ = pool_cache.delete("test").await?;
    let _ = combined_cache.delete("test").await?;
    let _ = env_cache.delete("test").await?;

    println!("\nExample completed successfully!");
    Ok(())
}
