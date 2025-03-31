# Database Transaction Testing Guide

This guide covers best practices for testing database transactions using the Navius Test Framework. It complements the code example in `workspace_migration/examples/crates/navius-test/examples/database_transaction_testing.rs`.

## Overview

Database transactions are critical for maintaining data integrity in applications. Proper testing of transaction behavior ensures that your application correctly handles commit, rollback, and error scenarios. The Navius Test Framework provides tools to simplify transaction testing without requiring a real database.

## Key Transaction Testing Scenarios

A comprehensive transaction testing strategy should cover these key scenarios:

1. **Successful Transactions**
   - Verify data is committed correctly
   - Ensure all operations within the transaction succeed
   - Check that the transaction completes with the expected result

2. **Transaction Rollbacks**
   - Test explicit rollbacks
   - Verify automatic rollbacks on errors
   - Ensure no partial changes are committed

3. **Error Handling**
   - Test error propagation from database operations
   - Verify that errors trigger appropriate rollbacks
   - Check that meaningful error information is returned

4. **Nested Transactions**
   - Test transactions within transactions
   - Verify savepoint behavior
   - Test rollback of inner transactions without affecting outer ones

5. **Transaction Isolation**
   - Test that changes within a transaction are not visible outside until committed
   - Verify isolation level behavior if your application uses specific isolation levels

## Using Mock Database Components

The Navius Test Framework provides mock database components that simplify transaction testing:

### MockDatabaseClient

The `MockDatabaseClient` allows you to simulate database operations:

```rust
use navius_test::mocks::db::MockDatabaseClient;

let db_client = MockDatabaseClient::new();

// Set up expectations for transaction operations
db_client.expect_begin_transaction()
    .returns(Ok(MockTransaction::new("tx1")));

db_client.expect_query(
    "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
    &["John Doe", "john@example.com"]
)
.returns(Ok(QueryResult::with_id(1)));
```

### MockTransaction

The `MockTransaction` simulates database transaction behavior:

```rust
use navius_test::mocks::db::MockTransaction;

// Create a transaction with a specific ID
let transaction = MockTransaction::new("tx1");

// Set up expectations for commit
transaction.expect_commit()
    .returns(Ok(()));

// Set up expectations for rollback
transaction.expect_rollback()
    .returns(Ok(()));
```

## Testing Transaction Patterns

### Basic Transaction Testing

Test a simple transaction that should commit successfully:

```rust
#[tokio::test]
async fn test_successful_transaction() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create and configure mock database
    let db = MockDatabaseClient::new();
    
    // Set up transaction expectations
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx1")));
        
    // Set up query expectations
    db.expect_query_in_transaction(
        "tx1",
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        &["John Doe", "john@example.com"]
    )
    .returns(Ok(QueryResult::with_id(1)));
    
    // Set up commit expectations
    db.expect_commit_transaction("tx1")
        .returns(Ok(()));
    
    // Create service and execute transaction
    let service = UserService::new(db.clone());
    let result = service.create_user("John Doe", "john@example.com").await?;
    
    // Verify result
    assert_eq(result, 1, "Should return the inserted user ID")?;
    
    Ok(())
}
```

### Testing Rollbacks

Test that a transaction rolls back correctly on error:

```rust
#[tokio::test]
async fn test_transaction_rollback() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create and configure mock database
    let db = MockDatabaseClient::new();
    
    // Set up transaction expectations
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx1")));
        
    // Set up query expectations - simulate a constraint violation
    db.expect_query_in_transaction(
        "tx1",
        "UPDATE users SET email = $1 WHERE id = $2",
        &["new@example.com", "999"]
    )
    .returns(Err(DbError::ConstraintViolation("No such user")));
    
    // Set up rollback expectations
    db.expect_rollback_transaction("tx1")
        .returns(Ok(()));
    
    // Create service and execute transaction
    let service = UserService::new(db.clone());
    let result = service.update_email(999, "new@example.com").await;
    
    // Verify error result
    assert_err(result, "Should return an error for non-existent user")?;
    
    Ok(())
}
```

### Testing Nested Transactions

Test that nested transactions work correctly:

```rust
#[tokio::test]
async fn test_nested_transactions() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create and configure mock database
    let db = MockDatabaseClient::new();
    
    // Set up outer transaction expectations
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx_outer")));
    
    // Set up first operation in outer transaction
    db.expect_query_in_transaction(
        "tx_outer",
        "INSERT INTO orders (user_id, total) VALUES ($1, $2) RETURNING id",
        &[1, 100.0]
    )
    .returns(Ok(QueryResult::with_id(101)));
    
    // Set up inner transaction (savepoint) expectations
    db.expect_begin_savepoint("tx_outer", "sp1")
        .returns(Ok(()));
    
    // Set up operation in inner transaction
    db.expect_query_in_savepoint(
        "tx_outer", "sp1",
        "INSERT INTO order_items (order_id, product_id, quantity) VALUES ($1, $2, $3)",
        &[101, "prod1", 2]
    )
    .returns(Ok(QueryResult::empty()));
    
    // Set up commit expectations for inner transaction
    db.expect_release_savepoint("tx_outer", "sp1")
        .returns(Ok(()));
    
    // Set up commit expectations for outer transaction
    db.expect_commit_transaction("tx_outer")
        .returns(Ok(()));
    
    // Create service and execute nested transactions
    let service = OrderService::new(db.clone());
    let result = service.create_order_with_items(1, 100.0, vec![("prod1", 2)]).await?;
    
    // Verify result
    assert_eq(result, 101, "Should return the inserted order ID")?;
    
    Ok(())
}
```

