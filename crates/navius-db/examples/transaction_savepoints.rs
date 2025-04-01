use navius_db::{DatabaseConnectionManager, DatabaseError, DatabaseResult, PgPool, PoolOptions};
use std::env;
use std::sync::Arc;
use std::time::Duration;

/// This example demonstrates the use of savepoints and nested transactions.
///
/// It showcases:
/// 1. Basic transaction management
/// 2. Savepoint creation and usage
/// 3. Nested transactions
/// 4. Retry logic for transient errors
///
/// To run this example:
/// ```bash
/// DATABASE_URL=postgres://user:password@localhost:5432/mydatabase cargo run --example transaction_savepoints
/// ```
#[tokio::main]
async fn main() -> DatabaseResult<()> {
    // Set up the database connection
    let db = setup_database().await?;

    // Set up test tables
    create_test_tables(&db).await?;

    println!("== Basic Savepoint Example ==");
    basic_savepoint_example(&db).await?;

    println!("\n== Nested Transaction Example ==");
    nested_transaction_example(&db).await?;

    println!("\n== Retry Logic Example ==");
    retry_logic_example(&db).await?;

    println!("\n== Fund Transfer Example ==");
    fund_transfer_example(&db).await?;

    Ok(())
}

/// Set up the database connection
async fn setup_database() -> DatabaseResult<Arc<DatabaseConnectionManager>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    println!("Connecting to database...");

    let pool_options = PoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(5));

    let pool = PgPool::connect_with_options(&database_url, pool_options)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database.");

    Ok(Arc::new(DatabaseConnectionManager::new(pool)))
}

/// Create test tables for the examples
async fn create_test_tables(db: &DatabaseConnectionManager) -> DatabaseResult<()> {
    let conn = db.get_connection().await?;

    println!("Creating test tables...");

    // Accounts table for fund transfers
    conn.execute(
        "DROP TABLE IF EXISTS accounts; 
         CREATE TABLE accounts (
             id SERIAL PRIMARY KEY,
             account_number VARCHAR(20) UNIQUE NOT NULL,
             balance DECIMAL(10, 2) NOT NULL DEFAULT 0
         );
         
         INSERT INTO accounts (account_number, balance) VALUES
         ('A001', 1000.00),
         ('A002', 500.00),
         ('A003', 250.00);",
    )
    .await?;

    // Transactions table to log transfers
    conn.execute(
        "DROP TABLE IF EXISTS transfer_logs;
         CREATE TABLE transfer_logs (
             id SERIAL PRIMARY KEY,
             from_account VARCHAR(20) NOT NULL,
             to_account VARCHAR(20) NOT NULL,
             amount DECIMAL(10, 2) NOT NULL,
             timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
             status VARCHAR(20) NOT NULL
         );",
    )
    .await?;

    // Items table for inventory management
    conn.execute(
        "DROP TABLE IF EXISTS inventory;
         CREATE TABLE inventory (
             id SERIAL PRIMARY KEY,
             item_code VARCHAR(20) UNIQUE NOT NULL,
             quantity INTEGER NOT NULL DEFAULT 0,
             reserved INTEGER NOT NULL DEFAULT 0
         );
         
         INSERT INTO inventory (item_code, quantity, reserved) VALUES
         ('ITEM001', 100, 0),
         ('ITEM002', 50, 0),
         ('ITEM003', 25, 0);",
    )
    .await?;

    println!("Test tables created.");

    Ok(())
}

