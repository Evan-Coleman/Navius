# Practical Caching with AWS ElastiCache (Redis)

## Overview
A lightweight, security-focused approach to caching for our Rust Axum backend that leverages AWS ElastiCache (Redis) to improve application performance without unnecessary complexity.

## Current State
Our application needs a structured caching approach to improve response times and reduce database load for frequently accessed data.

## Target State
A practical caching system that:
- Uses AWS ElastiCache (Redis) as the primary caching layer
- Secures cache access with proper authentication and key strategies
- Provides straightforward integration with Axum handlers
- Implements sensible cache invalidation strategies
- Includes minimal but effective monitoring

## Implementation Progress Tracking

### Phase 1: Core Redis Caching
1. **Secure Redis Connection**
   - [ ] Implement AWS ElastiCache connection with proper authentication
   - [ ] Create secure key generation strategies
   - [ ] Add connection pooling with retry handling
   
   *Updated at: Not started*

2. **Basic Cache Operations**
   - [ ] Implement typed get/set operations with serialization
   - [ ] Add TTL-based expiration for cached items
   - [ ] Create cache miss handling with source data loading
   
   *Updated at: Not started*

3. **Axum Integration**
   - [ ] Build cache middleware for Axum routes
   - [ ] Create response caching for appropriate endpoints
   - [ ] Add request deduplication for concurrent requests
   
   *Updated at: Not started*

### Phase 2: Security and Performance
1. **Cache Security Enhancements**
   - [ ] Implement tenant isolation in cache keys
   - [ ] Add Entra identity context in caching decisions
   - [ ] Create secure cache key generation to prevent enumeration
   
   *Updated at: Not started*

2. **Performance Optimization**
   - [ ] Implement batch operations for related data
   - [ ] Add compression for large cached values
   - [ ] Create background refresh for critical cached data
   
   *Updated at: Not started*

3. **Cache Invalidation**
   - [ ] Implement targeted invalidation on data changes
   - [ ] Add cache stampede protection
   - [ ] Create cache versioning for schema changes
   
   *Updated at: Not started*

### Phase 3: Monitoring and Reliability
1. **Observability**
   - [ ] Add basic CloudWatch metrics integration
   - [ ] Implement cache hit/miss logging
   - [ ] Create simple dashboard for cache performance
   
   *Updated at: Not started*

2. **Fallback Strategies**
   - [ ] Implement degraded mode for Redis outages
   - [ ] Add circuit breaker for cache operations
   - [ ] Create graceful performance degradation
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: Secure Redis Connection

## Success Criteria
- Cache operations are secure and respect tenant boundaries
- Cache hit rates exceed 80% for key operations
- Cache-related errors do not impact application availability
- Performance improvements are measurable and consistent
- Development team can easily leverage caching for new features

## Implementation Notes
This approach focuses on practical caching mechanisms that provide immediate performance benefits without unnecessary complexity. We'll leverage Redis for its speed and capabilities while ensuring security and simplicity are maintained.

### Example Implementation

