use std::time::{Duration, Instant};
use navius_cache::{
    operations::{Cache, CacheKey, CacheOperations, CacheOptions},
    error::CacheResult,
};
use navius_cache_redis::{
    connection::{RedisConnectionManager, RedisCacheConfig},
    operations::RedisCache,
    RedisCommandPipeline,
};
use serde::{Deserialize, Serialize};
use rand::{thread_rng, Rng};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Order {
    id: u32,
    customer_id: u32,
    items: Vec<OrderItem>,
    total: f64,
    status: OrderStatus,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct OrderItem {
    product_id: u32,
    quantity: u32,
    price: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Canceled,
}

impl CacheKey for Order {
    fn to_string(&self) -> String {
        format!("order:{}", self.id)
    }
}

impl CacheKey for u32 {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create Redis cache config
    let config = RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        key_prefix: "navius:example:pipeline".to_string(),
        connection_timeout_seconds: 5,
        connection_retries: 3,
        pool_size: 10,
    };

    // Create connection manager
    let connection_manager = RedisConnectionManager::new(config).expect("Failed to create connection manager");
    
    // Create Redis cache
    let cache = RedisCache::new(connection_manager.into()).with_lua_scripting();
    
    println!("=== Redis Pipeline Operations Example ===");
    
    // Generate test data
    let orders = generate_test_orders(10);
    
    // Example 1: Individual SET Operations vs. Pipeline
    println!("\n=== Example 1: Individual vs. Pipeline SET Operations ===");
    
    // Measure time for individual SET operations
    let start = Instant::now();
    
    for order in &orders {
        let key = format!("order:{}", order.id);
        cache.set(key, order, None).await?;
    }
    
    let individual_duration = start.elapsed();
    println!("Individual SET operations for {} orders took: {:?}", orders.len(), individual_duration);
    
    // Clear data
    for order in &orders {
        let key = format!("order:{}", order.id);
        cache.delete(key).await?;
    }
    
    // Measure time for pipeline SET operations
    let start = Instant::now();
    
    let mut pipeline = cache.pipeline();
    
    for order in &orders {
        let key = format!("order:{}", order.id);
        pipeline = pipeline.set(key, order);
    }
    
    cache.execute_pipeline(pipeline).await?;
    
    let pipeline_duration = start.elapsed();
    println!("Pipeline SET operations for {} orders took: {:?}", orders.len(), pipeline_duration);
    
    println!("Speed improvement: {:.2}x", individual_duration.as_secs_f64() / pipeline_duration.as_secs_f64());
    
    // Example 2: Batched Operations - Update Order Status
    println!("\n=== Example 2: Batch Update Order Status ===");
    
    // Update status for multiple orders in one pipeline
    let start = Instant::now();
    
    let order_ids = vec![1, 3, 5, 7, 9];
    
    let mut pipeline = cache.pipeline();
    
    // Get multiple orders in one pipeline operation
    for &id in &order_ids {
        let key = format!("order:{}", id);
        pipeline = pipeline.get(key);
    }
    
    cache.execute_pipeline(pipeline).await?;
    
    // Now retrieve the orders individually (in a real scenario, we'd parse the pipeline results)
    let mut orders_to_update = Vec::new();
    for &id in &order_ids {
        let key = format!("order:{}", id);
        if let Some(mut order) = cache.get::<_, Order>(key).await? {
            // Update status
            order.status = OrderStatus::Shipped;
            orders_to_update.push(order);
        }
    }
    
    // Update all orders in a single pipeline
    let mut pipeline = cache.pipeline();
    
    for order in &orders_to_update {
        let key = format!("order:{}", order.id);
        pipeline = pipeline.set(key, order);
    }
    
    cache.execute_pipeline(pipeline).await?;
    
    let batch_duration = start.elapsed();
    println!("Batch update of {} orders took: {:?}", orders_to_update.len(), batch_duration);
    
    // Example 3: Pipeline for Complex Data Structure Updates
    println!("\n=== Example 3: Pipeline for Complex Data Structure Updates ===");
    
    // Create customer-order index
    let start = Instant::now();
    
    // Group orders by customer
    let mut customer_orders = std::collections::HashMap::new();
    for order in &orders {
        customer_orders
            .entry(order.customer_id)
            .or_insert_with(Vec::new)
            .push(order.id);
    }
    
    // Create a sorted set of orders by customer
    let mut pipeline = cache.pipeline();
    
    for (customer_id, order_ids) in &customer_orders {
        let key = format!("customer:{}:orders", customer_id);
        
        // Add each order to the customer's sorted set with timestamp as score
        for &order_id in order_ids {
            let order = orders.iter().find(|o| o.id == order_id).unwrap();
            pipeline = pipeline.set(format!("order:{}", order_id), order);
        }
        
        // Also delete the key to ensure it's clean
        pipeline = pipeline.delete(key);
    }
    
    cache.execute_pipeline(pipeline).await?;
    
    // Now use individual operations to add to sorted sets
    for (customer_id, order_ids) in &customer_orders {
        for &order_id in order_ids {
            let order = orders.iter().find(|o| o.id == order_id).unwrap();
            let timestamp = chrono::NaiveDateTime::parse_from_str(&order.created_at, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .timestamp() as f64;
                
            cache.zset_add(
                format!("customer:{}:orders", customer_id),
                vec![(timestamp, order_id)],
            ).await?;
        }
    }
    
    let complex_duration = start.elapsed();
    println!("Creating customer-order index took: {:?}", complex_duration);
    
    // Verify the index - get the latest orders for each customer
    for (customer_id, _) in &customer_orders {
        let latest_orders: Vec<u32> = cache
            .zset_range(format!("customer:{}:orders", customer_id), -3, -1)
            .await?;
            
        println!("Latest 3 orders for customer {}: {:?}", customer_id, latest_orders);
    }
    
    // Example 4: Delete operations with pipeline
    println!("\n=== Example 4: Delete Operations with Pipeline ===");
    
    let start = Instant::now();
    
    let mut keys_to_delete = Vec::new();
    
    // Add order keys
    for order in &orders {
        keys_to_delete.push(format!("order:{}", order.id));
    }
    
    // Add customer order indices
    for customer_id in customer_orders.keys() {
        keys_to_delete.push(format!("customer:{}:orders", customer_id));
    }
    
    // Delete all keys in a single pipeline
    let mut pipeline = cache.pipeline();
    
    for key in &keys_to_delete {
        pipeline = pipeline.delete(key);
    }
    
    cache.execute_pipeline(pipeline).await?;
    
    let cleanup_duration = start.elapsed();
    println!("Deleted {} keys in: {:?}", keys_to_delete.len(), cleanup_duration);
    
    println!("\nSuccessfully completed all pipeline operations!");
    
    Ok(())
}

fn generate_test_orders(count: usize) -> Vec<Order> {
    let mut rng = thread_rng();
    let mut orders = Vec::with_capacity(count);
    
    let statuses = [
        OrderStatus::Pending,
        OrderStatus::Processing,
        OrderStatus::Shipped,
        OrderStatus::Delivered,
        OrderStatus::Canceled,
    ];
    
    for i in 1..=count {
        let customer_id = rng.gen_range(1..5);
        let num_items = rng.gen_range(1..6);
        
        let mut items = Vec::with_capacity(num_items);
        let mut total = 0.0;
        
        for _ in 0..num_items {
            let product_id = rng.gen_range(1..100);
            let quantity = rng.gen_range(1..5);
            let price = rng.gen_range(10.0..100.0);
            let item_total = price * quantity as f64;
            
            items.push(OrderItem {
                product_id,
                quantity,
                price,
            });
            
            total += item_total;
        }
        
        // Generate a random timestamp within the last 30 days
        let days_ago = rng.gen_range(0..30);
        let hours_ago = rng.gen_range(0..24);
        let mins_ago = rng.gen_range(0..60);
        
        let now = chrono::Local::now();
        let created_at = now
            .checked_sub_signed(chrono::Duration::days(days_ago))
            .unwrap()
            .checked_sub_signed(chrono::Duration::hours(hours_ago))
            .unwrap()
            .checked_sub_signed(chrono::Duration::minutes(mins_ago))
            .unwrap()
            .naive_local()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        
        orders.push(Order {
            id: i as u32,
            customer_id,
            items,
            total,
            status: statuses[rng.gen_range(0..statuses.len())].clone(),
            created_at,
        });
    }
    
    orders
} 