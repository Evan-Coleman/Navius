# Core Stability Roadmap Instructions

This document provides detailed instructions for implementing the Core Stability Roadmap with a focus on creating a Spring Boot-like developer experience in Rust. Follow these steps sequentially to ensure a smooth implementation.

## Phase 1: Fix Build Errors and Core Structure

### 1. Core Naming Standardization

#### Step 1.1: Create detailed file inventory and analysis (1-2 days)
1. Run `find src/core -type f -name "*.rs" | sort > core_files.txt` to create complete inventory
2. Categorize files by directory and identify potential naming conflicts
3. For each file, identify all import references:
   ```bash
   for file in $(find src -name "*.rs"); do
     echo "Analyzing $file"
     grep -n "use " $file | grep "core::"
   done > core_dependencies.txt
   ```
4. Generate dependency graph to visualize import relationships
5. Identify high-impact files that are referenced frequently
6. Review existing tests for affected components

#### Step 1.2: Implement Core Router renaming (2-3 days)
1. Create `src/core/router/core_router.rs` (copy from existing file)
2. Update `CoreRouter` struct and implementation in the new file
3. Rename `app_router.rs` to `core_app_router.rs`
4. Update `CoreRouterBuilder` struct and implementation
5. Modify `src/core/router/mod.rs` to export ONLY from new files (backwards compatibility NOT needed)
6. Run tests after each file change
7. Update all imports of router components using `find` and `sed`:
   ```bash
   find src -name "*.rs" -exec sed -i 's/use crate::core::router::/use crate::core::router::core_/g' {} \;
   ```
8. Remove old non-prefixed files after successful testing

#### Step 1.3: Model and Handler renaming (2-3 days)
1. Rename model files:
   - `models/response.rs` → `models/core_response.rs`
   - `models/error.rs` → `models/core_error.rs`
   - Update `models/mod.rs` to export ONLY from new files (no backwards compatibility exports)
   
2. Rename handler files:
   - `handlers/health.rs` → `handlers/core_health.rs`
   - `handlers/actuator.rs` → `handlers/core_actuator.rs`
   - `handlers/docs.rs` → `handlers/core_docs.rs`
   - Update `handlers/mod.rs` to export ONLY from new files

3. Rename utility files:
   - `utils/api_client.rs` → `utils/core_api_client.rs`
   - `utils/api_logger.rs` → `utils/core_api_logger.rs`
   - `utils/date.rs` → `utils/core_date.rs`
   - Update `utils/mod.rs` to export ONLY from new files

4. Run tests after each rename to catch errors early
5. Remove old non-prefixed files after tests pass

#### Step 1.4: Eliminate mod.rs files and centralize module declarations (2-3 days)

1. **Inventory and planning**:
   - Create a complete inventory of all mod.rs files in the codebase:
     ```bash
     find src -name "mod.rs" > mod_files.txt
     ```
   - For each mod.rs file, analyze its content and identify:
     - Module declarations
     - Re-exports
     - Public/private visibility settings

2. **Create module declarations structure for lib.rs**:
   - Create a draft structure of nested module declarations:
     ```rust
     // Example structure
     mod core {
         pub mod router {
             pub mod core_router;
             pub mod core_app_router;
             
             pub use core_router::*;
             pub use core_app_router::*;
         }
         
         pub mod models {
             pub mod core_response;
             pub mod error;
             pub mod extensions;
             
             pub use core_response::*;
             pub use error::*;
             pub use extensions::*;
         }
         
         // Other modules...
     }
     
     mod app {
         pub mod api {
             pub mod examples;
             
             pub use examples::*;
         }
         
         pub mod services {
             // Service module declarations...
         }
         
         // Other modules...
     }
     ```

3. **Implement in manageable chunks**:
   - Start with a single directory (e.g., `src/core/models/`)
   - Move module declarations and re-exports to lib.rs
   - Run `cargo check` to verify the changes
   - Delete the mod.rs file once changes are verified
   - Continue with other directories, committing after each successful change

4. **Update imports**:
   - As you remove mod.rs files, you may need to update imports in files that reference them
   - Run `cargo check` frequently to identify issues
   - Use find/replace tools to update imports in bulk when patterns are consistent:
     ```bash
     find src -name "*.rs" -exec sed -i 's/use crate::core::models::/use crate::core::models::/g' {} \;
     ```

5. **Handle special cases**:
   - For modules with complex nesting or re-export patterns, consider:
     - Using selective re-exports with explicit items
     - Using aliased imports for name conflicts
     - Creating helper re-export modules if necessary

6. **Final verification**:
   - After all mod.rs files are removed, run:
     ```bash
     find src -name "mod.rs"
     ```
   - This should return no results
   - Run comprehensive tests to ensure no functionality is broken
   - Verify that all public interfaces are still accessible

