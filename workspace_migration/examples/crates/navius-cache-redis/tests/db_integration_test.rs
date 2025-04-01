use std::sync::Arc;
use std::time::Duration;

use navius_cache_redis::{RedisCache, RedisCacheConfig, RedisCacheError};
use navius_db::provider::DatabaseProvider;
use navius_db_postgres::provider::{PostgresProvider, PostgresProviderOptions};
use sqlx::postgres::PgConnectOptions;
use tokio::time;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct TestEntity {
    id: String,
    name: String,
    data: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TestEntity {
    fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            data: format!("Test data for {}", name),
            created_at: Some(chrono::Utc::now()),
        }
    }
}

async fn setup_test_db_pool() -> Result<PostgresProvider, Box<dyn std::error::Error>> {
    // Database connection options (using a test database)
    let connect_options = PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .username("postgres")
        .password("postgres")
        .database("navius_test");
    
    // Create provider options with specific settings
    let options = PostgresProviderOptions::new(connect_options)
        .with_max_connections(5)
        .with_min_connections(1)
        .with_max_lifetime(Some(Duration::from_secs(1800)))
        .with_idle_timeout(Some(Duration::from_secs(600)));
    
    // Create the provider
    let provider = PostgresProvider::new(options).await?;
    
    // Prepare test table if needed
    let mut conn = provider.pool.acquire().await?;
    let create_table_query = r#"
    CREATE TABLE IF NOT EXISTS test_entities (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        data TEXT NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE
    )
    "#;
    sqlx::query(create_table_query).execute(&mut conn).await?;
    
    Ok(provider)
}

async fn setup_test_redis_cache() -> Result<RedisCache, RedisCacheError> {
    // Configure Redis connection
    let config = RedisCacheConfig::builder()
        .with_url("redis://localhost:6379")
        .with_connection_timeout(Duration::from_secs(5))
        .with_pool_size(5)
        .build();

    let cache = RedisCache::new(config).await?;

    // Flush the database before starting tests
    let mut conn = cache.connection().await?;
    let _: () = redis::cmd("FLUSHDB")
        .query_async(&mut conn)
        .await
        .map_err(|e| RedisCacheError::CommandError(format!("Failed to flush database: {}", e)))?;

    Ok(cache)
}