/// Basic example of using savepoints
async fn basic_savepoint_example(db: &DatabaseConnectionManager) -> DatabaseResult<()> {
    db.transaction(|mut tx| async move {
        // Query initial account balances
        println!("Initial account balances:");
        print_account_balances(&mut tx).await?;

        // Create a savepoint
        println!("Creating savepoint 'before_updates'...");
        tx.savepoint("before_updates").await?;

        // Update account A001
        println!("Updating account A001...");
        tx.execute("UPDATE accounts SET balance = balance + 100 WHERE account_number = 'A001'")
            .await?;

        // Update account A002
        println!("Updating account A002...");
        tx.execute("UPDATE accounts SET balance = balance - 50 WHERE account_number = 'A002'")
            .await?;

        // Show balances after updates
        println!("Account balances after updates:");
        print_account_balances(&mut tx).await?;

        // Roll back to the savepoint
        println!("Rolling back to savepoint 'before_updates'...");
        tx.rollback_to_savepoint("before_updates").await?;

        // Show balances after rollback
        println!("Account balances after rollback:");
        print_account_balances(&mut tx).await?;

        // Make different updates
        println!("Making different updates...");
        tx.execute("UPDATE accounts SET balance = balance + 200 WHERE account_number = 'A003'")
            .await?;

        // Show final balances
        println!("Final account balances:");
        print_account_balances(&mut tx).await?;

        // Commit the transaction
        tx.commit().await?;
        println!("Transaction committed.");

        Ok(())
    })
    .await?;

    Ok(())
}

/// Example of using nested transactions
async fn nested_transaction_example(db: &DatabaseConnectionManager) -> DatabaseResult<()> {
    db.transaction(|mut tx| async move {
        println!("Starting parent transaction...");

        // Query initial inventory
        println!("Initial inventory:");
        print_inventory(&mut tx).await?;

        // Update item in parent transaction
        println!("Updating ITEM001 in parent transaction...");
        tx.execute("UPDATE inventory SET quantity = quantity + 10 WHERE item_code = 'ITEM001'")
            .await?;

        // Start a nested transaction
        println!("Starting nested transaction...");
        let nested_result = tx
            .nested(|| async {
                // Update items in nested transaction
                println!("Updating ITEM002 in nested transaction...");
                tx.execute(
                    "UPDATE inventory SET quantity = quantity + 5 WHERE item_code = 'ITEM002'",
                )
                .await?;

                // Simulate a failure in the nested transaction
                let should_fail = true; // Toggle this to see different behavior
                if should_fail {
                    println!("Simulating failure in nested transaction...");
                    return Err(DatabaseError::ValidationError(
                        "Simulated failure in nested transaction".to_string(),
                    ));
                }

                // This won't execute if should_fail is true
                println!("Updating ITEM003 in nested transaction...");
                tx.execute(
                    "UPDATE inventory SET quantity = quantity + 3 WHERE item_code = 'ITEM003'",
                )
                .await?;

                Ok(())
            })
            .await;

        // Check the result of the nested transaction
        if let Err(e) = &nested_result {
            println!("Nested transaction failed: {}", e);
        } else {
            println!("Nested transaction succeeded.");
        }

        // Show inventory after nested transaction (regardless of its success/failure)
        println!("Inventory after nested transaction:");
        print_inventory(&mut tx).await?;

        // Continue with parent transaction
        println!("Continuing with parent transaction...");
        tx.execute("UPDATE inventory SET reserved = 5 WHERE item_code = 'ITEM001'")
            .await?;

        // Show final inventory
        println!("Final inventory:");
        print_inventory(&mut tx).await?;

        // Commit the parent transaction
        tx.commit().await?;
        println!("Parent transaction committed.");

        Ok(())
    })
    .await?;

    Ok(())
}

