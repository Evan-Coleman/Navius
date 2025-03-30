/*!
Navius DB - Database functionality for the Navius framework

This crate provides database connectivity, query execution, and ORM functionality
for the Navius framework with pluggable database backends.
*/

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
pub use pool::{
    DatabaseConnection, DatabasePool, DatabaseRowSet, DatabaseTransaction, PoolOptions,
};
pub use query::{Query, QueryBuilder, QueryExecutor, SortDirection};
pub use repository::{Entity, Repository};
pub use transaction::Transaction;

/// Database provider interface
pub trait DatabaseProvider: Send + Sync + 'static {
    /// Get the provider name
    fn name(&self) -> &'static str;

    /// Get the provider version
    fn version(&self) -> &'static str;
}

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
