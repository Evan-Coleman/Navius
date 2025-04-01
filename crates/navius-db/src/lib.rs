/*!
Navius DB - Database functionality for the Navius framework

This crate provides database connectivity, query execution, and ORM functionality
for the Navius framework with pluggable database backends.
*/

// Re-exports
pub use connection::{DatabaseConnection, PgConnection};
pub use error::DatabaseError;
pub use transaction::{DatabaseTransaction, PgTransaction};

pub mod connection;
pub mod error;
pub mod pool;
pub mod repository;
pub mod transaction;

/// The context for database operations
/// Used to track operation context and logging
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// The name of the operation
    pub operation: String,
    /// The name of the table or collection
    pub table: Option<String>,
    /// Additional context for error reporting
    pub additional_context: Option<String>,
}

impl OperationContext {
    pub fn new<S: Into<String>>(operation: S) -> Self {
        Self {
            operation: operation.into(),
            table: None,
            additional_context: None,
        }
    }

    pub fn with_table<S: Into<String>>(mut self, table: S) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn with_additional_context<S: Into<String>>(mut self, context: S) -> Self {
        self.additional_context = Some(context.into());
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
