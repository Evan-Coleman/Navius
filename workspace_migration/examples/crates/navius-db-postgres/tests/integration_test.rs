use std::path::PathBuf;

use navius_db::provider::{DatabaseProvider, ProviderOptions};
use navius_db::transaction::TransactionOptions;
use navius_db_postgres::{MigrationOptions, PostgresProvider, PostgresProviderOptions};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Helper function to create a test migration
async fn create_migration_file(
    dir: &std::path::Path,
    version: &str,
    description: &str,
    content: &str,
) -> std::path::PathBuf {
    let filename = format!("V{}__{}.sql", version, description.replace(' ', "_"));
    let path = dir.join(filename);

    let mut file = File::create(&path).await.expect("Failed to create file");
    file.write_all(content.as_bytes())
        .await
        .expect("Failed to write file");
    file.flush().await.expect("Failed to flush file");

    path
}

/// Set up a test database connection
async fn setup_test_provider() -> (PostgresProvider, TempDir) {
    // Initialize logging for tests
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    // Create a temp directory for migrations
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let migrations_dir = temp_dir.path();

    // Create test migrations
    create_migration_file(
        migrations_dir,
        "20250329000001",
        "Create test table",
        "CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL);",
    )
    .await;

    create_migration_file(
        migrations_dir,
        "20250329000002",
        "Add columns",
        "ALTER TABLE test_table ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW();",
    )
    .await;

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

    // Configure provider options
    let options = PostgresProviderOptions {
        pool_config: navius_db_postgres::pool::PgPoolConfig {
            url: database_url,
            max_connections: 5,
            min_connections: 1,
            max_lifetime: Some(std::time::Duration::from_secs(60 * 60)),
            idle_timeout: Some(std::time::Duration::from_secs(10 * 60)),
            acquire_timeout: std::time::Duration::from_secs(30),
        },
        migration_options: Some(MigrationOptions {
            migrations_dir: migrations_dir.to_path_buf(),
            migrations_table: Some(format!(
                "test_migrations_{}",
                chrono::Utc::now().timestamp()
            )),
            validate_checksums: true,
        }),
        run_migrations: false, // We'll run manually
        validate_migrations: true,
    };

    // Create provider
    let provider = PostgresProvider::new(options)
        .await
        .expect("Failed to create provider");

    (provider, temp_dir)
}

#[tokio::test]
async fn test_provider_creation() {
    let (provider, _temp_dir) = setup_test_provider().await;

    // Check provider metadata
    assert_eq!(provider.name(), "postgresql");
    assert!(provider.version().len() > 0);

    // Check health
    let health = provider
        .health_check()
        .await
        .expect("Health check should succeed");
    assert!(health, "Health check should return true");
}

