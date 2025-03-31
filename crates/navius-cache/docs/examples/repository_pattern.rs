//! Repository pattern integration example for navius-cache
//!
//! This example demonstrates how to integrate the cache with a repository pattern.
//! To run:
//! ```bash
//! cargo run --example repository_pattern --features redis
//! ```

use async_trait::async_trait;
use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// Entity model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
    role: String,
}

// Repository trait
#[async_trait]
trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> Result<Option<User>, String>;
    async fn find_all(&self) -> Result<Vec<User>, String>;
    async fn save(&self, user: User) -> Result<User, String>;
    async fn delete(&self, id: u64) -> Result<bool, String>;
}

// In-memory implementation (simulating a database)
struct InMemoryUserRepository {
    data: Arc<RwLock<HashMap<u64, User>>>,
}

impl InMemoryUserRepository {
    fn new() -> Self {
        // Populate with some initial data
        let mut data = HashMap::new();
        data.insert(
            1,
            User {
                id: 1,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                role: "user".to_string(),
            },
        );
        data.insert(
            2,
            User {
                id: 2,
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
                role: "admin".to_string(),
            },
        );

        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: u64) -> Result<Option<User>, String> {
        // Simulate database latency
        tokio::time::sleep(Duration::from_millis(50)).await;

        let data = self.data.read().unwrap();
        Ok(data.get(&id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<User>, String> {
        // Simulate database latency
        tokio::time::sleep(Duration::from_millis(100)).await;

        let data = self.data.read().unwrap();
        Ok(data.values().cloned().collect())
    }

    async fn save(&self, user: User) -> Result<User, String> {
        // Simulate database latency
        tokio::time::sleep(Duration::from_millis(75)).await;

        let mut data = self.data.write().unwrap();
        data.insert(user.id, user.clone());
        Ok(user)
    }

    async fn delete(&self, id: u64) -> Result<bool, String> {
        // Simulate database latency
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut data = self.data.write().unwrap();
        Ok(data.remove(&id).is_some())
    }
}

// Cached repository implementation
struct CachedUserRepository {
    repository: Box<dyn UserRepository>,
    cache: CacheConnectionManager,
    ttl: Duration,
}

impl CachedUserRepository {
    fn new(
        repository: Box<dyn UserRepository>,
        cache: CacheConnectionManager,
        ttl: Duration,
    ) -> Self {
        Self {
            repository,
            cache,
            ttl,
        }
    }

    // Helper to build the cache key for a user
    fn user_key(&self, id: u64) -> String {
        format!("user:{}", id)
    }

    // Helper to build the cache key for all users
    fn all_users_key(&self) -> String {
        "users:all".to_string()
    }
}

#[async_trait]
impl UserRepository for CachedUserRepository {
    async fn find_by_id(&self, id: u64) -> Result<Option<User>, String> {
        let key = self.user_key(id);

        // Try to get from cache first
        match self.cache.get::<_, User>(&key).await {
            Ok(Some(user)) => {
                println!("Cache HIT for {}", key);
                return Ok(Some(user));
            }
            Ok(None) => {
                println!("Cache MISS for {}", key);
                // Not in cache, get from repository
                match self.repository.find_by_id(id).await {
                    Ok(Some(user)) => {
                        // Store in cache for future requests
                        let options = CacheOptions::new().ttl(self.ttl);
                        if let Err(e) = self.cache.set(&key, &user, Some(options)).await {
                            println!("Warning: Failed to cache user {}: {}", id, e);
                        }
                        Ok(Some(user))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            Err(e) => {
                println!("Cache error for {}: {}", key, e);
                // Cache error, fallback to repository
                self.repository.find_by_id(id).await
            }
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, String> {
        let key = self.all_users_key();

        // Try to get from cache first
        match self.cache.get::<_, Vec<User>>(&key).await {
            Ok(Some(users)) => {
                println!("Cache HIT for {}", key);
                return Ok(users);
            }
            Ok(None) => {
                println!("Cache MISS for {}", key);
                // Not in cache, get from repository
                match self.repository.find_all().await {
                    Ok(users) => {
                        // Store in cache for future requests
                        let options = CacheOptions::new().ttl(self.ttl);
                        if let Err(e) = self.cache.set(&key, &users, Some(options)).await {
                            println!("Warning: Failed to cache all users: {}", e);
                        }
                        Ok(users)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => {
                println!("Cache error for {}: {}", key, e);
                // Cache error, fallback to repository
                self.repository.find_all().await
            }
        }
    }

    async fn save(&self, user: User) -> Result<User, String> {
        // Update the database first
        let result = self.repository.save(user.clone()).await?;

        // Then invalidate cache
        let user_key = self.user_key(user.id);
        let all_users_key = self.all_users_key();

        // We don't want to fail if cache invalidation fails, just log it
        if let Err(e) = self.cache.delete(&user_key).await {
            println!("Warning: Failed to invalidate user cache: {}", e);
        }

        if let Err(e) = self.cache.delete(&all_users_key).await {
            println!("Warning: Failed to invalidate all users cache: {}", e);
        }

        Ok(result)
    }

    async fn delete(&self, id: u64) -> Result<bool, String> {
        // Delete from the database first
        let result = self.repository.delete(id).await?;

        // Then invalidate cache
        let user_key = self.user_key(id);
        let all_users_key = self.all_users_key();

        // We don't want to fail if cache invalidation fails, just log it
        if let Err(e) = self.cache.delete(&user_key).await {
            println!("Warning: Failed to invalidate user cache: {}", e);
        }

        if let Err(e) = self.cache.delete(&all_users_key).await {
            println!("Warning: Failed to invalidate all users cache: {}", e);
        }

        Ok(result)
    }
}

// Service layer to demonstrate usage
struct UserService {
    repository: Box<dyn UserRepository>,
}

impl UserService {
    fn new(repository: Box<dyn UserRepository>) -> Self {
        Self { repository }
    }

    async fn get_user(&self, id: u64) -> Result<Option<User>, String> {
        self.repository.find_by_id(id).await
    }

    async fn get_all_users(&self) -> Result<Vec<User>, String> {
        self.repository.find_all().await
    }

    async fn create_user(&self, user: User) -> Result<User, String> {
        self.repository.save(user).await
    }

    async fn delete_user(&self, id: u64) -> Result<bool, String> {
        self.repository.delete(id).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Navius Cache: Repository Pattern Example");
    println!("=========================================");

    // Create cache configuration
    let config = CacheConfig::new(
        "redis://127.0.0.1:6379".to_string(),
        "repo-example:".to_string(),
        Duration::from_secs(300), // 5 minutes default TTL
    );

    println!("Connecting to Redis...");

    // Connect to Redis
    let cache = match CacheConnectionManager::new_redis(config.clone()).await {
        Ok(cache) => {
            println!("Successfully connected to Redis");
            cache
        }
        Err(e) => {
            println!("Failed to connect to Redis: {}", e);
            println!("This example requires a running Redis instance.");
            return Ok(());
        }
    };

    // Clear any existing data
    println!("Clearing any previous data...");
    cache.clear().await?;

    // Create base repository (simulates database)
    let base_repository = Box::new(InMemoryUserRepository::new());

    // Create cached repository with 1 minute TTL
    let cached_repository = Box::new(CachedUserRepository::new(
        base_repository,
        cache,
        Duration::from_secs(60),
    ));

    // Create user service with cached repository
    let user_service = UserService::new(cached_repository);

    // Demonstrate cache pattern
    println!("\n1. First request for all users (cache miss):");
    let users = user_service.get_all_users().await?;
    println!("Found {} users", users.len());

    println!("\n2. Second request for all users (cache hit):");
    let _users = user_service.get_all_users().await?;

    println!("\n3. First request for user 1 (cache miss):");
    let user = user_service.get_user(1).await?;
    println!("Found user: {:?}", user);

    println!("\n4. Second request for user 1 (cache hit):");
    let _user = user_service.get_user(1).await?;

    println!("\n5. Create a new user (invalidates caches):");
    let new_user = User {
        id: 3,
        name: "Bob Johnson".to_string(),
        email: "bob@example.com".to_string(),
        role: "user".to_string(),
    };
    user_service.create_user(new_user.clone()).await?;
    println!("Created user: {:?}", new_user);

    println!("\n6. Request for all users after update (cache miss due to invalidation):");
    let users = user_service.get_all_users().await?;
    println!("Found {} users (including new user)", users.len());

    println!("\n7. Delete a user (invalidates caches):");
    let deleted = user_service.delete_user(2).await?;
    println!("User deleted: {}", deleted);

    println!("\n8. Request for all users after deletion (cache miss due to invalidation):");
    let users = user_service.get_all_users().await?;
    println!("Found {} users (after deletion)", users.len());

    println!("\nExample completed successfully!");
    Ok(())
}
