# Practical Caching Roadmap

## Overview
A lightweight, security-focused approach to caching for Navius that improves application performance without unnecessary complexity.

## Current State
Our application needs a structured caching approach to improve response times and reduce database load for frequently accessed data.

## Target State
A practical caching system that:
- Uses Redis as the primary caching layer
- Secures cache access with proper authentication and key strategies
- Provides straightforward integration with Axum handlers
- Implements sensible cache invalidation strategies
- Includes minimal but effective monitoring

## Implementation Progress Tracking

### Phase 1: Core Redis Caching
1. **Redis Connection**
   - [ ] Implement Redis connection with proper error handling
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
   - [ ] Add identity context in caching decisions
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

### Phase 3: Reliability
1. **Fallback Strategies**
   - [ ] Implement degraded mode for Redis outages
   - [ ] Add circuit breaker for cache operations
   - [ ] Create graceful performance degradation
   
   *Updated at: Not started*

2. **Monitoring**
   - [ ] Add basic cache hit/miss metrics collection
   - [ ] Implement cache hit/miss logging
   - [ ] Create simple local dashboard for cache performance
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Adaptive TTL Implementation

## Success Criteria
- Cache operations are secure and respect tenant boundaries
- Cache hit rates exceed 80% for key operations
- Cache-related errors do not impact application availability
- Performance improvements are measurable and consistent
- Development team can easily leverage caching for new features

## Implementation Notes
This approach focuses on practical caching mechanisms that provide immediate performance benefits without unnecessary complexity. We'll leverage Redis for its speed and capabilities while ensuring security and simplicity are maintained.

Note: AWS-specific ElastiCache configuration, CloudWatch metrics integration, and AWS VPC security are managed in the AWS Integration roadmap.

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
        // Connect to Redis
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
    // Only cache GET requests
    if request.method() != axum::http::Method::GET {
        return Ok(next.run(request).await);
    }
    
    let path = request.uri().path().to_string();
    
    // Get tenant from request extensions (if available)
    let tenant_id = request
        .extensions()
        .get::<UserIdentity>()
        .map(|identity| identity.tenant_id.as_str());
    
    // Try to get from cache
    match cache.get::<Vec<u8>>(&path, tenant_id).await {
        Ok(Some(cached_bytes)) => {
            // Return cached response
            let response = axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("X-Cache", "HIT")
                .body(axum::body::Body::from(cached_bytes))
                .unwrap();
            
            Ok(response)
        }
        _ => {
            // Cache miss, get response and cache it
            let mut response = next.run(request).await;
            
            if response.status().is_success() {
                // Extract response body to cache it
                let (parts, body) = response.into_parts();
                let bytes = match hyper::body::to_bytes(body).await {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };
                
                // Cache response asynchronously (don't block the response)
                let cache_service = cache.clone();
                let path_clone = path.clone();
                let tenant_clone = tenant_id.map(|s| s.to_string());
                let bytes_clone = bytes.clone();
                tokio::spawn(async move {
                    if let Err(e) = cache_service
                        .set(&path_clone, &bytes_clone.to_vec(), 300, tenant_clone.as_deref())
                        .await
                    {
                        eprintln!("Failed to cache response: {}", e);
                    }
                });
                
                // Rebuild the response
                let response = axum::response::Response::from_parts(
                    parts,
                    axum::body::Body::from(bytes),
                );
                
                Ok(response)
            } else {
                // Don't cache non-success responses
                Ok(response)
            }
        }
    }
}

// Configure the application with caching middleware
pub fn configure_cache(app: Router, cache_service: CacheService) -> Router {
    app.layer(axum::middleware::from_fn_with_state(
        cache_service.clone(),
        cache_response,
    ))
    .with_state(cache_service)
}

// Usage in application setup
async fn create_cache_service() -> CacheService {
    // Get Redis configuration
    let redis_url = format!(
        "redis://{}:{}",
        std::env::var("REDIS_HOST").unwrap_or("localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or("6379".to_string())
    );

    CacheService::new(redis_url, "app".to_string())
        .expect("Failed to create cache service")
}

// Example usage in an Axum handler
async fn get_user_details(
    State(cache): State<CacheService>,
    State(db): State<DbPool>,
    Path(user_id): Path<Uuid>,
    Extension(identity): Extension<UserIdentity>,
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
- [redis-rs](https://docs.rs/redis/latest/redis/)
- [Axum middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
- [Caching Best Practices](https://docs.microsoft.com/en-us/azure/architecture/best-practices/caching)
- [Redis Documentation](https://redis.io/documentation) 