7. **Documentation**:
   - Update developer documentation to reflect the new module structure
   - Add comments in lib.rs explaining the module organization
   - Provide examples of how to add new modules in the future

#### Step 1.5: Create user-extensible templates (1-2 days)
1. Create `src/app/models.rs` with examples of extending core models
2. Create `src/app/handlers.rs` with examples of using core handlers
3. Update `src/app/router.rs` to reference new core router files
4. Add clear documentation comments explaining extension points

#### Step 1.6: Update documentation and finalize (1-2 days)
1. Update all documentation to reflect new file names and imports
2. Create a dedicated naming convention guide in docs
3. Create examples showing:
   - How to extend core functionality
   - User-defined files that would have naming conflicts
   - Before and after import patterns
4. Document that backward compatibility is not maintained

#### Step 1.7: Final testing and verification (1 day)
1. Run all unit tests
2. Run integration tests
3. Manually test example applications
4. Verify that all core functionality works as expected

### 2. Router Module Fixes

#### Step 1.1: Create app_router.rs file with Spring Boot-like design
1. Create a new file at `src/core/router/app_router.rs`
2. Define the `AppState` struct with the following fields:
   - `config`: App configuration
   - `start_time`: For tracking uptime
   - `client`: HTTP client (Optional)
   - `cache_registry`: For caching (Optional)
   - `token_client`: For authentication (Optional)
   - `metrics_handle`: For metrics (Optional)
   - `resource_registry`: For API resources (Optional)
   - `service_registry`: For services
3. Implement `Default` for `AppState`
4. Create intuitive router builder pattern similar to Spring Boot's application builder:
   ```rust
   pub struct RouterBuilder {
       app_state: AppState,
       cors_enabled: bool,
       metrics_enabled: bool,
       auth_enabled: bool,
       // Other configuration options
   }
   
   impl RouterBuilder {
       pub fn new() -> Self {
           Self {
               app_state: AppState::default(),
               cors_enabled: true,
               metrics_enabled: true,
               auth_enabled: false,
           }
       }
       
       pub fn with_config(mut self, config: Arc<Config>) -> Self {
           self.app_state.config = Some(config);
           self
       }
       
       pub fn with_cache(mut self, cache: Option<Arc<CacheRegistry>>) -> Self {
           self.app_state.cache_registry = cache;
           self
       }
       
       // Other builder methods
       
       pub fn build(self) -> Router {
           // Build the router with all configured components
       }
   }
   ```

#### Step 1.2: Fix AppState references and create Spring Boot-like dependency injection
1. Update all imports from `crate::core::router::AppState` to use the correct path
2. Fix any method calls on `AppState` to match the new implementation
3. Ensure `AppState` is properly re-exported from the `app` module
4. Implement a dependency injection pattern similar to Spring's autowiring:
   ```rust
   impl AppState {
       // Get component with type inference (similar to @Autowired)
       pub fn get<T: 'static>(&self) -> Option<&T> {
           // Implementation that finds the right component based on type
       }
       
       // Register a component (similar to @Bean)
       pub fn register<T: 'static>(&mut self, component: T) {
           // Implementation that registers a component
       }
   }
   ```

#### Step 1.3: Implement a simple public health endpoint
1. Create a simple health handler in `src/core/handlers/health.rs`:
   ```rust
   pub async fn simple_health_handler() -> impl IntoResponse {
       Json(json!({ "status": "UP" }))
   }
   ```

2. Add the route to the public router in `app_router.rs`:
   ```rust
   // In the build method of RouterBuilder
   let public_routes = Router::new()
       .route("/health", get(simple_health_handler));
       
   // Later merge with other routers
   Router::new()
       .merge(public_routes)
       // Other routers
   ```

3. Ensure the endpoint is accessible without authentication
4. Add a unit test to verify the endpoint returns the correct response:
   ```rust
   #[tokio::test]
   async fn test_simple_health_endpoint() {
       // Arrange
       let app = RouterBuilder::new().build();
       let request = Request::builder()
           .uri("/health")
           .method("GET")
           .body(Body::empty())
           .unwrap();
       
       // Act
       let response = app.oneshot(request).await.unwrap();
       
       // Assert
       assert_eq!(response.status(), StatusCode::OK);
       
       let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
       let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
       
       assert_eq!(body["status"], "UP");
   }
   ```

5. This endpoint should:
   - Return a 200 OK status code
   - Return a simple JSON object with a "status" field set to "UP"
   - Be accessible without any authentication
   - Respond quickly without any dependencies
   - Serve as a minimal healthcheck for load balancers and monitoring tools

### 2. Module Structure and Developer Experience Enhancement

