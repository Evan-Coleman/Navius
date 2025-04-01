use navius_cache::{
    config::CacheConfig,
    error::CacheResult,
    key::CacheKey,
    operations::Cache,
};
use navius_cache_redis::{
    config::RedisCacheConfig,
    connection::{RedisConnectionManager, RedisPipeline},
    operations::RedisCache,
};
use redis::{AsyncCommands, Pipeline};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

/**
 * This example demonstrates the use of Pipeline commands for batch operations in Redis cache.
 * 
 * Pipeline commands allow you to:
 * - Send multiple commands to Redis in a single network roundtrip
 * - Dramatically improve performance for batch operations
 * - Execute complex multi-key operations efficiently
 * 
 * Pipelining is useful for:
 * - Bulk data loading or migration
 * - Atomic multi-key operations
 * - Reducing network latency impact
 * - High-throughput data processing
 */

// Example data structure to demonstrate serialization with pipelines
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct User {
    id: u32,
    username: String,
    email: String,
    created_at: u64,
}

impl User {
    fn new(id: u32, username: &str, email: &str) -> Self {
        Self {
            id,
            username: username.to_string(),
            email: email.to_string(),
            created_at: chrono::Utc::now().timestamp() as u64,
        }
    }
}

// Implement CacheKey for User so it can be used as a key in the cache
impl CacheKey for User {
    fn to_cache_key(&self) -> String {
        format!("user:{}", self.id)
    }
}

// Simulate creating a large batch of users
fn generate_users(count: usize) -> Vec<User> {
    let mut users = Vec::with_capacity(count);
    for i in 1..=count as u32 {
        users.push(User::new(
            i,
            &format!("user{}", i),
            &format!("user{}@example.com", i),
        ));
    }
    users
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Configure Redis cache
    let config = RedisCacheConfig::builder()
        .with_url("redis://127.0.0.1:6379")
        .with_key_prefix("navius:example:pipeline")
        .with_default_ttl(Duration::from_secs(300))
        .with_pool_size(5)
        .build();

    // Create connection manager
    let conn_manager = Arc::new(
        RedisConnectionManager::new(config)
            .await
            .expect("Failed to create Redis connection manager"),
    );

    // Create cache instance
    let cache = RedisCache::new(conn_manager.clone())
        .expect("Failed to create Redis cache");

    println!("== Redis Pipeline Commands Example ==");

    // Basic Pipeline vs Individual Comparison
    println!("\n1. Performance Comparison: Individual Commands vs Pipeline");
    await compare_individual_vs_pipeline(&cache, conn_manager.clone()).await?;

    // Complex Pipeline Operations
    println!("\n2. Complex Pipeline Operations");
    await complex_pipeline_operations(&cache, conn_manager.clone()).await?;

    // Bulk Data Loading
    println!("\n3. Bulk Data Loading with Pipelines");
    await bulk_data_loading(&cache, conn_manager.clone()).await?;

    // Pipeline with Multiple Data Types
    println!("\n4. Pipeline with Multiple Data Types");
    await pipeline_with_multiple_data_types(&cache, conn_manager.clone()).await?;

    // Transactional Pipelines
    println!("\n5. Transaction-like Operations with Pipelines");
    await transaction_like_operations(&cache, conn_manager.clone()).await?;

    // Clean up
    cleanup(&cache).await?;

    println!("\nPipeline commands example completed successfully!");
    Ok(())
}

