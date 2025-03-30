use navius_db_postgres::{MigrationOptions, MigrationRunner};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Get migrations directory from command line or use default
    let migrations_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("migrations"));

    println!("Using migrations directory: {}", migrations_dir.display());

    // Configure migration options
    let options = MigrationOptions {
        migrations_dir,
        migrations_table: Some("schema_migrations".to_string()),
        validate_checksums: true,
    };

    // Create migration runner
    let runner = MigrationRunner::new(pool, options);

    // Initialize
    runner.initialize().await?;

    // Find all migrations
    let migrations = runner.find_migrations().await?;
    println!("Found {} migration(s):", migrations.len());
    for migration in &migrations {
        println!("  - {} ({})", migration.version, migration.description);
    }

    // Check for pending migrations
    let pending = runner.pending_migrations().await?;
    println!("Found {} pending migration(s):", pending.len());
    for migration in &pending {
        println!("  - {} ({})", migration.version, migration.description);
    }

    // Run migrations
    if !pending.is_empty() {
        println!("Running migrations...");
        let count = runner.run_migrations().await?;
        println!("Applied {} migration(s)", count);
    }

    // Validate migrations
    println!("Validating migrations...");
    let valid = runner.validate().await?;
    println!("Validation result: {}", if valid { "OK" } else { "FAILED" });

    // Show status
    println!("Migration status:");
    let status = runner.info().await?;
    for s in status {
        match s.state {
            navius_db_postgres::migration::MigrationState::Applied {
                applied_at,
                success,
            } => {
                println!(
                    "  - {} ({}) - Applied at {} ({})",
                    s.migration.version,
                    s.migration.description,
                    applied_at,
                    if success { "SUCCESS" } else { "FAILED" }
                );
            }
            navius_db_postgres::migration::MigrationState::Pending => {
                println!(
                    "  - {} ({}) - Pending",
                    s.migration.version, s.migration.description
                );
            }
            navius_db_postgres::migration::MigrationState::Missing {
                applied_at,
                success,
            } => {
                println!(
                    "  - {} ({}) - Missing from filesystem, applied at {} ({})",
                    s.migration.version,
                    s.migration.description,
                    applied_at,
                    if success { "SUCCESS" } else { "FAILED" }
                );
            }
        }
    }

    Ok(())
}
