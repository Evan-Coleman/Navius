//! Tests for the database module
//!
//! These tests verify the functionality of the database connection,
//! pool management, and transaction handling.

use std::time::Duration;

use tokio::time::timeout;

use crate::core::config::app_config::DatabaseConfig;
use crate::core::database::connection::{ConnectionStats, MockDatabaseConnection};
use crate::core::database::error::DatabaseError;
use crate::core::database::{DatabaseConnection, PgPool};

#[tokio::test]
async fn test_database_config_defaults() {
    // Test the default configuration values
    let config = DatabaseConfig::default();

    assert!(!config.enabled, "Database should be disabled by default");
    assert_eq!(
        config.url,
        "postgres://postgres:postgres@localhost:5432/app"
    );
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.connect_timeout_seconds, 30);
    assert_eq!(config.idle_timeout_seconds, Some(300));
}

#[tokio::test]
async fn test_create_pool_disabled() {
    // Test creating a pool with disabled config
    let config = DatabaseConfig::default();

    let result = super::connection::create_pool(&config).await;
    assert!(result.is_err());

    match result {
        Err(DatabaseError::NotEnabled) => (),
        _ => panic!("Expected NotEnabled error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_create_pool_enabled() {
    // Test creating a pool with enabled config
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let result = super::connection::create_pool(&config).await;
    assert!(result.is_ok());

    let pool = result.unwrap();
    let ping_result = pool.ping().await;
    assert!(ping_result.is_ok());
}

#[tokio::test]
async fn test_connection_stats() {
    // Test the connection stats functionality
    let mut config = DatabaseConfig::default();
    config.enabled = true;
    config.max_connections = 20;

    let pool = MockDatabaseConnection::new(&config);

    let stats_result = pool.stats().await;
    assert!(stats_result.is_ok());

    let stats = stats_result.unwrap();
    assert_eq!(stats.max_connections, 20);
    assert_eq!(stats.idle_connections, 5); // From the mock implementation
    assert_eq!(stats.active_connections, 2); // From the mock implementation
}

#[tokio::test]
async fn test_transaction_lifecycle() {
    // Test the transaction lifecycle
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Begin transaction
    let tx_result = pool.begin_transaction().await;
    assert!(tx_result.is_ok());

    let mut tx = tx_result.unwrap();

    // Commit transaction
    let commit_result = tx.commit().await;
    assert!(commit_result.is_ok());

    // Try to commit again (should fail)
    let second_commit = tx.commit().await;
    assert!(second_commit.is_err());

    match second_commit {
        Err(DatabaseError::TransactionError(_)) => (),
        _ => panic!("Expected TransactionError, got {:?}", second_commit),
    }
}

#[tokio::test]
async fn test_transaction_rollback() {
    // Test transaction rollback
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Begin transaction
    let tx_result = pool.begin_transaction().await;
    assert!(tx_result.is_ok());

    let mut tx = tx_result.unwrap();

    // Rollback transaction
    let rollback_result = tx.rollback().await;
    assert!(rollback_result.is_ok());

    // Try to commit after rollback (should fail)
    let commit_after_rollback = tx.commit().await;
    assert!(commit_after_rollback.is_err());

    match commit_after_rollback {
        Err(DatabaseError::TransactionError(_)) => (),
        _ => panic!("Expected TransactionError, got {:?}", commit_after_rollback),
    }
}

#[tokio::test]
async fn test_connection_timeout() {
    // Test that connection operations have a reasonable timeout
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // The ping operation should complete in a reasonable time
    let ping_result = timeout(Duration::from_secs(1), DatabaseConnection::ping(&pool)).await;
    assert!(ping_result.is_ok());
}

#[tokio::test]
async fn test_connection_pool_interface() {
    // Test that we can use the PgPool type with the MockDatabaseConnection
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let result = super::connection::create_pool(&config).await;
    assert!(result.is_ok());

    let pool = result.unwrap();

    // Test basic operations on the pool
    let ping_result = pool.ping().await;
    assert!(ping_result.is_ok());

    let stats_result = pool.stats().await;
    assert!(stats_result.is_ok());

    let stats = stats_result.unwrap();
    assert_eq!(stats.max_connections, config.max_connections);
}