## Testing Transaction Isolation

Test that changes within a transaction are isolated until committed:

```rust
#[tokio::test]
async fn test_transaction_isolation() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create and configure mock database
    let db = MockDatabaseClient::new();
    
    // Set up transaction expectations
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx1")));
    
    // Set up query within transaction
    db.expect_query_in_transaction(
        "tx1",
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2 RETURNING balance",
        &[100.0, 1]
    )
    .returns(Ok(QueryResult::with_row(vec![("balance", 900.0)])));
    
    // Set up query outside transaction (should not see uncommitted changes)
    db.expect_query(
        "SELECT balance FROM accounts WHERE id = $1",
        &[1]
    )
    .returns(Ok(QueryResult::with_row(vec![("balance", 1000.0)])));
    
    // Set up commit expectations
    db.expect_commit_transaction("tx1")
        .returns(Ok(()));
    
    // Query after commit (should see committed changes)
    db.expect_query(
        "SELECT balance FROM accounts WHERE id = $1",
        &[1]
    )
    .returns(Ok(QueryResult::with_row(vec![("balance", 900.0)])));
    
    // Create service and execute test
    let service = AccountService::new(db.clone());
    
    // Start transaction but don't commit yet
    let tx = service.begin_transaction().await?;
    
    // Update within transaction
    let new_balance = service.withdraw_in_transaction(&tx, 1, 100.0).await?;
    assert_eq(new_balance, 900.0, "Balance should be updated in transaction")?;
    
    // Query outside transaction should not see changes
    let balance_before_commit = service.get_balance(1).await?;
    assert_eq(balance_before_commit, 1000.0, "Changes should not be visible outside transaction")?;
    
    // Commit transaction
    service.commit_transaction(tx).await?;
    
    // Query after commit should see changes
    let balance_after_commit = service.get_balance(1).await?;
    assert_eq(balance_after_commit, 900.0, "Changes should be visible after commit")?;
    
    Ok(())
}
```

## Best Practices

### 1. Test Real-World Transaction Patterns

Structure your tests to match how your application actually uses transactions rather than testing the transaction API directly.

### 2. Test Both Success and Failure Paths

Ensure you test both the successful path (commit) and various failure paths (rollback).

### 3. Verify Transaction Boundaries

Verify that operations either fully complete or fully roll back, with no partial changes committed.

### 4. Test Explicit vs. Implicit Transactions

If your application uses both explicit transactions and implicit transactions (e.g., through an ORM), test both patterns.

### 5. Test Complex Scenarios

Test complex scenarios like multiple operations in one transaction and transactions that span multiple database calls.

### 6. Test Concurrent Transactions

When relevant, test how concurrent transactions interact, especially if your application uses specific isolation levels.

### 7. Use Helper Functions

Create helper functions to set up common transaction patterns in your tests:

```rust
async fn setup_transaction_test(db: &MockDatabaseClient, tx_id: &str) -> TestResult<()> {
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new(tx_id)));
    
    Ok(())
}

async fn expect_transaction_commit(db: &MockDatabaseClient, tx_id: &str) -> TestResult<()> {
    db.expect_commit_transaction(tx_id)
        .returns(Ok(()));
    
    Ok(())
}
```

## Common Pitfalls

1. **Testing the Database Rather Than Your Code**: Focus on testing your application's transaction handling logic, not the database itself.

2. **Not Testing Rollback Paths**: Ensure your tests verify that rollbacks work correctly when transactions fail.

3. **Ignoring Error Handling**: Test that your code properly handles and reports database errors within transactions.

4. **Incomplete Transaction Lifecycle**: Make sure your tests cover the entire transaction lifecycle, including proper cleanup.

5. **Forgetting Edge Cases**: Test edge cases like empty results, large result sets, and transactions with many operations.

## Specific Transaction Testing Patterns

### Testing Money Transfers (ACID Properties)

Money transfers are a classic example requiring ACID properties:

```rust
#[tokio::test]
async fn test_money_transfer() -> TestResult<()> {
    // Create test fixture and mock database
    let fixture = TestFixture::new();
    let db = MockDatabaseClient::new();
    
    // Set up transaction
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("transfer_tx")));
    
    // Debit from source account
    db.expect_query_in_transaction(
        "transfer_tx",
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2 AND balance >= $1 RETURNING id",
        &[50.0, 1]
    )
    .returns(Ok(QueryResult::with_id(1)));
    
    // Credit to destination account
    db.expect_query_in_transaction(
        "transfer_tx",
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        &[50.0, 2]
    )
    .returns(Ok(QueryResult::empty()));
    
    // Record transaction history
    db.expect_query_in_transaction(
        "transfer_tx",
        "INSERT INTO transfers (from_account, to_account, amount) VALUES ($1, $2, $3)",
        &[1, 2, 50.0]
    )
    .returns(Ok(QueryResult::empty()));
    
    // Commit transaction
    db.expect_commit_transaction("transfer_tx")
        .returns(Ok(()));
    
    // Execute test
    let service = TransferService::new(db.clone());
    let result = service.transfer(1, 2, 50.0).await;
    
    assert_ok(&result, "Transfer should succeed")?;
    
    Ok(())
}
```

### Testing Inventory Management (Concurrency)

Inventory updates often need to handle concurrency concerns:

```rust
#[tokio::test]
async fn test_inventory_update_with_optimistic_locking() -> TestResult<()> {
    // Create test fixture and mock database
    let fixture = TestFixture::new();
    let db = MockDatabaseClient::new();
    
    // Set up first attempted transaction
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx1")));
    
    // Get current version
    db.expect_query_in_transaction(
        "tx1",
        "SELECT quantity, version FROM inventory WHERE product_id = $1 FOR UPDATE",
        &["product1"]
    )
    .returns(Ok(QueryResult::with_rows(vec![
        vec![("quantity", 10), ("version", 1)]
    ])));
    
    // Update fails due to optimistic locking (another transaction updated it first)
    db.expect_query_in_transaction(
        "tx1",
        "UPDATE inventory SET quantity = $1, version = version + 1 WHERE product_id = $2 AND version = $3",
        &[5, "product1", 1]
    )
    .returns(Ok(QueryResult::with_affected_rows(0))); // No rows affected means version changed
    
    // Rollback first transaction
    db.expect_rollback_transaction("tx1")
        .returns(Ok(()));
    
    // Retry with new transaction
    db.expect_begin_transaction()
        .returns(Ok(MockTransaction::new("tx2")));
    
    // Get updated version
    db.expect_query_in_transaction(
        "tx2",
        "SELECT quantity, version FROM inventory WHERE product_id = $1 FOR UPDATE",
        &["product1"]
    )
    .returns(Ok(QueryResult::with_rows(vec![
        vec![("quantity", 8), ("version", 2)]
    ])));
    
    // Update succeeds this time
    db.expect_query_in_transaction(
        "tx2",
        "UPDATE inventory SET quantity = $1, version = version + 1 WHERE product_id = $2 AND version = $3",
        &[3, "product1", 2]
    )
    .returns(Ok(QueryResult::with_affected_rows(1)));
    
    // Commit second transaction
    db.expect_commit_transaction("tx2")
        .returns(Ok(()));
    
    // Execute test
    let service = InventoryService::new(db.clone());
    let result = service.reduce_inventory("product1", 5).await?;
    
    assert_eq(result, 3, "Final quantity should be 3")?;
    
    Ok(())
}
```

## Integration with Real Databases

For end-to-end tests, you may need to test with real databases. The Navius Test Framework supports this through integration fixtures:

```rust
#[tokio::test]
async fn test_real_database_transaction() -> TestResult<()> {
    // Create integration test context
    let ctx = IntegrationContext::new()?;
    
    // Get a real database connection
    let db = ctx.get_database("test_db").await?;
    
    // Set up test data
    db.execute("CREATE TABLE IF NOT EXISTS test_users (id SERIAL PRIMARY KEY, name TEXT)").await?;
    
    // Run transaction test with real database
    let tx = db.begin_transaction().await?;
    
    // Perform operations in transaction
    db.query_in_transaction(&tx, "INSERT INTO test_users (name) VALUES ($1)", &["Test User"]).await?;
    
    // Check that data exists in transaction
    let result = db.query_in_transaction(&tx, "SELECT id FROM test_users WHERE name = $1", &["Test User"]).await?;
    assert_eq(result.rows().len(), 1, "Should have inserted one row")?;
    
    // Check that data is not visible outside transaction
    let outside_result = db.query("SELECT id FROM test_users WHERE name = $1", &["Test User"]).await?;
    assert_eq(outside_result.rows().len(), 0, "Should not be visible outside transaction")?;
    
    // Commit transaction
    tx.commit().await?;
    
    // Now data should be visible
    let after_commit = db.query("SELECT id FROM test_users WHERE name = $1", &["Test User"]).await?;
    assert_eq(after_commit.rows().len(), 1, "Should be visible after commit")?;
    
    // Clean up
    db.execute("DROP TABLE test_users").await?;
    
    Ok(())
}
```

## Conclusion

Effective database transaction testing is crucial for ensuring data integrity in your application. By using the Navius Test Framework's database testing utilities, you can thoroughly test transaction behavior without requiring a real database in most cases, making your tests faster and more reliable.

For a complete code example, refer to `workspace_migration/examples/crates/navius-test/examples/database_transaction_testing.rs`. 