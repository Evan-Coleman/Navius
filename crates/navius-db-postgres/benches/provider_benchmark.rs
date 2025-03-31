use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use navius_db::provider::DatabaseProvider;
use navius_db::transaction::TransactionOptions;
use navius_db_postgres::provider::{PostgresProvider, PostgresProviderOptions};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use sqlx::postgres::PgConnectOptions;

fn create_benchmark_provider() -> (PostgresProvider, TempDir, Runtime) {
    let rt = Runtime::new().unwrap();
    
    // Create temp directory for migrations
    let temp_dir = TempDir::new().unwrap();
    let migrations_dir = temp_dir.path();

    // Create test migrations in the runtime
    rt.block_on(async {
        // Create migration directory
        tokio::fs::create_dir_all(migrations_dir).await.unwrap();
        
        // Create initial schema migration
        let migration1 = format!(
            "{}/__V20250329000001__Create_benchmark_table.sql",
            migrations_dir.display()
        );
        let migration1_content = r#"
        CREATE TABLE benchmark_table (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            value INTEGER NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_benchmark_name ON benchmark_table(name);
        "#;
        tokio::fs::write(migration1, migration1_content).await.unwrap();
    });

    // Configure database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/navius_test".to_string());

    // Create connection options
    let connect_options = rt.block_on(async {
        database_url.parse::<PgConnectOptions>().unwrap()
    });

    // Create provider options
    let options = PostgresProviderOptions {
        pool_config: navius_db_postgres::pool::PgPoolConfig {
            url: database_url,
            max_connections: 20,
            min_connections: 5,
            max_lifetime: Some(Duration::from_secs(3600)),
            idle_timeout: Some(Duration::from_secs(600)),
            acquire_timeout: Duration::from_secs(30),
        },
        migration_options: Some(navius_db_postgres::migration::MigrationOptions {
            migrations_dir: migrations_dir.to_path_buf(),
            migrations_table: Some(format!("benchmark_migrations_{}", chrono::Utc::now().timestamp())),
            validate_checksums: true,
        }),
        run_migrations: true,
        validate_migrations: true,
    };

    // Create provider
    let provider = rt.block_on(async {
        let p = PostgresProvider::new(options).await.unwrap();
        
        // Insert some benchmark data
        let pool = p.pool();
        for i in 0..1000 {
            sqlx::query(
                "INSERT INTO benchmark_table (name, value) VALUES ($1, $2)"
            )
            .bind(format!("benchmark_name_{}", i % 100))
            .bind(i)
            .execute(&pool)
            .await
            .unwrap();
        }
        
        p
    });

    (provider, temp_dir, rt)
}

fn benchmark_simple_query(c: &mut Criterion) {
    let (provider, _temp_dir, rt) = create_benchmark_provider();
    let pool = provider.pool();

    let mut group = c.benchmark_group("Simple Queries");
    group.throughput(criterion::Throughput::Elements(1));
    group.bench_function("SELECT by ID", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: (i32, String, i32) = sqlx::query_as(
                    "SELECT id, name, value FROM benchmark_table WHERE id = $1"
                )
                .bind(1)
                .fetch_one(&pool)
                .await
                .unwrap();
            })
        })
    });

    group.bench_function("SELECT with index", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: Vec<(i32, String, i32)> = sqlx::query_as(
                    "SELECT id, name, value FROM benchmark_table WHERE name = $1"
                )
                .bind("benchmark_name_1")
                .fetch_all(&pool)
                .await
                .unwrap();
            })
        })
    });

    group.bench_function("SELECT without index", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: Vec<(i32, String, i32)> = sqlx::query_as(
                    "SELECT id, name, value FROM benchmark_table WHERE value = $1"
                )
                .bind(100)
                .fetch_all(&pool)
                .await
                .unwrap();
            })
        })
    });
    group.finish();
}