#### Step 2.1: Create examples module with Spring Boot-like patterns
1. Create `src/app/api/examples.rs` with examples demonstrating:
   - Controller-style endpoint definition
   - Service injection
   - Repository pattern
   - Error handling
2. Implement a macro for controller definition similar to Spring's @RestController:
   ```rust
   // Example macro usage in examples.rs
   #[api_controller]
   pub struct ExampleController {
       service: Arc<dyn ExampleService>,
   }
   
   #[api_routes]
   impl ExampleController {
       #[get("/examples")]
       async fn get_examples(&self) -> Result<Json<Vec<Example>>, AppError> {
           let examples = self.service.get_all().await?;
           Ok(Json(examples))
       }
       
       #[post("/examples")]
       async fn create_example(&self, payload: Json<CreateExample>) -> Result<Json<Example>, AppError> {
           let example = self.service.create(payload.0).await?;
           Ok(Json(example))
       }
   }
   ```

#### Step 2.2: Fix module declarations and improve discoverability
1. Review all module declarations in `.rs` files
2. Ensure all referenced modules exist
3. Fix any incorrect re-exports
4. Create clear documentation for extending each module
5. Add explicit extension points for user customization in each core module

## Phase 2: Developer Experience Enhancement

### 1. Core Abstraction Development

#### Step 1.1: Create database abstractions similar to Spring Data
1. Implement a repository pattern with macros for common operations:
   ```rust
   #[repository]
   pub trait UserRepository: Send + Sync {
       async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
       async fn find_all(&self) -> Result<Vec<User>, AppError>;
       async fn save(&self, user: User) -> Result<User, AppError>;
       async fn delete(&self, id: Uuid) -> Result<(), AppError>;
       
       // Custom queries can be defined with a macro similar to Spring Data
       #[query("SELECT * FROM users WHERE email = $1")]
       async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
   }
   ```

2. Create a database connection manager with connection pooling:
   ```rust
   pub struct DatabaseManager {
       pool: PgPool,
   }
   
   impl DatabaseManager {
       pub async fn new(config: &DatabaseConfig) -> Result<Self, AppError> {
           // Implementation that creates a connection pool
       }
       
       pub fn get_pool(&self) -> &PgPool {
           &self.pool
       }
   }
   ```

#### Step 1.2: Implement caching abstractions similar to Spring Cache
1. Create a caching layer with annotations for method-level caching:
   ```rust
   #[cacheable(key = "user:{id}", ttl = "30m")]
   async fn get_user(&self, id: Uuid) -> Result<User, AppError> {
       // Implementation that will be cached
   }
   
   #[cache_evict(key = "user:{id}")]
   async fn update_user(&self, id: Uuid, user: User) -> Result<User, AppError> {
       // Implementation that will evict a cache entry
   }
   ```

#### Step 1.3: Develop error handling framework with clear patterns
1. Create a comprehensive error handling system:
   ```rust
   // Core error types similar to Spring's exception hierarchy
   pub enum AppError {
       NotFound(String),
       BadRequest(String),
       Unauthorized(String),
       Forbidden(String),
       InternalServerError(String),
       // Other error types
   }
   
   // Error conversion traits for easy interoperability
   impl From<sqlx::Error> for AppError {
       fn from(err: sqlx::Error) -> Self {
           // Map database errors to appropriate AppError variants
       }
   }
   
   // Global error handler similar to Spring's @ControllerAdvice
   pub async fn global_error_handler(err: AppError) -> (StatusCode, Json<ErrorResponse>) {
       // Convert AppError to appropriate HTTP response
   }
   ```

### 2. Health and Info Endpoint Enhancement

#### Step 2.1: Update health model to match Spring Boot's format
1. Modify `DetailedHealthResponse` in `src/core/models/` to match Spring Boot's format:
   ```json
   {
     "status": "UP",
     "components": {
       "db": {
         "status": "UP",
         "details": {
           "database": "PostgreSQL",
           "version": "14.4"
         }
       },
       "cache": {
         "status": "UP"
       },
       "diskSpace": {
         "status": "UP",
         "details": {
           "total": 1048576,
           "free": 524288,
           "threshold": 10485
         }
       }
     }
   }
   ```

#### Step 2.2: Enhance dependency reporting
1. Update `DependencyStatus` to include all required fields
2. Implement proper status reporting for all dependencies
3. Add details for each dependency similar to Spring Actuator
4. Include health indicators that can be easily extended by users

