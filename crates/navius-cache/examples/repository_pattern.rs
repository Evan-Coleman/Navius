//! Repository pattern example for navius-cache
//!
//! This example demonstrates using the cache with a repository pattern.
//! To run:
//! ```bash
//! cargo run --example repository_pattern
//! ```

use navius_cache::{
    CacheConfig, CacheOptions, MemoryCache,
    invalidation::{CacheInvalidation, CacheInvalidator, InvalidationStrategy},
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

// Define our domain entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: i32,
    name: String,
    email: String,
    active: bool,
}

// Mock database for this example
struct Database {
    users: Vec<User>,
}

impl Database {
    fn new() -> Self {
        // Pre-populate with some users
        let users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                active: true,
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                active: true,
            },
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
                active: false,
            },
        ];

        Self { users }
    }

    async fn get_user(&self, id: i32) -> Option<User> {
        // Simulate database query latency
        tokio::time::sleep(Duration::from_millis(50)).await;

        self.users
            .iter()
            .find(|user| user.id == id)
            .map(|user| user.clone())
    }

    async fn get_users_by_status(&self, active: bool) -> Vec<User> {
        // Simulate database query latency
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.users
            .iter()
            .filter(|user| user.active == active)
            .cloned()
            .collect()
    }

    async fn update_user(&mut self, updated_user: User) -> Option<User> {
        // Simulate database query latency
        tokio::time::sleep(Duration::from_millis(150)).await;

        if let Some(user) = self.users.iter_mut().find(|u| u.id == updated_user.id) {
            let old_user = user.clone();
            *user = updated_user;
            Some(old_user)
        } else {
            None
        }
    }

    async fn delete_user(&mut self, id: i32) -> bool {
        // Simulate database query latency
        tokio::time::sleep(Duration::from_millis(100)).await;

        let initial_len = self.users.len();
        self.users.retain(|user| user.id != id);
        self.users.len() < initial_len
    }
}

// Define our repository interface
#[async_trait::async_trait]
trait UserRepository: Send + Sync {
    async fn get_user(&self, id: i32) -> Option<User>;
    async fn get_users_by_status(&self, active: bool) -> Vec<User>;
    async fn update_user(&self, user: User) -> Option<User>;
    async fn delete_user(&self, id: i32) -> bool;
}

// Implementation using cache + database
struct CachedUserRepository {
    db: Arc<Mutex<Database>>,
    cache: MemoryCache,
    invalidator: CacheInvalidator<MemoryCache, String>,
}

impl CachedUserRepository {
    fn new(db: Arc<Mutex<Database>>, cache: MemoryCache) -> Self {
        // Create a cache invalidator with immediate invalidation strategy
        let invalidator = CacheInvalidator::new(cache.clone(), InvalidationStrategy::Immediate);

        Self {
            db,
            cache,
            invalidator,
        }
    }

    // Helper methods to generate cache keys
    fn user_key(id: i32) -> String {
        format!("user:{}", id)
    }

    fn user_status_key(active: bool) -> String {
        format!("users:status:{}", active)
    }
}

#[async_trait::async_trait]
impl UserRepository for CachedUserRepository {
    async fn get_user(&self, id: i32) -> Option<User> {
        let cache_key = Self::user_key(id);

        // Try to get from cache first
        if let Ok(Some(user)) = self.cache.get::<_, User>(&cache_key).await {
            println!("Cache HIT for user:{}", id);
            return Some(user);
        }

        println!("Cache MISS for user:{}", id);

        // If not in cache, get from database
        let db = self.db.lock().await;
        if let Some(user) = db.get_user(id).await {
            // Store in cache with TTL of 5 minutes
            let options = CacheOptions::new().ttl(Duration::from_secs(300));
            let _ = self.cache.set(&cache_key, &user, Some(options)).await;

            Some(user)
        } else {
            None
        }
    }