#[tokio::test]
async fn test_migration_functionality() {
    let (provider, _temp_dir) = setup_test_provider().await;

    // Run migrations
    let count = provider
        .run_migrations()
        .await
        .expect("Migrations should run successfully");
    assert_eq!(count, 2, "Should apply 2 migrations");

    // Validate migrations
    let valid = provider
        .validate_migrations()
        .await
        .expect("Validation should succeed");
    assert!(valid, "Migrations should be valid");

    // Check migration info
    let info = provider
        .migration_info()
        .await
        .expect("Should get migration info");
    assert_eq!(info.len(), 2, "Should have 2 migrations");

    // Verify migrations using a direct query
    let pool = provider.pool();
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'test_table'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("Query should execute successfully");

    assert!(table_exists, "test_table should exist");

    // Clean up
    sqlx::query("DROP TABLE IF EXISTS test_table")
        .execute(&pool)
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
async fn test_transaction_functionality() {
    let (provider, _temp_dir) = setup_test_provider().await;

    // Run migrations to create the test table
    provider
        .run_migrations()
        .await
        .expect("Migrations should run successfully");

    // Get transaction manager
    let tx_manager = provider.transaction_manager();

    // Test transaction commit
    {
        let mut tx = tx_manager
            .begin_transaction(None)
            .await
            .expect("Should begin transaction");

        // Insert data
        tx.execute_query("INSERT INTO test_table (name) VALUES ('test1'), ('test2'), ('test3')")
            .await
            .expect("Insert should succeed");

        // Commit
        tx.commit().await.expect("Commit should succeed");
    }

    // Verify data was committed
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
        .fetch_one(&provider.pool())
        .await
        .expect("Count query should succeed");
    assert_eq!(count, 3, "Should have 3 rows after commit");

    // Test transaction rollback
    {
        let mut tx = tx_manager
            .begin_transaction(None)
            .await
            .expect("Should begin transaction");

        // Insert more data
        tx.execute_query("INSERT INTO test_table (name) VALUES ('test4'), ('test5')")
            .await
            .expect("Insert should succeed");

        // Rollback
        tx.rollback().await.expect("Rollback should succeed");
    }

    // Verify data was rolled back
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
        .fetch_one(&provider.pool())
        .await
        .expect("Count query should succeed");
    assert_eq!(count, 3, "Should still have 3 rows after rollback");

    // Test transaction with callback
    let result = tx_manager
        .with_transaction::<_, _, navius_db::error::DatabaseError>(|mut tx| {
            Box::pin(async move {
                // Insert data
                tx.execute_query("INSERT INTO test_table (name) VALUES ('test6')")
                    .await?;

                // Return some result
                Ok("success")
            })
        })
        .await
        .expect("Transaction should succeed");

    assert_eq!(result, "success");

    // Verify data was committed
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
        .fetch_one(&provider.pool())
        .await
        .expect("Count query should succeed");
    assert_eq!(count, 4, "Should have 4 rows after transaction callback");

    // Test nested transactions
    {
        let mut tx = tx_manager
            .begin_transaction(None)
            .await
            .expect("Should begin transaction");

        // Insert in outer transaction
        tx.execute_query("INSERT INTO test_table (name) VALUES ('outer')")
            .await
            .expect("Insert should succeed");

        // Create nested transaction
        {
            let mut nested_tx = tx
                .create_nested_transaction()
                .await
                .expect("Should create nested transaction");

            // Insert in nested transaction
            nested_tx
                .execute_query("INSERT INTO test_table (name) VALUES ('nested')")
                .await
                .expect("Insert should succeed");

            // Rollback nested transaction
            nested_tx.rollback().await.expect("Rollback should succeed");
        }

        // Commit outer transaction
        tx.commit().await.expect("Commit should succeed");
    }

    // Verify only outer transaction was committed
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
        .fetch_one(&provider.pool())
        .await
        .expect("Count query should succeed");
    assert_eq!(count, 5, "Should have 5 rows after nested transactions");

    // Clean up
    sqlx::query("DROP TABLE IF EXISTS test_table")
        .execute(&provider.pool())
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
async fn test_provider_from_options() {
    // Create a temp directory for migrations
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let migrations_dir = temp_dir.path();

    // Create test migrations
    create_migration_file(
        migrations_dir,
        "20250329000001",
        "Create test table",
        "CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL);",
    )
    .await;

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

    // Create provider options
    let mut options = HashMap::new();
    options.insert("database_url".to_string(), database_url);
    options.insert("max_connections".to_string(), "5".to_string());
    options.insert(
        "migrations_dir".to_string(),
        migrations_dir.to_string_lossy().to_string(),
    );
    options.insert(
        "migrations_table".to_string(),
        format!("test_migrations_{}", chrono::Utc::now().timestamp()),
    );
    options.insert("run_migrations".to_string(), "true".to_string());
    options.insert("validate_migrations".to_string(), "true".to_string());

    // Create provider
    let provider = PostgresProvider::connect(options)
        .await
        .expect("Should create provider from options");

    // Check provider metadata
    assert_eq!(provider.name(), "postgresql");
    assert!(provider.version().len() > 0);

    // Check health
    let health = provider
        .health_check()
        .await
        .expect("Health check should succeed");
    assert!(health, "Health check should return true");

    // Check if test_table exists (since run_migrations was true)
    let pool = provider.pool();
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'test_table'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("Query should execute successfully");

    assert!(table_exists, "test_table should exist");

    // Clean up
    sqlx::query("DROP TABLE IF EXISTS test_table")
        .execute(&pool)
        .await
        .expect("Cleanup should succeed");
}
