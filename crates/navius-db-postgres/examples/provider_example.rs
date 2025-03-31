use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use navius_db::provider::{DatabaseProvider, ProviderOptions};
use navius_db::transaction::TransactionOptions;
use navius_db_postgres::provider::{PostgresProvider, PostgresProviderOptions};
use sqlx::postgres::PgConnectOptions;
use tempfile::tempdir;
use tokio::fs;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting PostgreSQL provider example");

    // Create a temporary directory for migrations
    let migration_dir = tempdir()?;
    let migration_path = migration_dir.path().to_path_buf();
    
    // Create example migration files
    create_migration_files(&migration_path).await?;
    
    // Method 1: Create provider directly with PostgresProviderOptions
    let provider = create_provider_directly(&migration_path).await?;
    
    // Use the provider for basic operations
    basic_operations(&provider).await?;
    
    // Use the provider for transaction operations
    transaction_operations(&provider).await?;
    
    // Method 2: Create provider using generic ProviderOptions
    let provider = create_provider_from_options(&migration_path).await?;
    
    // Get migration information
    migration_info(&provider).await?;
    
    info!("PostgreSQL provider example completed successfully");
    Ok(())
}

async fn create_migration_files(migration_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create migration directory
    fs::create_dir_all(migration_path).await?;
    
    // Create initial schema migration
    let migration1 = format!(
        "{}/__V20250329120000__Create_users_table.sql",
        migration_path.display()
    );
    let migration1_content = r#"
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        username VARCHAR(100) NOT NULL UNIQUE,
        email VARCHAR(255) NOT NULL UNIQUE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    );
    "#;
    fs::write(migration1, migration1_content).await?;
    
    // Create second migration
    let migration2 = format!(
        "{}/__V20250329120100__Add_user_profile.sql",
        migration_path.display()
    );
    let migration2_content = r#"
    CREATE TABLE user_profiles (
        id SERIAL PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users(id),
        bio TEXT,
        location VARCHAR(255),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    );
    "#;
    fs::write(migration2, migration2_content).await?;
    
    info!("Created example migration files in {}", migration_path.display());
    Ok(())
}

async fn create_provider_directly(migration_path: &PathBuf) -> Result<PostgresProvider, Box<dyn std::error::Error>> {
    // Database connection options (using a test database)
    let connect_options = PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .username("postgres")
        .password("postgres")
        .database("navius_test");
    
    // Create provider options with specific settings
    let options = PostgresProviderOptions::new(connect_options)
        .with_max_connections(5)
        .with_min_connections(1)
        .with_max_lifetime(Some(Duration::from_secs(1800)))
        .with_idle_timeout(Some(Duration::from_secs(600)))
        .with_migration_dir(migration_path.to_path_buf())
        .with_run_migrations(true)          // Run migrations on startup
        .with_validate_migrations(true);    // Validate migrations on startup
    
    // Create the provider
    let provider = PostgresProvider::new(options).await?;
    
    info!("Created PostgreSQL provider directly with custom options");
    info!("Provider name: {}, version: {}", provider.name(), provider.version());
    
    // Health check
    let health = provider.health_check().await?;
    info!("Provider health check: {}", health);
    
    Ok(provider)
}

async fn create_provider_from_options(migration_path: &PathBuf) -> Result<PostgresProvider, Box<dyn std::error::Error>> {
    // Create generic provider options
    let mut options = ProviderOptions::new("postgresql");
    
    // Set connection string
    options.insert("connection_string".to_string(), "postgres://postgres:postgres@localhost:5432/navius_test".to_string());
    
    // Set pool options
    options.insert("max_connections".to_string(), "5".to_string());
    options.insert("min_connections".to_string(), "1".to_string());
    options.insert("max_lifetime".to_string(), "1800".to_string());
    options.insert("idle_timeout".to_string(), "600".to_string());
    
    // Set migration options
    options.insert("migration_dir".to_string(), migration_path.to_string_lossy().to_string());
    options.insert("run_migrations".to_string(), "true".to_string());
    options.insert("validate_migrations".to_string(), "true".to_string());
    
    // Create the provider using the generic interface
    let provider = PostgresProvider::connect(options).await?;
    
    info!("Created PostgreSQL provider from generic ProviderOptions");
    
    Ok(provider)
}

