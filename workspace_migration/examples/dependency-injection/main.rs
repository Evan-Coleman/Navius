//! Example of using the dependency injection system
//!
//! This example demonstrates how to use the component registry and dependency
//! injection system from navius-core, including lifecycle hooks and async support.

use async_trait::async_trait;
use navius_core::{
    config::Config,
    di::{Application, ApplicationBuilder, AsyncLifecycle, ComponentScope, Environment, Lifecycle},
    error::Result,
};

// Define some example components with lifecycle hooks
#[derive(Debug, Clone)]
struct DatabaseService {
    connection_string: String,
    is_initialized: bool,
}

impl DatabaseService {
    fn new(connection_string: String) -> Self {
        println!(
            "Creating DatabaseService with connection string: {}",
            connection_string
        );
        Self {
            connection_string,
            is_initialized: false,
        }
    }

    fn query(&self, sql: &str) -> Result<String> {
        println!(
            "Executing query '{}' on connection {}",
            sql, self.connection_string
        );
        Ok(format!("Result from: {}", sql))
    }
}

impl Lifecycle for DatabaseService {
    fn on_initialize(&self) -> Result<()> {
        println!(
            "Initializing DatabaseService with connection: {}",
            self.connection_string
        );
        // In a real implementation, this would establish a connection
        // For demonstration purposes, we'll just set a flag
        let mut this = self as *const Self as *mut Self;
        unsafe {
            (*this).is_initialized = true;
        }
        Ok(())
    }

    fn on_destroy(&self) -> Result<()> {
        println!(
            "Destroying DatabaseService connection: {}",
            self.connection_string
        );
        // In a real implementation, this would close the connection
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CacheService {
    url: String,
    is_connected: bool,
}

impl CacheService {
    fn new(url: String) -> Self {
        println!("Creating CacheService with URL: {}", url);
        Self {
            url,
            is_connected: false,
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        println!("Getting '{}' from cache at {}", key, self.url);
        Some(format!("Cached value for {}", key))
    }
}

#[async_trait]
impl AsyncLifecycle for CacheService {
    async fn on_initialize_async(&self) -> Result<()> {
        println!("Async initializing CacheService with URL: {}", self.url);
        // In a real implementation, this would establish an async connection
        // Simulate some async initialization work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut this = self as *const Self as *mut Self;
        unsafe {
            (*this).is_connected = true;
        }
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        println!("Async destroying CacheService connection: {}", self.url);
        // In a real implementation, this would close the connection asynchronously
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct UserService {
    database: String,
    cache: String,
}

impl UserService {
    fn new(db: String, cache: String) -> Self {
        println!("Creating UserService with DB: {} and cache: {}", db, cache);
        Self {
            database: db,
            cache,
        }
    }

    fn get_user(&self, id: &str) -> Result<String> {
        println!(
            "Getting user {} using database {} and cache {}",
            id, self.database, self.cache
        );
        Ok(format!("User {}", id))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a configuration
    let mut config = Config::default();
    config.set("database.url", "postgres://localhost:5432/navius")?;
    config.set("cache.url", "redis://localhost:6379")?;

    // Create an application using the builder pattern with a specific environment
    let mut app = ApplicationBuilder::new()
        .with_config(config)
        .with_environment(Environment::Development)
        // Register singleton database component with lifecycle hooks
        .add_singleton::<DatabaseService, _>(|| {
            let connection_string = "postgres://localhost:5432/navius".to_string();
            DatabaseService::new(connection_string)
        })
        // Register singleton cache component with async lifecycle hooks
        .add_singleton::<CacheService, _>(|| {
            let url = "redis://localhost:6379".to_string();
            CacheService::new(url)
        })
        // Register a prototype component that creates a new instance each time
        .add_factory::<UserService, _>(
            || {
                let db = "postgres://localhost:5432/navius".to_string();
                let cache = "redis://localhost:6379".to_string();
                UserService::new(db, cache)
            },
            ComponentScope::Prototype,
        )
        .build();

    println!(
        "\n=== Application Environment: {} ===\n",
        app.environment().name()
    );

    println!("\n=== Synchronous Component Initialization ===\n");

    // Get the database service (singleton with sync lifecycle hooks)
    let db_service = app.get::<DatabaseService>()?;
    println!("Got database service: {:?}", db_service);
    println!("Database initialized: {}", db_service.is_initialized);
    let query_result = db_service.query("SELECT * FROM users")?;
    println!("Query result: {}", query_result);

    println!("\n=== Asynchronous Component Initialization ===\n");

    // Get the cache service (singleton with async lifecycle hooks)
    let cache_service = app.get_async::<CacheService>().await?;
    println!("Got cache service: {:?}", cache_service);
    println!("Cache connected: {}", cache_service.is_connected);
    let cache_result = cache_service.get("user:123");
    println!("Cache result: {:?}", cache_result);

    // Get the user service (prototype, will create a new instance each time)
    let user_service1 = app.get::<UserService>()?;
    println!("Got user service 1: {:?}", user_service1);
    let user_result1 = user_service1.get_user("123")?;
    println!("User result 1: {}", user_result1);

    // Get another instance of the user service
    let user_service2 = app.get::<UserService>()?;
    println!("Got user service 2: {:?}", user_service2);
    let user_result2 = user_service2.get_user("456")?;
    println!("User result 2: {}", user_result2);

    println!("\n=== Application Shutdown ===\n");
    // Proper shutdown with lifecycle hooks
    app.shutdown()?;

    println!("\n=== Dependency Injection Complete ===\n");

    Ok(())
}
