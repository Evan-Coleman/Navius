//! Basic Integration Example
//!
//! This example demonstrates the integration of multiple Navius crates:
//! - navius-core: Component registry and dependency injection
//! - navius-http: HTTP server and routing
//! - navius-auth: Authentication and authorization

use async_trait::async_trait;
use navius_auth::auth::{AuthProvider, AuthResult, Credentials, UserInfo};
use navius_core::{
    config::Config,
    di::{Application, ApplicationBuilder, AsyncLifecycle, ComponentScope, Environment, Lifecycle},
    error::{Error, Result},
};
use navius_http::{
    routing::{Route, Router},
    server::{HttpServer, ServerConfig},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Simple in-memory user store for demonstration
#[derive(Debug, Clone)]
struct InMemoryAuthProvider {
    users: Arc<Mutex<HashMap<String, UserInfo>>>,
    is_initialized: bool,
}

impl InMemoryAuthProvider {
    fn new() -> Self {
        let mut users = HashMap::new();

        // Add some demo users
        users.insert(
            "user1".to_string(),
            UserInfo {
                id: "user1".to_string(),
                username: "user1".to_string(),
                email: "user1@example.com".to_string(),
                roles: vec!["user".to_string()],
                metadata: HashMap::new(),
            },
        );

        users.insert(
            "admin".to_string(),
            UserInfo {
                id: "admin".to_string(),
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                roles: vec!["user".to_string(), "admin".to_string()],
                metadata: HashMap::new(),
            },
        );

        Self {
            users: Arc::new(Mutex::new(users)),
            is_initialized: false,
        }
    }
}

impl Lifecycle for InMemoryAuthProvider {
    fn on_initialize(&self) -> Result<()> {
        println!("Initializing InMemoryAuthProvider");

        // In a real implementation, this might load users from a database
        // For this example, we just set the initialized flag
        let mut this = self as *const Self as *mut Self;
        unsafe {
            (*this).is_initialized = true;
        }

        Ok(())
    }

    fn on_destroy(&self) -> Result<()> {
        println!("Destroying InMemoryAuthProvider");
        Ok(())
    }
}

impl AuthProvider for InMemoryAuthProvider {
    fn authenticate(&self, credentials: &Credentials) -> AuthResult<UserInfo> {
        let users = self.users.lock().unwrap();

        // For this simple example, we only support username/password auth
        match credentials {
            Credentials::UsernamePassword { username, password } => {
                // In a real implementation, we would verify the password
                // For this example, we just check if the user exists
                if let Some(user_info) = users.get(username) {
                    // In a real app, we would validate the password here
                    if password == "password" {
                        Ok(user_info.clone())
                    } else {
                        Err(Error::new("Invalid password"))
                    }
                } else {
                    Err(Error::new("User not found"))
                }
            }
            _ => Err(Error::new("Unsupported authentication method")),
        }
    }

    fn validate_token(&self, token: &str) -> AuthResult<UserInfo> {
        // In a real implementation, this would validate a JWT or other token
        // For this example, we just use the token as the username
        let users = self.users.lock().unwrap();
        if let Some(user_info) = users.get(token) {
            Ok(user_info.clone())
        } else {
            Err(Error::new("Invalid token"))
        }
    }
}

// Simple health check service
#[derive(Debug, Clone)]
struct HealthService {
    status: Arc<Mutex<String>>,
}

impl HealthService {
    fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new("OK".to_string())),
        }
    }

    fn check(&self) -> Result<String> {
        let status = self.status.lock().unwrap();
        Ok(status.clone())
    }

    fn set_status(&self, status: &str) -> Result<()> {
        let mut status_lock = self.status.lock().unwrap();
        *status_lock = status.to_string();
        Ok(())
    }
}

// HTTP Server wrapper that integrates with the component registry
#[derive(Debug)]
struct AppServer {
    server: HttpServer,
    auth_provider: InMemoryAuthProvider,
    health_service: HealthService,
}

impl AppServer {
    fn new(auth_provider: InMemoryAuthProvider, health_service: HealthService) -> Self {
        // Create a new HTTP server with default config
        let config = ServerConfig::default().with_address("127.0.0.1:8080".to_string());

        let server = HttpServer::new(config);

        Self {
            server,
            auth_provider,
            health_service,
        }
    }

    fn configure_routes(&mut self) -> Result<()> {
        let mut router = Router::new();

        // Health check route
        let health_service = self.health_service.clone();
        router.add_route(Route::get("/health", move |_req| {
            let status = health_service
                .check()
                .unwrap_or_else(|_| "ERROR".to_string());
            Ok(format!("{{\"status\": \"{}\"}}", status))
        }));

        // Protected route example
        let auth_provider = self.auth_provider.clone();
        router.add_route(Route::get("/api/protected", move |req| {
            // Get the token from the Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            // Remove Bearer prefix if present
            let token = auth_header.trim_start_matches("Bearer ").trim();

            // Validate the token
            match auth_provider.validate_token(token) {
                Ok(user_info) => Ok(format!(
                    "{{\"message\": \"Welcome, {}\"}}",
                    user_info.username
                )),
                Err(_) => {
                    let mut response = navius_http::response::Response::new(403);
                    response.set_body("{{\"error\": \"Unauthorized\"}}");
                    Err(Error::from(response))
                }
            }
        }));

        // Set the router
        self.server.set_router(router);

        Ok(())
    }
}

#[async_trait]
impl AsyncLifecycle for AppServer {
    async fn on_initialize_async(&self) -> Result<()> {
        println!("Initializing AppServer");
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        println!("Shutting down AppServer");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a configuration
    let mut config = Config::default();
    config.set("server.address", "127.0.0.1:8080")?;

    // Create the application using the builder pattern
    let mut app = ApplicationBuilder::new()
        .with_config(config)
        .with_environment(Environment::Development)
        // Register the auth provider as a singleton
        .add_singleton::<InMemoryAuthProvider, _>(|| InMemoryAuthProvider::new())
        // Register the health service as a singleton
        .add_singleton::<HealthService, _>(|| HealthService::new())
        // Register the server as a singleton
        .add_factory::<AppServer, _>(
            || {
                // Resolve dependencies from the registry
                let auth_provider = app.get::<InMemoryAuthProvider>().unwrap();
                let health_service = app.get::<HealthService>().unwrap();

                let mut server = AppServer::new(auth_provider, health_service);
                server.configure_routes().unwrap();

                server
            },
            ComponentScope::Singleton,
        )
        .build();

    println!(
        "🚀 Application started in {} environment",
        app.environment().name()
    );

    // Get the server and start it
    let server = app.get::<AppServer>()?;

    println!(
        "Server configured with {} routes",
        server.server.routes().len()
    );
    println!("HTTP server listening on {}", server.server.address());
    println!("Try accessing:");
    println!("  - http://127.0.0.1:8080/health");
    println!("  - http://127.0.0.1:8080/api/protected (with Authorization: Bearer user1)");

    // Normally we would start the server here, but for this example we'll just simulate it
    // server.server.start().await?;

    // Simulate the server running
    println!("\nPress Ctrl+C to stop the server...");

    // For this example, we'll just wait for a short time
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Shutdown the application (this will call lifecycle hooks)
    app.shutdown().await?;

    println!("Application shutdown complete");

    Ok(())
}
