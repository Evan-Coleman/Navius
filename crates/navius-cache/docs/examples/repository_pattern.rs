//! Repository Pattern Example with Cache Integration
//!
//! This example demonstrates how to use the cache with a repository pattern.
//! To run:
//! ```bash
//! cargo run --example repository_pattern
//! ```

use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// ===========================================================================
// Domain Models
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

impl User {
    pub fn new(id: u64, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
            active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

impl Product {
    pub fn new(id: u64, name: &str, price: f64) -> Self {
        Self {
            id,
            name: name.to_string(),
            price,
            in_stock: true,
        }
    }
}

// ===========================================================================
// Error Types
// ===========================================================================

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<navius_cache::error::CacheError> for RepositoryError {
    fn from(error: navius_cache::error::CacheError) -> Self {
        RepositoryError::CacheError(error.to_string())
    }
}

// ===========================================================================
// Mocked Database
// ===========================================================================

/// Mock database that uses hashmaps as storage
#[derive(Debug)]
pub struct MockDatabase {
    users: Arc<RwLock<HashMap<u64, User>>>,
    products: Arc<RwLock<HashMap<u64, Product>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            products: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_user(&self, id: u64) -> Result<Option<User>, RepositoryError> {
        // Simulate database latency
        std::thread::sleep(Duration::from_millis(50));

        let users = self
            .users
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(users.get(&id).cloned())
    }

    pub fn save_user(&self, user: User) -> Result<(), RepositoryError> {
        // Simulate database latency
        std::thread::sleep(Duration::from_millis(100));

        let mut users = self
            .users
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        users.insert(user.id, user);
        Ok(())
    }

    pub fn delete_user(&self, id: u64) -> Result<bool, RepositoryError> {
        // Simulate database latency
        std::thread::sleep(Duration::from_millis(75));

        let mut users = self
            .users
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(users.remove(&id).is_some())
    }

    pub fn get_product(&self, id: u64) -> Result<Option<Product>, RepositoryError> {
        // Simulate database latency
        std::thread::sleep(Duration::from_millis(50));

        let products = self
            .products
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(products.get(&id).cloned())
    }

    pub fn save_product(&self, product: Product) -> Result<(), RepositoryError> {
        // Simulate database latency
        std::thread::sleep(Duration::from_millis(100));

        let mut products = self
            .products
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        products.insert(product.id, product);
        Ok(())
    }

    pub fn delete_product(&self, id: u64) -> Result<bool, RepositoryError> {
        // Simulate database latency
        std::thread::sleep(Duration::from_millis(75));

        let mut products = self
            .products
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(products.remove(&id).is_some())
    }
}

// ===========================================================================
// Cached Repository Implementation
// ===========================================================================

/// Cache-aware user repository
pub struct UserRepository {
    db: Arc<MockDatabase>,
    cache: CacheConnectionManager,
    cache_ttl: Duration,
}

impl UserRepository {
    pub fn new(db: Arc<MockDatabase>, cache: CacheConnectionManager, cache_ttl: Duration) -> Self {
        Self {
            db,
            cache,
            cache_ttl,
        }
    }

    pub async fn get_user(&self, id: u64) -> Result<Option<User>, RepositoryError> {
        let cache_key = format!("user:{}", id);

        // Try to get from cache first
        if let Some(user) = self.cache.get::<_, User>(&cache_key).await? {
            println!("Cache HIT: Retrieved user {} from cache", id);
            return Ok(Some(user));
        }

        println!(
            "Cache MISS: User {} not in cache, fetching from database",
            id
        );

        // If not in cache, get from database
        match self.db.get_user(id)? {
            Some(user) => {
                // Store in cache for future requests
                let options = CacheOptions::new().ttl(self.cache_ttl);
                self.cache.set(&cache_key, &user, Some(options)).await?;
                println!(
                    "Cached user {} for {} seconds",
                    id,
                    self.cache_ttl.as_secs()
                );
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn save_user(&self, user: User) -> Result<(), RepositoryError> {
        // Save to database
        self.db.save_user(user.clone())?;

        // Update cache
        let cache_key = format!("user:{}", user.id);
        let options = CacheOptions::new().ttl(self.cache_ttl);
        self.cache.set(&cache_key, &user, Some(options)).await?;
        println!("Updated user {} in cache", user.id);

        Ok(())
    }

    pub async fn delete_user(&self, id: u64) -> Result<bool, RepositoryError> {
        // Delete from database
        let result = self.db.delete_user(id)?;

        // If successful, invalidate cache
        if result {
            let cache_key = format!("user:{}", id);
            self.cache.delete(&cache_key).await?;
            println!("Removed user {} from cache", id);
        }

        Ok(result)
    }
}

/// Cache-aware product repository
pub struct ProductRepository {
    db: Arc<MockDatabase>,
    cache: CacheConnectionManager,
    cache_ttl: Duration,
}

impl ProductRepository {
    pub fn new(db: Arc<MockDatabase>, cache: CacheConnectionManager, cache_ttl: Duration) -> Self {
        Self {
            db,
            cache,
            cache_ttl,
        }
    }

    pub async fn get_product(&self, id: u64) -> Result<Option<Product>, RepositoryError> {
        let cache_key = format!("product:{}", id);

        // Try to get from cache first
        if let Some(product) = self.cache.get::<_, Product>(&cache_key).await? {
            println!("Cache HIT: Retrieved product {} from cache", id);
            return Ok(Some(product));
        }

        println!(
            "Cache MISS: Product {} not in cache, fetching from database",
            id
        );

        // If not in cache, get from database
        match self.db.get_product(id)? {
            Some(product) => {
                // Store in cache for future requests
                let options = CacheOptions::new().ttl(self.cache_ttl);
                self.cache.set(&cache_key, &product, Some(options)).await?;
                println!(
                    "Cached product {} for {} seconds",
                    id,
                    self.cache_ttl.as_secs()
                );
                Ok(Some(product))
            }
            None => Ok(None),
        }
    }

    pub async fn save_product(&self, product: Product) -> Result<(), RepositoryError> {
        // Save to database
        self.db.save_product(product.clone())?;

        // Update cache
        let cache_key = format!("product:{}", product.id);
        let options = CacheOptions::new().ttl(self.cache_ttl);
        self.cache.set(&cache_key, &product, Some(options)).await?;
        println!("Updated product {} in cache", product.id);

        Ok(())
    }

    pub async fn delete_product(&self, id: u64) -> Result<bool, RepositoryError> {
        // Delete from database
        let result = self.db.delete_product(id)?;

        // If successful, invalidate cache
        if result {
            let cache_key = format!("product:{}", id);
            self.cache.delete(&cache_key).await?;
            println!("Removed product {} from cache", id);
        }

        Ok(result)
    }
}

// ===========================================================================
// Main Example
// ===========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Repository Pattern with Cache Integration Example");
    println!("================================================");

    // Create a mock database
    let db = Arc::new(MockDatabase::new());

    // Setup the cache
    let config = CacheConfig::new(
        "memory://".to_string(),
        "repository-example:".to_string(),
        Duration::from_secs(60),
    );

    println!("Creating in-memory cache...");

    // Create in-memory cache
    let cache = CacheConnectionManager::new_memory(config);
    println!("Successfully created in-memory cache");

    // Clear any previous data
    cache.clear().await?;

    // Create repositories
    let user_repo = UserRepository::new(db.clone(), cache.clone(), Duration::from_secs(30));
    let product_repo = ProductRepository::new(db.clone(), cache.clone(), Duration::from_secs(30));

    println!("\nTesting User Repository");
    println!("----------------------");

    // Create and save a user
    let alice = User::new(1, "Alice", "alice@example.com");
    user_repo.save_user(alice.clone()).await?;
    println!("Saved user: {:?}", alice);

    // First retrieval (should come from database with cache miss)
    let retrieved = user_repo.get_user(1).await?;
    println!("First retrieval result: {:?}", retrieved);

    // Second retrieval (should come from cache)
    let retrieved = user_repo.get_user(1).await?;
    println!("Second retrieval result: {:?}", retrieved);

    // Update the user
    let mut updated_alice = alice.clone();
    updated_alice.email = "alice.updated@example.com".to_string();
    user_repo.save_user(updated_alice.clone()).await?;
    println!("Updated user: {:?}", updated_alice);

    // Retrieval after update (should reflect new data)
    let retrieved = user_repo.get_user(1).await?;
    println!("Retrieval after update: {:?}", retrieved);

    // Delete the user
    let deleted = user_repo.delete_user(1).await?;
    println!("User deleted: {}", deleted);

    // Retrieval after delete (should be None)
    let retrieved = user_repo.get_user(1).await?;
    println!("Retrieval after delete: {:?}", retrieved);

    println!("\nTesting Product Repository");
    println!("-------------------------");

    // Create and save a product
    let laptop = Product::new(101, "Laptop", 999.99);
    product_repo.save_product(laptop.clone()).await?;
    println!("Saved product: {:?}", laptop);

    // First retrieval (should come from database with cache miss)
    let retrieved = product_repo.get_product(101).await?;
    println!("First retrieval result: {:?}", retrieved);

    // Second retrieval (should come from cache)
    let retrieved = product_repo.get_product(101).await?;
    println!("Second retrieval result: {:?}", retrieved);

    // Update the product
    let mut updated_laptop = laptop.clone();
    updated_laptop.price = 899.99;
    product_repo.save_product(updated_laptop.clone()).await?;
    println!("Updated product: {:?}", updated_laptop);

    // Retrieval after update (should reflect new data)
    let retrieved = product_repo.get_product(101).await?;
    println!("Retrieval after update: {:?}", retrieved);

    // Delete the product
    let deleted = product_repo.delete_product(101).await?;
    println!("Product deleted: {}", deleted);

    // Retrieval after delete (should be None)
    let retrieved = product_repo.get_product(101).await?;
    println!("Retrieval after delete: {:?}", retrieved);

    println!("\nExample completed successfully!");
    Ok(())
}
