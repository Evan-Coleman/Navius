//! # Spring Boot-like Annotation Macros
//!
//! This module provides structs and traits that simulate Spring Boot annotations
//! in Rust. These macros make it easier for Java Spring Boot developers to
//! transition to Rust using familiar patterns.
//!
//! ## Core Annotations
//!
//! The following Spring Boot-like patterns are supported:
//!
//! - **Controller/RestController**: Marker traits for controller components
//! - **RequestMapping**: Base path configuration for controllers
//! - **GetMapping/PostMapping/etc**: HTTP method-specific endpoint mappings
//! - **Service**: Marker trait for service components
//! - **Repository**: Marker trait for repository components
//! - **Cacheable**: Cache configuration for methods
//! - **CacheEvict**: Cache eviction configuration
//! - **Transactional**: Transaction behavior configuration
//! - **Validate**: Input validation markers
//!
//! ## Extending with Custom Annotations
//!
//! You can create your own annotation-like macros by following these steps:
//!
//! 1. Define a struct or trait that represents your annotation
//! 2. Implement methods that configure the behavior
//! 3. Use the struct/trait in your application code
//!
//! ### Example: Creating a Custom `@Scheduled` Annotation
//!
//! ```rust
//! // Define the annotation struct
//! pub struct Scheduled {
//!     cron: String,
//!     fixed_rate: Option<u64>,
//!     fixed_delay: Option<u64>,
//!     initial_delay: Option<u64>,
//! }
//!
//! impl Scheduled {
//!     pub fn new(cron: &str) -> Self {
//!         Self {
//!             cron: cron.to_string(),
//!             fixed_rate: None,
//!             fixed_delay: None,
//!             initial_delay: None,
//!         }
//!     }
//!
//!     pub fn with_fixed_rate(mut self, milliseconds: u64) -> Self {
//!         self.fixed_rate = Some(milliseconds);
//!         self
//!     }
//!
//!     pub fn with_fixed_delay(mut self, milliseconds: u64) -> Self {
//!         self.fixed_delay = Some(milliseconds);
//!         self
//!     }
//!
//!     pub fn with_initial_delay(mut self, milliseconds: u64) -> Self {
//!         self.initial_delay = Some(milliseconds);
//!         self
//!     }
//! }
//!
//! // Usage in application code
//! struct TaskService;
//!
//! impl TaskService {
//!     fn cleanup_expired_tokens(&self) {
//!         // Method implementation
//!     }
//!
//!     fn register_scheduled_tasks(&self, scheduler: &Scheduler) {
//!         let task = Scheduled::new("0 0 * * * *")  // Run at midnight
//!             .with_initial_delay(60000);           // 1 minute initial delay
//!
//!         scheduler.register(
//!             "cleanupExpiredTokens",
//!             task,
//!             Box::new(|| self.cleanup_expired_tokens())
//!         );
//!     }
//! }
//! ```

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::Deserialize;

use crate::core::{
    error::AppError,
    router::core_app_router::AppState,
};

/// Marker trait for controllers (similar to Spring Boot's @Controller)
pub trait Controller {}

/// Marker trait for REST controllers (similar to Spring Boot's @RestController)
pub trait RestController: Controller {}

/// Registers routes for a controller (similar to Spring Boot's component scanning)
pub trait RegisterRoutes {
    /// Register routes with the router
    fn register_routes(self: Arc<Self>, router: Router<Arc<AppState>>) -> Router<Arc<AppState>>;
}

/// Creates request mapping information (similar to Spring Boot's @RequestMapping)
#[derive(Clone, Debug)]
pub struct RequestMapping {
    /// Base path for the controller
    pub path: String,
}

impl RequestMapping {
    /// Create a new request mapping
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

/// Defines an endpoint (similar to Spring Boot's @GetMapping, @PostMapping, etc.)
#[derive(Clone, Debug)]
pub struct EndpointMapping {
    /// HTTP method for the endpoint
    pub method: EndpointMethod,
    /// Path for the endpoint (appended to the controller's base path)
    pub path: String,
}

/// HTTP methods for endpoint mappings
#[derive(Clone, Debug)]
pub enum EndpointMethod {
    /// GET request (similar to @GetMapping)
    Get,
    /// POST request (similar to @PostMapping)
    Post,
    /// PUT request (similar to @PutMapping)
    Put,
    /// DELETE request (similar to @DeleteMapping)
    Delete,
}

impl EndpointMapping {
    /// Create a GET mapping (similar to Spring Boot's @GetMapping)
    pub fn get(path: &str) -> Self {
        Self {
            method: EndpointMethod::Get,
            path: path.to_string(),
        }
    }
    
    /// Create a POST mapping (similar to Spring Boot's @PostMapping)
    pub fn post(path: &str) -> Self {
        Self {
            method: EndpointMethod::Post,
            path: path.to_string(),
        }
    }
    
    /// Create a PUT mapping (similar to Spring Boot's @PutMapping)
    pub fn put(path: &str) -> Self {
        Self {
            method: EndpointMethod::Put,
            path: path.to_string(),
        }
    }
    
