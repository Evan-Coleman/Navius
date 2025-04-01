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

    // Basic Set Operations Example
    println!("\n1. Basic Set Operations");
    await basic_set_operations(&cache).await?;

    // Set Membership Example
    println!("\n2. Set Membership Tests");
    await set_membership_test(&cache).await?;

    // Complex Set Operations Example
    println!("\n3. Complex Set Operations (Union, Intersection, Difference)");
    await complex_set_operations(&cache).await?;

    // Random Members Example
    println!("\n4. Retrieving Random Set Members");
    await random_members_example(&cache).await?;

    // Complex Objects in Sets Example
    println!("\n5. Using Complex Objects in Sets");
    await complex_objects_example(&cache).await?;

    // Clean up
    cleanup(&cache).await?;

    println!("\nSet operations example completed successfully!");
    Ok(())
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