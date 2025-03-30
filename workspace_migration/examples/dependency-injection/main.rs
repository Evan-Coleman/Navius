//! Example of using the dependency injection system
//!
//! This example demonstrates how to use the component registry and dependency
//! injection system from navius-core.

use navius_core::{
    config::Config,
    di::{Application, ApplicationBuilder, ComponentScope},
    error::Result,
};

// Define some example components
#[derive(Debug, Clone)]
struct DatabaseService {
    connection_string: String,
}

impl DatabaseService {
    fn new(connection_string: String) -> Self {
        println!(
            "Creating DatabaseService with connection string: {}",
            connection_string
        );
        Self { connection_string }
    }

    fn query(&self, sql: &str) -> Result<String> {
        println!(
            "Executing query '{}' on connection {}",
            sql, self.connection_string
        );
        Ok(format!("Result from: {}", sql))
    }
}

#[derive(Debug, Clone)]
struct CacheService {
    url: String,
}

impl CacheService {
    fn new(url: String) -> Self {
        println!("Creating CacheService with URL: {}", url);
        Self { url }
    }

    fn get(&self, key: &str) -> Option<String> {
        println!("Getting '{}' from cache at {}", key, self.url);
        Some(format!("Cached value for {}", key))
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

fn main() -> Result<()> {
    // Create a configuration
    let mut config = Config::default();
    config.set("database.url", "postgres://localhost:5432/navius")?;
    config.set("cache.url", "redis://localhost:6379")?;

    // Create an application using the builder pattern
    let mut app = ApplicationBuilder::new()
        .with_config(config)
        // Register singleton components
        .add_singleton::<DatabaseService, _>(|| {
            let connection_string = "postgres://localhost:5432/navius".to_string();
            DatabaseService::new(connection_string)
        })
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

    println!("\n=== Using the Application ===\n");

    // Get the database service (singleton)
    let db_service = app.get::<DatabaseService>()?;
    println!("Got database service: {:?}", db_service);
    let query_result = db_service.query("SELECT * FROM users")?;
    println!("Query result: {}", query_result);

    // Get the cache service (singleton)
    let cache_service = app.get::<CacheService>()?;
    println!("Got cache service: {:?}", cache_service);
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

    println!("\n=== Dependency Injection Complete ===\n");

    Ok(())
}