#### Step 2.3: Implement info endpoint with Spring Boot-like structure
1. Create an info model that matches Spring Boot's info endpoint:
   ```json
   {
     "app": {
       "name": "navius-application",
       "description": "My Navius Application",
       "version": "1.0.0",
       "encoding": "UTF-8",
       "java": {
         "equivalent": "Spring Boot 3.0"
       }
     },
     "build": {
       "artifact": "navius-app",
       "name": "navius-app",
       "time": "2023-06-15T16:34:00Z",
       "version": "1.0.0",
       "group": "com.example"
     },
     "git": {
       "branch": "main",
       "commit": {
         "id": "abc123",
         "time": "2023-06-15T16:30:00Z"
       }
     }
   }
   ```

## Phase 3: Documentation and Examples

### 1. Spring-to-Rust Migration Guides

#### Step 1.1: Create comprehensive documentation
1. Create markdown files in `docs/guides/` with:
   - Spring Boot to Navius conversion guide
   - Mapping of Spring Boot annotations to Navius macros
   - Database access patterns comparison
   - Authentication and security comparison
   - Configuration property mapping

#### Step 1.2: Develop side-by-side comparisons
1. Create documentation showing Spring Boot code alongside equivalent Navius code:
   ```
   // Spring Boot
   @RestController
   @RequestMapping("/api/users")
   public class UserController {
       @Autowired
       private UserService userService;
       
       @GetMapping("/{id}")
       public ResponseEntity<User> getUser(@PathVariable UUID id) {
           return ResponseEntity.ok(userService.getUser(id));
       }
   }
   
   // Navius
   #[api_controller]
   pub struct UserController {
       service: Arc<dyn UserService>,
   }
   
   #[api_routes]
   impl UserController {
       #[get("/api/users/:id")]
       async fn get_user(&self, Path(id): Path<Uuid>) -> Result<Json<User>, AppError> {
           let user = self.service.get_user(id).await?;
           Ok(Json(user))
       }
   }
   ```

### 2. Example Applications and Tutorials

#### Step 2.1: Create fully-functional example application
1. Develop a complete example application in `examples/` showing:
   - User authentication and authorization
   - Database CRUD operations
   - Caching
   - Error handling
   - API documentation
   - Testing

#### Step 2.2: Develop comprehensive tutorials
1. Create step-by-step tutorials for common tasks:
   - Setting up a new Navius project
   - Creating RESTful endpoints
   - Connecting to a database
   - Implementing authentication
   - Adding caching
   - Testing your application
   - Deploying to production

#### Step 2.3: Create a tutorial for extending the health endpoint
1. Show how to customize and extend the simple `/health` endpoint:
   ```rust
   // In user's app code
   pub async fn custom_health_handler() -> impl IntoResponse {
       // Example of adding custom health checks
       let db_status = check_database_connection().await;
       let redis_status = check_redis_connection().await;
       
       let status = if db_status && redis_status { "UP" } else { "DOWN" };
       
       Json(json!({
           "status": status,
           "custom": {
               "database": db_status,
               "redis": redis_status,
               "version": env!("CARGO_PKG_VERSION"),
               "environment": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
           }
       }))
   }
   
   // In the user's router configuration
   let app = NaviusApp::new()
       .with_default_config()
       // Override the default health endpoint with custom implementation
       .with_route("/health", get(custom_health_handler)) 
       .run();
   ```

## Phase 4: Testing and Refinement

### 1. Comprehensive Testing

#### Step 1.1: Create unit tests for all components
1. Develop comprehensive unit tests for:
   - Router builder
   - Controller macros
   - Repository implementations
   - Caching abstractions
   - Error handling

#### Step 1.2: Implement integration tests
1. Create integration tests that simulate real-world usage:
   - Full request-response cycle
   - Database interactions
   - Cache operations
   - Authentication flows

### 2. Performance Optimization

#### Step 2.1: Benchmark critical components
1. Create benchmarks for key components:
   - Router performance
   - Database access patterns
   - Serialization/deserialization
   - Error handling overhead

#### Step 2.2: Optimize and document performance
1. Identify and fix performance bottlenecks
2. Document performance characteristics:
   - Requests per second compared to Spring Boot
   - Memory usage
   - Startup time
   - Database operation throughput

## Implementation Notes

### Spring Boot-like Design Patterns
- Use builder patterns for configuration (similar to Spring's auto-configuration)
- Implement trait-based dependency injection (similar to Spring's @Autowired)
- Create macro-based annotations for common patterns
- Follow convention-over-configuration principles

### Dependency Status Reporting
- Database: Should report "UP" if connected, "DOWN" if failed, "DISABLED" if not configured
- Cache: Should report "UP" if working, "DOWN" if failed
- Authentication: Should report "UP"/"DOWN" based on configuration

### Status Codes
- 200 OK: All dependencies are up or disabled (intentionally)
- 503 Service Unavailable: One or more critical dependencies are down

### Testing Tips
- Use mock implementations for testing
- Test both success and failure paths
- Create test fixtures for common scenarios
- Use table-driven tests for comprehensive coverage 