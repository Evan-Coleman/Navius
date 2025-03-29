---
title: "Developer Onboarding Guide"
description: "# Find files in the auth component"
category: contributing
tags:
  - api
  - architecture
  - authentication
  - caching
  - database
  - development
  - documentation
  - integration
  - redis
  - testing
last_updated: March 27, 2025
version: 1.0
---
# Developer Onboarding Guide

**Updated At:** March 23, 2025

Welcome to the Navius project! This guide will help you get started as a developer on the project.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- Rust (latest stable version)
- Cargo (comes with Rust)
- Git
- Docker and Docker Compose
- VS Code (recommended) or your preferred IDE

### Setting Up Your Development Environment

1. **Clone the repository**:

```bash
git clone https://gitlab.com/navius/navius.git
cd navius
```

2. **Set up environment variables**:

Create a `.env` file in the project root with the following variables:

```
RUST_LOG=debug
CONFIG_DIR=./config
RUN_ENV=development
```

3. **Install IDE extensions**:

For VS Code, set up the recommended extensions:

```bash
mkdir -p .vscode
cp .devtools/ide/vscode/* .vscode/
```

Then restart VS Code and install the recommended extensions when prompted.

4. **Build the project**:

```bash
cargo build
```

This will also generate the API clients from OpenAPI specifications.

5. **Run the tests**:

```bash
cargo test
```

## Project Structure

Navius follows a modular architecture with a clean separation of concerns. See the [Project Navigation Guide](./) for a detailed explanation of the codebase structure.

Key directories:

- `src/core/` - Core business logic and framework functionality
- `src/app/` - User-extensible application code
- `config/` - Configuration files
- `docs/` - Documentation
- `.devtools/` - Development tools and scripts

## Development Workflow

### Running the Server

To run the development server:

```bash
.devtools/scripts/run_dev.sh
```

### Adding a New Feature

1. **Create a feature branch**:

```bash
git checkout -b feature/your-feature-name
```

2. **Implement the feature**:

- Add routes in `src/app/router.rs`
- Implement handlers in `src/app/api/`
- Add business logic in `src/app/services/`
- Add tests for your feature

3. **Run tests**:

```bash
cargo test
```

4. **Verify code style**:

```bash
cargo clippy
cargo fmt --check
```

5. **Create a merge request**:

Push your changes and create a merge request on GitLab.

### Useful Development Scripts

The project includes several helper scripts in the `.devtools/scripts/` directory:

- `run_dev.sh` - Run the development server
- `regenerate_api.sh` - Regenerate API clients from OpenAPI specs
- `navigate.sh` - Help navigate the codebase
- `verify-structure.sh` - Verify the project structure

Example usage:

```bash
# Find files in the auth component
.devtools/scripts/navigate.sh component auth

# Trace a request flow
.devtools/scripts/navigate.sh flow "GET /users"

# Verify project structure
.devtools/scripts/verify-structure.sh
```

## Debugging

VS Code launch configurations are provided for debugging:

1. Open the "Run and Debug" panel in VS Code
2. Select "Debug Navius Server" to debug the server
3. Set breakpoints in your code
4. Start debugging (F5)

For debugging tests, use the "Debug Unit Tests" configuration.

## Architecture Overview

Navius follows clean architecture principles:

1. **Core Layer** (`src/core/`):
   - Contains the core business logic
   - Independent from external frameworks
   - Defines interfaces for external dependencies

2. **Application Layer** (`src/app/`):
   - User-extensible scaffolding
   - Uses core functionality
   - Provides extension points for customization

3. **Framework Integration**:
   - Uses Axum for web framework
   - SQLx for database access
   - Redis for caching

See the [Module Dependencies Diagram](./) for a visual representation of the architecture.

## Code Examples

Here are practical examples to help you understand how to work with the Navius codebase:

### Adding an API Endpoint

Create a new handler in your application's API directory:

```rust
// src/app/api/users.rs
use axum::{
    Json,
    extract::{Path, State},
};
use std::sync::Arc;
use tracing::info;

use crate::core::{
    error::{AppError, Result},
    router::AppState,
};
use crate::app::services::user_service::UserService;

pub async fn get_user_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<User>> {
    info!("🔍 User lookup requested for ID: {}", user_id);
    
    // Access user service from state
    let user_service = &state.user_service;
    
    // Fetch user from service
    let user = user_service.get_user_by_id(&user_id).await?;
    
    // Return JSON response
    Ok(Json(user))
}
```

