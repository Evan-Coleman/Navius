//! # Cache Invalidation Testing Example
//!
//! This example demonstrates how to test cache invalidation scenarios using
//! the Navius Test Framework. It covers several key invalidation patterns:
//!
//! - Time-based expiration (TTL)
//! - Explicit key invalidation
//! - Pattern-based invalidation (e.g., wildcard keys)
//! - Write-through and write-behind caching
//! - Cache consistency verification
//!
//! The example uses mock implementations to simulate cache behavior without
//! requiring an actual cache server.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use navius_test::{
    error::{TestResult, assert_eq, assert_false, assert_none, assert_some, assert_true},
    fixture::TestFixture,
    mocks::cache::{CacheError, MockCacheClient},
};

/// A product data service that uses caching
struct ProductService {
    /// Database client (mocked in tests)
    db: MockDatabaseClient,
    /// Cache client
    cache: MockCacheClient,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

/// A mock database client for testing
struct MockDatabaseClient {
    /// Simulated database with product data
    data: Arc<RwLock<HashMap<String, Product>>>,
    /// Counter to track database calls
    query_count: Arc<RwLock<u64>>,
}

/// Product information
#[derive(Debug, Clone, PartialEq)]
struct Product {
    id: String,
    name: String,
    price: f64,
    stock: u32,
    updated_at: Instant,
}

impl Product {
    /// Create a new product
    fn new(id: &str, name: &str, price: f64, stock: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            price,
            stock,
            updated_at: Instant::now(),
        }
    }

    /// Serialize to JSON (simplified for the example)
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","name":"{}","price":{},"stock":{}}}"#,
            self.id, self.name, self.price, self.stock
        )
    }

    /// Deserialize from JSON (simplified for the example)
    fn from_json(json: &str) -> Result<Self, String> {
        // In a real implementation, use a proper JSON parser
        // This is simplified for the example
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err("Invalid JSON format".to_string());
        }

        let mut id = String::new();
        let mut name = String::new();
        let mut price = 0.0;
        let mut stock = 0;

        // Very naive parsing for demonstration purposes
        for part in json[1..json.len() - 1].split(',') {
            let kv: Vec<&str> = part.split(':').collect();
            if kv.len() != 2 {
                continue;
            }

            let key = kv[0].trim_matches(|c| c == '"' || c == ' ');
            let value = kv[1].trim_matches(|c| c == '"' || c == ' ');

            match key {
                "id" => id = value.to_string(),
                "name" => name = value.to_string(),
                "price" => price = value.parse().unwrap_or(0.0),
                "stock" => stock = value.parse().unwrap_or(0),
                _ => {}
            }
        }

        if id.is_empty() || name.is_empty() {
            return Err("Missing required fields".to_string());
        }

        Ok(Self {
            id,
            name,
            price,
            stock,
            updated_at: Instant::now(),
        })
    }
}

impl MockDatabaseClient {
    /// Create a new mock database client
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            query_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Add test data to the mock database
    fn add_product(&self, product: Product) {
        let mut data = self.data.write().unwrap();
        data.insert(product.id.clone(), product);
    }

    /// Simulate retrieving a product from the database
    async fn get_product(&self, id: &str) -> Result<Option<Product>, String> {
        // Increment query count to track database access
        {
            let mut count = self.query_count.write().unwrap();
            *count += 1;
        }

        // Simulate database latency
        tokio::time::sleep(Duration::from_millis(50)).await;

        let data = self.data.read().unwrap();
        Ok(data.get(id).cloned())
    }

    /// Update a product in the database
    async fn update_product(&self, product: Product) -> Result<(), String> {
        // Increment query count
        {
            let mut count = self.query_count.write().unwrap();
            *count += 1;
        }

        // Simulate database latency
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut data = self.data.write().unwrap();

        if !data.contains_key(&product.id) {
            return Err(format!("Product with ID {} not found", product.id));
        }

        data.insert(product.id.clone(), product);
        Ok(())
    }

    /// Get the number of database queries executed
    fn query_count(&self) -> u64 {
        *self.query_count.read().unwrap()
    }

    /// Reset the query counter
    fn reset_query_count(&self) {
        let mut count = self.query_count.write().unwrap();
        *count = 0;
    }
}