```rust
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use redis::{AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};

// Simple cache service with secure key handling
#[derive(Clone)]
pub struct CacheService {
    client: Arc<Client>,
    prefix: String,
}

impl CacheService {
    pub fn new(redis_url: String, prefix: String) -> Result<Self, redis::RedisError> {
        // Connect to Redis with proper AWS ElastiCache configuration
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client: Arc::new(client),
            prefix,
        })
    }
    
    // Secure key generation that includes tenant isolation
    fn make_key(&self, key: &str, tenant_id: Option<&str>) -> String {
        match tenant_id {
            Some(tenant) => format!("{}:{}:{}", self.prefix, tenant, key),
            None => format!("{}:global:{}", self.prefix, key),
        }
    }
    
    // Type-safe cache get with automatic deserialization
    pub async fn get<T>(&self, key: &str, tenant_id: Option<&str>) -> Result<Option<T>, redis::RedisError>
    where
        T: DeserializeOwned,
    {
        let key = self.make_key(key, tenant_id);
        let mut conn = self.client.get_async_connection().await?;
        
        let result: Option<String> = conn.get(&key).await?;
        
        match result {
            Some(data) => {
                match serde_json::from_str(&data) {
                    Ok(value) => Ok(Some(value)),
                    Err(_) => {
                        // Invalid data in cache, remove it
                        let _: () = conn.del(&key).await?;
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }
    
    // Type-safe cache set with automatic serialization and TTL
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: u64, tenant_id: Option<&str>) -> Result<(), redis::RedisError>
    where
        T: Serialize,
    {
        let key = self.make_key(key, tenant_id);
        let serialized = serde_json::to_string(value)
            .map_err(|e| redis::RedisError::from((redis::ErrorKind::IoError, "Serialization error", e.to_string())))?;
        
        let mut conn = self.client.get_async_connection().await?;
        
        // Set with expiration
        conn.set_ex(key, serialized, ttl_seconds).await
    }
    
    // Cache invalidation
    pub async fn invalidate(&self, key: &str, tenant_id: Option<&str>) -> Result<(), redis::RedisError> {
        let key = self.make_key(key, tenant_id);
        let mut conn = self.client.get_async_connection().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }
    
    // Pattern-based cache invalidation (e.g., for related items)
    pub async fn invalidate_pattern(&self, pattern: &str, tenant_id: Option<&str>) -> Result<(), redis::RedisError> {
        let pattern = self.make_key(&format!("{}*", pattern), tenant_id);
        let mut conn = self.client.get_async_connection().await?;
        
        // Get keys matching pattern
        let keys: Vec<String> = conn.keys(&pattern).await?;
        
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        
        Ok(())
    }
}

// Simple response caching middleware for Axum
pub async fn cache_response<B>(
    State(cache): State<CacheService>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Send,
{
    // Extract cache key from request
    let path = request.uri().path();
    let query = request.uri().query().unwrap_or("");
    let cache_key = format!("response:{}?{}", path, query);
    
    // Extract tenant ID from request extensions (set by auth middleware)
    let tenant_id = request
        .extensions()
        .get::<EntraIdentity>()
        .map(|identity| identity.tenant_id.as_str());
    
    // Try to get from cache first
    match cache.get::<String>(&cache_key, tenant_id).await {
        Ok(Some(cached_response)) => {
            // Return cached response
            Ok(cached_response.into_response())
        }
        _ => {
            // Cache miss, get fresh response
            let response = next.run(request).await;
            
            // Only cache successful responses
            if response.status().is_success() {
                // Convert response to bytes
                let (parts, body) = response.into_parts();
                let bytes = hyper::body::to_bytes(body)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                // Recreate response for caching and returning
                let response = Response::from_parts(parts, bytes.clone().into());
                
                // Cache in background (don't block response)
                let body_str = String::from_utf8_lossy(&bytes);
                let cache_service = cache.clone();
                tokio::spawn(async move {
                    if let Err(e) = cache_service
                        .set(&cache_key, &body_str, 300, tenant_id)
                        .await
                    {
                        tracing::error!("Failed to cache response: {}", e);
                    }
                });
                
                Ok(response)
            } else {
                // Don't cache error responses
                Ok(response)
            }
        }
    }
}

// Add cache middleware to Axum router
pub fn configure_cache(app: Router, cache_service: CacheService) -> Router {
    app.layer(axum::middleware::from_fn_with_state(
        cache_service.clone(),
        cache_response,
    ))
    .with_state(cache_service)
}

// Usage in application setup
async fn create_cache_service() -> CacheService {
    // Get AWS ElastiCache configuration from parameters
    let redis_url = format!(
        "redis://{}:{}",
        std::env::var("ELASTICACHE_HOST").unwrap_or("localhost".to_string()),
        std::env::var("ELASTICACHE_PORT").unwrap_or("6379".to_string())
    );

    CacheService::new(redis_url, "app".to_string())
        .expect("Failed to create cache service")
}

// Example usage in an Axum handler
async fn get_user_details(
    State(cache): State<CacheService>,
    State(db): State<DbPool>,
    Path(user_id): Path<Uuid>,
    Extension(identity): Extension<EntraIdentity>,
) -> impl IntoResponse {
    let tenant_id = identity.tenant_id.as_str();
    let cache_key = format!("user:{}", user_id);
    
    // Try to get from cache first
    if let Ok(Some(user)) = cache.get::<User>(&cache_key, Some(tenant_id)).await {
        return (StatusCode::OK, Json(user)).into_response();
    }
    
    // Cache miss, get from database
    match db.get_user(user_id).await {
        Ok(user) => {
            // Cache the result in background
            let cache_service = cache.clone();
            let user_clone = user.clone();
            let tenant = tenant_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = cache_service
                    .set(&cache_key, &user_clone, 300, Some(&tenant))
                    .await
                {
                    tracing::error!("Failed to cache user: {}", e);
                }
            });
            
            (StatusCode::OK, Json(user)).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
```

## References
- [AWS ElastiCache for Redis](https://docs.aws.amazon.com/elasticache/latest/red-ug/WhatIs.html)
- [redis-rs](https://docs.rs/redis/latest/redis/)
- [Axum middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
- [AWS ElastiCache Best Practices](https://docs.aws.amazon.com/elasticache/latest/red-ug/BestPractices.html) 