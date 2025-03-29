/*!
Navius DB - Database functionality for the Navius framework

This crate provides database connectivity, query execution, and ORM functionality
for the Navius framework. It supports PostgreSQL through SQLx.
*/

// Re-export dependencies for convenience
#[cfg(feature = "postgres")]
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
pub use config::DatabaseConfig;
pub use connection::DatabaseConnectionManager;
pub use error::{DatabaseError, DatabaseResult};
#[cfg(feature = "postgres")]
pub use pool::PgPool;
pub use pool::{DatabasePool, PoolOptions};
pub use query::{Query, QueryBuilder, QueryExecutor, SortDirection};
pub use repository::{Entity, Repository};
pub use transaction::Transaction;

/// Database module to be used in applications
pub mod database {
    pub use super::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
