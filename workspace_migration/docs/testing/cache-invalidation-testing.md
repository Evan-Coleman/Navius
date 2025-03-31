# Cache Invalidation Testing Guide

This guide covers best practices for testing cache invalidation using the Navius Test Framework. It complements the code example in `workspace_migration/examples/crates/navius-test/examples/cache_invalidation_testing.rs`.

## Overview

Cache invalidation is one of the two hard problems in computer science (along with naming things and off-by-one errors). Proper testing of cache invalidation strategies ensures that your application maintains consistency between cached data and the source of truth. The Navius Test Framework provides tools to simplify cache invalidation testing without requiring a real cache server.

## Key Cache Invalidation Testing Scenarios

A comprehensive cache invalidation testing strategy should cover these key scenarios:

1. **Time-Based Invalidation**
   - Test that items expire correctly after their TTL (Time To Live)
   - Verify that expired items are removed or refreshed when accessed

2. **Explicit Key Invalidation**
   - Test that individual keys can be explicitly invalidated
   - Verify that invalidated keys are removed from the cache

3. **Pattern-Based Invalidation**
   - Test invalidation of multiple related cache entries using patterns (e.g., wildcards)
   - Verify that all matching keys are properly invalidated

4. **Write-Through and Write-Behind Caching**
   - Test that cache is updated when underlying data changes in write-through scenarios
   - Verify that cache is marked for invalidation in write-behind scenarios

5. **Cache Consistency**
   - Test that cache data remains consistent with the source of truth after operations
   - Verify that concurrent operations don't lead to inconsistent cache states

## Using Mock Cache Components

The Navius Test Framework provides mock cache components that simplify testing:

### MockCacheClient

The `MockCacheClient` allows you to simulate cache operations:

```rust
use navius_test::mocks::cache::MockCacheClient;

let cache_client = MockCacheClient::new();

// Set up expectations for cache operations
cache_client.expect_get("products:123")
    .returns(Ok(Some(r#"{"id":"123","name":"Test Product","price":99.99}"#.to_string())));

cache_client.expect_set("products:123", r#"{"id":"123","name":"Test Product","price":99.99}"#, Some(3600))
    .returns(Ok(()));

cache_client.expect_del("products:123")
    .returns(Ok(true));
```

## Testing Invalidation Strategies

### Time-Based Invalidation

Test that items expire correctly after their TTL:

```rust
#[tokio::test]
async fn test_time_based_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    
    // Set up cache expectations - first request should miss
    cache.expect_get("products:123")
        .returns(Ok(None));
    
    // Set up database query expectation
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Test Product",
        "price": 99.99
    }))));
    
    // Set up cache set expectation with TTL
    cache.expect_set(
        "products:123",
        json!({
            "id": "123",
            "name": "Test Product",
            "price": 99.99
        }).to_string(),
        Some(60) // 60 second TTL
    )
    .returns(Ok(()));
    
    // First fetch should query DB and cache result
    let service = ProductService::new(db.clone(), cache.clone());
    let product1 = service.get_product("123").await?;
    
    assert_eq(product1.name, "Test Product", "Should return product from DB")?;
    
    // Set up expectation for second fetch - cache hit
    cache.expect_get("products:123")
        .returns(Ok(Some(json!({
            "id": "123",
            "name": "Test Product",
            "price": 99.99
        }).to_string())));
    
    // Second fetch should hit cache
    let product2 = service.get_product("123").await?;
    assert_eq(product2.name, "Test Product", "Should return product from cache")?;
    
    // Simulate passage of time - cache item expired
    cache.expect_get("products:123")
        .returns(Ok(None)); // Cache miss due to expiration
    
    // Set up database query expectation again
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Test Product",
        "price": 99.99
    }))));
    
    // Set up cache set expectation with TTL again
    cache.expect_set(
        "products:123",
        json!({
            "id": "123",
            "name": "Test Product",
            "price": 99.99
        }).to_string(),
        Some(60)
    )
    .returns(Ok(()));
    
    // After expiration, should query DB again
    let product3 = service.get_product("123").await?;
    assert_eq(product3.name, "Test Product", "Should return refreshed product from DB")?;
    
    Ok(())
}
```