async fn compare_individual_vs_pipeline(
    cache: &RedisCache,
    conn_manager: Arc<RedisConnectionManager>,
) -> CacheResult<()> {
    const NUM_OPERATIONS: usize = 1000;
    
    // Generate some test data - key/value pairs
    let pairs: Vec<(String, i32)> = (0..NUM_OPERATIONS)
        .map(|i| (format!("key:{}", i), i as i32))
        .collect();
    
    // Individual SET operations
    println!("Setting {} key-value pairs individually...", NUM_OPERATIONS);
    let start = Instant::now();
    
    for (key, value) in &pairs {
        cache.set(key, value).await?;
    }
    
    let individual_duration = start.elapsed();
    println!("Individual operations duration: {:?}", individual_duration);
    
    // Delete the keys to start fresh
    for (key, _) in &pairs {
        cache.delete(key).await?;
    }
    
    // Pipeline SET operations
    println!("\nSetting {} key-value pairs with pipeline...", NUM_OPERATIONS);
    let start = Instant::now();
    
    conn_manager.execute_pipeline_command(|connection| {
        let mut pipeline = Pipeline::new();
        
        for (key, value) in &pairs {
            let key = format!("{}:{}", conn_manager.get_prefix(), key);
            pipeline.set(key, value.to_string());
        }
        
        Ok(pipeline) // Return the pipeline for execution
    }).await?;
    
    let pipeline_duration = start.elapsed();
    println!("Pipeline operations duration: {:?}", pipeline_duration);
    
    // Calculate the speedup
    let speedup = individual_duration.as_micros() as f64 / pipeline_duration.as_micros() as f64;
    println!("\nPipeline is {:.1}x faster than individual operations", speedup);
    
    // Verify data was inserted correctly
    println!("\nVerifying a few values were set correctly:");
    for i in 0..5 {
        let key = format!("key:{}", i);
        let value: Option<i32> = cache.get(&key).await?;
        println!("{} = {:?}", key, value);
    }
    
    Ok(())
}

async fn complex_pipeline_operations(
    cache: &RedisCache,
    conn_manager: Arc<RedisConnectionManager>,
) -> CacheResult<()> {
    // We'll create a shopping cart with various operations
    println!("Creating a shopping cart with pipeline commands");
    
    // User's cart ID
    let cart_id = "user:12345:cart";
    // List of products
    let products = vec![
        ("product:1001", "Laptop", 1299.99),
        ("product:1002", "Smartphone", 899.99),
        ("product:1003", "Headphones", 199.99),
        ("product:1004", "Mouse", 49.99),
        ("product:1005", "Keyboard", 79.99),
    ];
    
    // Ensure cart is empty to start
    let _ = cache.delete(cart_id).await;
    
    // Use pipeline for multiple operations:
    // 1. Add each product to the cart hash
    // 2. Add the product ID to the cart items set
    // 3. Set expiration (TTL) on the cart
    // 4. Count the items in the cart
    println!("\nAdding products to cart with pipeline");

    let (cart_size, cart_total): (usize, f64) = conn_manager.execute_pipeline_command(|connection| {
        let mut pipeline = Pipeline::new();
        let prefixed_cart_id = format!("{}:{}", conn_manager.get_prefix(), cart_id);
        let prefixed_cart_items = format!("{}:{}:items", conn_manager.get_prefix(), cart_id);
        
        // For each product:
        // - Add to hash with product details
        // - Add to set of cart items
        let mut total = 0.0;
        for (product_id, name, price) in &products {
            // Add to hash: HSET cart_id product_id {json with name and price}
            let product_data = serde_json::json!({
                "id": product_id,
                "name": name,
                "price": price,
                "added_at": chrono::Utc::now().timestamp(),
            });
            pipeline.hset(&prefixed_cart_id, product_id, product_data.to_string());
            
            // Add to set: SADD cart_id:items product_id
            pipeline.sadd(&prefixed_cart_items, *product_id);
            
            total += *price;
        }
        
        // Set expiration on cart (30 minutes)
        pipeline.expire(&prefixed_cart_id, 30 * 60);
        pipeline.expire(&prefixed_cart_items, 30 * 60);
        
        // Get cart size (number of items)
        pipeline.scard(&prefixed_cart_items);
        
        // Execute all commands and get results
        Ok(pipeline)
    }).await?;
    
    println!("Added items to cart");
    println!("Cart has {} items with total price: ${:.2}", cart_size, cart_total);
    
    // Now read cart items with a pipeline
    println!("\nReading cart items with pipeline");
    
    let cart_items: Vec<String> = cache.set_members(format!("{}:items", cart_id)).await?;
    
    println!("Cart contains {} items:", cart_items.len());
    
    // Get all product details using pipeline
    let product_details = conn_manager.execute_pipeline_command(|connection| {
        let mut pipeline = Pipeline::new();
        let prefixed_cart_id = format!("{}:{}", conn_manager.get_prefix(), cart_id);
        
        // For each product ID, get the JSON data from the hash
        for product_id in &cart_items {
            pipeline.hget(&prefixed_cart_id, product_id);
        }
        
        Ok(pipeline)
    }).await?;
    
    // Print the cart contents
    let mut total = 0.0;
    for (i, json_str) in product_details.iter().enumerate() {
        if let Ok(product_data) = serde_json::from_str::<serde_json::Value>(json_str) {
            let id = product_data["id"].as_str().unwrap_or("unknown");
            let name = product_data["name"].as_str().unwrap_or("unknown");
            let price = product_data["price"].as_f64().unwrap_or(0.0);
            
            println!("{}. {} - ${:.2}", i + 1, name, price);
            total += price;
        }
    }
    
    println!("Cart total: ${:.2}", total);
    
    Ok(())
}