/// Example of using retry logic for transient errors
async fn retry_logic_example(db: &DatabaseConnectionManager) -> DatabaseResult<()> {
    db.transaction(|mut tx| async move {
        println!("Starting transaction with retry logic...");

        // Create a savepoint for the retry operation
        tx.savepoint("retry_point").await?;

        // Initialize attempt counter
        let mut attempt_counter = Arc::new(std::sync::Mutex::new(0));
        let attempt_counter_clone = Arc::clone(&attempt_counter);

        // Use the with_retry method to automatically handle retries
        tx.with_retry(3, || {
            let counter = Arc::clone(&attempt_counter_clone);

            async move {
                // Increment the attempt counter
                let mut count = counter.lock().unwrap();
                *count += 1;
                let current_attempt = *count;
                drop(count);

                println!("Attempt {}/3...", current_attempt);

                // Simulate a transient error on the first two attempts
                if current_attempt < 3 {
                    println!("Simulating a transient error...");
                    return Err(DatabaseError::TransactionError(format!(
                        "Simulated transient error on attempt {}",
                        current_attempt
                    )));
                }

                // Third attempt succeeds
                println!("Attempt {} succeeded!", current_attempt);
                tx.execute(
                    "UPDATE inventory SET quantity = quantity + 1 WHERE item_code = 'ITEM003'",
                )
                .await?;

                Ok(())
            }
        })
        .await?;

        // Show final inventory after retries
        println!("Inventory after retry operation:");
        print_inventory(&mut tx).await?;

        // Commit the transaction
        tx.commit().await?;
        println!("Transaction committed.");

        Ok(())
    })
    .await?;

    Ok(())
}

/// Practical example: Fund transfer between accounts
async fn fund_transfer_example(db: &DatabaseConnectionManager) -> DatabaseResult<()> {
    // Function to transfer funds between accounts
    async fn transfer_funds(
        db: &DatabaseConnectionManager,
        from_account: &str,
        to_account: &str,
        amount: f64,
    ) -> DatabaseResult<()> {
        db.transaction(|mut tx| async move {
            println!("Starting fund transfer transaction...");
            println!(
                "Transferring ${:.2} from {} to {}",
                amount, from_account, to_account
            );

            // Check if source account exists and has sufficient funds
            let rows = tx
                .query_with(
                    "SELECT balance FROM accounts WHERE account_number = $1",
                    &[&from_account],
                )
                .await?;

            if rows.is_empty() {
                return Err(DatabaseError::NotFoundError(format!(
                    "Source account {} not found",
                    from_account
                )));
            }

            let balance: f64 = rows.get(0).get("balance");

            if balance < amount {
                return Err(DatabaseError::ValidationError(format!(
                    "Insufficient funds: ${:.2} available, ${:.2} requested",
                    balance, amount
                )));
            }

            // Create savepoint before making any changes
            tx.savepoint("before_transfer").await?;

            // Try to update the source account
            let rows_affected = tx
                .execute_with(
                    "UPDATE accounts SET balance = balance - $1 
                 WHERE account_number = $2 AND balance >= $1",
                    &[&amount, &from_account],
                )
                .await?;

            if rows_affected == 0 {
                // This could happen if another transaction modified the balance
                // Roll back to savepoint
                tx.rollback_to_savepoint("before_transfer").await?;

                return Err(DatabaseError::ConcurrencyError(
                    "Account balance was modified by another transaction".to_string(),
                ));
            }

            // Log the debit operation
            tx.execute_with(
                "INSERT INTO transfer_logs (from_account, to_account, amount, status)
                 VALUES ($1, $2, $3, 'DEBIT_COMPLETE')",
                &[&from_account, &to_account, &amount],
            )
            .await?;

            // Create another savepoint
            tx.savepoint("after_debit").await?;

            // Try to update the destination account
            // Use nested transaction for this part
            tx.nested(|| async {
                // Check if destination account exists
                let rows = tx
                    .query_with(
                        "SELECT id FROM accounts WHERE account_number = $1",
                        &[&to_account],
                    )
                    .await?;

                if rows.is_empty() {
                    return Err(DatabaseError::NotFoundError(format!(
                        "Destination account {} not found",
                        to_account
                    )));
                }

                // Update the destination account
                tx.execute_with(
                    "UPDATE accounts SET balance = balance + $1 
                     WHERE account_number = $2",
                    &[&amount, &to_account],
                )
                .await?;

                // Log the credit operation
                tx.execute_with(
                    "INSERT INTO transfer_logs (from_account, to_account, amount, status)
                     VALUES ($1, $2, $3, 'CREDIT_COMPLETE')",
                    &[&from_account, &to_account, &amount],
                )
                .await?;

                Ok(())
            })
            .await?;

            // Update the final status
            tx.execute_with(
                "INSERT INTO transfer_logs (from_account, to_account, amount, status)
                 VALUES ($1, $2, $3, 'TRANSFER_COMPLETE')",
                &[&from_account, &to_account, &amount],
            )
            .await?;

            // Commit the transaction
            tx.commit().await?;
            println!("Fund transfer successful!");

            Ok(())
        })
        .await
    }

    // Print initial account balances
    let conn = db.get_connection().await?;
    println!("Initial account balances:");
    let rows = conn
        .query("SELECT account_number, balance FROM accounts ORDER BY account_number")
        .await?;
    for row in rows.iter() {
        let account: &str = row.get("account_number");
        let balance: f64 = row.get("balance");
        println!("  {} = ${:.2}", account, balance);
    }

    // Perform a valid transfer
    println!("\nPerforming valid transfer: A001 -> A002 ($200.00)");
    let result = transfer_funds(db, "A001", "A002", 200.00).await;

    match &result {
        Ok(_) => println!("Transfer completed successfully."),
        Err(e) => println!("Transfer failed: {}", e),
    }

    // Try an invalid transfer (insufficient funds)
    println!("\nPerforming invalid transfer: A003 -> A001 ($1000.00)");
    let result = transfer_funds(db, "A003", "A001", 1000.00).await;

    match &result {
        Ok(_) => println!("Transfer completed successfully."),
        Err(e) => println!("Transfer failed: {}", e),
    }

    // Print final account balances
    println!("\nFinal account balances:");
    let rows = conn
        .query("SELECT account_number, balance FROM accounts ORDER BY account_number")
        .await?;
    for row in rows.iter() {
        let account: &str = row.get("account_number");
        let balance: f64 = row.get("balance");
        println!("  {} = ${:.2}", account, balance);
    }

    // Print transfer logs
    println!("\nTransfer logs:");
    let rows = conn
        .query(
            "SELECT from_account, to_account, amount, status, timestamp 
         FROM transfer_logs 
         ORDER BY timestamp",
        )
        .await?;

    for row in rows.iter() {
        let from: &str = row.get("from_account");
        let to: &str = row.get("to_account");
        let amount: f64 = row.get("amount");
        let status: &str = row.get("status");
        let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");

        println!(
            "  {} | {} -> {} | ${:.2} | {}",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            from,
            to,
            amount,
            status
        );
    }

    Ok(())
}

