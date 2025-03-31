//! # Database Transaction Testing Example
//!
//! This example demonstrates how to test database transaction behavior using
//! the Navius Test Framework. It covers common scenarios including:
//!
//! - Testing successful transaction commits
//! - Testing transaction rollbacks
//! - Testing error propagation during transactions
//! - Testing nested transactions
//! - Verifying transaction isolation
//!
//! The example uses mock implementations to simulate database behavior
//! without requiring an actual database connection.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use navius_test::{
    error::{TestResult, assert_eq, assert_false, assert_true},
    fixture::TestFixture,
    mocks::database::{MockDatabaseClient, MockDatabaseError, MockQueryResult, MockValue},
};

/// A service that uses database transactions to perform operations
struct UserService {
    db: MockDatabaseClient,
}

impl UserService {
    /// Create a new user service with the given database client
    fn new(db: MockDatabaseClient) -> Self {
        Self { db }
    }

    /// Create a user with the given information, using a transaction
    async fn create_user(&self, name: &str, email: &str) -> Result<i64, MockDatabaseError> {
        // Begin a transaction
        let transaction = self.db.begin_transaction().await?;

        // Insert the user
        transaction
            .execute(
                "INSERT INTO users (name, email) VALUES (?, ?)",
                &[name, email],
            )
            .await?;

        // Get the user ID
        let result = transaction
            .query(
                "SELECT id FROM users WHERE email = ? ORDER BY id DESC LIMIT 1",
                &[email],
            )
            .await?;

        if result.rows.is_empty() {
            // Roll back the transaction if we couldn't find the user
            transaction.rollback().await?;
            return Err(MockDatabaseError::new("Failed to create user"));
        }

        let user_id = result.rows[0]
            .get("id")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| MockDatabaseError::new("Invalid user ID"))?;

        // Create initial settings for the user
        transaction.execute(
            "INSERT INTO user_settings (user_id, theme, notifications) VALUES (?, 'default', true)",
            &[&user_id],
        ).await?;

        // Commit the transaction
        transaction.commit().await?;

        Ok(user_id)
    }

    /// Update a user's settings, rolling back if there's an error
    async fn update_user_settings(
        &self,
        user_id: i64,
        theme: &str,
        notifications: bool,
    ) -> Result<bool, MockDatabaseError> {
        // Begin a transaction
        let transaction = self.db.begin_transaction().await?;

        // Check if the user exists
        let user_exists = transaction
            .query("SELECT 1 FROM users WHERE id = ?", &[&user_id])
            .await?;

        if user_exists.rows.is_empty() {
            transaction.rollback().await?;
            return Err(MockDatabaseError::new("User not found"));
        }

        // Update the settings
        transaction
            .execute(
                "UPDATE user_settings SET theme = ?, notifications = ? WHERE user_id = ?",
                &[theme, &notifications, &user_id],
            )
            .await?;

        // Commit the transaction
        transaction.commit().await?;

        Ok(true)
    }

    /// Delete a user and all associated data in a transaction
    async fn delete_user(&self, user_id: i64) -> Result<bool, MockDatabaseError> {
        // Begin a transaction
        let transaction = self.db.begin_transaction().await?;

        // Delete user settings
        transaction
            .execute("DELETE FROM user_settings WHERE user_id = ?", &[&user_id])
            .await?;

        // Delete user posts
        transaction
            .execute("DELETE FROM posts WHERE user_id = ?", &[&user_id])
            .await?;

        // Delete the user
        let result = transaction
            .execute("DELETE FROM users WHERE id = ?", &[&user_id])
            .await?;

        if result == 0 {
            // No user was deleted, roll back
            transaction.rollback().await?;
            return Err(MockDatabaseError::new("User not found"));
        }

        // Commit the transaction
        transaction.commit().await?;

        Ok(true)
    }
}

/// A mock database transaction for testing
#[derive(Clone)]
struct MockTransaction {
    /// The ID of the transaction
    id: String,
    /// The state of the transaction (active, committed, rolled back)
    state: Arc<Mutex<TransactionState>>,
    /// The mock database client
    db: MockDatabaseClient,
}

/// The state of a transaction
#[derive(Debug, Clone, PartialEq)]
enum TransactionState {
    /// The transaction is active
    Active,
    /// The transaction has been committed
    Committed,
    /// The transaction has been rolled back
    RolledBack,
}