async fn bulk_data_loading(
    cache: &RedisCache,
    conn_manager: Arc<RedisConnectionManager>,
) -> CacheResult<()> {
    // Generate a large number of users
    const NUM_USERS: usize = 1000;
    println!("Generating {} users for bulk loading", NUM_USERS);
    
    let users = generate_users(NUM_USERS);
    
    // First, measure performance without pipeline
    println!("\nLoading users individually (this may take a while)...");
    let start = Instant::now();
    
    // Store user data in individual SET operations (only do 50 to save time)
    for user in users.iter().take(50) {
        let key = format!("user:{}", user.id);
        cache.set(&key, user).await?;
    }
    
    let individual_duration = start.elapsed();
    println!("Time to load 50 users individually: {:?}", individual_duration);
    
    // Now use pipeline for bulk operation on all 1000 users
    println!("\nLoading {} users with pipeline...", NUM_USERS);
    let start = Instant::now();
    
    // Group users into batches of 100 for better memory management
    const BATCH_SIZE: usize = 100;
    for chunk in users.chunks(BATCH_SIZE) {
        conn_manager.execute_pipeline_command(|connection| {
            let mut pipeline = Pipeline::new();
            
            for user in chunk {
                let key = format!("{}:user:{}", conn_manager.get_prefix(), user.id);
                // Serialize user to JSON
                let json = serde_json::to_string(user).unwrap();
                // Set with expiration
                pipeline.set(&key, json);
                pipeline.expire(&key, 3600); // 1 hour TTL
                
                // Also maintain an index of usernames to IDs
                let username_key = format!("{}:username:{}", conn_manager.get_prefix(), user.username);
                pipeline.set(&username_key, user.id.to_string());
            }
            
            Ok(pipeline)
        }).await?;
    }
    
    let pipeline_duration = start.elapsed();
    println!("Time to load {} users with pipeline: {:?}", NUM_USERS, pipeline_duration);
    
    // Calculate theoretical speedup if we had done all 1000 individually
    let theoretical_individual_time = individual_duration.as_millis() as f64 * (NUM_USERS as f64 / 50.0);
    let speedup = theoretical_individual_time / pipeline_duration.as_millis() as f64;
    
    println!("\nEstimated speedup for {} users: {:.1}x faster with pipeline", 
             NUM_USERS, speedup);
    
    // Verify some random users were saved correctly
    println!("\nVerifying some random users:");
    for &id in &[1, 42, 777, 999] {
        let key = format!("user:{}", id);
        let user: Option<User> = cache.get(&key).await?;
        
        match user {
            Some(u) => println!("User {}: {} ({})", u.id, u.username, u.email),
            None => println!("User {} not found", id),
        }
    }
    
    Ok(())
}