fn benchmark_transactions(c: &mut Criterion) {
    let (provider, _temp_dir, rt) = create_benchmark_provider();
    let tx_manager = provider.transaction_manager();

    let mut group = c.benchmark_group("Transactions");

    // Simple transaction with single operation
    group.bench_function("Single operation transaction", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut tx = tx_manager.begin_transaction(None).await.unwrap();
                
                tx.execute_query(
                    "INSERT INTO benchmark_table (name, value) VALUES ('transaction_benchmark', 999)"
                ).await.unwrap();
                
                tx.commit().await.unwrap();
            })
        })
    });

    // Multiple operations in a single transaction
    group.bench_function("Multiple operations transaction", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut tx = tx_manager.begin_transaction(None).await.unwrap();
                
                for i in 0..5 {
                    tx.execute_query(
                        &format!("INSERT INTO benchmark_table (name, value) VALUES ('transaction_multi_{}', {})", i, i)
                    ).await.unwrap();
                }
                
                tx.commit().await.unwrap();
            })
        })
    });

    // Nested transactions
    group.bench_function("Nested transactions", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut tx = tx_manager.begin_transaction(None).await.unwrap();
                
                tx.execute_query(
                    "INSERT INTO benchmark_table (name, value) VALUES ('nested_parent', 1000)"
                ).await.unwrap();
                
                let mut nested_tx = tx.create_nested_transaction().await.unwrap();
                
                nested_tx.execute_query(
                    "INSERT INTO benchmark_table (name, value) VALUES ('nested_child', 1001)"
                ).await.unwrap();
                
                nested_tx.commit().await.unwrap();
                tx.commit().await.unwrap();
            })
        })
    });

    // Transaction callback API
    group.bench_function("Transaction callback API", |b| {
        b.iter(|| {
            rt.block_on(async {
                tx_manager.with_transaction::<_, (), Box<dyn std::error::Error>>(|mut tx| {
                    Box::pin(async move {
                        tx.execute_query(
                            "INSERT INTO benchmark_table (name, value) VALUES ('callback_api', 2000)"
                        ).await?;
                        
                        Ok(())
                    })
                }).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_connection_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("Connection Pool");
    
    // Benchmark different pool sizes
    for pool_size in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("Pool size impact", pool_size), 
            pool_size, 
            |b, &size| {
                let rt = Runtime::new().unwrap();
                
                // Create provider with specific pool size
                let database_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/navius_test".to_string());
                
                let options = rt.block_on(async {
                    PostgresProviderOptions {
                        pool_config: navius_db_postgres::pool::PgPoolConfig {
                            url: database_url,
                            max_connections: size,
                            min_connections: size / 5,
                            max_lifetime: Some(Duration::from_secs(3600)),
                            idle_timeout: Some(Duration::from_secs(600)),
                            acquire_timeout: Duration::from_secs(30),
                        },
                        migration_options: None,
                        run_migrations: false,
                        validate_migrations: false,
                    }
                });
                
                let provider = rt.block_on(async {
                    PostgresProvider::new(options).await.unwrap()
                });
                
                // Test concurrent connections
                b.iter(|| {
                    rt.block_on(async {
                        let pool = provider.pool();
                        
                        // Create multiple concurrent queries
                        let mut handles = vec![];
                        
                        for i in 0..size {
                            let pool_clone = pool.clone();
                            handles.push(tokio::spawn(async move {
                                let _: (i32,) = sqlx::query_as("SELECT $1::int")
                                    .bind(i)
                                    .fetch_one(&pool_clone)
                                    .await
                                    .unwrap();
                            }));
                        }
                        
                        // Wait for all queries to complete
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    })
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_migration_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("Migration Execution");
    
    group.bench_function("Run migrations", |b| {
        b.iter_with_setup(
            || {
                // Setup: Create a fresh provider with migrations
                let rt = Runtime::new().unwrap();
                let temp_dir = TempDir::new().unwrap();
                let migrations_dir = temp_dir.path();
                
                // Create migrations
                rt.block_on(async {
                    tokio::fs::create_dir_all(migrations_dir).await.unwrap();
                    
                    for i in 1..=5 {
                        let migration_name = format!(
                            "{}/__V2025032900000{}__{}.sql",
                            migrations_dir.display(),
                            i,
                            format!("Benchmark_migration_{}", i)
                        );
                        
                        let content = format!(r#"
                        CREATE TABLE benchmark_migration_table_{} (
                            id SERIAL PRIMARY KEY,
                            name TEXT NOT NULL,
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                        );
                        "#, i);
                        
                        tokio::fs::write(migration_name, content).await.unwrap();
                    }
                });
                
                // Configure database connection
                let database_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/navius_test".to_string());
                
                let options = PostgresProviderOptions {
                    pool_config: navius_db_postgres::pool::PgPoolConfig {
                        url: database_url,
                        max_connections: 10,
                        min_connections: 2,
                        max_lifetime: Some(Duration::from_secs(3600)),
                        idle_timeout: Some(Duration::from_secs(600)),
                        acquire_timeout: Duration::from_secs(30),
                    },
                    migration_options: Some(navius_db_postgres::migration::MigrationOptions {
                        migrations_dir: migrations_dir.to_path_buf(),
                        migrations_table: Some(format!("benchmark_migrations_{}", chrono::Utc::now().timestamp())),
                        validate_checksums: true,
                    }),
                    run_migrations: false, // Important: don't run migrations during setup
                    validate_migrations: false,
                };
                
                let provider = rt.block_on(async {
                    PostgresProvider::new(options).await.unwrap()
                });
                
                (provider, rt, temp_dir)
            },
            |(provider, rt, _temp_dir)| {
                rt.block_on(async {
                    // Benchmark the migration execution
                    provider.run_migrations().await.unwrap();
                })
            }
        );
    });
    
    group.bench_function("Validate migrations", |b| {
        b.iter_with_setup(
            || {
                // Setup: Create a provider with already executed migrations
                let rt = Runtime::new().unwrap();
                let temp_dir = TempDir::new().unwrap();
                let migrations_dir = temp_dir.path();
                
                // Create migrations
                rt.block_on(async {
                    tokio::fs::create_dir_all(migrations_dir).await.unwrap();
                    
                    for i in 1..=5 {
                        let migration_name = format!(
                            "{}/__V2025032900000{}__{}.sql",
                            migrations_dir.display(),
                            i,
                            format!("Benchmark_validation_{}", i)
                        );
                        
                        let content = format!(r#"
                        CREATE TABLE benchmark_validation_table_{} (
                            id SERIAL PRIMARY KEY,
                            name TEXT NOT NULL,
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                        );
                        "#, i);
                        
                        tokio::fs::write(migration_name, content).await.unwrap();
                    }
                });
                
                // Configure database connection
                let database_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/navius_test".to_string());
                
                let options = PostgresProviderOptions {
                    pool_config: navius_db_postgres::pool::PgPoolConfig {
                        url: database_url,
                        max_connections: 10,
                        min_connections: 2,
                        max_lifetime: Some(Duration::from_secs(3600)),
                        idle_timeout: Some(Duration::from_secs(600)),
                        acquire_timeout: Duration::from_secs(30),
                    },
                    migration_options: Some(navius_db_postgres::migration::MigrationOptions {
                        migrations_dir: migrations_dir.to_path_buf(),
                        migrations_table: Some(format!("benchmark_migrations_{}", chrono::Utc::now().timestamp())),
                        validate_checksums: true,
                    }),
                    run_migrations: true, // Run migrations during setup
                    validate_migrations: false,
                };
                
                let provider = rt.block_on(async {
                    PostgresProvider::new(options).await.unwrap()
                });
                
                (provider, rt, temp_dir)
            },
            |(provider, rt, _temp_dir)| {
                rt.block_on(async {
                    // Benchmark the migration validation
                    provider.validate_migrations().await.unwrap();
                })
            }
        );
    });
    
    group.finish();
}

criterion_group!(
    name = benches; 
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets = benchmark_simple_query, benchmark_transactions, benchmark_connection_pool, benchmark_migration_execution
);
criterion_main!(benches); 