    async fn get_users_by_status(&self, active: bool) -> Vec<User> {
        let cache_key = Self::user_status_key(active);

        // Try to get from cache first
        if let Ok(Some(users)) = self.cache.get::<_, Vec<User>>(&cache_key).await {
            println!("Cache HIT for users with status:{}", active);
            return users;
        }

        println!("Cache MISS for users with status:{}", active);

        // If not in cache, get from database
        let db = self.db.lock().await;
        let users = db.get_users_by_status(active).await;

        // Store in cache with TTL of 5 minutes
        let options = CacheOptions::new().ttl(Duration::from_secs(300));
        let _ = self.cache.set(&cache_key, &users, Some(options)).await;

        users
    }

    async fn update_user(&self, user: User) -> Option<User> {
        let user_id = user.id;
        let cache_key = Self::user_key(user_id);

        // Update in database first
        let mut db = self.db.lock().await;
        let old_user = db.update_user(user.clone()).await;

        if old_user.is_some() {
            // Invalidate cache for the specific user
            let _ = self.invalidator.invalidate(&cache_key).await;

            // Also invalidate status-based caches since status might have changed
            let _ = self
                .invalidator
                .invalidate(&Self::user_status_key(true))
                .await;
            let _ = self
                .invalidator
                .invalidate(&Self::user_status_key(false))
                .await;

            // Update the cache with new value
            let options = CacheOptions::new().ttl(Duration::from_secs(300));
            let _ = self.cache.set(&cache_key, &user, Some(options)).await;
        }

        old_user
    }

    async fn delete_user(&self, id: i32) -> bool {
        let cache_key = Self::user_key(id);

        // Delete from database first
        let mut db = self.db.lock().await;
        let deleted = db.delete_user(id).await;

        if deleted {
            // Invalidate cache for the specific user
            let _ = self.invalidator.invalidate(&cache_key).await;

            // Also invalidate status-based caches
            let _ = self
                .invalidator
                .invalidate(&Self::user_status_key(true))
                .await;
            let _ = self
                .invalidator
                .invalidate(&Self::user_status_key(false))
                .await;
        }

        deleted
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Navius Cache: Repository Pattern Example");
    println!("========================================");

    // Create database
    let db = Arc::new(Mutex::new(Database::new()));

    // Create cache configuration
    let config = CacheConfig::new(
        "memory://".to_string(),
        "repository-example:".to_string(),
        Duration::from_secs(300), // 5 minutes default TTL
    );

    println!("Creating in-memory cache...");

    // Create memory cache
    let cache = MemoryCache::new(config.key_prefix.unwrap_or_default(), config.default_ttl);
    println!("Successfully created in-memory cache");

    // Clear any existing data from previous runs
    println!("Clearing any previous data...");
    cache.clear().await?;

    // Create repository
    let repo = CachedUserRepository::new(db.clone(), cache);

    // Test repository operations
    println!("\n1. Get User (First Access - Cache Miss)");
    let user1 = repo.get_user(1).await;
    println!("User 1: {:?}", user1);

    println!("\n2. Get User (Second Access - Cache Hit)");
    let user1_cached = repo.get_user(1).await;
    println!("User 1 (cached): {:?}", user1_cached);

    println!("\n3. Get Users by Status (First Access - Cache Miss)");
    let active_users = repo.get_users_by_status(true).await;
    println!("Active users count: {}", active_users.len());
    for user in &active_users {
        println!("- {}: {}", user.id, user.name);
    }

    println!("\n4. Update User");
    let mut updated_user = user1.unwrap().clone();
    updated_user.name = "Alice Updated".to_string();
    let old_user = repo.update_user(updated_user.clone()).await;
    println!("Old user data: {:?}", old_user);
    println!("Updated user: {:?}", updated_user);

    println!("\n5. Get User After Update (Cache should be updated)");
    let user1_after_update = repo.get_user(1).await;
    println!("User 1 after update: {:?}", user1_after_update);

    println!("\n6. Delete User");
    let deleted = repo.delete_user(3).await;
    println!("User 3 deleted: {}", deleted);

    println!("\n7. Get Active Users After Changes (Cache should be invalidated)");
    let active_users_updated = repo.get_users_by_status(true).await;
    println!("Active users count: {}", active_users_updated.len());
    for user in &active_users_updated {
        println!("- {}: {}", user.id, user.name);
    }

    println!("\nExample completed successfully!");
    Ok(())
}