impl MockTransaction {
    /// Create a new mock transaction
    fn new(id: &str, db: MockDatabaseClient) -> Self {
        Self {
            id: id.to_string(),
            state: Arc::new(Mutex::new(TransactionState::Active)),
            db,
        }
    }

    /// Execute a query in the transaction
    async fn execute(
        &self,
        query: &str,
        params: &[&dyn std::fmt::Debug],
    ) -> Result<u64, MockDatabaseError> {
        // Check if the transaction is still active
        let state = self.state.lock().unwrap();
        if *state != TransactionState::Active {
            return Err(MockDatabaseError::new("Transaction is no longer active"));
        }
        drop(state);

        // Forward to the database client with transaction context
        self.db
            .execute_with_transaction(self.id.clone(), query, params)
            .await
    }

    /// Query the database in the transaction
    async fn query(
        &self,
        query: &str,
        params: &[&dyn std::fmt::Debug],
    ) -> Result<MockQueryResult, MockDatabaseError> {
        // Check if the transaction is still active
        let state = self.state.lock().unwrap();
        if *state != TransactionState::Active {
            return Err(MockDatabaseError::new("Transaction is no longer active"));
        }
        drop(state);

        // Forward to the database client with transaction context
        self.db
            .query_with_transaction(self.id.clone(), query, params)
            .await
    }

    /// Commit the transaction
    async fn commit(&self) -> Result<(), MockDatabaseError> {
        let mut state = self.state.lock().unwrap();
        if *state != TransactionState::Active {
            return Err(MockDatabaseError::new("Transaction is no longer active"));
        }
        *state = TransactionState::Committed;

        // Tell the database client about the commit
        self.db.commit_transaction(self.id.clone()).await
    }

    /// Roll back the transaction
    async fn rollback(&self) -> Result<(), MockDatabaseError> {
        let mut state = self.state.lock().unwrap();
        if *state != TransactionState::Active {
            return Err(MockDatabaseError::new("Transaction is no longer active"));
        }
        *state = TransactionState::RolledBack;

        // Tell the database client about the rollback
        self.db.rollback_transaction(self.id.clone()).await
    }
}

/// Extension methods for MockDatabaseClient to support transactions
impl MockDatabaseClient {
    /// Begin a new transaction
    async fn begin_transaction(&self) -> Result<MockTransaction, MockDatabaseError> {
        // Generate a unique transaction ID
        let tx_id = format!("tx_{}", uuid::Uuid::new_v4());

        // Return a new transaction with this client
        Ok(MockTransaction::new(&tx_id, self.clone()))
    }

    /// Execute a query within a transaction context
    async fn execute_with_transaction(
        &self,
        tx_id: String,
        query: &str,
        params: &[&dyn std::fmt::Debug],
    ) -> Result<u64, MockDatabaseError> {
        // In a real implementation, we would track the transaction state
        // and apply changes only on commit
        // For this example, we'll just pass through to execute
        self.execute(query, params).await
    }

    /// Query the database within a transaction context
    async fn query_with_transaction(
        &self,
        tx_id: String,
        query: &str,
        params: &[&dyn std::fmt::Debug],
    ) -> Result<MockQueryResult, MockDatabaseError> {
        // In a real implementation, we would track the transaction state
        // and apply changes only on commit
        // For this example, we'll just pass through to query
        self.query(query, params).await
    }

    /// Commit a transaction
    async fn commit_transaction(&self, tx_id: String) -> Result<(), MockDatabaseError> {
        // In a real implementation, we would apply all changes made in the transaction
        // For this example, we'll just return success
        Ok(())
    }

    /// Roll back a transaction
    async fn rollback_transaction(&self, tx_id: String) -> Result<(), MockDatabaseError> {
        // In a real implementation, we would discard all changes made in the transaction
        // For this example, we'll just return success
        Ok(())
    }
}

