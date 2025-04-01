use futures::future::join_all;
use navius_cache_redis::{CacheOptions, RedisCache, RedisError, RedisHash};
use navius_db_postgres::{PostgresProvider, ProviderError};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::cell::RefCell;
use std::time::Duration;
use std::time::Instant;
use tempfile::TempDir;
use tokio::time::sleep;
use tracing::{error, info};

// Thread-local storage for UserKey string conversion
thread_local! {
    static USER_KEY_BUFFER: RefCell<String> = RefCell::new(String::new());
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct User {
    id: i64,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct UserKey(i64);

impl AsRef<str> for UserKey {
    fn as_ref(&self) -> &str {
        USER_KEY_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buffer.clear();
            buffer.push_str(&format!("user:{}", self.0));
            // Safety: We're returning a reference to the thread-local buffer
            // which is valid for the duration of this function call
            unsafe { std::mem::transmute(buffer.as_str()) }
        })
    }
}

impl ToString for UserKey {
    fn to_string(&self) -> String {
        format!("user:{}", self.0)
    }
}

async fn setup_database() -> Result<(PgPool, TempDir), Box<dyn std::error::Error>> {
    // Create temporary directory for migrations
    let temp_dir = TempDir::new()?;
    let migration_path = temp_dir.path().to_str().unwrap();

    // Create SQL migration files
    std::fs::write(
        format!("{}/V1__create_users_table.sql", migration_path),
        r#"
        CREATE TABLE users (
            id BIGSERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )?;

    // Initialize database connection
    let database_url = "postgres://postgres:postgres@localhost:5432/postgres";
    let pool = PgPool::connect(database_url).await?;

    // Run migrations
    sqlx::migrate::Migrator::new(std::path::Path::new(migration_path))
        .await?
        .run(&pool)
        .await?;

    Ok((pool, temp_dir))
}

async fn get_or_create_user(
    db: &PostgresProvider,
    cache: &RedisCache,
    user_id: i64,
    name: &str,
    email: &str,
) -> Result<User, Box<dyn std::error::Error>> {
    let key = UserKey(user_id);
    
    // Try to get from cache first
    if let Ok(Some(user)) = cache.get::<_, User>(&key).await {
        return Ok(user);
    }
    
    // Not in cache, check database
    let user = match db
        .execute_query_one::<User, _>(
            "SELECT id, name, email, created_at FROM users WHERE id = $1",
            &[&user_id],
        )
        .await
    {
        Ok(user) => user,
        Err(ProviderError::NotFound) => {
            // Create new user
            let now = chrono::Utc::now();
            let user = User {
                id: user_id,
                name: name.to_string(),
                email: email.to_string(),
                created_at: now,
            };
            
            // Insert into database
            db.execute_query(
                "INSERT INTO users (id, name, email, created_at) VALUES ($1, $2, $3, $4)",
                &[&user.id, &user.name, &user.email, &user.created_at],
            )
            .await?;
            
            user
        },
        Err(e) => return Err(Box::new(e)),
    };
    
    // Store in cache with TTL
    let ttl = Duration::from_secs(60); // 1 minute cache
    cache
        .set(&key, &user, Some(CacheOptions::new().with_ttl(ttl)))
        .await?;
    
    Ok(user)
}

async fn update_user_name(
    db: &PostgresProvider,
    cache: &RedisCache,
    user_id: i64,
    new_name: &str,
) -> Result<User, Box<dyn std::error::Error>> {
    let key = UserKey(user_id);
    
    // Update in database
    db.execute_query(
        "UPDATE users SET name = $1 WHERE id = $2",
        &[&new_name, &user_id],
    )
    .await?;
    
    // Get updated user
    let user = db
        .execute_query_one::<User, _>(
            "SELECT id, name, email, created_at FROM users WHERE id = $1",
            &[&user_id],
        )
        .await?;
    
    // Update cache
    cache.set(&key, &user, None).await?;
    
    Ok(user)
}

async fn delete_user(
    db: &PostgresProvider,
    cache: &RedisCache,
    user_id: i64,
) -> Result<bool, Box<dyn std::error::Error>> {
    let key = UserKey(user_id);
    
    // Delete from database
    let result = db
        .execute_query("DELETE FROM users WHERE id = $1", &[&user_id])
        .await?;
    
    // Delete from cache
    cache.delete(&key).await?;
    
    Ok(result > 0)
}

#[tokio::test]
async fn test_redis_cache_with_postgres_db() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Setup Redis cache
    let redis_url = "redis://localhost:6379/0";
    let cache = RedisCache::new(redis_url).await?;
    
    // Setup database
    let (pool, _temp_dir) = setup_database().await?;
    let db = PostgresProvider::new(pool);
    
    // Test create and retrieve user
    let user_id = 1;
    let user = get_or_create_user(&db, &cache, user_id, "John Doe", "john@example.com").await?;
    assert_eq!(user.id, user_id);
    assert_eq!(user.name, "John Doe");
    
    // Test cache hit
    let start = Instant::now();
    let cached_user = get_or_create_user(&db, &cache, user_id, "Should not matter", "should@not.matter").await?;
    let cache_duration = start.elapsed();
    assert_eq!(cached_user, user); // Should be the same user
    info!("Cache hit duration: {:?}", cache_duration);
    
    // Test update
    let updated_user = update_user_name(&db, &cache, user_id, "Jane Doe").await?;
    assert_eq!(updated_user.id, user_id);
    assert_eq!(updated_user.name, "Jane Doe");
    
    // Verify update propagated to cache
    let cached_updated_user = cache.get::<_, User>(&UserKey(user_id)).await?;
    assert!(cached_updated_user.is_some());
    assert_eq!(cached_updated_user.unwrap().name, "Jane Doe");
    
    // Test cache expiration
    let expiring_key = UserKey(999);
    let expiring_user = User {
        id: 999,
        name: "Expiring User".to_string(),
        email: "expiring@example.com".to_string(),
        created_at: chrono::Utc::now(),
    };
    
    // Set with very short TTL
    cache
        .set(
            &expiring_key,
            &expiring_user,
            Some(CacheOptions::new().with_ttl(Duration::from_millis(100))),
        )
        .await?;
    
    // Verify it's in cache
    let cached = cache.get::<_, User>(&expiring_key).await?;
    assert!(cached.is_some());
    
    // Wait for expiration
    sleep(Duration::from_millis(150)).await;
    
    // Verify it's gone
    let cached_after_expiry = cache.get::<_, User>(&expiring_key).await?;
    assert!(cached_after_expiry.is_none());
    
    // Test delete
    let deleted = delete_user(&db, &cache, user_id).await?;
    assert!(deleted);
    
    // Verify deleted from cache
    let cached_after_delete = cache.get::<_, User>(&UserKey(user_id)).await?;
    assert!(cached_after_delete.is_none());
    
    // Test concurrent access
    let mut handles = vec![];
    for i in 100..110 {
        let db_clone = db.clone();
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let email = format!("user{}@example.com", i);
            let name = format!("User {}", i);
            get_or_create_user(&db_clone, &cache_clone, i, &name, &email).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let results = join_all(handles).await;
    for result in results {
        let user = result??;
        info!("Created user: {:?}", user);
    }
    
    // Test hash operations with database
    let hash_key = "users:metadata";
    
    // Store user counts in hash
    let count: i64 = db
        .execute_query_one_scalar("SELECT COUNT(*) FROM users", &[])
        .await?;
    
    cache.hset(hash_key, "total_users", count).await?;
    cache.hset(hash_key, "last_updated", chrono::Utc::now().to_string()).await?;
    
    // Retrieve from hash
    let stored_count: i64 = cache.hget(hash_key, "total_users").await?;
    assert_eq!(stored_count, count);
    
    // Clean up
    cache.delete(hash_key).await?;
    
    Ok(())
} 