async fn pipeline_with_multiple_data_types(
    cache: &RedisCache,
    conn_manager: Arc<RedisConnectionManager>,
) -> CacheResult<()> {
    println!("Creating different data structures in a single pipeline");
    
    // We'll create:
    // 1. A string for user profile
    // 2. A hash for user preferences
    // 3. A set for user roles
    // 4. A sorted set for user activity scores
    // 5. A list for user notifications
    
    let user_id = 42;
    let profile_key = format!("user:{}:profile", user_id);
    let prefs_key = format!("user:{}:preferences", user_id);
    let roles_key = format!("user:{}:roles", user_id);
    let activity_key = format!("user:{}:activity", user_id);
    let notifications_key = format!("user:{}:notifications", user_id);
    
    // Clean up any existing keys
    for key in &[&profile_key, &prefs_key, &roles_key, &activity_key, &notifications_key] {
        let _ = cache.delete(key).await;
    }
    
    // Create all data structures in a single pipeline
    conn_manager.execute_pipeline_command(|connection| {
        let mut pipeline = Pipeline::new();
        
        // Add prefix to keys
        let prefix = conn_manager.get_prefix();
        let profile_key = format!("{}:{}", prefix, profile_key);
        let prefs_key = format!("{}:{}", prefix, prefs_key);
        let roles_key = format!("{}:{}", prefix, roles_key);
        let activity_key = format!("{}:{}", prefix, activity_key);
        let notifications_key = format!("{}:{}", prefix, notifications_key);
        
        // 1. Set profile (string)
        let profile = serde_json::json!({
            "id": user_id,
            "name": "Jane Smith",
            "email": "jane.smith@example.com",
            "bio": "Software engineer and open source contributor"
        });
        pipeline.set(&profile_key, profile.to_string());
        
        // 2. Set preferences (hash)
        pipeline.hset(&prefs_key, "theme", "dark");
        pipeline.hset(&prefs_key, "notifications", "enabled");
        pipeline.hset(&prefs_key, "language", "en-US");
        pipeline.hset(&prefs_key, "timezone", "America/New_York");
        
        // 3. Set roles (set)
        pipeline.sadd(&roles_key, "user");
        pipeline.sadd(&roles_key, "editor");
        pipeline.sadd(&roles_key, "moderator");
        
        // 4. Set activity scores (sorted set)
        pipeline.zadd(&activity_key, "login", 100);
        pipeline.zadd(&activity_key, "post_create", 75);
        pipeline.zadd(&activity_key, "comment", 50);
        pipeline.zadd(&activity_key, "profile_update", 25);
        
        // 5. Add notifications (list)
        pipeline.lpush(&notifications_key, "You have a new follower!");
        pipeline.lpush(&notifications_key, "Your post received 10 likes");
        pipeline.lpush(&notifications_key, "New comment on your post");
        
        // Set TTL on all keys (24 hours)
        for key in &[&profile_key, &prefs_key, &roles_key, &activity_key, &notifications_key] {
            pipeline.expire(key, 24 * 60 * 60);
        }
        
        Ok(pipeline)
    }).await?;
    
    println!("Created multiple data structures for user {}", user_id);
    
    // Now retrieve all the data in a single operation
    println!("\nRetrieving user data with pipeline:");
    
    // This pipeline will fetch all of the user's data at once
    let results = conn_manager.execute_pipeline_command(|connection| {
        let mut pipeline = Pipeline::new();
        
        // Add prefix to keys
        let prefix = conn_manager.get_prefix();
        let profile_key = format!("{}:{}", prefix, profile_key);
        let prefs_key = format!("{}:{}", prefix, prefs_key);
        let roles_key = format!("{}:{}", prefix, roles_key);
        let activity_key = format!("{}:{}", prefix, activity_key);
        let notifications_key = format!("{}:{}", prefix, notifications_key);
        
        // Get profile (string)
        pipeline.get(&profile_key);
        
        // Get all preferences (hash)
        pipeline.hgetall(&prefs_key);
        
        // Get all roles (set)
        pipeline.smembers(&roles_key);
        
        // Get activity scores (sorted set)
        pipeline.zrange_withscores(&activity_key, 0, -1);
        
        // Get notifications (list)
        pipeline.lrange(&notifications_key, 0, -1);
        
        Ok(pipeline)
    }).await?;
    
    // Parse and display the results
    if results.len() >= 5 {
        // 1. Display profile
        let profile: serde_json::Value = serde_json::from_str(&results[0]).unwrap_or_default();
        println!("\nUser Profile:");
        println!("Name: {}", profile["name"]);
        println!("Email: {}", profile["email"]);
        println!("Bio: {}", profile["bio"]);
        
        // 2. Display preferences
        let preferences: HashMap<String, String> = serde_json::from_value(serde_json::to_value(&results[1]).unwrap()).unwrap_or_default();
        println!("\nUser Preferences:");
        for (key, value) in &preferences {
            println!("- {}: {}", key, value);
        }
        
        // 3. Display roles
        let roles: Vec<String> = serde_json::from_value(serde_json::to_value(&results[2]).unwrap()).unwrap_or_default();
        println!("\nUser Roles:");
        for role in &roles {
            println!("- {}", role);
        }
        
        // 4. Display activity scores
        let activities: Vec<(String, f64)> = serde_json::from_value(serde_json::to_value(&results[3]).unwrap()).unwrap_or_default();
        println!("\nActivity Scores:");
        for (activity, score) in &activities {
            println!("- {}: {}", activity, score);
        }
        
        // 5. Display notifications
        let notifications: Vec<String> = serde_json::from_value(serde_json::to_value(&results[4]).unwrap()).unwrap_or_default();
        println!("\nRecent Notifications:");
        for (i, notification) in notifications.iter().enumerate() {
            println!("{}. {}", i + 1, notification);
        }
    }
    
    Ok(())
}