    /// Create a DELETE mapping (similar to Spring Boot's @DeleteMapping)
    pub fn delete(path: &str) -> Self {
        Self {
            method: EndpointMethod::Delete,
            path: path.to_string(),
        }
    }
}

/// Marker trait for service components (similar to Spring Boot's @Service)
pub trait Service {}

/// Marker trait for repositories (similar to Spring Boot's @Repository)
pub trait Repository {}

/// Defines caching behavior for a method (similar to Spring Boot's @Cacheable)
#[derive(Clone, Debug)]
pub struct Cacheable {
    /// Name of the cache
    pub cache_name: String,
    /// Key pattern for the cache (similar to Spring Boot's key = "#id")
    pub key: String,
    /// Time to live in seconds
    pub ttl: Option<u64>,
}

impl Cacheable {
    /// Create a new cacheable definition
    pub fn new(cache_name: &str, key: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            key: key.to_string(),
            ttl: None,
        }
    }
    
    /// Set the time to live for cached items
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

/// Defines cache eviction behavior (similar to Spring Boot's @CacheEvict)
#[derive(Clone, Debug)]
pub struct CacheEvict {
    /// Name of the cache
    pub cache_name: String,
    /// Key pattern for the cache (similar to Spring Boot's key = "#id")
    pub key: String,
    /// Whether to evict all entries (similar to Spring Boot's allEntries = true)
    pub all_entries: bool,
}

impl CacheEvict {
    /// Create a new cache evict definition
    pub fn new(cache_name: &str, key: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            key: key.to_string(),
            all_entries: false,
        }
    }
    
    /// Set whether to evict all entries
    pub fn with_all_entries(mut self, all_entries: bool) -> Self {
        self.all_entries = all_entries;
        self
    }
}

/// Defines transactional behavior (similar to Spring Boot's @Transactional)
#[derive(Clone, Debug)]
pub struct Transactional {
    /// Whether the transaction is read-only
    pub read_only: bool,
    /// Isolation level for the transaction
    pub isolation: TransactionIsolation,
    /// Propagation behavior for the transaction
    pub propagation: TransactionPropagation,
}

/// Transaction isolation levels (similar to Spring Boot's Isolation enum)
#[derive(Clone, Debug)]
pub enum TransactionIsolation {
    /// Default isolation level
    Default,
    /// Read uncommitted isolation level
    ReadUncommitted,
    /// Read committed isolation level
    ReadCommitted,
    /// Repeatable read isolation level
    RepeatableRead,
    /// Serializable isolation level
    Serializable,
}

/// Transaction propagation behaviors (similar to Spring Boot's Propagation enum)
#[derive(Clone, Debug)]
pub enum TransactionPropagation {
    /// Required propagation
    Required,
    /// Supports propagation
    Supports,
    /// Mandatory propagation
    Mandatory,
    /// Requires new propagation
    RequiresNew,
    /// Not supported propagation
    NotSupported,
    /// Never propagation
    Never,
    /// Nested propagation
    Nested,
}

impl Transactional {
    /// Create a new transactional definition
    pub fn new() -> Self {
        Self {
            read_only: false,
            isolation: TransactionIsolation::Default,
            propagation: TransactionPropagation::Required,
        }
    }
    
    /// Set whether the transaction is read-only
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    
    /// Set the isolation level
    pub fn with_isolation(mut self, isolation: TransactionIsolation) -> Self {
        self.isolation = isolation;
        self
    }
    
    /// Set the propagation behavior
    pub fn with_propagation(mut self, propagation: TransactionPropagation) -> Self {
        self.propagation = propagation;
        self
    }
}

/// Validates input (similar to Spring Boot's @Valid annotation)
pub trait Validate {
    /// Validate the input
    fn validate(&self) -> Result<(), AppError>;
}

/// Example of how to use these macros in a controller:
/// 
/// ```
/// use crate::core::macros::core_macros::*;
/// 
/// struct UserController {
///     service: Arc<dyn UserService>,
/// }
/// 
/// impl RestController for UserController {}
/// 
/// impl UserController {
///     // This would be annotated with @RequestMapping("/api/users") in Spring Boot
///     fn base_path() -> RequestMapping {
///         RequestMapping::new("/api/users")
///     }
///     
///     // This would be annotated with @GetMapping in Spring Boot
///     async fn get_users(&self) -> Result<Json<Vec<User>>, AppError> {
///         // Implementation...
///     }
///     
///     // This would be annotated with @GetMapping("/{id}") in Spring Boot
///     async fn get_user_by_id(&self, id: Path<String>) -> Result<Json<User>, AppError> {
///         // Implementation...
///     }
///     
///     // This would be annotated with @PostMapping in Spring Boot
///     async fn create_user(&self, user: Json<CreateUserRequest>) -> Result<Json<User>, AppError> {
///         // Implementation...
///     }
/// }
/// 
/// impl RegisterRoutes for UserController {
///     fn register_routes(self: Arc<Self>, router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
///         let base_path = Self::base_path();
///         
///         // Register routes using the base path and endpoint mappings
///         router
///             .route(
///                 &format!("{}", base_path.path),
///                 get(|state: State<Arc<AppState>>| async move {
///                     let controller = state.service_registry.get::<Arc<UserController>>().unwrap();
///                     controller.get_users().await
///                 })
///             )
///             .route(
///                 &format!("{}/:id", base_path.path),
///                 get(|state: State<Arc<AppState>>, path: Path<String>| async move {
///                     let controller = state.service_registry.get::<Arc<UserController>>().unwrap();
///                     controller.get_user_by_id(path).await
///                 })
///             )
///             // Additional routes...
///     }
/// }
/// ``` 