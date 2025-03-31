// Basic dependency injection example
//
// This example demonstrates the core features of the Navius DI system:
// - Component registration and resolution
// - Configuration binding and injection
// - Lifecycle management
// - Application building

use async_trait::async_trait;
use navius_di::{
    Application, ApplicationBuilder, AsyncLifecycle, ComponentRef, ComponentRegistry,
    ComponentScope, ConfigRef, Error, Lifecycle, Result, autowire,
};
use serde::Deserialize;
use std::sync::Arc;

// A simple component with lifecycle hooks
struct DatabaseService {
    connection_string: String,
}

impl DatabaseService {
    fn new(connection_string: String) -> Self {
        println!(
            "Creating DatabaseService with connection: {}",
            connection_string
        );
        Self { connection_string }
    }

    fn query(&self, sql: &str) -> String {
        format!(
            "Executing '{}' on connection {}",
            sql, self.connection_string
        )
    }
}

impl Lifecycle for DatabaseService {
    fn on_initialize(&self) -> Result<()> {
        println!("Initializing DatabaseService");
        // Simulate connection initialization
        println!("Connected to database at {}", self.connection_string);
        Ok(())
    }

    fn on_destroy(&self) -> Result<()> {
        println!("Destroying DatabaseService");
        // Simulate connection cleanup
        println!("Disconnected from database at {}", self.connection_string);
        Ok(())
    }
}

// A component that depends on another component
struct UserService {
    db: ComponentRef<DatabaseService>,
}

impl UserService {
    fn new(db: ComponentRef<DatabaseService>) -> Self {
        Self { db }
    }

    fn get_user(&self, id: &str) -> String {
        self.db
            .query(&format!("SELECT * FROM users WHERE id = '{}'", id))
    }
}

// An async component with async lifecycle hooks
struct AsyncService {
    name: String,
}

impl AsyncService {
    fn new(name: String) -> Self {
        println!("Creating AsyncService: {}", name);
        Self { name }
    }

    async fn get_data(&self) -> String {
        format!("Data from async service: {}", self.name)
    }
}

#[async_trait]
impl AsyncLifecycle for AsyncService {
    async fn on_initialize_async(&self) -> Result<()> {
        println!("Async initialization of AsyncService: {}", self.name);
        // Simulate async initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("AsyncService {} initialized", self.name);
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        println!("Async destruction of AsyncService: {}", self.name);
        // Simulate async cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        println!("AsyncService {} destroyed", self.name);
        Ok(())
    }
}

// A configuration struct
#[derive(Debug, Clone, Deserialize)]
struct AppConfig {
    name: String,
    version: String,
    database_url: String,
}

// A configuration-aware service
struct ConfigAwareService {
    config: ConfigRef<AppConfig>,
}

impl ConfigAwareService {
    fn new(config: ConfigRef<AppConfig>) -> Self {
        Self { config }
    }

    fn get_info(&self) -> String {
        format!(
            "Application: {} v{} (DB: {})",
            self.config.name, self.config.version, self.config.database_url
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an application builder
    let app = Application::builder()
        // Set configuration
        .with_config("app.name", "Navius Example".to_string())
        .with_config("app.version", "1.0.0".to_string())
        .with_config(
            "app.database_url",
            "jdbc:postgresql://localhost:5432/mydb".to_string(),
        )
        // Register the database service as a singleton
        .with_factory(
            || DatabaseService::new("jdbc:postgresql://localhost:5432/mydb".to_string()),
            ComponentScope::Singleton,
        )
        // Register the user service with a factory that depends on the database service
        .with_factory(
            || {
                let registry = ComponentRegistry::new();
                let db = registry.get::<DatabaseService>().unwrap();
                UserService::new(db)
            },
            ComponentScope::Prototype,
        )
        // Register an async service
        .with_factory(
            || AsyncService::new("MainAsyncService".to_string()),
            ComponentScope::Singleton,
        )
        // Build the application
        .build()
        .await?;

    // Get and use services
    println!("\n=== Using Services ===\n");

    // Get the user service
    let user_service = app.get::<UserService>()?;
    println!("User service result: {}", user_service.get_user("user-1"));

    // Get the async service
    let async_service = app.get::<AsyncService>()?;
    println!("Async service result: {}", async_service.get_data().await);

    // Get the config and create a config-aware service
    let app_config = app.config::<AppConfig>("app")?;
    println!("App config: {:?}", app_config);

    let config_service = ConfigAwareService::new(ConfigRef::new(app_config));
    println!("Config service info: {}", config_service.get_info());

    // Shutdown the application
    println!("\n=== Shutting Down ===\n");
    app.shutdown_async().await?;

    Ok(())
}