### Explicit Key Invalidation

Test that individual keys can be explicitly invalidated:

```rust
#[tokio::test]
async fn test_explicit_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    
    // Set up initial state with cached data
    cache.expect_get("products:123")
        .returns(Ok(Some(json!({
            "id": "123",
            "name": "Original Name",
            "price": 99.99
        }).to_string())));
    
    // First fetch should hit cache
    let service = ProductService::new(db.clone(), cache.clone());
    let product1 = service.get_product("123").await?;
    
    assert_eq(product1.name, "Original Name", "Should return product from cache")?;
    
    // Set up update operation in database
    db.expect_query(
        "UPDATE products SET name = $1 WHERE id = $2",
        &["Updated Name", "123"]
    )
    .returns(Ok(QueryResult::with_affected_rows(1)));
    
    // Set up cache invalidation expectation
    cache.expect_del("products:123")
        .returns(Ok(true));
    
    // Perform update and invalidate
    service.update_product_name("123", "Updated Name").await?;
    
    // After invalidation, next get should miss cache
    cache.expect_get("products:123")
        .returns(Ok(None));
    
    // Set up database query for refreshed data
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Updated Name",
        "price": 99.99
    }))));
    
    // Set up cache set expectation for refreshed data
    cache.expect_set(
        "products:123",
        json!({
            "id": "123",
            "name": "Updated Name",
            "price": 99.99
        }).to_string(),
        Some(60)
    )
    .returns(Ok(()));
    
    // After invalidation, should get fresh data
    let product2 = service.get_product("123").await?;
    assert_eq(product2.name, "Updated Name", "Should return updated product from DB")?;
    
    Ok(())
}
```

### Pattern-Based Invalidation

Test invalidation of multiple related cache entries using patterns:

```rust
#[tokio::test]
async fn test_pattern_invalidation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    
    // Set up cache expectations for collection
    cache.expect_get("category:electronics:products")
        .returns(Ok(Some(json!([
            {"id": "123", "name": "Smartphone"},
            {"id": "456", "name": "Laptop"}
        ]).to_string())));
    
    // Fetch the collection from cache
    let service = ProductService::new(db.clone(), cache.clone());
    let products = service.get_products_by_category("electronics").await?;
    
    assert_eq(products.len(), 2, "Should have 2 products from cache")?;
    
    // Add a new product to the category in the database
    db.expect_query(
        "INSERT INTO products (id, name, category) VALUES ($1, $2, $3)",
        &["789", "Tablet", "electronics"]
    )
    .returns(Ok(QueryResult::empty()));
    
    // Set up pattern invalidation using wildcard
    cache.expect_del_pattern("category:electronics:*")
        .returns(Ok(2)); // Invalidated 2 keys
    
    // Create new product (which should invalidate the category cache)
    service.create_product("789", "Tablet", "electronics").await?;
    
    // Next fetch of the collection should miss cache
    cache.expect_get("category:electronics:products")
        .returns(Ok(None));
    
    // Set up database query for all products in category
    db.expect_query(
        "SELECT * FROM products WHERE category = $1",
        &["electronics"]
    )
    .returns(Ok(QueryResult::with_json_array(vec![
        json!({"id": "123", "name": "Smartphone", "category": "electronics"}),
        json!({"id": "456", "name": "Laptop", "category": "electronics"}),
        json!({"id": "789", "name": "Tablet", "category": "electronics"})
    ])));
    
    // Set up cache set expectation for refreshed collection
    cache.expect_set(
        "category:electronics:products",
        json!([
            {"id": "123", "name": "Smartphone", "category": "electronics"},
            {"id": "456", "name": "Laptop", "category": "electronics"},
            {"id": "789", "name": "Tablet", "category": "electronics"}
        ]).to_string(),
        Some(300)
    )
    .returns(Ok(()));
    
    // After invalidation, should get fresh data
    let updated_products = service.get_products_by_category("electronics").await?;
    assert_eq(updated_products.len(), 3, "Should have 3 products including new one")?;
    
    Ok(())
}
```

