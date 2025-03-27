---
title: "Two-Tier Cache Implementation Example"
description: "Code examples demonstrating how to implement and use the two-tier caching system in Navius applications"
category: examples
tags:
  - examples
  - caching
  - redis
  - performance
  - two-tier
  - code
related:
  - ../guides/caching-strategies.md
  - ../reference/configuration/cache-config.md
  - ../reference/patterns/caching-patterns.md
last_updated: March 26, 2024
version: 1.0
---

# Two-Tier Cache Implementation Example

This example demonstrates how to implement and use the two-tier caching system in a Navius application.

## Basic Implementation

```rust
use navius::app::cache::{create_two_tier_cache, create_typed_two_tier_cache};
use navius::core::services::cache_service::CacheService;
use std::time::Duration;
use serde::{Serialize, Deserialize};

// Define a type to cache
#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Create a function to set up the cache
async fn setup_user_cache(cache_service: &CacheService) -> Result<(), AppError> {
    // Create a two-tier cache for user data
    // Fast cache (memory) TTL: 60 seconds
    // Slow cache (Redis) TTL: 1 hour
    let user_cache = create_typed_two_tier_cache::<User>(
        "users",
        cache_service,
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(3600)),
    ).await?;
    
    // Store a user in the cache
    let user = User {
        id: "123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    user_cache.set("user:123", &user, None).await?;
    
    // Later, retrieve the user from the cache
    if let Some(cached_user) = user_cache.get("user:123").await? {
        println!("Found user: {:?}", cached_user);
    }
    
    Ok(())
}
```

## Fallback Behavior Demonstration

This example shows how the two-tier cache handles fallback behavior:

```rust
use navius::app::cache::create_two_tier_cache;
use navius::core::services::cache_service::CacheService;
use std::time::Duration;

async fn demonstrate_fallback(cache_service: &CacheService) -> Result<(), AppError> {
    // Create a two-tier cache
    let cache = create_two_tier_cache(
        "demo-cache",
        cache_service,
        Some(Duration::from_secs(30)),   // Fast cache TTL
        Some(Duration::from_secs(300)),  // Slow cache TTL
    ).await?;
    
    // Set a value in both caches
    cache.set("key1", "value1".as_bytes(), None).await?;
    
    // This will fetch from the fast cache (in-memory)
    let fast_result = cache.get("key1").await?;
    println!("Fast cache result: {:?}", fast_result);
    
    // Simulate clearing the fast cache
    // In a real scenario, this might happen when the app restarts
    cache.fast_cache.clear().await?;
    
    // This will now fetch from the slow cache (Redis) and promote to fast cache
    let fallback_result = cache.get("key1").await?;
    println!("After fallback result: {:?}", fallback_result);
    
    // This will now fetch from the fast cache again as the value was promoted
    let promoted_result = cache.get("key1").await?;
    println!("After promotion result: {:?}", promoted_result);
    
    Ok(())
}
```

## Development Environment Setup

For development environments without Redis, you can use the memory-only two-tier cache:

```rust
use navius::app::cache::create_memory_only_two_tier_cache;
use navius::core::services::cache_service::CacheService;
use std::time::Duration;

async fn setup_dev_cache(cache_service: &CacheService) -> Result<(), AppError> {
    // Create a memory-only two-tier cache
    // Small fast cache TTL: 10 seconds
    // Larger slow cache TTL: 60 seconds
    let dev_cache = create_memory_only_two_tier_cache(
        "dev-cache",
        cache_service,
        Some(Duration::from_secs(10)),
        Some(Duration::from_secs(60)),
    ).await?;
    
    // Use it like a normal cache
    dev_cache.set("dev-key", "dev-value".as_bytes(), None).await?;
    
    Ok(())
}
```

## Integration with Service Layer

Here's how to integrate the two-tier cache with a service layer:

```rust
use navius::app::cache::create_typed_two_tier_cache;
use navius::core::services::cache_service::CacheService;
use std::time::Duration;
use std::sync::Arc;

struct UserService {
    cache: Arc<Box<dyn TypedCache<User>>>,
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    async fn new(
        cache_service: &CacheService,
        repository: Arc<dyn UserRepository>,
    ) -> Result<Self, AppError> {
        let cache = Arc::new(create_typed_two_tier_cache::<User>(
            "users",
            cache_service,
            Some(Duration::from_secs(60)),
            Some(Duration::from_secs(3600)),
        ).await?);
        
        Ok(Self { cache, repository })
    }
    
    async fn get_user(&self, id: &str) -> Result<Option<User>, AppError> {
        let cache_key = format!("user:{}", id);
        
        // Try to get from cache first
        if let Some(user) = self.cache.get(&cache_key).await? {
            return Ok(Some(user));
        }
        
        // If not in cache, get from repository
        if let Some(user) = self.repository.find_by_id(id).await? {
            // Store in cache for next time
            self.cache.set(&cache_key, &user, None).await?;
            return Ok(Some(user));
        }
        
        Ok(None)
    }
}
```

## Complete Application Example

Here's a complete example showing how to set up and use the two-tier cache in an API endpoint:

```rust
use axum::{
    routing::get,
    Router,
    extract::{State, Path},
    response::Json,
};
use navius::app::cache::create_typed_two_tier_cache;
use navius::core::services::cache_service::CacheService;
use std::sync::Arc;
use std::time::Duration;

// Application state with cache service
struct AppState {
    user_service: Arc<UserService>,
}

async fn setup_app() -> Router {
    // Create cache service
    let cache_service = CacheService::new().await.unwrap();
    
    // Create user repository
    let user_repository = Arc::new(UserRepositoryImpl::new());
    
    // Create user service with caching
    let user_service = Arc::new(
        UserService::new(&cache_service, user_repository).await.unwrap()
    );
    
    // Create application state
    let app_state = Arc::new(AppState { user_service });
    
    // Create router with API endpoints
    Router::new()
        .route("/users/:id", get(get_user))
        .with_state(app_state)
}

// API endpoint that uses the cached service
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Option<User>> {
    let user = state.user_service.get_user(&id).await.unwrap();
    Json(user)
}
```

## Best Practices

1. **Correctly size your caches**: Small, frequently accessed data works best
2. **Set appropriate TTLs**: Fast cache should have shorter TTL than slow cache
3. **Handle errors gracefully**: The two-tier cache handles most errors internally
4. **Monitor performance**: Track hit/miss rates to fine-tune cache settings
5. **Use typed caches**: They provide type safety and easier code maintenance

## Read More

For more details on caching strategies, see the [Caching Strategies Guide](../guides/caching-strategies.md). 