async fn basic_operations(provider: &PostgresProvider) -> Result<(), Box<dyn std::error::Error>> {
    info!("Performing basic operations...");
    
    // Simple query execution
    let query = "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id";
    let user_id: i32 = sqlx::query_scalar(query)
        .bind("john_doe")
        .bind("john@example.com")
        .fetch_one(&provider.pool)
        .await?;
    
    info!("Inserted user with ID: {}", user_id);
    
    // Simple select query
    let query = "SELECT username, email FROM users WHERE id = $1";
    let (username, email): (String, String) = sqlx::query_as(query)
        .bind(user_id)
        .fetch_one(&provider.pool)
        .await?;
    
    info!("Retrieved user: {} ({})", username, email);
    
    Ok(())
}

async fn transaction_operations(provider: &PostgresProvider) -> Result<(), Box<dyn std::error::Error>> {
    info!("Demonstrating transaction operations...");
    
    // Get the transaction manager
    let tx_manager = provider.transaction_manager();
    
    // Example 1: Using the transaction directly
    info!("Example 1: Direct transaction usage");
    let mut tx = tx_manager.begin_transaction(None).await?;
    
    // Execute queries within the transaction
    tx.execute_query("INSERT INTO users (username, email) VALUES ('jane_doe', 'jane@example.com')")
        .await?;
    tx.execute_query("INSERT INTO users (username, email) VALUES ('bob_smith', 'bob@example.com')")
        .await?;
    
    // Commit the transaction
    tx.commit().await?;
    
    // Example 2: Using nested transactions
    info!("Example 2: Nested transactions");
    let mut tx = tx_manager.begin_transaction(Some(TransactionOptions::default())).await?;
    
    tx.execute_query("INSERT INTO users (username, email) VALUES ('alice_wonder', 'alice@example.com')")
        .await?;
    
    // Create a nested transaction
    let mut nested_tx = tx.create_nested_transaction().await?;
    
    // Execute in nested transaction
    nested_tx.execute_query("INSERT INTO users (username, email) VALUES ('charlie_brown', 'charlie@example.com')")
        .await?;
    
    // Roll back the nested transaction - this will not affect the parent transaction
    nested_tx.rollback().await?;
    
    // Continue with parent transaction
    tx.execute_query("INSERT INTO users (username, email) VALUES ('david_jones', 'david@example.com')")
        .await?;
    
    // Commit the parent transaction
    tx.commit().await?;
    
    // Example 3: Using the transaction callback API
    info!("Example 3: Transaction callback API");
    let result = tx_manager.with_transaction::<_, String, Box<dyn std::error::Error>>(|mut tx| {
        Box::pin(async move {
            // Execute queries
            tx.execute_query("INSERT INTO users (username, email) VALUES ('emma_wilson', 'emma@example.com')")
                .await?;
                
            // Create a nested transaction within the callback
            let mut nested = tx.create_nested_transaction().await?;
            nested.execute_query("UPDATE users SET email = 'emma.wilson@example.com' WHERE username = 'emma_wilson'")
                .await?;
                
            // Commit the nested transaction
            nested.commit().await?;
            
            Ok("Transaction completed successfully".to_string())
        })
    }).await?;
    
    info!("Transaction callback result: {}", result);
    
    Ok(())
}

async fn migration_info(provider: &PostgresProvider) -> Result<(), Box<dyn std::error::Error>> {
    info!("Retrieving migration information...");
    
    // Get migration information
    let migration_status = provider.migration_info().await?;
    
    // Display migration information
    info!("Found {} migrations:", migration_status.len());
    for status in migration_status {
        info!(
            "Migration: {} ({}), Version: {}, Applied: {}, Checksum Valid: {}",
            status.description,
            status.filename,
            status.version,
            status.applied,
            status.checksum_valid.unwrap_or(false)
        );
    }
    
    // Check for pending migrations
    let pending_migrations = migration_status.iter().filter(|s| !s.applied).count();
    if pending_migrations > 0 {
        info!("There are {} pending migrations", pending_migrations);
    } else {
        info!("All migrations have been applied");
    }
    
    Ok(())
} 