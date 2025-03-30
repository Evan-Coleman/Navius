/*!
Navius DB PostgreSQL - PostgreSQL implementation for the Navius database framework

This crate provides PostgreSQL connectivity for the Navius database framework using SQLx.
It implements the core database interfaces defined in navius-db.
*/

// Re-export sqlx for convenience
pub use sqlx;

// Internal modules
mod config;
mod connection;
mod error;
mod pool;
mod query;
mod repository;
mod transaction;

// Public exports
pub use config::PgDatabaseConfig;
pub use connection::PgConnectionManager;
pub use error::PgDatabaseError;
pub use pool::{PgConnection, PgPool, PgPoolOptions, PgRow, PgTransaction};
pub use query::{PgQuery, PgQueryExecutor};
pub use repository::PgRepository;
pub use transaction::PgTransaction as Transaction;

/// PostgreSQL provider for navius-db
pub struct PostgresProvider;

/// Implementation of the DatabaseProvider trait for PostgreSQL
impl navius_db::DatabaseProvider for PostgresProvider {
    fn name(&self) -> &'static str {
        "postgresql"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
