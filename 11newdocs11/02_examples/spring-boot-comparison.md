---
title: Spring Boot vs Navius Framework Comparison
description: Comprehensive comparison between Spring Boot (Java) and Navius (Rust) frameworks to help Java developers transition to Rust
category: examples
tags:
  - comparison
  - spring-boot
  - java
  - rust
  - migration
  - web-framework
related:
  - 02_examples/rest-api-example.md
  - 02_examples/dependency-injection-example.md
  - 01_getting_started/first-steps.md
last_updated: March 27, 2025
version: 1.0
status: stable
---

# Spring Boot vs Navius Developer Experience

This document illustrates the similarities between Spring Boot and Navius frameworks, showing how Java Spring Boot developers can easily transition to Rust using Navius.

## Overview

Navius was designed with Spring Boot developers in mind, providing a familiar programming model while leveraging Rust's performance and safety benefits. This guide highlights the parallel patterns between the two frameworks.

## Quick Navigation

- [Application Bootstrap](#application-bootstrap)
- [Module Organization](#module-organization)
- [Simple Health Endpoint](#simple-health-endpoint)
- [REST Controller](#rest-controller)
- [Service Layer](#service-layer)
- [Dependency Injection](#dependency-injection)
- [Configuration](#configuration)
- [Database Access](#database-access)
- [Testing](#testing)
- [Common Design Patterns](#common-design-patterns)

## Application Bootstrap

### Spring Boot (Java)

```java
@SpringBootApplication
public class DemoApplication {
    public static void main(String[] args) {
        SpringApplication.run(DemoApplication.class, args);
    }
}
```rust

### Navius (Rust)

```rust
fn main() {
    NaviusApp::new()
        .with_default_config()
        .with_actuator()
        .with_swagger()
        .run();
}
```rust

### Key Similarities

- Both frameworks provide a single entry point for application bootstrap
- Fluent API for configuration
- Convention over configuration approach
- Built-in support for common production-ready features

## Module Organization

### Spring Boot (Java)

Java Spring Boot follows a package-based organization where components are organized by feature or layer:

```rust
com.example.demo/
├── DemoApplication.java
├── config/
│   └── SecurityConfig.java
├── controller/
│   └── UserController.java
├── service/
│   └── UserService.java
├── repository/
│   └── UserRepository.java
└── model/
    └── User.java
```rust

### Navius (Rust)

Navius uses a flat module structure with centralized declarations in lib.rs:

```rust
// In lib.rs
mod core {
    pub mod router {
        pub mod core_router;
        pub mod core_app_router;
        
        pub use core_router::*;
        pub use core_app_router::*;
    }
    
    pub mod models { /* ... */ }
    pub mod handlers { /* ... */ }
}

mod app {
    pub mod api { /* ... */ }
    pub mod services { /* ... */ }
}

// Directory structure
src/
├── lib.rs
├── main.rs
├── core/
│   ├── router/
│   │   ├── core_router.rs
│   │   └── core_app_router.rs
│   └── models/
│       └── core_response.rs
└── app/
    └── api/
        └── examples.rs
```rust

This approach eliminates the need for mod.rs files in each directory, reducing file clutter and making the module structure more immediately apparent in a single location.

### Key Similarities

- Logical separation of concerns (controllers, services, repositories)
- Clear distinction between framework components and application code
- Support for modular architecture
- Ability to organize by feature or by layer

## Simple Health Endpoint

### Spring Boot (Java)

```java
@RestController
public class SimpleHealthController {
    @GetMapping("/health")
    public ResponseEntity<Map<String, String>> health() {
        Map<String, String> response = new HashMap<>();
        response.put("status", "UP");
        return ResponseEntity.ok(response);
    }
}
```rust

### Navius (Rust)

```rust
// In src/app/router.rs - User's custom router implementation
use navius::core::core_router::{Router, get};
use navius::core::core_response::IntoResponse;
use axum::Json;
use serde_json::json;

// Define your custom router configuration
pub fn configure_routes(router: &mut Router) {
    router.route("/health", get(health_handler));
}

// Your custom health endpoint implementation
async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "UP" }))
}

// Register your routes in main.rs
fn main() {
    NaviusApp::new()
        .with_default_config()
        .with_routes(configure_routes)
        .run();
}
```rust

### Extending the Health Endpoint in Navius

```rust
// In src/app/health.rs - User's custom health implementation
use navius::core::core_router::{Router, get};
use navius::core::core_response::IntoResponse;
use axum::Json;
use serde_json::json;

// Custom health implementation with more details
pub async fn custom_health_handler() -> impl IntoResponse {
    // Custom checks you might want to add
    let db_status = check_database().await;
    
    Json(json!({
        "status": db_status ? "UP" : "DOWN",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "details": {
            "database": db_status,
            "version": env!("CARGO_PKG_VERSION")
        }
    }))
}

// Register in your router (src/app/router.rs)
pub fn configure_routes(router: &mut Router) {
    router.route("/health", get(custom_health_handler));
}
```rust

### Key Similarities

- Similar endpoint declaration syntax
- JSON response generation pattern
- Endpoint registration mechanism
- Support for custom health information
- Built-in health check system

## REST Controller

### Spring Boot (Java)

```java
@RestController
@RequestMapping("/api/users")
public class UserController {
    @Autowired
    private UserService userService;
    
    @GetMapping
    public List<User> getAllUsers() {
        return userService.findAll();
    }
    
    @GetMapping("/{id}")
    public ResponseEntity<User> getUserById(@PathVariable UUID id) {
        return userService.findById(id)
            .map(ResponseEntity::ok)
            .orElse(ResponseEntity.notFound().build());
    }
    
    @PostMapping
    public ResponseEntity<User> createUser(@RequestBody @Valid UserRequest request) {
        User user = userService.create(request);
        return ResponseEntity
            .created(URI.create("/api/users/" + user.getId()))
            .body(user);
    }
}
```rust

### Navius (Rust)

```rust
// In src/app/controllers/user_controller.rs
use navius::core::core_macros::{api_controller, api_routes, request_mapping, get, post};
use navius::core::core_error::AppError;
use navius::app::services::UserService;
use axum::{Json, extract::Path};
use uuid::Uuid;
use std::sync::Arc;

#[api_controller]
#[request_mapping("/api/users")]
pub struct UserController {
    service: Arc<dyn UserService>,
}

#[api_routes]
impl UserController {
    #[get("")]
    async fn get_all_users(&self) -> Result<Json<Vec<User>>, AppError> {
        let users = self.service.find_all().await?;
        Ok(Json(users))
    }
    
    #[get("/:id")]
    async fn get_user_by_id(&self, Path(id): Path<Uuid>) -> Result<Json<User>, AppError> {
        match self.service.find_by_id(id).await? {
            Some(user) => Ok(Json(user)),
            None => Err(AppError::not_found("User not found"))
        }
    }
    
    #[post("")]
    async fn create_user(&self, Json(request): Json<UserRequest>) -> Result<(StatusCode, Json<User>), AppError> {
        // Validation happens via a derive macro on UserRequest
        let user = self.service.create(request).await?;
        Ok((StatusCode::CREATED, Json(user)))
    }
}
```rust

### Key Similarities

- Controllers organized by resource
- Base path mapping for resource collections
- Similar HTTP method annotations
- Path parameter extraction
- Request body validation
- Structured error handling
- Status code management

## Service Layer

### Spring Boot (Java)

```java
@Service
public class UserServiceImpl implements UserService {
    @Autowired
    private UserRepository userRepository;
    
    @Override
    public List<User> findAll() {
        return userRepository.findAll();
    }
    
    @Override
    public Optional<User> findById(UUID id) {
        return userRepository.findById(id);
    }
    
    @Override
    public User create(UserRequest request) {
        User user = new User();
        user.setName(request.getName());
        user.setEmail(request.getEmail());
        return userRepository.save(user);
    }
}
```rust

### Navius (Rust)

```rust
// In src/app/services/user_service.rs
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use navius::core::core_error::AppError;
use crate::app::repositories::UserRepository;
use crate::app::models::{User, UserRequest};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn create(&self, request: UserRequest) -> Result<User, AppError>;
}

pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
}

impl UserServiceImpl {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        self.repository.find_all().await
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        self.repository.find_by_id(id).await
    }
    
    async fn create(&self, request: UserRequest) -> Result<User, AppError> {
        let user = User {
            id: Uuid::new_v4(),
            name: request.name,
            email: request.email,
            created_at: chrono::Utc::now(),
        };
        
        self.repository.save(user).await
    }
}
```rust

### Key Similarities

- Services implement interfaces/traits for testability
- Clear method contracts
- Separation of business logic from persistence
- Similar CRUD operation patterns
- Error propagation patterns

## Dependency Injection

### Spring Boot (Java)

```java
// Component definition
@Service
public class EmailService {
    public void sendEmail(String to, String subject, String body) {
        // Implementation
    }
}

// Component usage
@Service
public class NotificationService {
    private final EmailService emailService;
    
    @Autowired
    public NotificationService(EmailService emailService) {
        this.emailService = emailService;
    }
    
    public void notifyUser(User user, String message) {
        emailService.sendEmail(user.getEmail(), "Notification", message);
    }
}

// Multiple implementations with qualifier
@Service("simpleEmail")
public class SimpleEmailService implements EmailService { /*...*/ }

@Service("advancedEmail")
public class AdvancedEmailService implements EmailService { /*...*/ }

// Usage with qualifier
@Service
public class NotificationService {
    private final EmailService emailService;
    
    @Autowired
    public NotificationService(@Qualifier("advancedEmail") EmailService emailService) {
        this.emailService = emailService;
    }
    // ...
}
```rust

### Navius (Rust)

```rust
// Service registry setup
use navius::core::core_registry::{ServiceRegistry, ServiceProvider};
use std::sync::Arc;

// Setting up the dependencies
fn configure_services(registry: &mut ServiceRegistry) {
    // Register the repository
    let repository = Arc::new(UserRepositoryImpl::new());
    registry.register::<dyn UserRepository>(repository);
    
    // Register the service, with dependency on repository
    let repository = registry.resolve::<dyn UserRepository>().unwrap();
    let service = Arc::new(UserServiceImpl::new(repository));
    registry.register::<dyn UserService>(service);
}

// In main.rs
fn main() {
    NaviusApp::new()
        .with_default_config()
        .with_services(configure_services)
        .run();
}

// Usage in controllers
#[api_controller]
#[request_mapping("/api/users")]
pub struct UserController {
    // Automatically injected by Navius
    service: Arc<dyn UserService>,
}

// Multiple implementations with named registrations
fn configure_services(registry: &mut ServiceRegistry) {
    // Register different implementations
    let simple_email = Arc::new(SimpleEmailService::new());
    let advanced_email = Arc::new(AdvancedEmailService::new());
    
    registry.register_named::<dyn EmailService>("simple", simple_email);
    registry.register_named::<dyn EmailService>("advanced", advanced_email);
    
    // Resolve named service
    let email_service = registry.resolve_named::<dyn EmailService>("advanced").unwrap();
    let notification_service = Arc::new(NotificationServiceImpl::new(email_service));
    registry.register::<dyn NotificationService>(notification_service);
}
```rust

### Key Similarities

- Container-managed dependency injection
- Constructor-based injection
- Support for interface/trait-based programming
- Multiple implementations with qualifiers/named registration
- Singleton lifecycle by default
- Ability to resolve dependencies from the container

## Configuration

### Spring Boot (Java)

```java
// application.properties or application.yml
server.port=8080
app.name=MyApplication
app.feature.enabled=true
database.url=jdbc:postgresql://localhost:5432/mydb
database.username=user
database.password=pass

// Configuration class
@Configuration
@ConfigurationProperties(prefix = "database")
public class DatabaseConfig {
    private String url;
    private String username;
    private String password;
    
    // Getters and setters
    
    @Bean
    public DataSource dataSource() {
        HikariConfig config = new HikariConfig();
        config.setJdbcUrl(url);
        config.setUsername(username);
        config.setPassword(password);
        return new HikariDataSource(config);
    }
}

// Using environment-specific configurations
// application-dev.properties, application-prod.properties
// Activated with: spring.profiles.active=dev
```rust

### Navius (Rust)

```rust
// config/default.yaml
server:
  port: 8080
app:
  name: MyApplication
  feature:
    enabled: true
database:
  url: postgres://localhost:5432/mydb
  username: user
  password: pass

// Configuration struct
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub app: ApplicationConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

// Loading configuration
impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", env::var("NAVIUS_ENV").unwrap_or_else(|_| "development".into()))).required(false))
            .add_source(Environment::with_prefix("NAVIUS").separator("__"))
            .build()?;
            
        builder.try_deserialize()
    }
}

// Using configuration
fn main() {
    let config = AppConfig::load().expect("Failed to load configuration");
    
    NaviusApp::new()
        .with_config(config)
        .run();
}
```rust

### Key Similarities

- External configuration files
- Environment-specific configurations
- Type-safe configuration objects
- Hierarchical configuration structure
- Environment variable overrides
- Default values for missing properties

## Database Access

### Spring Boot (Java)

```java
// Entity definition
@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue
    private UUID id;
    
    @Column(nullable = false)
    private String name;
    
    @Column(nullable = false, unique = true)
    private String email;
    
    @Column(name = "created_at")
    private LocalDateTime createdAt;
    
    // Getters and setters
}

// Repository definition
@Repository
public interface UserRepository extends JpaRepository<User, UUID> {
    List<User> findByNameContaining(String namePart);
    
    @Query("SELECT u FROM User u WHERE u.email = :email")
    Optional<User> findByEmail(@Param("email") String email);
}

// Usage
@Service
public class UserService {
    private final UserRepository repository;
    
    @Autowired
    public UserService(UserRepository repository) {
        this.repository = repository;
    }
    
    public List<User> findByName(String name) {
        return repository.findByNameContaining(name);
    }
}
```rust

### Navius (Rust)

```rust
// Entity definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

// Repository definition
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn save(&self, user: User) -> Result<User, AppError>;
}

// Implementation
pub struct UserRepositoryImpl {
    pool: Arc<PgPool>,
}

impl UserRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| AppError::database_error(e))?;
            
        Ok(users)
    }
    
    async fn find_by_name(&self, name: &str) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE name LIKE $1")
            .bind(format!("%{}%", name))
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| AppError::database_error(e))?;
            
        Ok(users)
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| AppError::database_error(e))?;
            
        Ok(user)
    }
    
    // Other methods...
}
```rust

### Key Similarities

- Entity-based database modeling
- Repository pattern for data access
- Support for complex queries
- Connection pooling
- Parameter binding for SQL safety
- Type-safe result mapping

## Testing

### Spring Boot (Java)

```java
// Service unit test
@RunWith(MockitoJUnitRunner.class)
public class UserServiceTest {
    @Mock
    private UserRepository userRepository;
    
    @InjectMocks
    private UserServiceImpl userService;
    
    @Test
    public void findById_shouldReturnUser_whenUserExists() {
        // Arrange
        UUID id = UUID.randomUUID();
        User user = new User();
        user.setId(id);
        user.setName("Test User");
        
        when(userRepository.findById(id)).thenReturn(Optional.of(user));
        
        // Act
        Optional<User> result = userService.findById(id);
        
        // Assert
        assertTrue(result.isPresent());
        assertEquals("Test User", result.get().getName());
        verify(userRepository).findById(id);
    }
}

// Controller integration test
@RunWith(SpringRunner.class)
@WebMvcTest(UserController.class)
public class UserControllerTest {
    @Autowired
    private MockMvc mockMvc;
    
    @MockBean
    private UserService userService;
    
    @Test
    public void getUserById_shouldReturnUser_whenUserExists() throws Exception {
        // Arrange
        UUID id = UUID.randomUUID();
        User user = new User();
        user.setId(id);
        user.setName("Test User");
        
        when(userService.findById(id)).thenReturn(Optional.of(user));
        
        // Act & Assert
        mockMvc.perform(get("/api/users/{id}", id))
            .andExpect(status().isOk())
            .andExpect(jsonPath("$.id").value(id.toString()))
            .andExpect(jsonPath("$.name").value("Test User"));
    }
}
```rust

### Navius (Rust)

```rust
// Service unit test
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    
    mock! {
        UserRepository {}
        
        #[async_trait]
        impl UserRepository for UserRepository {
            async fn find_all(&self) -> Result<Vec<User>, AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
            async fn save(&self, user: User) -> Result<User, AppError>;
        }
    }
    
    #[tokio::test]
    async fn find_by_id_should_return_user_when_user_exists() {
        // Arrange
        let id = Uuid::new_v4();
        let user = User {
            id,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        let mut repository = MockUserRepository::new();
        repository.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Ok(Some(user.clone())));
            
        let service = UserServiceImpl::new(Arc::new(repository));
        
        // Act
        let result = service.find_by_id(id).await;
        
        // Assert
        assert!(result.is_ok());
        let user_opt = result.unwrap();
        assert!(user_opt.is_some());
        let found_user = user_opt.unwrap();
        assert_eq!(found_user.name, "Test User");
    }
}

// Controller integration test
#[cfg(test)]
mod tests {
    use super::*;
    use navius::core::core_test::TestApp;
    use axum::http::StatusCode;
    
    #[tokio::test]
    async fn get_user_by_id_should_return_user_when_user_exists() {
        // Arrange
        let id = Uuid::new_v4();
        let user = User {
            id,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        let app = TestApp::new()
            .with_mock::<dyn UserService, _>(move |mut mock| {
                mock.expect_find_by_id()
                    .with(eq(id))
                    .returning(move |_| Ok(Some(user.clone())));
                mock
            })
            .build();
        
        // Act
        let response = app.get(&format!("/api/users/{}", id)).await;
        
        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        
        let body: User = response.json().await;
        assert_eq!(body.id, id);
        assert_eq!(body.name, "Test User");
    }
}
```rust

### Key Similarities

- Unit testing with mocks
- Integration testing with test clients
- Clear Arrange-Act-Assert pattern
- Declarative test case structure
- Mock expectations and verifications
- Support for testing async code
- JSON response validation

## Common Design Patterns

Both Spring Boot and Navius encourage the use of similar design patterns, making the transition between frameworks more intuitive:

### Factory Pattern

**Spring Boot**:
```java
@Component
public class PaymentMethodFactory {
    @Autowired
    private List<PaymentProcessor> processors;
    
    public PaymentProcessor getProcessor(String type) {
        return processors.stream()
                .filter(p -> p.supports(type))
                .findFirst()
                .orElseThrow(() -> new IllegalArgumentException("No processor for type: " + type));
    }
}
```rust

**Navius**:
```rust
pub struct PaymentMethodFactory {
    processors: HashMap<String, Arc<dyn PaymentProcessor>>,
}

impl PaymentMethodFactory {
    pub fn new(registry: &ServiceRegistry) -> Self {
        let processors = registry.resolve_all::<dyn PaymentProcessor>();
        let mut map = HashMap::new();
        
        for processor in processors {
            map.insert(processor.get_type().to_string(), processor);
        }
        
        Self { processors: map }
    }
    
    pub fn get_processor(&self, type_: &str) -> Result<Arc<dyn PaymentProcessor>, AppError> {
        self.processors.get(type_)
            .cloned()
            .ok_or_else(|| AppError::not_found(&format!("No processor for type: {}", type_)))
    }
}
```rust

### Observer Pattern

**Spring Boot**:
```java
// With Spring Events
@Component
public class OrderService {
    @Autowired
    private ApplicationEventPublisher eventPublisher;
    
    public Order createOrder(OrderRequest request) {
        Order order = // create order
        
        // Publish event for observers
        eventPublisher.publishEvent(new OrderCreatedEvent(order));
        
        return order;
    }
}

@Component
public class EmailNotifier {
    @EventListener
    public void onOrderCreated(OrderCreatedEvent event) {
        // Send email notification
    }
}
```rust

**Navius**:
```rust
// With Navius Event Bus
pub struct OrderService {
    event_bus: Arc<EventBus>,
}

impl OrderService {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
    
    pub async fn create_order(&self, request: OrderRequest) -> Result<Order, AppError> {
        let order = // create order
        
        // Publish event for observers
        self.event_bus.publish(OrderCreatedEvent::new(order.clone())).await?;
        
        Ok(order)
    }
}

pub struct EmailNotifier {
    // ...
}

impl EventHandler<OrderCreatedEvent> for EmailNotifier {
    async fn handle(&self, event: &OrderCreatedEvent) -> Result<(), AppError> {
        // Send email notification
        Ok(())
    }
}

// Register in service configuration
fn configure_services(registry: &mut ServiceRegistry) {
    let event_bus = registry.resolve::<EventBus>().unwrap();
    
    let notifier = Arc::new(EmailNotifier::new());
    event_bus.subscribe::<OrderCreatedEvent, _>(notifier);
}
```rust

## Migration Tips for Spring Boot Developers

When transitioning from Spring Boot to Navius, keep these key points in mind:

1. **Understand Rust Ownership**: Rust's ownership model differs from Java's garbage collection. Use `Arc<T>` for shared ownership where needed.

2. **Trait Objects Instead of Interfaces**: Use Rust traits (with `dyn` for dynamic dispatch) as you would Java interfaces.

3. **Async/Await vs Blocking**: Navius uses async/await for concurrency, not threads like Spring Boot. Add `.await` to async function calls.

4. **Error Handling with Result**: Replace exceptions with Rust's `Result` type for robust error handling.

5. **Explicit Dependencies**: Navius requires explicit dependency registration, while Spring Boot has more automatic component scanning.

6. **Immutable by Default**: Embrace Rust's immutability by default instead of the mutable objects common in Java.

7. **Testing Approaches**: Both frameworks support mocking, but Rust tests use different libraries like `mockall` instead of Mockito.

8. **Configuration Loading**: Both frameworks support structured configuration, but with different approaches to deserialization.

9. **Database Access**: Replace Spring Data repositories with explicit SQL in Navius using SQLx or Diesel.

10. **Macros vs Annotations**: Use Rust macros like `#[api_controller]` similarly to Spring's `@RestController`.

## Conclusion

Navius provides a familiar development experience for Spring Boot developers while leveraging Rust's performance, memory safety, and concurrency benefits. The similar architectural patterns and programming model allow for a smoother transition between the two frameworks.

By following the patterns demonstrated in this comparison guide, Spring Boot developers can quickly become productive with Navius and build high-performance, type-safe web applications with many of the same conveniences they're accustomed to.

For more detailed examples, refer to:
- [REST API Example](./rest-api-example.md)
- [Dependency Injection Example](./dependency-injection-example.md)
- [Getting Started with Navius](../01_getting_started/README.md)

## Troubleshooting Common Issues

### "Cannot move out of borrowed content" errors

**Spring Boot approach**: In Java, you can freely copy and pass objects.

**Navius solution**: Use `clone()` for objects that implement `Clone`, or use references when possible.

### Type parameter issues with trait objects

**Spring Boot approach**: Java generics erase at runtime.

**Navius solution**: Use `dyn Trait` for trait objects and be mindful of type parameter constraints.

### Async confusion

**Spring Boot approach**: Blocking code is common.

**Navius solution**: Use `.await` on all async function calls and ensure proper async function signatures.

### Missing type information

**Spring Boot approach**: Type inference is less strict.

**Navius solution**: Add type annotations when the compiler can't infer types.