### Write-Through Caching

Test that the cache is updated when underlying data changes:

```rust
#[tokio::test]
async fn test_write_through_caching() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    
    // Set up database update
    db.expect_query(
        "UPDATE products SET price = $1 WHERE id = $2 RETURNING *",
        &[149.99, "123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Test Product",
        "price": 149.99
    }))));
    
    // Set up cache update (write-through)
    cache.expect_set(
        "products:123",
        json!({
            "id": "123",
            "name": "Test Product",
            "price": 149.99
        }).to_string(),
        Some(60)
    )
    .returns(Ok(()));
    
    // Perform update with write-through caching
    let service = ProductService::new(db.clone(), cache.clone());
    let updated_product = service.update_product_price_write_through("123", 149.99).await?;
    
    assert_eq(updated_product.price, 149.99, "Should return updated product")?;
    
    // Next fetch should hit cache with updated data
    cache.expect_get("products:123")
        .returns(Ok(Some(json!({
            "id": "123",
            "name": "Test Product",
            "price": 149.99
        }).to_string())));
    
    // Should get updated data from cache without DB query
    let product = service.get_product("123").await?;
    assert_eq(product.price, 149.99, "Should return updated price from cache")?;
    
    Ok(())
}
```

### Cache Consistency

Test that cache data remains consistent with the source of truth:

```rust
#[tokio::test]
async fn test_cache_consistency() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    
    // Set up initial state
    cache.expect_get("products:123")
        .returns(Ok(None)); // Cache miss
    
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Original Product",
        "price": 99.99,
        "stock": 10
    }))));
    
    cache.expect_set(
        "products:123",
        json!({
            "id": "123",
            "name": "Original Product",
            "price": 99.99,
            "stock": 10
        }).to_string(),
        Some(60)
    )
    .returns(Ok(()));
    
    // First request fetches from DB and caches
    let service = ProductService::new(db.clone(), cache.clone());
    let product1 = service.get_product("123").await?;
    
    assert_eq(product1.stock, 10, "Should have initial stock level")?;
    
    // Setup stock update operation
    db.expect_query(
        "UPDATE products SET stock = stock - $1 WHERE id = $2 RETURNING stock",
        &[3, "123"]
    )
    .returns(Ok(QueryResult::with_row(vec![("stock", 7)])));
    
    // Setup cache invalidation
    cache.expect_del("products:123")
        .returns(Ok(true));
    
    // Perform stock update operation (which should invalidate cache)
    let new_stock = service.update_stock("123", 3).await?;
    assert_eq(new_stock, 7, "Should return updated stock level")?;
    
    // Next product fetch should miss cache
    cache.expect_get("products:123")
        .returns(Ok(None)); // Cache miss after invalidation
    
    // Set up database query for refreshed data
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Original Product",
        "price": 99.99,
        "stock": 7
    }))));
    
    // Set up cache set expectation for refreshed data
    cache.expect_set(
        "products:123",
        json!({
            "id": "123",
            "name": "Original Product",
            "price": 99.99,
            "stock": 7
        }).to_string(),
        Some(60)
    )
    .returns(Ok(()));
    
    // After invalidation, should get fresh data
    let product2 = service.get_product("123").await?;
    assert_eq(product2.stock, 7, "Should have updated stock level")?;
    
    Ok(())
}
```

## Best Practices

### 1. Test Cache Miss Behavior

Ensure your code handles cache misses gracefully by testing scenarios where items are not in the cache or have expired.

### 2. Test All Invalidation Strategies

Test each invalidation strategy your application uses, including time-based expiration, explicit invalidation, and pattern-based invalidation.

### 3. Verify Cache Consistency

Verify that your cache remains consistent with the underlying data source after updates, especially in concurrent operations.

### 4. Test TTL Behavior

Test that cache items expire correctly after their Time To Live (TTL) has elapsed and that your code handles the expiration gracefully.

### 5. Test Pattern-Based Invalidation Carefully

When using pattern-based invalidation, test that the patterns match exactly what you expect them to match and nothing more.

### 6. Simulate Failure Scenarios