// Trait extension for simplicity
trait PostgresProviderExt {
    async fn save_entity(&self, entity: &TestEntity) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_entity(&self, id: &str) -> Result<TestEntity, Box<dyn std::error::Error>>;
    async fn update_entity(&self, entity: &TestEntity) -> Result<(), Box<dyn std::error::Error>>;
    async fn delete_entity(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;
}

impl PostgresProviderExt for PostgresProvider {
    async fn save_entity(&self, entity: &TestEntity) -> Result<(), Box<dyn std::error::Error>> {
        let query = "INSERT INTO test_entities (id, name, data, created_at) VALUES ($1, $2, $3, $4)";
        sqlx::query(query)
            .bind(&entity.id)
            .bind(&entity.name)
            .bind(&entity.data)
            .bind(&entity.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn get_entity(&self, id: &str) -> Result<TestEntity, Box<dyn std::error::Error>> {
        let query = "SELECT id, name, data, created_at FROM test_entities WHERE id = $1";
        let entity = sqlx::query_as::<_, TestEntity>(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(entity)
    }
    
    async fn update_entity(&self, entity: &TestEntity) -> Result<(), Box<dyn std::error::Error>> {
        let query = "UPDATE test_entities SET name = $2, data = $3, created_at = $4 WHERE id = $1";
        sqlx::query(query)
            .bind(&entity.id)
            .bind(&entity.name)
            .bind(&entity.data)
            .bind(&entity.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn delete_entity(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let query = "DELETE FROM test_entities WHERE id = $1";
        sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// We'll mark this test as ignored by default since it requires both Redis and PostgreSQL to be running
#[tokio::test]
#[ignore]
async fn test_redis_cache_with_db_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test environment
    let db_pool = setup_test_db_pool().await?;
    let cache = setup_test_redis_cache().await?;
    
    // Create test entity
    let test_entity = TestEntity::new("test-integration");
    let entity_id = test_entity.id.clone();
    
    // Save to database
    db_pool.save_entity(&test_entity).await?;
    
    // Cache the entity
    cache.set(&format!("entity:{}", entity_id), &test_entity, None).await?;
    
    // Verify retrieval from cache
    let cached_entity: TestEntity = cache.get(&format!("entity:{}", entity_id)).await?;
    assert_eq!(cached_entity.id, test_entity.id);
    assert_eq!(cached_entity.name, test_entity.name);
    
    // Update entity in database
    let mut updated_entity = test_entity.clone();
    updated_entity.name = "updated-name".to_string();
    updated_entity.data = "Updated test data".to_string();
    
    db_pool.update_entity(&updated_entity).await?;
    
    // Verify cache still has old version
    let cached_entity: TestEntity = cache.get(&format!("entity:{}", entity_id)).await?;
    assert_eq!(cached_entity.name, test_entity.name); // Still has original name
    
    // Invalidate cache
    cache.delete(&format!("entity:{}", entity_id)).await?;
    
    // Verify cache no longer has the entity
    let result = cache.get::<TestEntity, _>(&format!("entity:{}", entity_id)).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(RedisCacheError::KeyNotFound(_))));
    
    // Retrieve from database and verify it has updated version
    let db_entity = db_pool.get_entity(&entity_id).await?;
    assert_eq!(db_entity.name, updated_entity.name);
    assert_eq!(db_entity.data, updated_entity.data);
    
    // Clean up
    db_pool.delete_entity(&entity_id).await?;
    
    Ok(())
}

// Test for batch operations integration
#[tokio::test]
#[ignore]
async fn test_batch_operations_with_db() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test environment
    let db_pool = setup_test_db_pool().await?;
    let cache = setup_test_redis_cache().await?;
    
    // Create multiple test entities
    let entities: Vec<TestEntity> = (0..10)
        .map(|i| TestEntity::new(&format!("batch-entity-{}", i)))
        .collect();
    
    // Save all entities to database
    for entity in &entities {
        db_pool.save_entity(entity).await?;
    }
    
    // Use pipeline to cache all entities
    let mut pipeline = cache.pipeline();
    for entity in &entities {
        pipeline.set(&format!("entity:{}", entity.id), entity);
    }
    pipeline.execute().await?;
    
    // Verify all entities are in cache
    for entity in &entities {
        let cached_entity: TestEntity = cache.get(&format!("entity:{}", entity.id)).await?;
        assert_eq!(cached_entity.id, entity.id);
    }
    
    // Update multiple entities
    let updated_entities: Vec<TestEntity> = entities
        .iter()
        .enumerate()
        .map(|(i, entity)| {
            let mut updated = entity.clone();
            updated.name = format!("updated-batch-entity-{}", i);
            updated
        })
        .collect();
    
    // Update all entities in database
    for entity in &updated_entities {
        db_pool.update_entity(entity).await?;
    }
    
    // Batch invalidate cache
    let mut pipeline = cache.pipeline();
    for entity in &entities {
        pipeline.delete(&format!("entity:{}", entity.id));
    }
    pipeline.execute().await?;
    
    // Verify entities are removed from cache
    for entity in &entities {
        let result = cache.get::<TestEntity, _>(&format!("entity:{}", entity.id)).await;
        assert!(result.is_err());
    }
    
    // Clean up
    for entity in &entities {
        db_pool.delete_entity(&entity.id).await?;
    }
    
    Ok(())
}

// Performance test for pipeline operations
#[tokio::test]
#[ignore]
async fn test_cache_performance() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test environment
    let cache = setup_test_redis_cache().await?;
    
    // Create large test dataset
    let entity_count = 1000;
    let entities: Vec<TestEntity> = (0..entity_count)
        .map(|i| TestEntity::new(&format!("perf-entity-{}", i)))
        .collect();
    
    // Measure time for individual operations
    let start = std::time::Instant::now();
    for entity in &entities[0..10] {
        cache.set(&format!("entity:{}", entity.id), entity, None).await?;
    }
    let individual_time = start.elapsed();
    
    // Measure time for pipeline operations
    let start = std::time::Instant::now();
    let mut pipeline = cache.pipeline();
    for entity in &entities {
        pipeline.set(&format!("entity:{}", entity.id), entity);
    }
    pipeline.execute().await?;
    let pipeline_time = start.elapsed();
    
    println!("Time for 10 individual operations: {:?}", individual_time);
    println!("Time for {} pipeline operations: {:?}", entity_count, pipeline_time);
    println!("Pipeline performance improvement factor: {:.2}x", 
        (individual_time.as_micros() as f64 / 10.0) / 
        (pipeline_time.as_micros() as f64 / entity_count as f64));
    
    // Cleanup
    let mut pipeline = cache.pipeline();
    for entity in &entities {
        pipeline.delete(&format!("entity:{}", entity.id));
    }
    pipeline.execute().await?;
    
    Ok(())
} 