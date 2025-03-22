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

// Import the User model for tests
use crate::repository::Repository;
use crate::repository::User;
use crate::repository::models::UserRole;
use crate::repository::user::UserRepository;
use chrono::{DateTime, Utc};
use uuid::Uuid;

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

#[tokio::test]
async fn test_connection_pool_management() {
    // Test pool management and configuration
    let mut config = DatabaseConfig::default();
    config.enabled = true;
    config.max_connections = 5;
    config.connect_timeout_seconds = 10;
    config.idle_timeout_seconds = Some(60);

    let pool = MockDatabaseConnection::new(&config);

    // Test that the pool respects configuration limits
    let stats_result = pool.stats().await;
    assert!(stats_result.is_ok());

    let stats = stats_result.unwrap();
    assert_eq!(stats.max_connections, 5);
}

#[tokio::test]
async fn test_pool_concurrent_connections() {
    use std::sync::Arc;
    use tokio::sync::Barrier;
    use tokio::task;

    // Test concurrent connections to the pool
    let mut config = DatabaseConfig::default();
    config.enabled = true;
    config.max_connections = 10;

    let pool = MockDatabaseConnection::new(&config);
    let pool = Arc::new(pool);
    let barrier = Arc::new(Barrier::new(5)); // 5 concurrent tasks

    // Spawn 5 concurrent tasks that will all access the pool simultaneously
    let mut handles = Vec::new();
    for i in 0..5 {
        let pool_clone = Arc::clone(&pool);
        let barrier_clone = Arc::clone(&barrier);

        let handle = task::spawn(async move {
            // Wait for all tasks to reach this point
            barrier_clone.wait().await;

            // Perform a ping operation
            let result = PgPool::ping(&*pool_clone).await;
            assert!(result.is_ok(), "Task {i} failed ping");

            // Get pool stats
            let stats_result = pool_clone.stats().await;
            assert!(stats_result.is_ok(), "Task {i} failed stats");

            i // Return task index for verification
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    // Verify that all tasks completed successfully
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Task future {i} did not complete successfully"
        );
        assert_eq!(
            result.as_ref().unwrap(),
            &i,
            "Task returned incorrect index"
        );
    }
}

#[tokio::test]
async fn test_pg_pool_trait_methods() {
    // Test the PgPool trait methods on MockDatabaseConnection
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Test ping
    let ping_result = PgPool::ping(&pool).await;
    assert!(ping_result.is_ok());

    // Test begin transaction
    let tx_result = pool.begin().await;
    assert!(tx_result.is_ok());

    // Test transaction commit
    let tx = tx_result.unwrap();
    let commit_result = tx.commit().await;
    assert!(commit_result.is_ok());

    // Test as_any downcasting
    let any_ref = pool.as_any();
    let downcast_result = any_ref.downcast_ref::<MockDatabaseConnection>();
    assert!(downcast_result.is_some());
}

#[tokio::test]
async fn test_init_database_disabled() {
    // Test initializing database with disabled configuration
    let config = DatabaseConfig::default(); // Disabled by default

    let result = super::connection::init_database(&config).await;
    assert!(result.is_err());

    match result {
        Err(crate::core::error::AppError::DatabaseError(msg)) => {
            assert!(msg.contains("disabled"));
        }
        _ => panic!("Expected DatabaseError with 'disabled' message, got unexpected error type"),
    }
}

#[tokio::test]
async fn test_ping_database_function() {
    // Test the standalone ping_database function
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    let result = super::ping_database(&pool).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transaction_safety() {
    // Test transaction safety properties
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Create a transaction using the PgPool trait
    let tx_result = pool.begin().await;
    assert!(tx_result.is_ok());

    let tx = tx_result.unwrap();

    // Commit should work
    let commit_result = tx.commit().await;
    assert!(commit_result.is_ok());
}

#[tokio::test]
async fn test_transaction_rollback_with_pg_transaction() {
    // Test transaction rollback using PgTransaction trait
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Create a transaction using the PgPool trait
    let tx_result = pool.begin().await;
    assert!(tx_result.is_ok());

    let tx = tx_result.unwrap();

    // Rollback should work
    let rollback_result = tx.rollback().await;
    assert!(rollback_result.is_ok());
}

#[tokio::test]
async fn test_nested_transactions() {
    // Test behavior with nested transactions, which should still be supported
    // even though they are simulated in the mock
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Begin outer transaction
    let outer_tx_result = pool.begin_transaction().await;
    assert!(outer_tx_result.is_ok());
    let mut outer_tx = outer_tx_result.unwrap();

    // Begin inner transaction
    let inner_tx_result = pool.begin_transaction().await;
    assert!(inner_tx_result.is_ok());
    let mut inner_tx = inner_tx_result.unwrap();

    // Commit inner transaction
    let inner_commit_result = inner_tx.commit().await;
    assert!(inner_commit_result.is_ok());

    // Commit outer transaction
    let outer_commit_result = outer_tx.commit().await;
    assert!(outer_commit_result.is_ok());
}

#[tokio::test]
async fn test_concurrent_transactions() {
    use std::sync::Arc;
    use tokio::sync::Barrier;
    use tokio::task;

    // Test concurrent transactions
    let mut config = DatabaseConfig::default();
    config.enabled = true;
    config.max_connections = 10;

    let pool = Arc::new(MockDatabaseConnection::new(&config));
    let barrier = Arc::new(Barrier::new(3)); // 3 concurrent transactions

    // Spawn concurrent tasks that will all create transactions
    let mut handles = Vec::new();
    for i in 0..3 {
        let pool_clone = Arc::clone(&pool);
        let barrier_clone = Arc::clone(&barrier);

        let handle = task::spawn(async move {
            // Wait for all tasks to reach this point
            barrier_clone.wait().await;

            // Begin transaction
            let tx_result = pool_clone.begin_transaction().await;
            assert!(tx_result.is_ok(), "Task {i} failed begin transaction");

            let mut tx = tx_result.unwrap();

            // Small delay to simulate work
            tokio::time::sleep(Duration::from_millis(10 * (i as u64 + 1))).await;

            // Commit transaction
            let commit_result = tx.commit().await;
            assert!(commit_result.is_ok(), "Task {i} failed commit");

            i // Return task index for verification
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    // Verify that all tasks completed successfully
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Task future {i} did not complete successfully"
        );
        assert_eq!(
            result.as_ref().unwrap(),
            &i,
            "Task returned incorrect index"
        );
    }
}

#[tokio::test]
async fn test_transaction_isolation() {
    // Test transaction isolation properties using MockDatabaseConnection
    // Since this is a mock, we're testing the API contracts rather than actual isolation
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let pool = MockDatabaseConnection::new(&config);

    // Begin two separate transactions
    let tx1_result = pool.begin_transaction().await;
    assert!(tx1_result.is_ok());
    let mut tx1 = tx1_result.unwrap();

    let tx2_result = pool.begin_transaction().await;
    assert!(tx2_result.is_ok());
    let mut tx2 = tx2_result.unwrap();

    // Commit first transaction
    let commit1_result = tx1.commit().await;
    assert!(commit1_result.is_ok());

    // Rollback second transaction
    let rollback2_result = tx2.rollback().await;
    assert!(rollback2_result.is_ok());
}

#[tokio::test]
async fn test_query_building_for_user_repository() {
    // Test the query building functionality using the UserRepository
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    // Create a mock connection
    let mock_conn =
        std::sync::Arc::new(Box::new(MockDatabaseConnection::new(&config)) as Box<dyn PgPool>);

    // Create a user repository with the mock connection
    let repo = UserRepository::new(mock_conn);

    // Create a test user
    let test_user = User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        is_active: true,
        role: UserRole::User,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Test finding the user by ID
    // This will exercise the query building code even though it uses a mock
    let result = repo.find_by_id(test_user.id).await;
    assert!(result.is_ok());

    // Test finding the user by username
    let result = repo.find_by_username(&test_user.username).await;
    assert!(result.is_ok());

    // Test finding the user by email
    let result = repo.find_by_email(&test_user.email).await;
    assert!(result.is_ok());

    // Test counting users
    let result = repo.count().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pg_connection_pool_contract() {
    // Test that the PgPool trait works as expected with a real connection type
    // In this case we're using a mock, but the contract should be the same
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    // Create a PgPool instance
    let pool =
        std::sync::Arc::new(Box::new(MockDatabaseConnection::new(&config)) as Box<dyn PgPool>);

    // Test ping
    let ping_result = pool.ping().await;
    assert!(ping_result.is_ok());

    // Test begin transaction
    let tx_result = pool.begin().await;
    assert!(tx_result.is_ok());

    // Get a transaction
    let tx = tx_result.unwrap();

    // Test commit
    let commit_result = tx.commit().await;
    assert!(commit_result.is_ok());
}

#[tokio::test]
async fn test_transaction_with_queries() {
    // Test transaction usage with query execution patterns
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    // Create a mock connection
    let pool = MockDatabaseConnection::new(&config);

    // Begin a transaction
    let tx_result = pool.begin().await;
    assert!(tx_result.is_ok());

    let tx = tx_result.unwrap();

    // In a real scenario, we'd run queries inside the transaction
    // Here we'll just simulate the pattern

    // Commit the transaction
    let commit_result = tx.commit().await;
    assert!(commit_result.is_ok());
}

#[tokio::test]
async fn test_add_user_to_mock_db() {
    // Test that we can add a user to the mock database
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    // Create mock connection
    let mock_conn = MockDatabaseConnection::new(&config);

    // Create a test user
    let user = User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        is_active: true,
        role: UserRole::User,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Add the user to the mock database
    mock_conn.add_user(user.clone());

    // Test finding the user
    let found_user = mock_conn.find_user_by_id(user.id);
    assert!(found_user.is_some());

    // Verify user data
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.username, user.username);
    assert_eq!(found_user.email, user.email);
}

#[tokio::test]
async fn test_user_queries_by_criteria() {
    // Test querying users by different criteria
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    // Create mock connection
    let mock_conn = MockDatabaseConnection::new(&config);

    // Add several test users
    let user1 = User {
        id: Uuid::new_v4(),
        username: "user1".to_string(),
        email: "user1@example.com".to_string(),
        full_name: Some("User One".to_string()),
        is_active: true,
        role: UserRole::User,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let user2 = User {
        id: Uuid::new_v4(),
        username: "user2".to_string(),
        email: "user2@example.com".to_string(),
        full_name: Some("User Two".to_string()),
        is_active: true,
        role: UserRole::Admin,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    mock_conn.add_user(user1.clone());
    mock_conn.add_user(user2.clone());

    // Test finding by username
    let found_user = mock_conn.find_user_by_username(&user1.username);
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().id, user1.id);

    // Test finding by email
    let found_user = mock_conn.find_user_by_email(&user2.email);
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().id, user2.id);

    // Test non-existent user
    let not_found = mock_conn.find_user_by_username("nonexistent");
    assert!(not_found.is_none());
}
