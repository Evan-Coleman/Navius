use navius_db::{DatabaseConnectionManager, DatabaseError, DatabaseResult, PgPool, PoolOptions};
use std::sync::Arc;
use tokio::sync::Mutex;
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

// Helper to create a test database connection
async fn create_test_db() -> Arc<DatabaseConnectionManager> {
    let pool_options = PoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_timeout(std::time::Duration::from_secs(5));

    // Use a test database URL - in a real test this would be a separate test database
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());

    let pool = PgPool::connect_with_options(&url, pool_options)
        .await
        .expect("Failed to connect to database");

    Arc::new(DatabaseConnectionManager::new(pool))
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

// This is a mock test that doesn't require an actual database connection
// It demonstrates the API usage rather than actual functionality
#[tokio::test]
async fn test_savepoint_usage_mock() -> DatabaseResult<()> {
    // This counter simulates rows affected by each operation
    let counter = Arc::new(Mutex::new(0));

    // Create a closure that uses the savepoint API
    let result = async {
        let counter_clone = Arc::clone(&counter);

        // Simulate a transaction with savepoints
        let simulate_transaction = |mut tx: navius_db::Transaction<'_>| async move {
            // First operation
            *counter_clone.lock().await += 1;
            println!("Executed first operation");

            // Create a savepoint
            tx.savepoint("point1").await?;
            println!("Created savepoint 'point1'");

            // Second operation - might fail
            let should_fail = false; // toggle this to simulate failure
            if should_fail {
                return Err(DatabaseError::ValidationError(
                    "Simulated failure".to_string(),
                ));
            }

            *counter_clone.lock().await += 1;
            println!("Executed second operation");

            // Create another savepoint
            tx.savepoint("point2").await?;
            println!("Created savepoint 'point2'");

            // Third operation in a nested transaction
            tx.nested(|| async {
                *counter_clone.lock().await += 1;
                println!("Executed third operation in nested transaction");
                Ok(())
            })
            .await?;

            // Release the second savepoint
            tx.release_savepoint("point2").await?;
            println!("Released savepoint 'point2'");

            // Fourth operation with retry logic
            tx.with_retry(3, || async {
                *counter_clone.lock().await += 1;
                println!("Executed fourth operation with retry");
                Ok(())
            })
            .await?;

            // Commit the transaction
            tx.commit().await?;
            println!("Committed transaction");

            Ok(())
        };

        // Since we don't have a real database, we're just using the API
        // to verify the code compiles and the pattern works
        Ok(())
    }
    .await;

    // In a real test with a database, we would assert on database state
    // Here we're just checking that our counter was incremented correctly
    let final_count = *counter.lock().await;
    assert_eq!(final_count, 4, "Expected 4 operations, got {}", final_count);

    result
}

// This test should only run when we have a real database connection
// Use DATABASE_URL environment variable to point to a test database
#[tokio::test]
#[ignore] // Skip by default, run with `cargo test -- --ignored`
async fn test_transaction_with_savepoints() -> DatabaseResult<()> {
    // Create test database with tables
    let db = create_test_db().await;

    // Set up test table
    db.connection()
        .await?
        .execute(
            "DROP TABLE IF EXISTS test_savepoints;
         CREATE TABLE test_savepoints (
            id SERIAL PRIMARY KEY,
            value TEXT NOT NULL
         );",
            &[],
        )
        .await?;

    // Test transaction with savepoints
    db.transaction(|mut tx| async move {
        // Insert initial record
        tx.execute("INSERT INTO test_savepoints (value) VALUES ('initial')")
            .await?;

        // Create a savepoint
        tx.savepoint("point1").await?;

        // Insert another record
        tx.execute("INSERT INTO test_savepoints (value) VALUES ('after_savepoint1')")
            .await?;

        // Create nested savepoint
        tx.nested(|| async {
            // Insert within nested transaction
            tx.execute("INSERT INTO test_savepoints (value) VALUES ('in_nested_tx')")
                .await?;

            // This would only be rolled back if we return an error
            Ok(())
        })
        .await?;

        // Check that all records exist
        let rows = tx
            .query("SELECT value FROM test_savepoints ORDER BY id")
            .await?;
        let count = rows.len();
        assert_eq!(count, 3, "Expected 3 rows, got {}", count);

        // Rollback to the first savepoint
        tx.rollback_to_savepoint("point1").await?;

        // Check that only the first record exists
        let rows = tx
            .query("SELECT value FROM test_savepoints ORDER BY id")
            .await?;
        let count = rows.len();
        assert_eq!(count, 1, "Expected 1 row after rollback, got {}", count);

        // Add a new record after rollback
        tx.execute("INSERT INTO test_savepoints (value) VALUES ('after_rollback')")
            .await?;

        // Try the retry functionality
        let mut attempts = 0;
        tx.with_retry(3, || async {
            attempts += 1;

            // Simulate success on second attempt
            if attempts == 1 {
                return Err(DatabaseError::TransactionError(
                    "Simulated transient error".to_string(),
                ));
            }

            tx.execute("INSERT INTO test_savepoints (value) VALUES ('after_retry')")
                .await?;

            Ok(())
        })
        .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    })
    .await?;

    // Verify final state
    let conn = db.connection().await?;
    let rows = conn
        .query("SELECT value FROM test_savepoints ORDER BY id", &[])
        .await?;
    let count = rows.len();
    assert_eq!(count, 3, "Expected 3 rows in final state, got {}", count);

    // Clean up
    conn.execute("DROP TABLE test_savepoints", &[]).await?;

    Ok(())
}

// Test that savepoint names are properly validated
#[tokio::test]
async fn test_savepoint_name_validation() {
    // Create test database
    let db = create_test_db().await;

    // Test invalid savepoint names
    let result = db
        .transaction(|mut tx| async move {
            // Valid name should succeed
            tx.savepoint("valid_name_123").await?;

            // Invalid names should fail
            let invalid_names = [
                "invalid-name",  // Dash
                "invalid;name",  // Semicolon
                "invalid name",  // Space
                "invalid'name",  // Quote
                "DROP TABLE;--", // SQL Injection attempt
            ];

            for name in invalid_names {
                let result = tx.savepoint(name).await;
                assert!(
                    result.is_err(),
                    "Savepoint with invalid name '{}' should fail",
                    name
                );

                // Verify the error type
                match result {
                    Err(DatabaseError::SavepointError(_)) => {
                        // Expected error
                    }
                    Err(e) => {
                        // Wrong error type
                        panic!("Expected SavepointError, got {:?}", e);
                    }
                    Ok(_) => {
                        // Should not succeed
                        panic!(
                            "Expected error for savepoint name '{}', but it succeeded",
                            name
                        );
                    }
                }
            }

            // The transaction succeeds, but we'll roll it back for cleanliness
            tx.rollback().await?;

            Ok(())
        })
        .await;

    // Transaction should complete successfully
    assert!(result.is_ok(), "Transaction failed: {:?}", result);

    Ok(())
}
