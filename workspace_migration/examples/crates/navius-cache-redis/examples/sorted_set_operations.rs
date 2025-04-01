use navius_cache::{
    config::CacheConfig,
    error::CacheResult,
    key::CacheKey,
    operations::Cache,
};
use navius_cache_redis::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    operations::{RedisCache, SortedSetOperations},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use rand::{thread_rng, Rng};

/**
 * This example demonstrates the use of Sorted Set (ZSet) operations in the Redis cache.
 * 
 * Sorted Sets allow you to:
 * - Store items with associated scores
 * - Retrieve items by score range
 * - Retrieve items by rank (position in the sorted order)
 * - Perform complex operations like unions and intersections with score aggregation
 * 
 * Sorted Sets are useful for:
 * - Leaderboards and rankings
 * - Time-series data with timestamps as scores
 * - Priority queues
 * - Rate limiting with timestamps
 * - Weighted recommendations
 * - Range queries
 */

// Example data structure for demonstrating serialization with sorted sets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Player {
    id: String,
    name: String,
    team: String,
}

impl Player {
    fn new(id: &str, name: &str, team: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            team: team.to_string(),
        }
    }
}

// Implement CacheKey for Player so it can be used as a key in the cache
impl CacheKey for Player {
    fn to_cache_key(&self) -> String {
        format!("player:{}", self.id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    popularity: f64,
    category: Category,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum Category {
    Electronics,
    Clothing,
    Books,
    HomeGoods,
    Toys,
}

impl CacheKey for Product {
    fn to_string(&self) -> String {
        format!("product:{}", self.id)
    }
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create Redis cache config
    let config = RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        key_prefix: "navius:example:zset".to_string(),
        connection_timeout_seconds: 5,
        connection_retries: 3,
        pool_size: 10,
    };

    // Create connection manager
    let connection_manager = RedisConnectionManager::new(config).expect("Failed to create connection manager");
    
    // Create Redis cache
    let cache = RedisCache::new(connection_manager.into()).with_lua_scripting();
    
    println!("=== Redis Sorted Set (ZSet) Operations Example ===");
    
    // Generate test data
    let products = generate_test_products(30);
    
    // Example 1: Basic Sorted Set Operations
    println!("\n=== Example 1: Basic Sorted Set Operations ===");
    
    // Add products to a sorted set by price
    let zset_key = "products:by_price";
    let product_prices: Vec<(f64, u32)> = products.iter()
        .map(|p| (p.price, p.id))
        .collect();
    
    println!("Adding {} products to sorted set by price...", products.len());
    let count = cache.zset_add(zset_key, product_prices).await?;
    println!("Added {} products to sorted set", count);
    
    // Get product count
    let set_size = cache.zset_length(zset_key).await?;
    println!("Total products in sorted set: {}", set_size);
    
    // Get 5 cheapest products
    let cheapest: Vec<u32> = cache.zset_range(zset_key, 0, 4).await?;
    println!("5 cheapest products: {:?}", cheapest);
    
    // Get 5 most expensive products
    let most_expensive: Vec<u32> = cache.zset_range(zset_key, -5, -1).await?;
    println!("5 most expensive products: {:?}", most_expensive);
    
    // Get 5 most expensive products with prices
    let expensive_with_prices: Vec<(u32, f64)> = cache.zset_range_with_scores(zset_key, -5, -1).await?;
    println!("5 most expensive products with prices:");
    for (id, price) in expensive_with_prices {
        let product = products.iter().find(|p| p.id == id).unwrap();
        println!("  {}: ${:.2} - {}", id, price, product.name);
    }
    
    // Example 2: Range Queries
    println!("\n=== Example 2: Range Queries ===");
    
    // Get products priced between $20 and $50
    let mid_range: Vec<u32> = cache.zset_range_by_score(zset_key, 20.0, 50.0).await?;
    println!("Products priced between $20 and $50 ({}): {:?}", mid_range.len(), mid_range);
    
    // Get products priced between $20 and $50 with prices
    let mid_range_with_prices: Vec<(u32, f64)> = 
        cache.zset_range_by_score_with_scores(zset_key, 20.0, 50.0).await?;
    println!("Products priced between $20 and $50 with prices:");
    for (id, price) in mid_range_with_prices {
        let product = products.iter().find(|p| p.id == id).unwrap();
        println!("  {}: ${:.2} - {}", id, price, product.name);
    }
    
    // Count products in price ranges
    let budget_count = cache.zset_count(zset_key, 0.0, 25.0).await?;
    let mid_count = cache.zset_count(zset_key, 25.01, 75.0).await?;
    let premium_count = cache.zset_count(zset_key, 75.01, 200.0).await?;
    
    println!("Product count by price range:");
    println!("  Budget ($0-$25): {}", budget_count);
    println!("  Mid-range ($25-$75): {}", mid_count);
    println!("  Premium ($75+): {}", premium_count);
    
    // Example 3: Rank and Score Operations
    println!("\n=== Example 3: Rank and Score Operations ===");
    
    // Check rank of specific products
    let cheapest_id = products.iter().min_by(|a, b| a.price.partial_cmp(&b.price).unwrap()).unwrap().id;
    let most_expensive_id = products.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap()).unwrap().id;
    
    let cheapest_rank = cache.zset_rank(zset_key, &cheapest_id).await?;
    let most_expensive_rank = cache.zset_rank(zset_key, &most_expensive_id).await?;
    
    println!("Rank of cheapest product (ID {}): {}", cheapest_id, cheapest_rank.unwrap());
    println!("Rank of most expensive product (ID {}): {}", most_expensive_id, most_expensive_rank.unwrap());
    
    // Get reverse ranks (from highest to lowest)
    let cheapest_reverse_rank = cache.zset_reverse_rank(zset_key, &cheapest_id).await?;
    let most_expensive_reverse_rank = cache.zset_reverse_rank(zset_key, &most_expensive_id).await?;
    
    println!("Reverse rank of cheapest product (ID {}): {}", cheapest_id, cheapest_reverse_rank.unwrap());
    println!("Reverse rank of most expensive product (ID {}): {}", most_expensive_id, most_expensive_reverse_rank.unwrap());
    
    // Get scores (prices) of specific products
    let middle_id = products[products.len() / 2].id;
    let price1 = cache.zset_score(zset_key, &cheapest_id).await?;
    let price2 = cache.zset_score(zset_key, &most_expensive_id).await?;
    let price3 = cache.zset_score(zset_key, &middle_id).await?;
    
    println!("Price of cheapest product (ID {}): ${:.2}", cheapest_id, price1.unwrap());
    println!("Price of most expensive product (ID {}): ${:.2}", most_expensive_id, price2.unwrap());
    println!("Price of middle product (ID {}): ${:.2}", middle_id, price3.unwrap());
    
    // Example 4: Modifying Scores
    println!("\n=== Example 4: Modifying Scores ===");
    
    // Apply a 10% price increase to middle product
    let original_price = price3.unwrap();
    let price_increase = original_price * 0.1;
    
    let new_price = cache.zset_increment_score(zset_key, &middle_id, price_increase).await?;
    println!("Increased price of product {} by 10%: ${:.2} -> ${:.2}", 
             middle_id, original_price, new_price);
    
    // Apply a 15% discount to most expensive product
    let original_price = price2.unwrap();
    let price_discount = -1.0 * original_price * 0.15;
    
    let new_price = cache.zset_increment_score(zset_key, &most_expensive_id, price_discount).await?;
    println!("Applied 15% discount to product {}: ${:.2} -> ${:.2}", 
             most_expensive_id, original_price, new_price);
    
    // Re-check ranks after price changes
    let new_rank = cache.zset_rank(zset_key, &most_expensive_id).await?;
    println!("New rank of product {} after discount: {}", most_expensive_id, new_rank.unwrap());
    
    // Example 5: Popularity by Category
    println!("\n=== Example 5: Popularity by Category ===");
    
    // Create sorted sets for each category
    let categories = [
        "products:electronics", 
        "products:clothing", 
        "products:books", 
        "products:homegoods", 
        "products:toys"
    ];
    
    // Add products to category-specific sorted sets
    for product in &products {
        let category_key = match product.category {
            Category::Electronics => categories[0],
            Category::Clothing => categories[1],
            Category::Books => categories[2],
            Category::HomeGoods => categories[3],
            Category::Toys => categories[4],
        };
        
        // Add product to category sorted set, scored by popularity
        cache.zset_add(category_key, vec![(product.popularity, product.id)]).await?;
    }
    
    // Get most popular product in each category
    println!("Most popular products by category:");
    for &category in &categories {
        let popular: Vec<u32> = cache.zset_range(category, -1, -1).await?;
        if !popular.is_empty() {
            let id = popular[0];
            let product = products.iter().find(|p| p.id == id).unwrap();
            println!("  {}: {} (Popularity: {:.1})", 
                    category, product.name, product.popularity);
        }
    }
    
    // Example 6: Set Operations with Sorted Sets
    println!("\n=== Example 6: Set Operations with Sorted Sets ===");
    
    // Find the union of electronics and toys, weighting electronics higher
    let destination = "products:tech_and_toys";
    let count = cache.zset_union_store(
        destination, 
        vec![categories[0], categories[4]], 
        Some(vec![1.5, 1.0]), 
        Some("SUM".to_string())
    ).await?;
    
    println!("Created union of electronics and toys with {} products", count);
    
    // Get top 3 items from the union with scores
    let top_items: Vec<(u32, f64)> = cache.zset_range_with_scores(destination, -3, -1).await?;
    println!("Top 3 items from tech_and_toys union:");
    for (id, score) in top_items {
        let product = products.iter().find(|p| p.id == id).unwrap();
        println!("  {}: {} (Union Score: {:.1})", id, product.name, score);
    }
    
    // Clean up
    println!("\nCleaning up...");
    cache.delete(zset_key).await?;
    for &category in &categories {
        cache.delete(category).await?;
    }
    cache.delete(destination).await?;
    
    println!("Successfully completed all sorted set operations!");
    
    Ok(())
}

fn generate_test_products(count: usize) -> Vec<Product> {
    let mut rng = thread_rng();
    let mut products = Vec::with_capacity(count);
    
    let categories = [
        Category::Electronics,
        Category::Clothing,
        Category::Books,
        Category::HomeGoods,
        Category::Toys,
    ];
    
    let product_names = [
        "Smartphone", "Laptop", "Headphones", "Camera", "Tablet", 
        "T-shirt", "Jeans", "Dress", "Jacket", "Shoes",
        "Novel", "Cookbook", "Biography", "Textbook", "Comic",
        "Sofa", "Lamp", "Table", "Chair", "Vase",
        "Action Figure", "Board Game", "Puzzle", "Doll", "Remote Car",
    ];
    
    for i in 1..=count {
        let category_idx = rng.gen_range(0..categories.len());
        let name_base_idx = category_idx * 5 + rng.gen_range(0..5);
        let name = format!("{} {}", product_names[name_base_idx % product_names.len()], i);
        
        let price = match categories[category_idx] {
            Category::Electronics => rng.gen_range(50.0..200.0),
            Category::Clothing => rng.gen_range(20.0..100.0),
            Category::Books => rng.gen_range(10.0..50.0),
            Category::HomeGoods => rng.gen_range(30.0..150.0),
            Category::Toys => rng.gen_range(15.0..80.0),
        };
        
        products.push(Product {
            id: i as u32,
            name,
            price,
            popularity: rng.gen_range(1.0..100.0),
            category: categories[category_idx].clone(),
        });
    }
    
    products
} 