/// Example of testing successful transaction commits
#[tokio::test]
async fn test_successful_transaction_commit() -> TestResult<()> {
    // Create a test fixture
    let fixture = TestFixture::new();

    // Create a mock database client
    let db = MockDatabaseClient::new();

    // Configure database expectations

    // The transaction should start
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx_1", db.clone())));

    // First an INSERT will be executed
    db.expect_execute_with_transaction(
        "tx_1",
        "INSERT INTO users (name, email) VALUES (?, ?)",
        &["Test User", "test@example.com"],
    )
    .returns(Ok(1));

    // Then a query to get the user ID
    db.expect_query_with_transaction(
        "tx_1",
        "SELECT id FROM users WHERE email = ? ORDER BY id DESC LIMIT 1",
        &["test@example.com"],
    )
    .returns(Ok(MockQueryResult::new().add_row({
        let mut row = HashMap::new();
        row.insert("id".to_string(), MockValue::Integer(123));
        row
    })));

    // Then another INSERT for the settings
    db.expect_execute_with_transaction(
        "tx_1",
        "INSERT INTO user_settings (user_id, theme, notifications) VALUES (?, 'default', true)",
        &[&123],
    )
    .returns(Ok(1));

    // Finally, the transaction should be committed
    db.expect_commit_transaction("tx_1").returns(Ok(()));

    // Create the service to test
    let service = UserService::new(db.clone());

    // Test the create_user method
    let user_id = service.create_user("Test User", "test@example.com").await?;

    // Verify the result
    assert_eq(user_id, 123, "Should return the correct user ID")?;

    // Verify that all mock expectations were met
    // In a real implementation, we would verify that each method was called exactly once

    Ok(())
}

/// Example of testing transaction rollbacks
#[tokio::test]
async fn test_transaction_rollback() -> TestResult<()> {
    // Create a test fixture
    let fixture = TestFixture::new();

    // Create a mock database client
    let db = MockDatabaseClient::new();

    // Configure database expectations

    // The transaction should start
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx_1", db.clone())));

    // Check if the user exists - it doesn't
    db.expect_query_with_transaction("tx_1", "SELECT 1 FROM users WHERE id = ?", &[&456])
        .returns(Ok(MockQueryResult::new())); // Empty result means user doesn't exist

    // The transaction should be rolled back
    db.expect_rollback_transaction("tx_1").returns(Ok(()));

    // Create the service to test
    let service = UserService::new(db.clone());

    // Test the update_user_settings method with a non-existent user
    let result = service.update_user_settings(456, "dark", true).await;

    // Verify that the method returns an error
    assert_true(
        result.is_err(),
        "Should return an error for non-existent user",
    )?;

    // Verify the error message
    let error = result.unwrap_err();
    assert_eq(
        error.message(),
        "User not found",
        "Should return the correct error message",
    )?;

    // Verify that all mock expectations were met

    Ok(())
}

/// Example of testing error propagation during transactions
#[tokio::test]
async fn test_transaction_error_propagation() -> TestResult<()> {
    // Create a test fixture
    let fixture = TestFixture::new();

    // Create a mock database client
    let db = MockDatabaseClient::new();

    // Configure database expectations

    // The transaction should start
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx_1", db.clone())));

    // The first operation succeeds
    db.expect_execute_with_transaction(
        "tx_1",
        "DELETE FROM user_settings WHERE user_id = ?",
        &[&789],
    )
    .returns(Ok(1));

    // The second operation fails
    db.expect_execute_with_transaction("tx_1", "DELETE FROM posts WHERE user_id = ?", &[&789])
        .returns(Err(MockDatabaseError::new("Database connection lost")));

    // The transaction should be rolled back automatically (not explicitly called in this test)

    // Create the service to test
    let service = UserService::new(db.clone());

    // Test the delete_user method
    let result = service.delete_user(789).await;

    // Verify that the method returns an error
    assert_true(
        result.is_err(),
        "Should return an error when a database operation fails",
    )?;

    // Verify the error message
    let error = result.unwrap_err();
    assert_eq(
        error.message(),
        "Database connection lost",
        "Should propagate the database error",
    )?;

    // Verify that all mock expectations were met

    Ok(())
}