### Adding Routes

Register your new endpoints in the application router:

```rust
// In src/app/router.rs
use crate::app::api::users::{get_user_handler, create_user_handler};

// Inside your router function
pub fn app_routes() -> axum::Router<Arc<AppState>> {
    let router = axum::Router::new();
    
    // Public routes (no authentication)
    let public_routes = Router::new()
        .route("/users/:id", get(get_user_handler));
    
    // Full access routes (require authentication)
    let full_access_routes = Router::new()
        .route("/users", post(create_user_handler));
        
    // Combine routes
    router
        .merge(public_routes)
        .nest("/full", full_access_routes)
}
```

### Creating a Service

Implement a service for business logic:

```rust
// src/app/services/user_service.rs
use async_trait::async_trait;
use crate::core::error::Result;
use crate::app::models::user_entity::User;
use crate::core::repository::UserRepository;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user_by_id(&self, id: &str) -> Result<User>;
    async fn create_user(&self, user: User) -> Result<User>;
}

pub struct DefaultUserService {
    user_repository: Arc<dyn UserRepository>,
}

impl DefaultUserService {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl UserService for DefaultUserService {
    async fn get_user_by_id(&self, id: &str) -> Result<User> {
        self.user_repository.find_by_id(id).await
    }
    
    async fn create_user(&self, user: User) -> Result<User> {
        self.user_repository.save(user).await
    }
}
```

### Using the Cache System

Implement a handler that uses the caching system:

```rust
// src/app/api/products.rs
use axum::{
    Json,
    extract::{Path, State},
};
use std::sync::Arc;

use crate::core::{
    error::Result,
    router::AppState,
    utils::api_resource::{ApiHandlerOptions, ApiResource, create_api_handler},
};
use crate::models::Product;

impl ApiResource for Product {
    type Id = String;

    fn resource_type() -> &'static str {
        "product"
    }

    fn api_name() -> &'static str {
        "ProductAPI"
    }
}

pub async fn get_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Product>> {
    // Define the fetch function
    let fetch_fn = move |state: &Arc<AppState>, id: String| -> futures::future::BoxFuture<'static, Result<Product>> {
        let state = state.clone();
        
        Box::pin(async move {
            // Your product fetch logic here
            state.product_service.get_product(&id).await
        })
    };

    // Create a handler with caching enabled
    let handler = create_api_handler(
        fetch_fn,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
            max_retry_attempts: 3,
            cache_ttl_seconds: state.config.cache.ttl_seconds,
            detailed_logging: true,
        },
    );

    // Execute the handler
    handler(State(state), Path(id)).await
}
```

### Testing Your Code

Write unit tests for your implementations:

```rust
// In your service implementation file
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use crate::core::repository::MockUserRepository;

    #[tokio::test]
    async fn test_get_user_by_id() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        let user_id = "user-123";
        let expected_user = User {
            id: user_id.to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        mock_repo
            .expect_find_by_id()
            .with(eq(user_id))
            .returning(move |_| {
                Ok(expected_user.clone())
            });
            
        let service = DefaultUserService::new(Arc::new(mock_repo));
        
        // Act
        let result = service.get_user_by_id(user_id).await;
        
        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.name, "Test User");
    }
}
```

## Documentation

All features should be documented. The project uses the following documentation structure:

- `docs/guides/` - User guides and tutorials
- `docs/reference/` - API and technical reference
- `docs/architecture/` - Architecture documentation
- `docs/contributing/` - Contribution guidelines
- `docs/roadmaps/` - Development roadmaps

## Getting Help

If you need help with the codebase:

1. Consult the [Project Navigation Guide](./)
2. Use the navigation scripts to explore the codebase
3. Read the documentation in the `docs/` directory
4. Reach out to the team on the project's communication channels

## Related Documents
- [Contributing Guide](./) - How to contribute to the project
- [Development Setup](./) - Setting up your development environment
