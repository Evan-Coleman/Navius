use navius_db::{DatabaseConnectionManager, DatabaseError, PgPool, PoolOptions, Transaction};
use std::sync::Arc;
use tokio::test;

// Helper function to create a test database pool
async fn create_test_pool() -> PgPool {
    let options = PoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(3));

    // You would typically use an env var or test config here
    // For tests, we're using a mock implementation
    PgPool::new_mock(options)
}

#[test]
async fn test_transaction_commit() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);

    // Act & Assert
    let result = db
        .transaction(|mut tx| async move {
            // Execute some queries in the transaction
            let affected_rows = tx
                .execute("INSERT INTO test_table (name) VALUES ('test')")
                .await?;
            assert_eq!(affected_rows, 1);

            // Commit happens automatically at the end of the closure if no error
            Ok::<_, DatabaseError>(42)
        })
        .await;

    // Verify transaction was successful
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
async fn test_transaction_rollback() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);

    // Act & Assert
    let result: Result<(), DatabaseError> = db
        .transaction(|mut tx| async move {
            // Execute a query that will succeed
            let affected_rows = tx
                .execute("INSERT INTO test_table (name) VALUES ('test')")
                .await?;
            assert_eq!(affected_rows, 1);

            // Then force an error to trigger rollback
            Err(DatabaseError::ValidationError(
                "Forced error for testing".to_string(),
            ))
        })
        .await;

    // Verify transaction was rolled back due to the error
    assert!(result.is_err());
    match result {
        Err(DatabaseError::ValidationError(msg)) => {
            assert_eq!(msg, "Forced error for testing");
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
async fn test_transaction_explicit_commit_rollback() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);
    let mut conn = db.get_connection().await.unwrap();

    // Test explicit commit
    {
        let tx = conn.begin().await.unwrap();
        // The transaction is explicitly committed
        tx.commit().await.unwrap();
    }

    // Get a new connection for the rollback test
    let mut conn = db.get_connection().await.unwrap();

    // Test explicit rollback
    {
        let tx = conn.begin().await.unwrap();
        // The transaction is explicitly rolled back
        tx.rollback().await.unwrap();
    }
}

#[test]
async fn test_transaction_drop_behavior() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);

    // Act - Create a transaction scope where the transaction will be dropped without commit/rollback
    {
        let mut conn = db.get_connection().await.unwrap();
        let _tx = conn.begin().await.unwrap();
        // Transaction goes out of scope here without explicit commit/rollback
    }

    // Assert - This is mostly checking that no panic occurs
    // The actual rollback behavior is handled by the database driver in its Drop implementation
    // and is logged but we can't easily verify it in a unit test
}

#[test]
async fn test_multiple_queries_in_transaction() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);

    // Act & Assert
    let result = db
        .transaction(|mut tx| async move {
            // First query
            tx.execute("INSERT INTO test_table (name) VALUES ('test1')")
                .await?;

            // Second query
            tx.execute("INSERT INTO test_table (name) VALUES ('test2')")
                .await?;

            // Query that returns data
            let rows = tx
                .query("SELECT * FROM test_table WHERE name = 'test1'")
                .await?;

            // Return success
            Ok::<_, DatabaseError>(())
        })
        .await;

    // Verify transaction was successful
    assert!(result.is_ok());
}

#[test]
async fn test_transaction_already_committed() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);
    let mut conn = db.get_connection().await.unwrap();
    let tx = conn.begin().await.unwrap();

    // Act - Commit the transaction
    tx.commit().await.unwrap();

    // Assert - Attempting a second commit should fail
    // Note: tx is consumed by commit, so we can't test this directly
    // This test is mostly for documentation
}

#[test]
async fn test_nested_transactions() {
    // Arrange
    let pool = create_test_pool().await;
    let db = DatabaseConnectionManager::new(pool);

    // Act & Assert - Start an outer transaction
    let result = db
        .transaction(|mut tx_outer| async move {
            // Do something in the outer transaction
            tx_outer
                .execute("INSERT INTO test_table (name) VALUES ('outer')")
                .await?;

            // Get a new transaction - in real database this would be a savepoint
            let db_inner = DatabaseConnectionManager::new(pool);
            let result_inner = db_inner
                .transaction(|mut tx_inner| async move {
                    // Do something in the inner transaction
                    tx_inner
                        .execute("INSERT INTO test_table (name) VALUES ('inner')")
                        .await?;

                    // Return success from inner
                    Ok::<_, DatabaseError>(())
                })
                .await?;

            // Return success from outer
            Ok::<_, DatabaseError>(())
        })
        .await;

    // Verify transaction was successful
    assert!(result.is_ok());
}
