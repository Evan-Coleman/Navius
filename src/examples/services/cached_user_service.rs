//! # Cached User Service Example
//!
//! This example demonstrates how to use the Spring Boot-like annotation macros
//! to create a service with caching.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use crate::core::error::AppError;
use crate::examples::controllers::user_controller::{
    CreateUserRequest, UpdateUserRequest, User, UserService,
};

/// Simple cache implementation for the example
pub struct SimpleCache<T> {
    data: Mutex<HashMap<String, (T, Option<u64>)>>,
}

impl<T: Clone> SimpleCache<T> {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let data = self.data.lock().unwrap();
        data.get(key).map(|(value, _)| value.clone())
    }

    pub fn set(&self, key: &str, value: T, ttl_seconds: Option<u64>) {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), (value, ttl_seconds));
    }

    pub fn remove(&self, key: &str) {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
    }
}

/// Cached user service implementation
pub struct CachedUserService {
    inner_service: Arc<dyn UserService>,
    cache: Arc<SimpleCache<User>>,
}

impl CachedUserService {
    pub fn new(inner_service: Arc<dyn UserService>, cache: Arc<SimpleCache<User>>) -> Self {
        Self {
            inner_service,
            cache,
        }
    }

    /// Find user by ID with caching (similar to @Cacheable in Spring Boot)
    pub async fn find_by_id_cached(&self, id: Uuid) -> Option<User> {
        // Define cacheable config (similar to @Cacheable(value = "users", key = "#id"))
        let cacheable = Cacheable::new("users", "{id}").with_ttl(300); // 5 minutes TTL

        // Check cache first
        let cache_key = format!("user:{}", id);
        if let Some(cached_user) = self.cache.get(&cache_key) {
            return Some(cached_user);
        }

        // If not in cache, fetch from inner service
        let user = self.inner_service.find_by_id(id);

        // Store in cache if found
        if let Some(ref user_data) = user {
            self.cache.set(&cache_key, user_data.clone(), cacheable.ttl);
        }

        user
    }

    /// Create a user and update cache (similar to @CachePut in Spring Boot)
    pub async fn create_cached(&self, request: CreateUserRequest) -> User {
        let user = self.inner_service.create(request);

        // Store in cache (similar to @CachePut(value = "users", key = "#result.id"))
        let cache_key = format!("user:{}", user.id);
        self.cache.set(&cache_key, user.clone(), Some(300));

        user
    }

    /// Update a user and update cache (similar to @CachePut in Spring Boot)
    pub async fn update_cached(&self, id: Uuid, request: UpdateUserRequest) -> Option<User> {
        let updated_user = self.inner_service.update(id, request);

        if let Some(ref user) = updated_user {
            // Update cache if successful
            let cache_key = format!("user:{}", id);
            self.cache.set(&cache_key, user.clone(), Some(300));
        }

        updated_user
    }

    /// Delete a user and evict from cache (similar to @CacheEvict in Spring Boot)
    pub async fn delete_cached(&self, id: Uuid) -> bool {
        let result = self.inner_service.delete(id);

        if result {
            // Define cache evict config (similar to @CacheEvict(value = "users", key = "#id"))
            let _cache_evict = CacheEvict::new("users", "{id}");

            // Evict from cache if deletion was successful
            let cache_key = format!("user:{}", id);
            self.cache.remove(&cache_key);
        }

        result
    }

    /// Clear all users from cache (similar to @CacheEvict(value = "users", allEntries = true))
    pub async fn clear_cache(&self) {
        // Define cache evict config
        let _cache_evict = CacheEvict::new("users", "*").with_all_entries(true);

        // In a real implementation, this would clear all cache entries with the "users" prefix
        // For simplicity, we don't implement the actual cache clearing logic here
        println!("Clearing all user cache entries");
    }
}

/// Example of using the cached service
pub async fn cached_service_example() -> Result<(), AppError> {
    // Create dependencies
    let inner_service =
        Arc::new(crate::examples::controllers::user_controller::InMemoryUserService::new())
            as Arc<dyn UserService>;
    let cache = Arc::new(SimpleCache::<User>::new());
    let service = CachedUserService::new(inner_service, cache);

    // Create a user
    let user = service
        .create_cached(CreateUserRequest {
            name: "Cached User".to_string(),
            email: "cached@example.com".to_string(),
        })
        .await;
    println!("Created user: {:?}", user);

    // Find by ID (should be from cache)
    let cached_user = service.find_by_id_cached(user.id).await;
    println!("Found user from cache: {:?}", cached_user);

    // Update user
    let updated_user = service
        .update_cached(
            user.id,
            UpdateUserRequest {
                name: Some("Updated Cached User".to_string()),
                email: None,
            },
        )
        .await;
    println!("Updated user: {:?}", updated_user);

    // Delete user
    let deleted = service.delete_cached(user.id).await;
    println!("Deleted user: {}", deleted);

    Ok(())
}

// Annotation-style structs (will be implemented in a real framework)

pub struct Cacheable {
    cache_name: String,
    key_pattern: String,
    pub ttl: Option<u64>,
}

impl Cacheable {
    pub fn new(cache_name: &str, key_pattern: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            key_pattern: key_pattern.to_string(),
            ttl: None,
        }
    }

    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.ttl = Some(seconds);
        self
    }
}

pub struct CacheEvict {
    cache_name: String,
    key_pattern: String,
    all_entries: bool,
}

impl CacheEvict {
    pub fn new(cache_name: &str, key_pattern: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            key_pattern: key_pattern.to_string(),
            all_entries: false,
        }
    }

    pub fn with_all_entries(mut self, all_entries: bool) -> Self {
        self.all_entries = all_entries;
        self
    }
}

pub trait Service {}