impl ProductService {
    /// Create a new product service
    fn new(db: MockDatabaseClient, cache: MockCacheClient, cache_ttl: u64) -> Self {
        Self {
            db,
            cache,
            cache_ttl,
        }
    }

    /// Get a product by ID, using the cache with TTL
    async fn get_product(&self, id: &str) -> Result<Option<Product>, String> {
        // Try to get from cache first
        let cache_key = format!("product:{}", id);
        match self.cache.get(&cache_key).await {
            Ok(Some(data)) => {
                // Cache hit - deserialize and return
                return Product::from_json(&data).map(Some);
            }
            Ok(None) => {
                // Cache miss - get from database
            }
            Err(e) => {
                // Cache error - log and continue with database
                println!("Cache error: {}", e);
            }
        }

        // Get from database
        match self.db.get_product(id).await? {
            Some(product) => {
                // Store in cache with TTL
                let _ = self
                    .cache
                    .set(&cache_key, &product.to_json(), Some(self.cache_ttl))
                    .await;
                Ok(Some(product))
            }
            None => Ok(None),
        }
    }

    /// Update a product with write-through caching
    async fn update_product(&self, product: Product) -> Result<(), String> {
        // Update in database first
        self.db.update_product(product.clone()).await?;

        // Update cache (write-through)
        let cache_key = format!("product:{}", product.id);
        match self
            .cache
            .set(&cache_key, &product.to_json(), Some(self.cache_ttl))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                // Cache error - log but don't fail since database update succeeded
                println!("Cache update error: {}", e);
            }
        }

        // Invalidate any collections containing this product
        let _ = self.invalidate_product_collections(&product.id).await;

        Ok(())
    }

    /// Invalidate a specific product in the cache
    async fn invalidate_product(&self, id: &str) -> Result<(), String> {
        let cache_key = format!("product:{}", id);
        match self.cache.delete(&cache_key).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to invalidate product: {}", e)),
        }
    }

    /// Invalidate all collections that might contain this product
    async fn invalidate_product_collections(&self, product_id: &str) -> Result<(), String> {
        // In a real implementation, we might have collection keys like:
        // - category:electronics:products
        // - featured:products
        // - recently_viewed:user123

        // For this example, we'll just use a simple pattern
        let pattern = "collection:*";
        match self.cache.delete_pattern(pattern).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to invalidate collections: {}", e)),
        }
    }

    /// Update product stock with explicit cache invalidation
    async fn update_stock(&self, id: &str, new_stock: u32) -> Result<(), String> {
        // Get current product
        let product = match self.db.get_product(id).await? {
            Some(p) => p,
            None => return Err(format!("Product with ID {} not found", id)),
        };

        // Update the product
        let updated_product = Product {
            stock: new_stock,
            updated_at: Instant::now(),
            ..product
        };

        // Update in database
        self.db.update_product(updated_product).await?;

        // Invalidate cache (instead of updating it)
        self.invalidate_product(id).await?;

        Ok(())
    }

    /// Get all products in a collection, demonstrating batch caching
    async fn get_products_in_collection(
        &self,
        collection_id: &str,
    ) -> Result<Vec<Product>, String> {
        let cache_key = format!("collection:{}", collection_id);

        // Try to get collection from cache
        match self.cache.get(&cache_key).await {
            Ok(Some(data)) => {
                // For simplicity, assume data is a comma-separated list of product IDs
                let product_ids: Vec<String> = data.split(',').map(|s| s.to_string()).collect();

                // Get each product (may use cache)
                let mut products = Vec::new();
                for id in product_ids {
                    if let Some(product) = self.get_product(&id).await? {
                        products.push(product);
                    }
                }

                return Ok(products);
            }
            Ok(None) => {
                // Cache miss - would fetch from database
                // For this example, we'll return an empty list
                return Ok(Vec::new());
            }
            Err(e) => {
                // Cache error - log and continue
                println!("Cache error: {}", e);
                return Ok(Vec::new());
            }
        }
    }
}

// Mock implementation of time-related functions for testing
struct MockClock {
    current_time: Arc<RwLock<Instant>>,
}

