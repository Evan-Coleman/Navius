# Navius Dependency Injection

A lightweight dependency injection system for Navius applications, inspired by the spring-rs framework.

## Features

- **Component Registry**: Type-safe dependency resolution
- **Multiple Scopes**: Singleton, Prototype, Request, and Session scopes
- **Lifecycle Hooks**: Both synchronous and asynchronous lifecycle management
- **Qualifiers**: Support for disambiguating components of the same type
- **Factory Support**: Factory-based component creation
- **Autowiring**: Automatic dependency resolution
- **Comprehensive Error Handling**: Clear error messages for common DI issues

## Usage

```rust
use navius_di::{ComponentRegistry, ComponentScope, Result};

// Define your components
struct DatabaseService {
    connection_string: String,
}

impl DatabaseService {
    fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    fn query(&self, sql: &str) -> String {
        format!("Executing '{}' on connection {}", sql, self.connection_string)
    }
}

struct UserService {
    db: DatabaseService,
}

impl UserService {
    fn new(db: DatabaseService) -> Self {
        Self { db }
    }

    fn get_user(&self, id: &str) -> String {
        self.db.query(&format!("SELECT * FROM users WHERE id = '{}'", id))
    }
}

fn main() -> Result<()> {
    // Create a component registry
    let registry = ComponentRegistry::new();

    // Register a database service as a singleton
    registry.register_with_factory(
        || DatabaseService::new("jdbc:postgresql://localhost:5432/mydb".to_string()),
        ComponentScope::Singleton,
    );

    // Register a user service that depends on the database service
    registry.register_with_factory(
        || {
            let db = registry.get::<DatabaseService>().unwrap();
            UserService::new((*db).clone())
        },
        ComponentScope::Prototype,
    );

    // Get the user service and use it
    let user_service = registry.get::<UserService>()?;
    let user = user_service.get_user("user-1");
    println!("User: {}", user);

    Ok(())
}
```

## Component Scopes

- **Singleton**: Components are instantiated once and shared across the application
- **Prototype**: Components are instantiated each time they are requested
- **Request**: Components are instantiated for each request (useful in web applications)
- **Session**: Components are instantiated for each session (useful in web applications)

## Lifecycle Hooks

Components can implement lifecycle hooks to be notified when they are created, initialized, or destroyed:

```rust
use navius_di::{Lifecycle, LifecyclePhase, Result};

impl Lifecycle for DatabaseService {
    fn on_create(&self) -> Result<()> {
        println!("DatabaseService created");
        Ok(())
    }

    fn on_initialize(&self) -> Result<()> {
        println!("DatabaseService initialized");
        Ok(())
    }

    fn on_destroy(&self) -> Result<()> {
        println!("DatabaseService destroyed");
        Ok(())
    }
}
```

Async lifecycle hooks are also supported:

```rust
use navius_di::{AsyncLifecycle, LifecyclePhase, Result};

#[async_trait::async_trait]
impl AsyncLifecycle for DatabaseService {
    async fn on_create_async(&self) -> Result<()> {
        println!("DatabaseService created asynchronously");
        Ok(())
    }

    async fn on_initialize_async(&self) -> Result<()> {
        println!("DatabaseService initialized asynchronously");
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        println!("DatabaseService destroyed asynchronously");
        Ok(())
    }
}
```

## Qualifiers

Qualifiers can be used to disambiguate components of the same type:

```rust
// Register multiple database services with different qualifiers
registry.register_with_qualifier(
    DatabaseService::new("jdbc:postgresql://localhost:5432/users".to_string()),
    "users-db",
)?;

registry.register_with_qualifier(
    DatabaseService::new("jdbc:postgresql://localhost:5432/products".to_string()),
    "products-db",
)?;

// Get a specific database service by qualifier
let users_db = registry.get_by_qualifier::<DatabaseService>("users-db")?;
let products_db = registry.get_by_qualifier::<DatabaseService>("products-db")?;
```

## Examples

For more complete examples, see the [examples directory](./examples/).

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT) 