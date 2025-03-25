# Spring Boot vs Navius Developer Experience

This document illustrates the similarities between Spring Boot and Navius frameworks, showing how Java Spring Boot developers can easily transition to Rust using Navius.

## Application Bootstrap

### Spring Boot (Java)

```java
@SpringBootApplication
public class DemoApplication {
    public static void main(String[] args) {
        SpringApplication.run(DemoApplication.class, args);
    }
}
```

### Navius (Rust)

```rust
fn main() {
    NaviusApp::new()
        .with_default_config()
        .with_actuator()
        .with_swagger()
        .run();
}
```

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
```

### Navius (Rust)

```rust
// In src/app/router.rs or similar
pub fn configure_routes(router: &mut Router) {
    router.route("/health", get(health_handler));
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "UP" }))
}
```

### Extending the Health Endpoint in Navius

```rust
// Custom health implementation with more details
async fn custom_health_handler() -> impl IntoResponse {
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

// Register in your router
pub fn configure_routes(router: &mut Router) {
    router.route("/health", get(custom_health_handler));
}
```

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
```

### Navius (Rust)

```rust
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
```

## Service Layer

### Spring Boot (Java)

```java
@Service
public class UserServiceImpl implements UserService {
    @Autowired
    private UserRepository userRepository;
    
    @Autowired
    private PasswordEncoder passwordEncoder;
    
    @Override
    public List<User> findAll() {
        return userRepository.findAll();
    }
    
    @Override
    public Optional<User> findById(UUID id) {
        return userRepository.findById(id);
    }
    
    @Override
    @Transactional
    public User create(UserRequest request) {
        if (userRepository.existsByEmail(request.getEmail())) {
            throw new BadRequestException("Email already in use");
        }
        
        User user = new User();
        user.setName(request.getName());
        user.setEmail(request.getEmail());
        user.setPassword(passwordEncoder.encode(request.getPassword()));
        
        return userRepository.save(user);
    }
}
```

### Navius (Rust)

```rust
#[service]
pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
    password_encoder: Arc<dyn PasswordEncoder>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        self.repository.find_all().await
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        self.repository.find_by_id(id).await
    }
    
    #[transactional]
    async fn create(&self, request: UserRequest) -> Result<User, AppError> {
        if self.repository.exists_by_email(&request.email).await? {
            return Err(AppError::bad_request("Email already in use"));
        }
        
        let user = User {
            id: Uuid::new_v4(),
            name: request.name,
            email: request.email,
            password: self.password_encoder.encode(&request.password).await?,
            created_at: Utc::now(),
        };
        
        self.repository.save(user).await
    }
}
```

## Repository Layer

### Spring Boot (Java)

```java
@Repository
public interface UserRepository extends JpaRepository<User, UUID> {
    boolean existsByEmail(String email);
    
    @Query("SELECT u FROM User u WHERE u.email = :email")
    Optional<User> findByEmail(String email);
}
```

### Navius (Rust)

```rust
#[repository]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn save(&self, user: User) -> Result<User, AppError>;
    async fn exists_by_email(&self, email: &str) -> Result<bool, AppError>;
    
    #[query("SELECT * FROM users WHERE email = $1")]
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
}
```

## Configuration Properties

### Spring Boot (Java)

```java
@Configuration
@ConfigurationProperties(prefix = "app.security")
public class SecurityProperties {
    private boolean enabled = true;
    private int tokenExpirationMinutes = 60;
    private String issuer = "navius";
    // Getters and setters
}
```

### Navius (Rust)

```rust
#[derive(Debug, Deserialize, ConfigurationProperties)]
#[config(prefix = "app.security")]
pub struct SecurityConfig {
    #[config(default = true)]
    pub enabled: bool,
    
    #[config(default = 60)]
    pub token_expiration_minutes: i32,
    
    #[config(default = "navius")]
    pub issuer: String,
}
```

## Caching

### Spring Boot (Java)

```java
@Service
public class UserServiceImpl implements UserService {
    // ...
    
    @Override
    @Cacheable(value = "users", key = "#id")
    public Optional<User> findById(UUID id) {
        return userRepository.findById(id);
    }
    
    @Override
    @CacheEvict(value = "users", key = "#result.id")
    public User create(UserRequest request) {
        // Implementation
    }
}
```

### Navius (Rust)

```rust
#[async_trait]
impl UserService for UserServiceImpl {
    #[cacheable(cache = "users", key = "{id}")]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        self.repository.find_by_id(id).await
    }
    
    #[cache_evict(cache = "users", key = "{result.id}")]
    async fn create(&self, request: UserRequest) -> Result<User, AppError> {
        // Implementation
    }
}
```

## Validation

### Spring Boot (Java)

```java
public class UserRequest {
    @NotBlank(message = "Name is required")
    private String name;
    
    @NotBlank(message = "Email is required")
    @Email(message = "Email must be valid")
    private String email;
    
    @NotBlank(message = "Password is required")
    @Size(min = 8, message = "Password must be at least 8 characters")
    private String password;
    
    // Getters and setters
}
```

### Navius (Rust)

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct UserRequest {
    #[validate(required(message = "Name is required"))]
    pub name: String,
    
    #[validate(required(message = "Email is required"))]
    #[validate(email(message = "Email must be valid"))]
    pub email: String,
    
    #[validate(required(message = "Password is required"))]
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}
```

## Error Handling

### Spring Boot (Java)

```java
@RestControllerAdvice
public class GlobalExceptionHandler {
    @ExceptionHandler(ResourceNotFoundException.class)
    public ResponseEntity<ErrorResponse> handleNotFound(ResourceNotFoundException ex) {
        return ResponseEntity
            .status(HttpStatus.NOT_FOUND)
            .body(new ErrorResponse("NOT_FOUND", ex.getMessage()));
    }
    
    @ExceptionHandler(BadRequestException.class)
    public ResponseEntity<ErrorResponse> handleBadRequest(BadRequestException ex) {
        return ResponseEntity
            .status(HttpStatus.BAD_REQUEST)
            .body(new ErrorResponse("BAD_REQUEST", ex.getMessage()));
    }
}
```

### Navius (Rust)

```rust
#[exception_handler]
pub async fn global_error_handler(err: AppError) -> impl IntoResponse {
    let (status, error_response) = match err {
        AppError::NotFound(message) => (
            StatusCode::NOT_FOUND,
            ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message,
            }
        ),
        AppError::BadRequest(message) => (
            StatusCode::BAD_REQUEST,
            ErrorResponse {
                code: "BAD_REQUEST".to_string(),
                message,
            }
        ),
        // Other error mappings
    };
    
    (status, Json(error_response))
}
```

## Application Properties/Configuration

### Spring Boot (Java)

```properties
# application.properties or application.yml
spring.application.name=demo-app
server.port=8080

spring.datasource.url=jdbc:postgresql://localhost:5432/demoapp
spring.datasource.username=postgres
spring.datasource.password=password

app.security.enabled=true
app.security.token-expiration-minutes=120
```

### Navius (Rust)

```yaml
# config/default.yaml
app:
  name: demo-app
  server:
    port: 8080
    
database:
  url: postgres://postgres:password@localhost:5432/demoapp
  
app.security:
  enabled: true
  token_expiration_minutes: 120
```

## Benefits of Navius for Spring Boot Developers

1. **Familiar Patterns**: Navius implements Spring Boot's core patterns like controllers, services, and repositories
2. **Macro-Based Annotations**: Similar to Spring annotations but with Rust's compile-time safety
3. **Enhanced Performance**: Get the speed and resource efficiency of Rust with Spring Boot's developer experience
4. **Type Safety**: Benefit from Rust's strong type system while using familiar Spring patterns
5. **Memory Safety**: Eliminate whole classes of runtime errors with Rust's ownership model
6. **Async by Default**: First-class support for asynchronous programming with similar simplicity
7. **Production Ready**: Built-in support for metrics, health checks, and observability 