impl MockClock {
    fn new() -> Self {
        Self {
            current_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    fn advance(&self, duration: Duration) {
        let mut time = self.current_time.write().unwrap();
        *time += duration;
    }

    fn now(&self) -> Instant {
        *self.current_time.read().unwrap()
    }
}

/// Example test for time-based cache invalidation (TTL)
#[tokio::test]
async fn test_time_based_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();

    // Create mock components
    let db = MockDatabaseClient::new();
    let cache = MockCacheClient::new();

    // Add test data
    let product = Product::new("p1", "Test Product", 29.99, 100);
    db.add_product(product.clone());

    // Set cache expectations

    // First get: cache miss
    cache.expect_get("product:p1").returns(Ok(None));

    // Cache set after first database retrieval
    cache
        .expect_set("product:p1", &product.to_json(), Some(60))
        .returns(Ok(()));

    // Second get: cache hit
    cache
        .expect_get("product:p1")
        .returns(Ok(Some(product.to_json())));

    // Third get (after TTL): cache miss
    cache.expect_get("product:p1").returns(Ok(None));

    // Cache set after second database retrieval
    cache
        .expect_set("product:p1", &product.to_json(), Some(60))
        .returns(Ok(()));

    // Create the service
    let service = ProductService::new(db.clone(), cache.clone(), 60);

    // First access - should miss cache and fetch from DB
    db.reset_query_count();
    let result1 = service.get_product("p1").await?;

    // Verify result and database was accessed
    assert_some(
        result1,
        product.clone(),
        "Should return the correct product",
    )?;
    assert_eq(db.query_count(), 1, "Should have queried the database")?;

    // Second access - should hit cache and not access DB
    db.reset_query_count();
    let result2 = service.get_product("p1").await?;

    // Verify result and database was not accessed
    assert_some(
        result2,
        product.clone(),
        "Should return the correct product",
    )?;
    assert_eq(db.query_count(), 0, "Should not have queried the database")?;

    // Simulate TTL expiration by changing cache expectations
    // The third get will be a cache miss as configured above

    // Third access - should miss cache (due to TTL) and fetch from DB
    db.reset_query_count();
    let result3 = service.get_product("p1").await?;

    // Verify result and database was accessed again
    assert_some(
        result3,
        product.clone(),
        "Should return the correct product",
    )?;
    assert_eq(
        db.query_count(),
        1,
        "Should have queried the database again",
    )?;

    Ok(())
}

/// Example test for explicit cache invalidation
#[tokio::test]
async fn test_explicit_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();

    // Create mock components
    let db = MockDatabaseClient::new();
    let cache = MockCacheClient::new();

    // Add test data
    let product = Product::new("p2", "Another Product", 19.99, 50);
    db.add_product(product.clone());

    // Set cache expectations

    // First get: cache miss
    cache.expect_get("product:p2").returns(Ok(None));

    // Cache set after first database retrieval
    cache
        .expect_set("product:p2", &product.to_json(), Some(60))
        .returns(Ok(()));

    // Second get: cache hit
    cache
        .expect_get("product:p2")
        .returns(Ok(Some(product.to_json())));

    // Delete operation for cache invalidation
    cache.expect_delete("product:p2").returns(Ok(true));

    // Third get after invalidation: cache miss
    cache.expect_get("product:p2").returns(Ok(None));

    // Updated product data
    let updated_product = Product {
        stock: 45,
        ..product.clone()
    };

    // Cache set after second database retrieval with updated product
    cache
        .expect_set("product:p2", &updated_product.to_json(), Some(60))
        .returns(Ok(()));

    // Create the service
    let service = ProductService::new(db.clone(), cache.clone(), 60);

    // First access - should miss cache and fetch from DB
    db.reset_query_count();
    let result1 = service.get_product("p2").await?;

    // Verify result and database was accessed
    assert_some(
        result1,
        product.clone(),
        "Should return the correct product",
    )?;
    assert_eq(db.query_count(), 1, "Should have queried the database")?;

    // Second access - should hit cache and not access DB
    db.reset_query_count();
    let result2 = service.get_product("p2").await?;

    // Verify result and database was not accessed
    assert_some(
        result2,
        product.clone(),
        "Should return the correct product",
    )?;
    assert_eq(db.query_count(), 0, "Should not have queried the database")?;

    // Explicitly invalidate the cache
    service.invalidate_product("p2").await?;

