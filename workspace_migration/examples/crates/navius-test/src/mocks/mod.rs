// Re-export mock implementations
pub mod cache;
pub mod database;
pub mod filesystem;

// Import commonly used types
pub use database::{
    DatabaseClient, MockDatabaseClient, MockDatabaseError, MockQueryResult, MockValue, QueryResult,
    Row, Value,
};

pub use filesystem::{
    FileMetadata, FileSystem, MockFileHandle, MockFileMetadata, MockFileSystem, MockFileSystemError,
};

// Add new mocks below as they are implemented

// Module for HTTP client mocks
pub mod http;

// Module for configuration mocks
pub mod config;

// Module for authentication mocks
pub mod auth;

// Module for logger mocks
pub mod logger;

/// Utility function to create a mock registry with common mocks
pub fn setup_common_mocks() -> crate::mock::MockRegistry {
    use crate::mock::MockRegistry;
    use std::sync::Arc;

    let registry = MockRegistry::new();

    // Register database mock
    let db = database::MockDatabaseClient::new();
    let _ = db.register(&registry);

    // Register filesystem mock
    let fs = filesystem::MockFileSystem::new();
    let _ = fs.register(&registry);

    // Add more common mocks here as they are implemented

    registry
}

/// Trait for accessing a mock database in tests
pub trait HasMockDatabase {
    /// Get the mock database client
    fn database(&self) -> Arc<database::MockDatabaseClient>;
}

/// Trait for accessing a mock filesystem in tests
pub trait HasMockFileSystem {
    /// Get the mock filesystem
    fn filesystem(&self) -> Arc<filesystem::MockFileSystem>;
}

/// Trait for mock configurations that provide common mocks
pub trait CommonMocks: HasMockDatabase + HasMockFileSystem {}

/// A test fixture that provides common mocks
#[derive(Debug)]
pub struct MockFixture {
    /// The mock registry
    registry: Arc<crate::mock::MockRegistry>,

    /// The mock database client
    database: Arc<database::MockDatabaseClient>,

    /// The mock filesystem
    filesystem: Arc<filesystem::MockFileSystem>,
}

impl MockFixture {
    /// Create a new mock fixture
    pub fn new() -> Self {
        let registry = Arc::new(setup_common_mocks());

        let database = registry
            .get::<dyn database::DatabaseClient, database::MockDatabaseClient>()
            .expect("Failed to get mock database client");

        let filesystem = registry
            .get::<dyn filesystem::FileSystem, filesystem::MockFileSystem>()
            .expect("Failed to get mock filesystem");

        Self {
            registry,
            database,
            filesystem,
        }
    }

    /// Get the mock registry
    pub fn registry(&self) -> Arc<crate::mock::MockRegistry> {
        Arc::clone(&self.registry)
    }

    /// Verify that all expectations have been met
    pub fn verify(&self) -> crate::error::TestResult<()> {
        self.registry.verify()
    }
}

impl HasMockDatabase for MockFixture {
    fn database(&self) -> Arc<database::MockDatabaseClient> {
        Arc::clone(&self.database)
    }
}

impl HasMockFileSystem for MockFixture {
    fn filesystem(&self) -> Arc<filesystem::MockFileSystem> {
        Arc::clone(&self.filesystem)
    }
}

impl CommonMocks for MockFixture {}

// Default implementation
impl Default for MockFixture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_fixture() {
        let fixture = MockFixture::new();

        // Access the database mock
        let db = fixture.database();

        // Access the filesystem mock
        let fs = fixture.filesystem();

        // Set up database expectations
        db.expect_query(
            "SELECT * FROM users WHERE id = ?",
            Ok(database::MockQueryResult::new().add_row({
                let mut row = std::collections::HashMap::new();
                row.insert("id".to_string(), database::MockValue::Integer(1));
                row.insert(
                    "name".to_string(),
                    database::MockValue::String("Test User".to_string()),
                );
                row
            })),
        );

        // Set up filesystem expectations
        fs.create_dir("/test").unwrap();
        fs.write_file("/test/data.txt", b"Test data").unwrap();

        // Use the mocks
        let result = db.query("SELECT * FROM users WHERE id = ?").unwrap();
        assert_eq!(result.len(), 1);

        let file_content = fs.read_file("/test/data.txt").unwrap();
        assert_eq!(file_content, b"Test data");

        // Verify that all expectations have been met
        fixture.verify().unwrap();
    }
}