async fn transaction_like_operations(
    cache: &RedisCache,
    conn_manager: Arc<RedisConnectionManager>,
) -> CacheResult<()> {
    println!("Performing transaction-like operations with pipelines");
    println!("(Note: This is not a true ACID transaction but demonstrates atomic operations)");
    
    // Example: Transfer funds from one account to another
    // We'll simulate a banking operation that needs to be performed atomically
    
    let user1_account = "account:user1";
    let user2_account = "account:user2";
    
    // Initialize account balances
    cache.set(user1_account, 1000.0).await?;
    cache.set(user2_account, 500.0).await?;
    
    println!("\nInitial balances:");
    let user1_balance: f64 = cache.get(user1_account).await?.unwrap_or(0.0);
    let user2_balance: f64 = cache.get(user2_account).await?.unwrap_or(0.0);
    println!("{}: ${:.2}", user1_account, user1_balance);
    println!("{}: ${:.2}", user2_account, user2_balance);
    
    // Now perform a transfer of $200 from user1 to user2
    // This should be done atomically to avoid race conditions
    let transfer_amount = 200.0;
    println!("\nTransferring ${:.2} from {} to {}", transfer_amount, user1_account, user2_account);
    
    // Create a transaction log
    let transaction_id = format!("tx:{}", chrono::Utc::now().timestamp());
    
    conn_manager.execute_pipeline_command(|connection| {
        let mut pipeline = Pipeline::new();
        
        // Add prefix to keys
        let prefix = conn_manager.get_prefix();
        let user1_key = format!("{}:{}", prefix, user1_account);
        let user2_key = format!("{}:{}", prefix, user2_account);
        let tx_key = format!("{}:{}", prefix, transaction_id);
        
        // Get current balances (for validation)
        pipeline.get(&user1_key);
        pipeline.get(&user2_key);
        
        // Record transaction start
        pipeline.set(&tx_key, "PENDING");
        
        // Update balances
        pipeline.set(&user1_key, (user1_balance - transfer_amount).to_string());
        pipeline.set(&user2_key, (user2_balance + transfer_amount).to_string());
        
        // Mark transaction as complete
        pipeline.set(&tx_key, "COMPLETED");
        
        // Set transaction TTL (keep for 30 days)
        pipeline.expire(&tx_key, 30 * 24 * 60 * 60);
        
        Ok(pipeline)
    }).await?;
    
    println!("Transfer completed with transaction ID: {}", transaction_id);
    
    // Verify new balances
    println!("\nNew balances:");
    let new_user1_balance: f64 = cache.get(user1_account).await?.unwrap_or(0.0);
    let new_user2_balance: f64 = cache.get(user2_account).await?.unwrap_or(0.0);
    println!("{}: ${:.2}", user1_account, new_user1_balance);
    println!("{}: ${:.2}", user2_account, new_user2_balance);
    
    // Verify total (should be unchanged)
    println!("\nTotal funds in system: ${:.2}", new_user1_balance + new_user2_balance);
    
    Ok(())
}

async fn cleanup(cache: &RedisCache) -> CacheResult<()> {
    // Clean up all the keys created in this example
    println!("\nCleaning up example keys...");
    
    // List prefixes to clean up
    let prefixes = [
        "key:",
        "user:",
        "username:",
        "account:",
        "tx:",
    ];
    
    // Use scan to find all keys with our prefixes
    for prefix in &prefixes {
        let pattern = format!("{}*", prefix);
        let keys: Vec<String> = cache.get_keys_by_pattern(&pattern).await?;
        
        println!("Deleting {} keys with prefix '{}'", keys.len(), prefix);
        
        for key in keys {
            let _ = cache.delete(&key).await;
        }
    }
    
    println!("Cleanup complete.");
    Ok(())
} 