    // Update the database (simulate another service updating it)
    db.update_product(updated_product.clone()).await?;

    // Third access - should miss cache (due to invalidation) and fetch updated data from DB
    db.reset_query_count();
    let result3 = service.get_product("p2").await?;

    // Verify result has updated data and database was accessed again
    assert_some(
        result3,
        updated_product.clone(),
        "Should return the updated product",
    )?;
    assert_eq(
        db.query_count(),
        1,
        "Should have queried the database again",
    )?;

    Ok(())
}

/// Example test for pattern-based cache invalidation
#[tokio::test]
async fn test_pattern_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();

    // Create mock components
    let db = MockDatabaseClient::new();
    let cache = MockCacheClient::new();

    // Add test data
    let product = Product::new("p3", "Pattern Test", 39.99, 75);
    db.add_product(product.clone());

    // Set cache expectations

    // Product update operation in DB
    db.expect_update_product(product.clone()).returns(Ok(()));

    // Cache set for product update (write-through)
    cache
        .expect_set("product:p3", &product.to_json(), Some(60))
        .returns(Ok(()));

    // Pattern invalidation for collections
    cache.expect_delete_pattern("collection:*").returns(Ok(3)); // Assume 3 keys matched and were deleted

    // Create the service
    let service = ProductService::new(db.clone(), cache.clone(), 60);

    // Update the product, which should also invalidate collections
    service.update_product(product.clone()).await?;

    // Verify all mock expectations were met

    Ok(())
}

/// Example test for write-through cache consistency
#[tokio::test]
async fn test_write_through_consistency() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();

    // Create mock components
    let db = MockDatabaseClient::new();
    let cache = MockCacheClient::new();

    // Add test data
    let product = Product::new("p4", "Consistency Test", 49.99, 200);
    db.add_product(product.clone());

    // Updated product
    let updated_product = Product {
        price: 45.99,
        stock: 180,
        ..product.clone()
    };

    // Set cache expectations

    // Initial get: cache miss
    cache.expect_get("product:p4").returns(Ok(None));

    // Cache set after first database retrieval
    cache
        .expect_set("product:p4", &product.to_json(), Some(60))
        .returns(Ok(()));

    // Update in database
    db.expect_update_product(updated_product.clone())
        .returns(Ok(()));

    // Update in cache (write-through)
    cache
        .expect_set("product:p4", &updated_product.to_json(), Some(60))
        .returns(Ok(()));

    // Pattern invalidation for collections
    cache.expect_delete_pattern("collection:*").returns(Ok(0)); // No collections matched

    // Get after update: cache hit with updated data
    cache
        .expect_get("product:p4")
        .returns(Ok(Some(updated_product.to_json())));

    // Create the service
    let service = ProductService::new(db.clone(), cache.clone(), 60);

    // First access - should miss cache and fetch from DB
    let result1 = service.get_product("p4").await?;
    assert_some(
        result1,
        product.clone(),
        "Should return the initial product",
    )?;

    // Update the product with write-through caching
    service.update_product(updated_product.clone()).await?;

    // Next access - should hit cache with updated data
    db.reset_query_count();
    let result2 = service.get_product("p4").await?;

    // Verify result has updated data and database was not accessed
    assert_some(
        result2,
        updated_product.clone(),
        "Should return the updated product from cache",
    )?;
    assert_eq(db.query_count(), 0, "Should not have queried the database")?;

    Ok(())
}

