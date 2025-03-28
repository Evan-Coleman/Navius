# Application Structure

This guide explains the recommended structure for Navius applications, focusing on organization, modularity, and maintainability.

## Directory Structure

A typical Navius application follows this directory structure:

```
my-navius-app/
├── .devtools/             # Development tools and scripts
├── config/                # Configuration files
│   ├── default.yaml       # Default configuration
│   ├── development.yaml   # Development environment overrides
│   └── production.yaml    # Production environment overrides
├── src/
│   ├── main.rs            # Application entry point
│   ├── lib.rs             # Library exports
│   ├── core/              # Core domain logic
│   │   ├── models/        # Domain models
│   │   ├── services/      # Business services
│   │   └── errors/        # Domain errors
│   ├── app/               # Application layer
│   │   ├── api/           # API handlers
│   │   ├── services/      # Application services
│   │   └── state.rs       # Application state
│   ├── infra/             # Infrastructure layer
│   │   ├── database/      # Database access
│   │   ├── cache/         # Caching implementations
│   │   └── external/      # External service integrations
│   └── utils/             # Utility functions and helpers
├── tests/                 # Integration tests
│   ├── api/               # API tests
│   └── common/            # Test helpers and fixtures
└── Cargo.toml             # Project manifest
```

## Layer Separation

Navius applications follow a clean architecture approach with three main layers:

### Core Layer (`src/core/`)

The core layer contains the essential business logic and domain rules:

- **Domain Models**: Business entities that represent the core concepts
- **Services**: Core business logic that operates on domain models
- **Errors**: Domain-specific error types

The core layer should have minimal dependencies and should not depend on external frameworks or libraries.

### Application Layer (`src/app/`)

The application layer connects the core domain to the outside world:

- **API Handlers**: HTTP handlers that expose services as API endpoints
- **Application Services**: Orchestrate domain services to fulfill use cases
- **Application State**: Manages service instances and dependency injection

### Infrastructure Layer (`src/infra/`)

The infrastructure layer provides implementation details for external systems:

- **Database**: Data persistence implementations
- **Cache**: Caching mechanisms
- **External Services**: Integration with third-party services

## Module Organization

### Main Entry Point

The `main.rs` file serves as the application entry point:

```rust
// src/main.rs
use my_navius_app::app::state::AppState;
use my_navius_app::app::router;

#[tokio::main]
async fn main() {
    // Initialize application state
    let app_state = AppState::new().await;
    
    // Build the router
    let app = router::build_router(app_state);
    
    // Start the server
    // ...
}
```

### Library Exports

The `lib.rs` file exports modules for external use:

```rust
// src/lib.rs
pub mod app;
pub mod core;
pub mod infra;
pub mod utils;
```

### Application State

The `state.rs` file manages dependency injection:

```rust
// src/app/state.rs
use std::sync::Arc;

use crate::core::services::UserService;
use crate::infra::database::DatabaseConnection;
use crate::infra::cache::CacheProvider;

pub struct AppState {
    pub user_service: Arc<UserService>,
    pub db_connection: Arc<DatabaseConnection>,
    pub cache: Arc<CacheProvider>,
}

impl AppState {
    pub async fn new() -> Self {
        // Initialize services
        // ...
    }
}
```

### Router Setup

The router connects API handlers to routes:

```rust
// src/app/router.rs
use axum::Router;

use crate::app::api::user_handler;
use crate::app::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/users/:id", get(user_handler::get_user))
        .route("/users", post(user_handler::create_user))
        // ...
        .with_state(state)
}
```

## Benefits of This Structure

1. **Clear Separation of Concerns**: Each layer has a distinct responsibility
2. **Testability**: Core logic can be tested independently of infrastructure
3. **Maintainability**: Changes in one layer have minimal impact on others
4. **Scalability**: New features can be added without modifying existing code
5. **Flexibility**: Infrastructure implementations can be swapped easily

## Recommendations

1. Keep the core domain logic free from infrastructure dependencies
2. Use traits to define interfaces between layers
3. Inject dependencies through constructors rather than global state
4. Use `Arc<T>` for shared ownership of services across threads
5. Group related functionality into modules based on domain concepts, not technical aspects
6. Keep module APIs small and focused

## Related Guides

- [Service Registration](service-registration.md) for managing service instances
- [Dependency Injection](dependency-injection.md) for wiring dependencies
- [Configuration](configuration.md) for customizing application behavior 