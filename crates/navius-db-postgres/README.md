# Navius PostgreSQL Provider

PostgreSQL implementation of the `DatabaseProvider` interface for the Navius framework.

## Features

- Fully-featured PostgreSQL provider implementation
- Advanced connection pooling with auto-scaling
- Comprehensive transaction support with savepoints and nested transactions
- Robust database migration system
- Comprehensive error handling

## Migration System

The navius-db-postgres crate includes a robust migration system for managing your database schema. It supports:

- Versioned migrations with a timestamp-based naming convention
- Transactional migration execution (all-or-nothing)
- Checksum validation to detect modified migrations
- Detailed status reporting
- Automatic discovery of migration files

### Migration File Format

Migration files should follow this naming convention:

```
V20250329000001__Create_users_table.sql
```

Where:
- `V` is a fixed prefix
- `20250329000001` is a timestamp-based version (YYYYMMDDhhmmss)
- `__` (double underscore) separates the version from the description
- `Create_users_table` is a description (words separated by underscores)
- `.sql` is the file extension

Each migration file should contain valid SQL statements for your schema changes.

### Using Migrations with the Provider

The PostgreSQL provider can be configured to automatically run and validate migrations on startup:

```rust
use navius_db_postgres::{MigrationOptions, PostgresProvider, PostgresProviderOptions};
use std::path::PathBuf;

// Configure provider options
let options = PostgresProviderOptions {
    pool_config: navius_db_postgres::pool::PgPoolConfig {
        url: "postgres://user:password@localhost/dbname".to_string(),
        max_connections: 10,
        min_connections: 1,
        max_lifetime: Some(std::time::Duration::from_secs(60 * 60)),
        idle_timeout: Some(std::time::Duration::from_secs(10 * 60)),
        acquire_timeout: std::time::Duration::from_secs(30),
    },
    migration_options: Some(MigrationOptions {
        migrations_dir: PathBuf::from("migrations"),
        migrations_table: Some("schema_migrations".to_string()),
        validate_checksums: true,
    }),
    run_migrations: true,  // Run migrations on startup
    validate_migrations: true,  // Validate migrations on startup
};

// Create provider
let provider = PostgresProvider::new(options).await?;
```

### Manual Migration Operations

You can also run and manage migrations manually:

```rust
// Run pending migrations
let count = provider.run_migrations().await?;
println!("Applied {} migrations", count);

// Validate migrations
let valid = provider.validate_migrations().await?;
if valid {
    println!("Migrations are valid");
} else {
    println!("Migration validation failed");
}

// Get migration status
let status = provider.migration_info().await?;
for s in status {
    match s.state {
        MigrationState::Applied { applied_at, success } => {
            println!(
                "Migration {} ({}) - Applied at {} ({})",
                s.migration.version,
                s.migration.description,
                applied_at,
                if success { "SUCCESS" } else { "FAILED" }
            );
        }
        MigrationState::Pending => {
            println!(
                "Migration {} ({}) - Pending",
                s.migration.version, s.migration.description
            );
        }
        MigrationState::Missing { applied_at, success } => {
            println!(
                "Migration {} ({}) - Missing from filesystem, applied at {} ({})",
                s.migration.version,
                s.migration.description,
                applied_at,
                if success { "SUCCESS" } else { "FAILED" }
            );
        }
    }
}
```

### Using the Migration API Directly

For more advanced use cases, you can use the migration API directly:

```rust
use navius_db_postgres::{MigrationOptions, MigrationRunner};
use sqlx::postgres::PgPoolOptions;
use std::path::PathBuf;

// Create connection pool
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect("postgres://user:password@localhost/dbname")
    .await?;

// Configure migration options
let options = MigrationOptions {
    migrations_dir: PathBuf::from("migrations"),
    migrations_table: Some("schema_migrations".to_string()),
    validate_checksums: true,
};

// Create migration runner
let runner = MigrationRunner::new(pool, options);

// Run migrations
let count = runner.run_migrations().await?;
println!("Applied {} migrations", count);
```

## Transaction Support

The PostgreSQL provider includes comprehensive transaction support:

```rust
// Get transaction manager
let tx_manager = provider.transaction_manager();

// Begin a transaction
let mut tx = tx_manager
    .begin_transaction(None)
    .await?;

// Execute queries
tx.execute_query("INSERT INTO users (name) VALUES ('John Doe')")
    .await?;

// Create a nested transaction (savepoint)
let mut nested_tx = tx
    .create_nested_transaction()
    .await?;

// Execute queries in the nested transaction
nested_tx
    .execute_query("UPDATE users SET name = 'Jane Doe' WHERE name = 'John Doe'")
    .await?;

// Commit or rollback the nested transaction
nested_tx.commit().await?; // or nested_tx.rollback().await?;

// Commit or rollback the main transaction
tx.commit().await?; // or tx.rollback().await?;

// Or use the transaction callback API
let result = tx_manager
    .with_transaction(|mut tx| {
        Box::pin(async move {
            // Execute queries
            tx.execute_query("INSERT INTO users (name) VALUES ('John Doe')")
                .await?;
            
            // Return a result
            Ok("success")
        })
    })
    .await?;
```

## Using the Provider

The navius-db-postgres crate implements the `DatabaseProvider` interface from the navius-db crate:

```rust
use navius_db::provider::{DatabaseProvider, ProviderOptions};
use std::collections::HashMap;

// Create provider options
let mut options = HashMap::new();
options.insert("database_url".to_string(), "postgres://user:password@localhost/dbname".to_string());
options.insert("max_connections".to_string(), "10".to_string());
options.insert("migrations_dir".to_string(), "/path/to/migrations".to_string());
options.insert("run_migrations".to_string(), "true".to_string());

// Create provider
let provider = navius_db_postgres::PostgresProvider::connect(options).await?;

// Use the provider
let health = provider.health_check().await?;
println!("Database health: {}", if health { "OK" } else { "FAILED" });
```

## How Migration Execution Works

1. The runner scans the migrations directory for files matching the naming pattern
2. It sorts migrations by version (timestamp)
3. It queries the database for applied migrations
4. It identifies pending migrations (those in the directory but not in the database)
5. For each pending migration, it:
   - Executes the SQL within a transaction
   - Records the migration in the migrations table
   - Commits the transaction if successful, or rolls back if an error occurs

## Error Handling

The migration system includes comprehensive error handling:

- Migration files that don't match the naming pattern are reported
- Empty migration files are rejected
- SQL syntax errors are caught and reported
- Version order issues are detected
- Checksum mismatches for modified migrations are identified

## Best Practices

- Use unique, descriptive names for migrations
- Keep migrations atomic (focused on a single change)
- Include both "up" and "down" logic in your migrations when possible
- Use transactions within your migrations for related changes
- Version control your migrations alongside your code
- Run migrations as part of your deployment process

## Performance Considerations

- The migration system is designed to be efficient and fast
- Migrations are executed in transactions for atomicity
- Checksums are cached to avoid redundant recalculation
- Only pending migrations are executed, minimizing database operations
- Version validation is optimized for quick verification

## Performance Tuning

Based on comprehensive benchmarks, we recommend the following configurations for optimal performance.

### Connection Pool Configuration

The connection pool configuration has a significant impact on performance. Based on benchmarks, we recommend:

**Production Environments:**
```rust
let options = PostgresProviderOptions {
    pool_config: PgPoolConfig {
        // Other settings...
        max_connections: 20, // 20-30 per instance
        min_connections: 5,  // 5-10 per instance
        max_lifetime: Some(Duration::from_secs(3600)), // 30-60 minutes
        idle_timeout: Some(Duration::from_secs(300)),  // 5-10 minutes
        acquire_timeout: Duration::from_secs(30),      // 30 seconds
    },
    // Other options...
}
```

**Development Environments:**
```rust
let options = PostgresProviderOptions {
    pool_config: PgPoolConfig {
        // Other settings...
        max_connections: 5,  // 5-10 connections
        min_connections: 2,  // 2-3 connections
        max_lifetime: Some(Duration::from_secs(900)),  // 10-15 minutes
        idle_timeout: Some(Duration::from_secs(180)),  // 2-5 minutes
        acquire_timeout: Duration::from_secs(10),      // 10 seconds
    },
    // Other options...
}
```

### Transaction Usage

For optimal transaction performance:

1. **Use the Transaction Callback API** when possible:
   ```rust
   let result = tx_manager.with_transaction(|mut tx| {
       Box::pin(async move {
           // Your transaction operations here
           Ok(result)
       })
   }).await?;
   ```

2. **Keep transactions short and focused** - Long-running transactions can lead to resource contention.

3. **Avoid nested transactions when possible** - Benchmarks show a ~20% overhead for nested transactions.

4. **Use prepared statements** within transactions for better performance.

### Query Optimization

1. **Use indexed columns** in WHERE clauses when possible.
   - Primary key lookups are ~30x faster than non-indexed queries
   - Index-based queries are ~12x faster than non-indexed queries

2. **Limit result sets** to reduce memory usage and network transfer times.

3. **Use prepared statements** for frequently executed queries.

### Migration Best Practices

1. **Keep migrations small and focused** - Large migrations can block database operations.

2. **Use indexes effectively** - Add indexes in separate migrations after data is loaded.

3. **Consider transaction boundaries** - Ensure transactional integrity within migrations.

4. **Run migrations during low-traffic periods** - While efficient, migrations do create additional database load.

## Benchmarking

The provider includes comprehensive benchmarks for measuring performance:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- "Transaction"  # Runs only transaction benchmarks
```

Benchmark categories include:
- Simple Queries (primary key, indexed, non-indexed)
- Transactions (simple, multi-operation, nested, callback API)
- Connection Pool (various pool sizes)
- Migration Execution (running and validating migrations)

## License

Apache 2.0, at your option.

## Examples

### Comprehensive Provider Example

The crate includes a comprehensive example that demonstrates all the key features of the PostgreSQL provider, including:

- Creating a provider with custom options
- Running and validating migrations
- Executing basic database operations
- Working with transactions (including nested transactions)
- Using the transaction callback API
- Getting migration information

To run the example:

```bash
cargo run --example provider_example
```

Note: This example requires a PostgreSQL server running on localhost with the default credentials (postgres/postgres) and a database named `navius_test`. You can modify the connection parameters in the example code if needed.

The example demonstrates:

1. **Provider Creation**: Two methods of creating a provider (direct and from options)
2. **Migration Management**: Creating and applying migration files
3. **Transaction Operations**: Using transactions, nested transactions, and the callback API
4. **Migration Information**: Getting detailed information about migrations

See the source code in `examples/provider_example.rs` for full details.

### Migration Example

// ... existing code ...

// ... existing code ... 