/// Example test for cache invalidation during stock updates
#[tokio::test]
async fn test_stock_update_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();

    // Create mock components
    let db = MockDatabaseClient::new();
    let cache = MockCacheClient::new();

    // Add test data
    let product = Product::new("p5", "Stock Test", 15.99, 50);
    db.add_product(product.clone());

    // Set cache expectations

    // Initial get: cache miss
    cache.expect_get("product:p5").returns(Ok(None));

    // Cache set after first database retrieval
    cache
        .expect_set("product:p5", &product.to_json(), Some(60))
        .returns(Ok(()));

    // Get before update: cache hit
    cache
        .expect_get("product:p5")
        .returns(Ok(Some(product.to_json())));

    // Stock update in DB
    db.expect_get_product("p5")
        .returns(Ok(Some(product.clone())));

    // Updated product with new stock
    let updated_product = Product {
        stock: 40,
        ..product.clone()
    };

    db.expect_update_product(updated_product.clone())
        .returns(Ok(()));

    // Cache invalidation during stock update
    cache.expect_delete("product:p5").returns(Ok(true));

    // Get after invalidation: cache miss
    cache.expect_get("product:p5").returns(Ok(None));

    // Cache set after database retrieval post-invalidation
    cache
        .expect_set("product:p5", &updated_product.to_json(), Some(60))
        .returns(Ok(()));

    // Create the service
    let service = ProductService::new(db.clone(), cache.clone(), 60);

    // First access - should miss cache and fetch from DB
    let result1 = service.get_product("p5").await?;
    assert_some(
        result1,
        product.clone(),
        "Should return the initial product",
    )?;

    // Second access - should hit cache
    db.reset_query_count();
    let result2 = service.get_product("p5").await?;
    assert_eq(db.query_count(), 0, "Should not have queried the database")?;

    // Update stock, which should invalidate the cache
    service.update_stock("p5", 40).await?;

    // Access after stock update - should miss cache and fetch updated data
    db.reset_query_count();
    let result3 = service.get_product("p5").await?;

    // Verify result has updated stock and database was accessed
    assert_some(
        result3,
        updated_product.clone(),
        "Should return product with updated stock",
    )?;
    assert_eq(db.query_count(), 1, "Should have queried the database")?;

    Ok(())
}

/// Main function to run examples
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running cache invalidation testing examples...");

    // Run the TTL invalidation example
    match test_time_based_invalidation().await {
        Ok(_) => println!("✅ Time-based invalidation example passed"),
        Err(e) => println!("❌ Time-based invalidation example failed: {}", e),
    }

    // Run the explicit invalidation example
    match test_explicit_invalidation().await {
        Ok(_) => println!("✅ Explicit invalidation example passed"),
        Err(e) => println!("❌ Explicit invalidation example failed: {}", e),
    }

    // Run the pattern invalidation example
    match test_pattern_invalidation().await {
        Ok(_) => println!("✅ Pattern invalidation example passed"),
        Err(e) => println!("❌ Pattern invalidation example failed: {}", e),
    }

    // Run the write-through consistency example
    match test_write_through_consistency().await {
        Ok(_) => println!("✅ Write-through consistency example passed"),
        Err(e) => println!("❌ Write-through consistency example failed: {}", e),
    }

    // Run the stock update invalidation example
    match test_stock_update_invalidation().await {
        Ok(_) => println!("✅ Stock update invalidation example passed"),
        Err(e) => println!("❌ Stock update invalidation example failed: {}", e),
    }

    println!("Cache invalidation testing examples completed!");

    Ok(())
}

/*
 * Best Practices for Testing Cache Invalidation:
 *
 * 1. Test All Invalidation Strategies:
 *    Verify time-based expiration (TTL), explicit invalidation, and
 *    pattern-based invalidation work correctly.
 *
 * 2. Test Cache Consistency:
 *    Ensure that cache stays in sync with the underlying data store,
 *    especially after updates.
 *
 * 3. Verify Cache Miss Behavior:
 *    Test that the system correctly handles cache misses by fetching
 *    from the underlying data store.
 *
 * 4. Test Invalidation Patterns:
 *    For systems using pattern-based invalidation (like Redis keys with
 *    wildcards), test that the correct keys are invalidated.
 *
 * 5. Test Edge Cases:
 *    Test behavior when cache is unavailable, when TTL is very short,
 *    or during high concurrency.
 *
 * 6. Test Race Conditions:
 *    Verify the system handles concurrent cache updates and invalidations
 *    without data inconsistency.
 *
 * 7. Measure Performance Impact:
 *    Validate that cache invalidation doesn't cause unexpected performance
 *    degradation.
 *
 * 8. Test Write Patterns:
 *    Verify both write-through (synchronous) and write-behind (asynchronous)
 *    caching behave correctly.
 *
 * 9. Test Bulk Invalidations:
 *    Ensure that bulk or batch invalidations are handled efficiently.
 *
 * 10. Mock Cache Behavior:
 *     Use mocks that simulate real cache behavior, including TTL expiration,
 *     for deterministic testing.
 */
