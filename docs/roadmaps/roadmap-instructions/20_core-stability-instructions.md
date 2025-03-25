# Core Stability Roadmap Instructions

This document provides detailed instructions for implementing the Core Stability Roadmap with a focus on creating a Spring Boot-like developer experience in Rust. Follow these steps sequentially to ensure a smooth implementation.

## Phase 1: Fix Build Errors and Core Structure

### 1. Core Naming Standardization

#### Step 1.1: Create consistent naming pattern for core files
1. Create a detailed inventory of all files in the `src/core` directory
2. Identify files that might conflict with common user-defined filenames:
   - `router.rs` → `core_router.rs` 
   - `app.rs` → `core_app.rs`
   - `handler.rs` → `core_handler.rs`
   - `model.rs` → `core_model.rs`
   - `error.rs` → `core_error.rs`
   - Any other files that users would commonly create

#### Step 1.2: Rename files with unclear names
1. Identify files with unclear names (like `@core.rs`)
2. Determine their actual functionality by examining code
3. Rename with descriptive names that reflect their purpose
4. Example: `@core.rs` might become `core_resource_manager.rs` if it manages resources

#### Step 1.3: Update imports and references
1. For each renamed file, identify all import statements across the codebase
2. Update all references to use the new file names
3. Update all module declarations in `mod.rs` files
4. Test building the application after each major rename to catch errors early

#### Step 1.4: Create user-extensible files
1. For each core file that has user-extensible functionality, create a corresponding template in the `src/app` directory
2. Make these templates clearly import and use the core functionality
3. Add clear documentation comments explaining extension points
4. Example: `src/app/router.rs` could be a template that imports `src/core/core_router.rs` and shows how to add custom routes

#### Step 1.5: Document naming conventions
1. Create a markdown file explaining the naming conventions
2. Clearly describe which files are for core functionality vs. user extensions
3. Document how users should extend functionality without modifying core files
4. Add examples of proper usage

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