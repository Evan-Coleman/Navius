use navius_cache::{
    config::CacheConfig,
    error::CacheResult,
    key::CacheKey,
    operations::Cache,
};
use navius_cache_redis::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    operations::{RedisCache, SetOperations},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use rand::{thread_rng, Rng};

/**
 * This example demonstrates the use of Set operations in the Redis cache.
 * 
 * Set operations allow you to:
 * - Store multiple unique items in a single key
 * - Check for item existence in the set
 * - Perform set operations like union, intersection, and difference
 * - Retrieve random members from the set
 * 
 * Sets are useful for:
 * - Tracking unique items (e.g., unique visitors)
 * - Managing relationships (e.g., followers, friends)
 * - Implementing features that require unique collections
 * - Efficiently checking membership of items
 */

// Example data structure to demonstrate serialization with sets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    category: String,
}

impl Product {
    fn new(id: &str, name: &str, category: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category: category.to_string(),
        }
    }
}

// Implement CacheKey for Product so it can be used as a key in the cache
impl CacheKey for Product {
    fn to_cache_key(&self) -> String {
        format!("product:{}", self.id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct User {
    id: u32,
    name: String,
    score: f64,
}

impl CacheKey for User {
    fn to_string(&self) -> String {
        format!("user:{}", self.id)
    }
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Configure Redis cache
    let config = RedisCacheConfig::builder()
        .with_url("redis://127.0.0.1:6379")
        .with_key_prefix("navius:example:sets")
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
    let cache = RedisCache::new(conn_manager)
        .expect("Failed to create Redis cache");

    println!("== Redis Set Operations Example ==");

    // First, let's create some test data
    let users = generate_test_users(20);
    
    // Basic Set Operations
    println!("\n=== Basic Set Operations ===");
    
    // Add users to a set
    let set_key = "users";
    let user_ids: Vec<u32> = users.iter().map(|u| u.id).collect();
    
    println!("Adding users to set '{}'...", set_key);
    let count = cache.set_add(set_key, user_ids.clone()).await?;
    println!("Added {} users to set", count);
    
    // Verify set members
    let members: Vec<u32> = cache.set_members(set_key).await?;
    println!("Set members: {:?}", members);
    
    // Check set membership
    let test_id = user_ids[0];
    let contains = cache.set_contains(set_key, &test_id).await?;
    println!("Set contains user {}: {}", test_id, contains);
    
    // Get set size
    let size = cache.set_length(set_key).await?;
    println!("Set size: {}", size);
    
    // Remove some members
    let to_remove = vec![user_ids[0], user_ids[1]];
    let removed = cache.set_remove(set_key, to_remove.clone()).await?;
    println!("Removed {} users from set", removed);
    
    // Verify set members after removal
    let members: Vec<u32> = cache.set_members(set_key).await?;
    println!("Set members after removal: {:?}", members);
    
    // Create additional sets for set operations
    let set_key2 = "users:premium";
    let premium_ids: Vec<u32> = users.iter()
        .filter(|u| u.score > 75.0)
        .map(|u| u.id)
        .collect();
    
    cache.set_add(set_key2, premium_ids.clone()).await?;
    println!("\nCreated premium users set with {} members", premium_ids.len());
    
    // Set operations: intersection
    let intersection: Vec<u32> = cache.set_intersection(vec![set_key, set_key2]).await?;
    println!("Intersection of sets: {:?}", intersection);
    
    // Set operations: union
    let union: Vec<u32> = cache.set_union(vec![set_key, set_key2]).await?;
    println!("Union of sets: {:?}", union);
    
    // Set operations: difference
    let difference: Vec<u32> = cache.set_difference(vec![set_key, set_key2]).await?;
    println!("Difference of sets (set1 - set2): {:?}", difference);
    
    // Store operations results
    cache.set_intersection_store("users:intersection", vec![set_key, set_key2]).await?;
    cache.set_union_store("users:union", vec![set_key, set_key2]).await?;
    cache.set_difference_store("users:difference", vec![set_key, set_key2]).await?;
    
    println!("\n=== SortedSet Operations ===");
    
    // Create a sorted set of users with their scores
    let zset_key = "users:ranked";
    let user_scores: Vec<(f64, u32)> = users.iter()
        .map(|u| (u.score, u.id))
        .collect();
    
    println!("Adding users to sorted set '{}'...", zset_key);
    let count = cache.zset_add(zset_key, user_scores).await?;
    println!("Added {} users to sorted set", count);
    
    // Get top 5 users
    let top5: Vec<u32> = cache.zset_range(zset_key, -5, -1).await?;
    println!("Top 5 users: {:?}", top5);
    
    // Get top 5 users with scores
    let top5_with_scores: Vec<(u32, f64)> = cache.zset_range_with_scores(zset_key, -5, -1).await?;
    println!("Top 5 users with scores:");
    for (id, score) in top5_with_scores {
        println!("  User ID: {}, Score: {}", id, score);
    }
    
    // Get users with scores above 80
    let high_scorers: Vec<u32> = cache.zset_range_by_score(zset_key, 80.0, 100.0).await?;
    println!("Users with scores above 80: {:?}", high_scorers);
    
    // Get rank of a specific user
    let test_user = user_ids[5];
    if let Some(rank) = cache.zset_rank(zset_key, &test_user).await? {
        println!("Rank of user {}: {}", test_user, rank);
    }
    
    // Increment a user's score
    let new_score = cache.zset_increment_score(zset_key, &test_user, 10.0).await?;
    println!("Incremented score of user {} to {}", test_user, new_score);
    
    // Remove some users from the sorted set
    let to_remove = vec![user_ids[0], user_ids[1]];
    let removed = cache.zset_remove(zset_key, to_remove).await?;
    println!("Removed {} users from sorted set", removed);
    
    // Count users within a score range
    let count = cache.zset_count(zset_key, 60.0, 80.0).await?;
    println!("Number of users with scores between 60 and 80: {}", count);
    
    // Clean up
    cache.delete(set_key).await?;
    cache.delete(set_key2).await?;
    cache.delete(zset_key).await?;
    cache.delete("users:intersection").await?;
    cache.delete("users:union").await?;
    cache.delete("users:difference").await?;
    
    println!("\nSuccessfully completed all set and sorted set operations!");
    
    Ok(())
}

fn generate_test_users(count: usize) -> Vec<User> {
    let mut rng = thread_rng();
    let mut users = Vec::with_capacity(count);
    
    for i in 1..=count {
        users.push(User {
            id: i as u32,
            name: format!("User {}", i),
            score: rng.gen_range(0.0..100.0),
        });
    }
    
    users
}

async fn basic_set_operations(cache: &RedisCache) -> CacheResult<()> {
    let set_key = "simple_set";
    
    // Add items to a set
    println!("Adding items to set '{}'", set_key);
    let added = cache.set_add(set_key, vec!["apple", "banana", "cherry"]).await?;
    println!("Added {} new items to the set", added);

    // Add more items, including duplicates
    let added = cache.set_add(set_key, vec!["banana", "dragonfruit", "elderberry"]).await?;
    println!("Added {} more unique items to the set", added);

    // Check the set's size
    let count = cache.set_length(set_key).await?;
    println!("Set '{}' has {} unique items", set_key, count);
    
    // Get all members of the set
    let members: Vec<String> = cache.set_members(set_key).await?;
    println!("Set members: {:?}", members);
    
    // Remove some items
    let removed = cache.set_remove(set_key, vec!["banana", "nonexistent"]).await?;
    println!("Removed {} items from the set", removed);
    
    // Get the updated set
    let updated_members: Vec<String> = cache.set_members(set_key).await?;
    println!("Updated set members: {:?}", updated_members);
    
    // Get the updated count
    let updated_count = cache.set_length(set_key).await?;
    println!("Updated set size: {}", updated_count);
    
    Ok(())
}

async fn set_membership_test(cache: &RedisCache) -> CacheResult<()> {
    let set_key = "membership_test_set";
    
    // Add items to a set
    println!("Creating set '{}'", set_key);
    cache.set_add(set_key, vec!["user1", "user2", "user3", "user4", "user5"]).await?;
    
    // Check if various items exist in the set
    let items_to_test = vec!["user1", "user3", "user6", "user7"];
    for item in items_to_test {
        let exists = cache.set_contains(set_key, &item).await?;
        println!("Is '{}' in the set? {}", item, exists);
    }
    
    Ok(())
}

async fn complex_set_operations(cache: &RedisCache) -> CacheResult<()> {
    // Create three sets to demonstrate set operations
    let set_a = "set_a";
    let set_b = "set_b";
    let set_c = "set_c";
    
    println!("Creating three sets for demonstration");
    
    // Set A: Fruits
    cache.set_add(set_a, vec!["apple", "banana", "cherry", "date", "elderberry"]).await?;
    println!("Set A (fruits): {:?}", cache.set_members::<String, _>(set_a).await?);
    
    // Set B: Fruits that are red
    cache.set_add(set_b, vec!["apple", "cherry", "strawberry", "raspberry"]).await?;
    println!("Set B (red fruits): {:?}", cache.set_members::<String, _>(set_b).await?);
    
    // Set C: Fruits that are berries
    cache.set_add(set_c, vec!["blackberry", "blueberry", "elderberry", "raspberry", "strawberry"]).await?;
    println!("Set C (berries): {:?}", cache.set_members::<String, _>(set_c).await?);
    
    // Intersection: Items that are in both Set A and Set B
    // In this case: red fruits from our fruits list
    println!("\nIntersection operations:");
    let intersection = cache.set_intersection::<String, _>(vec![set_a, set_b]).await?;
    println!("Intersection of Set A and Set B (fruits that are red): {:?}", intersection);

    // Store the intersection in a new key
    let dest_key = "red_fruits_from_our_list";
    let count = cache.set_intersection_store(dest_key, vec![set_a, set_b]).await?;
    println!("Stored {} items in '{}' (you can retrieve this later)", count, dest_key);

    // Union: Items that are in either Set B or Set C
    // In this case: all red fruits and all berries combined
    println!("\nUnion operations:");
    let union = cache.set_union::<String, _>(vec![set_b, set_c]).await?;
    println!("Union of Set B and Set C (red fruits OR berries): {:?}", union);

    // Store the union in a new key
    let dest_key = "red_fruits_or_berries";
    let count = cache.set_union_store(dest_key, vec![set_b, set_c]).await?;
    println!("Stored {} items in '{}' (you can retrieve this later)", count, dest_key);

    // Difference: Items in Set A that are not in Set B
    // In this case: fruits that are not red
    println!("\nDifference operations:");
    let difference = cache.set_difference::<String, _>(vec![set_a, set_b]).await?;
    println!("Difference of Set A and Set B (fruits that are not red): {:?}", difference);

    // Store the difference in a new key
    let dest_key = "non_red_fruits";
    let count = cache.set_difference_store(dest_key, vec![set_a, set_b]).await?;
    println!("Stored {} items in '{}' (you can retrieve this later)", count, dest_key);

    // Complex operation example: berries from our list that are not red
    println!("\nComplex set operation example:");
    // First find berries in our list (intersection of A and C)
    let berries_in_our_list = "berries_in_our_list";
    cache.set_intersection_store(berries_in_our_list, vec![set_a, set_c]).await?;
    
    // Then find which of these berries are not red (difference with set_b)
    let non_red_berries = "non_red_berries";
    let count = cache.set_difference_store(non_red_berries, vec![berries_in_our_list, set_b]).await?;
    
    let result: Vec<String> = cache.set_members(non_red_berries).await?;
    println!("Berries from our list that are not red ({}): {:?}", count, result);
    
    Ok(())
}

async fn random_members_example(cache: &RedisCache) -> CacheResult<()> {
    let set_key = "large_set";
    
    // Create a larger set for this example
    println!("Creating a large set for random sampling");
    let items: Vec<String> = (1..=100).map(|i| format!("item_{}", i)).collect();
    cache.set_add(set_key, items).await?;
    
    let set_size = cache.set_length(set_key).await?;
    println!("Set contains {} items", set_size);
    
    // Get a single random member
    println!("\nRetrieving a single random member:");
    let single_item: Vec<String> = cache.set_random_members(set_key, 1).await?;
    println!("Random item: {:?}", single_item);
    
    // Get multiple random members
    println!("\nRetrieving 5 random members:");
    let items: Vec<String> = cache.set_random_members(set_key, 5).await?;
    println!("Random items: {:?}", items);
    
    // Get more members than exist in the set (should return all members)
    println!("\nAttempting to retrieve more members than exist:");
    let count = 200; // More than the 100 items in the set
    let items: Vec<String> = cache.set_random_members(set_key, count).await?;
    println!("Requested {} random members, received {}", count, items.len());
    
    Ok(())
}

async fn complex_objects_example(cache: &RedisCache) -> CacheResult<()> {
    // Create products
    let products = vec![
        Product::new("p1", "Laptop", "Electronics"),
        Product::new("p2", "Smartphone", "Electronics"),
        Product::new("p3", "T-shirt", "Clothing"),
        Product::new("p4", "Jeans", "Clothing"),
        Product::new("p5", "Sneakers", "Footwear"),
        Product::new("p6", "Sandals", "Footwear"),
    ];
    
    // Create sets for different categories
    println!("Creating product category sets");
    
    // Electronics products
    let electronics_set = "category:electronics";
    let electronics = products.iter().filter(|p| p.category == "Electronics").collect::<Vec<_>>();
    cache.set_add(electronics_set, electronics).await?;
    
    // Clothing products
    let clothing_set = "category:clothing";
    let clothing = products.iter().filter(|p| p.category == "Clothing").collect::<Vec<_>>();
    cache.set_add(clothing_set, clothing).await?;
    
    // Footwear products
    let footwear_set = "category:footwear";
    let footwear = products.iter().filter(|p| p.category == "Footwear").collect::<Vec<_>>();
    cache.set_add(footwear_set, footwear).await?;
    
    // Create a set of featured products (products from different categories)
    println!("\nCreating a set of featured products");
    let featured_set = "featured:products";
    cache.set_add(featured_set, vec![&products[0], &products[3], &products[4]]).await?;
    
    // Get all featured products
    let featured_products: Vec<Product> = cache.set_members(featured_set).await?;
    println!("\nFeatured products:");
    for product in &featured_products {
        println!("- {} ({})", product.name, product.category);
    }
    
    // Check if a product is featured
    let product_to_check = &products[1]; // Smartphone
    let is_featured = cache.set_contains(featured_set, product_to_check).await?;
    println!("\nIs {} featured? {}", product_to_check.name, is_featured);
    
    // Find featured electronics (intersection)
    println!("\nFinding featured electronics (intersection):");
    let featured_electronics: Vec<Product> = cache.set_intersection(vec![featured_set, electronics_set]).await?;
    for product in &featured_electronics {
        println!("- {} ({})", product.name, product.category);
    }
    
    // Find non-featured clothing (difference)
    println!("\nFinding non-featured clothing (difference):");
    let non_featured_clothing: Vec<Product> = cache.set_difference(vec![clothing_set, featured_set]).await?;
    for product in &non_featured_clothing {
        println!("- {} ({})", product.name, product.category);
    }
    
    Ok(())
}

async fn cleanup(cache: &RedisCache) -> CacheResult<()> {
    // Clean up all the keys created in this example
    println!("\nCleaning up example keys...");
    
    let keys = vec![
        "simple_set",
        "membership_test_set",
        "set_a",
        "set_b",
        "set_c",
        "red_fruits_from_our_list",
        "red_fruits_or_berries",
        "non_red_fruits",
        "berries_in_our_list",
        "non_red_berries",
        "large_set",
        "category:electronics",
        "category:clothing",
        "category:footwear",
        "featured:products",
    ];
    
    for key in keys {
        let _ = cache.delete(key).await;
    }
    
    println!("Cleanup complete.");
    Ok(())
} 