Test how your cache handling code behaves when the cache service is slow or unavailable.

### 7. Use Helper Functions

Create helper functions to set up common cache patterns in your tests:

```rust
async fn setup_cached_product(
    cache: &MockCacheClient, 
    id: &str, 
    name: &str,
    price: f64
) -> TestResult<()> {
    cache.expect_get(format!("products:{}", id))
        .returns(Ok(Some(json!({
            "id": id,
            "name": name,
            "price": price
        }).to_string())));
    
    Ok(())
}

async fn expect_cache_invalidation(
    cache: &MockCacheClient,
    key: &str
) -> TestResult<()> {
    cache.expect_del(key)
        .returns(Ok(true));
    
    Ok(())
}
```

## Common Pitfalls

1. **Cache Stampede**: Not testing how your code handles cache stampedes (multiple simultaneous requests for the same uncached item).

2. **Testing Only Happy Paths**: Failing to test cache misses, expiration, and error scenarios.

3. **Ignoring Timing Issues**: Not properly simulating the passage of time for TTL-based expiration.

4. **Overlooking Partial Invalidation**: Not testing scenarios where only some related cache entries are invalidated.

5. **Forgetting Consistency Checks**: Not verifying that the cache remains consistent with the data source after updates.

## Advanced Cache Testing Scenarios

### Cache Stampede Prevention

Test that your code prevents cache stampedes using techniques like distributed locking:

```rust
#[tokio::test]
async fn test_cache_stampede_prevention() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    let lock_manager = MockLockManager::new();
    
    // Set up cache miss
    cache.expect_get("products:high_demand_item")
        .returns(Ok(None));
    
    // Set up lock acquisition
    lock_manager.expect_acquire_lock("cache:lock:products:high_demand_item", 10000)
        .returns(Ok("lock1"));
    
    // Set up database query (should happen only once)
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["high_demand_item"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "high_demand_item",
        "name": "High Demand Product",
        "price": 299.99
    }))));
    
    // Set up cache set
    cache.expect_set(
        "products:high_demand_item",
        json!({
            "id": "high_demand_item",
            "name": "High Demand Product",
            "price": 299.99
        }).to_string(),
        Some(60)
    )
    .returns(Ok(()));
    
    // Set up lock release
    lock_manager.expect_release_lock("cache:lock:products:high_demand_item", "lock1")
        .returns(Ok(()));
    
    // Create service with lock manager
    let service = ProductService::new_with_lock_manager(
        db.clone(), 
        cache.clone(),
        lock_manager.clone()
    );
    
    // Simulate multiple concurrent requests
    let futures = vec![
        service.get_product_with_lock_prevention("high_demand_item"),
        service.get_product_with_lock_prevention("high_demand_item"),
        service.get_product_with_lock_prevention("high_demand_item")
    ];
    
    // All should resolve to the same product
    let results = futures::future::join_all(futures).await;
    for result in results {
        let product = result?;
        assert_eq(product.name, "High Demand Product", "All requests should get the product")?;
    }
    
    // Verify that the DB was only queried once through the lock mechanism
    
    Ok(())
}
```

### Stale-While-Revalidate Pattern

Test the pattern where stale content is served while fresh content is being fetched:

```rust
#[tokio::test]
async fn test_stale_while_revalidate() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create mock components
    let cache = MockCacheClient::new();
    let db = MockDatabaseClient::new();
    
    // Initial state - cached item with metadata
    cache.expect_get("products:123:data")
        .returns(Ok(Some(json!({
            "id": "123",
            "name": "Original Product",
            "price": 99.99,
            "timestamp": "2025-03-25T10:00:00Z"
        }).to_string())));
    
    cache.expect_get("products:123:metadata")
        .returns(Ok(Some(json!({
            "last_updated": "2025-03-25T10:00:00Z",
            "ttl": 3600,
            "grace_period": 300
        }).to_string())));
    
    // Service checks that the item is stale (TTL exceeded but within grace period)
    // and should return stale data while revalidating
    
    // Expect background refresh query
    db.expect_query(
        "SELECT * FROM products WHERE id = $1",
        &["123"]
    )
    .returns(Ok(QueryResult::with_json(json!({
        "id": "123",
        "name": "Updated Product",
        "price": 109.99
    }))));
    
    // Expect cache update with new data
    cache.expect_set(
        "products:123:data",
        json!({
            "id": "123",
            "name": "Updated Product",
            "price": 109.99,
            "timestamp": mockito::matches_pattern(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")
        }).to_string(),
        Some(3600)
    )
    .returns(Ok(()));
    
    // Expect metadata update
    cache.expect_set(
        "products:123:metadata",
        mockito::any_string(), // Don't need to check exact contents
        Some(3900) // TTL + grace period
    )
    .returns(Ok(()));
    
    // Create service with stale-while-revalidate capability
    let service = ProductService::new_with_swr(db.clone(), cache.clone());
    
    // Should return stale data immediately while triggering background refresh
    let result = service.get_product_swr("123").await?;
    
    // First request should return stale data
    assert_eq(result.name, "Original Product", "Should return stale data initially")?;
    
    // Wait for background refresh to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Next request should get fresh data
    cache.expect_get("products:123:data")
        .returns(Ok(Some(json!({
            "id": "123",
            "name": "Updated Product",
            "price": 109.99,
            "timestamp": mockito::matches_pattern(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")
        }).to_string())));
    
    cache.expect_get("products:123:metadata")
        .returns(Ok(Some(json!({
            "last_updated": mockito::matches_pattern(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z"),
            "ttl": 3600,
            "grace_period": 300
        }).to_string())));
    
    let updated_result = service.get_product_swr("123").await?;
    assert_eq(updated_result.name, "Updated Product", "Should return fresh data after background refresh")?;
    
    Ok(())
}
```

## Integration with Real Cache Servers

For end-to-end tests, you may need to test with real cache servers. The Navius Test Framework supports this through integration fixtures:

```rust
#[tokio::test]
async fn test_real_cache_integration() -> TestResult<()> {
    // Create integration test context
    let ctx = IntegrationContext::new()?;
    
    // Get a real cache client and database connection
    let cache = ctx.get_cache().await?;
    let db = ctx.get_database("test_db").await?;
    
    // Clear existing data
    cache.flush_all().await?;
    db.execute("DELETE FROM test_products").await?;
    
    // Insert test data
    db.execute(
        "INSERT INTO test_products (id, name, price) VALUES ($1, $2, $3)",
        &["test1", "Test Product", 99.99]
    ).await?;
    
    // Create service with real components
    let service = ProductService::new_real(db.clone(), cache.clone());
    
    // First fetch should cache the product
    let product1 = service.get_product("test1").await?;
    assert_eq(product1.name, "Test Product", "Should return product from DB")?;
    
    // Update product in database directly
    db.execute(
        "UPDATE test_products SET name = $1 WHERE id = $2",
        &["Updated Product", "test1"]
    ).await?;
    
    // Without invalidation, should still get old data from cache
    let product2 = service.get_product("test1").await?;
    assert_eq(product2.name, "Test Product", "Should return cached product")?;
    
    // Explicitly invalidate the cache
    service.invalidate_product("test1").await?;
    
    // After invalidation, should get updated data
    let product3 = service.get_product("test1").await?;
    assert_eq(product3.name, "Updated Product", "Should return updated product from DB")?;
    
    // Clean up
    cache.flush_all().await?;
    db.execute("DELETE FROM test_products").await?;
    
    Ok(())
}
```

## Conclusion

Effective cache invalidation testing is crucial for ensuring data consistency in your application. By using the Navius Test Framework's cache testing utilities, you can thoroughly test cache invalidation strategies without requiring a real cache server in most cases, making your tests faster and more reliable.

For a complete code example, refer to `workspace_migration/examples/crates/navius-test/examples/cache_invalidation_testing.rs`.

## Additional Resources

- [Cache-Aside Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/cache-aside)
- [Stale-While-Revalidate Pattern](https://web.dev/stale-while-revalidate/)
- [Cache Stampede Prevention Techniques](https://en.wikipedia.org/wiki/Cache_stampede) 