/// Helper function to print account balances
async fn print_account_balances(tx: &mut navius_db::Transaction<'_>) -> DatabaseResult<()> {
    let rows = tx
        .query("SELECT account_number, balance FROM accounts ORDER BY account_number")
        .await?;

    // We need to implement a way to iterate through rows
    let mut result_rows = Vec::new();
    let mut row_set = rows;
    while let Some(row) = row_set.next().await? {
        result_rows.push(row);
    }

    for row in result_rows {
        let account = row.get_column_value("account_number")?.unwrap();
        let balance = row.get_column_value("balance")?.unwrap();
        println!("Account: {}, Balance: {}", account, balance);
    }

    Ok(())
}

/// Helper function to print inventory
async fn print_inventory(tx: &mut navius_db::Transaction<'_>) -> DatabaseResult<()> {
    let rows = tx
        .query("SELECT item_code, quantity, reserved FROM inventory ORDER BY item_code")
        .await?;

    // We need to implement a way to iterate through rows
    let mut result_rows = Vec::new();
    let mut row_set = rows;
    while let Some(row) = row_set.next().await? {
        result_rows.push(row);
    }

    for row in result_rows {
        let item = row.get_column_value("item_code")?.unwrap();
        let quantity = row.get_column_value("quantity")?.unwrap();
        let reserved = row.get_column_value("reserved")?.unwrap();
        println!(
            "Item: {}, Quantity: {}, Reserved: {}",
            item, quantity, reserved
        );
    }

    Ok(())
}
