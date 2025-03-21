# User Application

This directory contains the user application components that you can modify to customize the server. The framework is designed to hide complexity while providing a clean interface for adding your own routes and handlers.

## Key Files

- `user_router.rs` - **The main file you'll modify** to add your own routes and handlers
- `router.rs` - A thin wrapper around the core router implementation (generally shouldn't need modifications)

## How to Add Your Own Routes

The simplest way to add your own routes is to modify the `user_router.rs` file. This file contains the `UserRouter` struct with a `create_user_routes` method that returns custom routes.

### Example

```rust
// In src/app/user_router.rs
impl UserRouter {
    pub fn create_user_routes(state: Arc<AppState>) -> Router {
        // Define your routes here
        let public_routes = Router::new()
            .route("/pet/{id}", get(pet::fetch_pet_handler))
            .route("/hello", get(|| async { "Hello, World!" }))
            .route("/users", get(your_user_handler));
            
        // ... Rest of the method ...
    }
}
```

## Standard Route Groups

The framework uses four standard route groups:

1. **Public Routes** - Available without authentication
   - Example: `/pet/{id}`, `/hello`, etc.

2. **Read-Only Routes** - Requires read-only role (when auth enabled)
   - Example: `/read/pet/{id}`, etc.
   - Accessed via `/read/...` prefix

3. **Full Access Routes** - Requires full access role (when auth enabled)
   - Example: `/full/pet/{id}`, etc.
   - Accessed via `/full/...` prefix

4. **Actuator Routes** - For system monitoring and management (when auth enabled, requires admin role)
   - Example: `/actuator/health`, `/actuator/info`, etc.
   - These are primarily provided by the core framework
   - Accessed via `/actuator/...` prefix

## Creating Custom Handlers

Create your handlers in the appropriate location (e.g., `src/handlers/your_feature/`) and import them in `user_router.rs`.

Example:
```rust
// In src/handlers/your_feature/mod.rs
pub async fn your_handler() -> impl IntoResponse {
    "Your response"
}

// In src/app/user_router.rs
use crate::handlers::your_feature::your_handler;

// Add to routes
let public_routes = Router::new()
    .route("/your-path", get(your_handler));
```

## Authentication

The framework automatically applies authentication middleware to the appropriate route groups when auth is enabled in the configuration. You don't need to manually apply authentication middleware to your routes.

## Project Structure

This project follows a clear separation between core framework functionality and user-extensible code:

- `/src/core/` - Contains core framework code that should not be modified
- `/src/app/` - Contains user application code that you can customize
- Other directories like `/src/handlers/` are extensible for adding your own handlers

When extending the application, focus on adding to the user-extensible parts rather than modifying the core framework. 