/// Example of testing nested transaction scenarios
#[tokio::test]
async fn test_nested_transactions() -> TestResult<()> {
    // Create a test fixture
    let fixture = TestFixture::new();

    // Create a mock database client
    let db = MockDatabaseClient::new();

    // Configure expectations for the outer transaction
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx_outer", db.clone())));

    // Configure expectations for the inner transaction
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx_inner", db.clone())));

    // Operation in the outer transaction
    db.expect_execute_with_transaction(
        "tx_outer",
        "INSERT INTO audit_log (action, description) VALUES (?, ?)",
        &["begin_update", "Starting user update"],
    )
    .returns(Ok(1));

    // Operation in the inner transaction
    db.expect_execute_with_transaction(
        "tx_inner",
        "UPDATE users SET name = ? WHERE id = ?",
        &["New Name", &123],
    )
    .returns(Ok(1));

    // Commit the inner transaction
    db.expect_commit_transaction("tx_inner").returns(Ok(()));

    // Another operation in the outer transaction
    db.expect_execute_with_transaction(
        "tx_outer",
        "INSERT INTO audit_log (action, description) VALUES (?, ?)",
        &["end_update", "Completed user update"],
    )
    .returns(Ok(1));

    // Commit the outer transaction
    db.expect_commit_transaction("tx_outer").returns(Ok(()));

    // Execute a nested transaction scenario
    // This would typically be wrapped in a service method, but we'll test it directly

    // Start the outer transaction
    let outer_tx = db.begin_transaction().await?;

    // Perform an operation in the outer transaction
    outer_tx
        .execute(
            "INSERT INTO audit_log (action, description) VALUES (?, ?)",
            &["begin_update", "Starting user update"],
        )
        .await?;

    // Start an inner transaction
    let inner_tx = db.begin_transaction().await?;

    // Perform an operation in the inner transaction
    inner_tx
        .execute(
            "UPDATE users SET name = ? WHERE id = ?",
            &["New Name", &123],
        )
        .await?;

    // Commit the inner transaction
    inner_tx.commit().await?;

    // Perform another operation in the outer transaction
    outer_tx
        .execute(
            "INSERT INTO audit_log (action, description) VALUES (?, ?)",
            &["end_update", "Completed user update"],
        )
        .await?;

    // Commit the outer transaction
    outer_tx.commit().await?;

    // Verify that all mock expectations were met

    Ok(())
}

/// Main function to run examples
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running database transaction testing examples...");

    // Run the transaction commit example
    match test_successful_transaction_commit().await {
        Ok(_) => println!("✅ Transaction commit example passed"),
        Err(e) => println!("❌ Transaction commit example failed: {}", e),
    }

    // Run the transaction rollback example
    match test_transaction_rollback().await {
        Ok(_) => println!("✅ Transaction rollback example passed"),
        Err(e) => println!("❌ Transaction rollback example failed: {}", e),
    }

    // Run the error propagation example
    match test_transaction_error_propagation().await {
        Ok(_) => println!("✅ Transaction error propagation example passed"),
        Err(e) => println!("❌ Transaction error propagation example failed: {}", e),
    }

    // Run the nested transactions example
    match test_nested_transactions().await {
        Ok(_) => println!("✅ Nested transactions example passed"),
        Err(e) => println!("❌ Nested transactions example failed: {}", e),
    }

    println!("Database transaction testing examples completed!");

    Ok(())
}

/*
 * Best Practices for Testing Database Transactions:
 *
 * 1. Test Both Success and Failure Paths:
 *    Always test both successful transactions and various failure scenarios.
 *
 * 2. Verify Transaction Boundaries:
 *    Ensure that operations properly commit or roll back as expected.
 *
 * 3. Test Data Consistency:
 *    Verify that related data remains consistent after transactions.
 *
 * 4. Mock Transaction Behavior:
 *    Create mock implementations that simulate transaction behavior.
 *
 * 5. Test Isolation Levels:
 *    If your application uses specific isolation levels, test their effects.
 *
 * 6. Test Concurrency:
 *    Test how transactions behave when multiple operations occur simultaneously.
 *
 * 7. Test Performance:
 *    For critical paths, test transaction performance characteristics.
 *
 * 8. Test Error Handling:
 *    Ensure that transaction errors are properly handled and don't leave
 *    the system in an inconsistent state.
 *
 * 9. Test Nested Transactions:
 *    If your system uses nested or savepoint transactions, test their behavior.
 *
 * 10. Use a Consistent Mocking Approach:
 *     Standardize how you mock transaction behavior across tests.
 */
