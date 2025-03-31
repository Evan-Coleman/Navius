// Re-export mock implementations
pub mod auth;
pub mod cache;
pub mod config;
pub mod database;
pub mod filesystem;
pub mod http;
pub mod logger;

// Import commonly used types
pub use database::{
    DatabaseClient, MockDatabaseClient, MockDatabaseError, MockQueryResult, MockValue, QueryResult,
    Row, Value,
};

pub use filesystem::{
    FileMetadata, FileSystem, MockFileHandle, MockFileMetadata, MockFileSystem, MockFileSystemError,
};

pub use http::{
    AuthToken, HttpClient, HttpMethod, HttpResponse, MockHttpClient, MockHttpError, RequestBody,
    ResponseBody,
};

pub use config::{ConfigValue, ConfigurationProvider, MockConfigError, MockConfigurationProvider};

pub use auth::{
    AuthProvider, LogEntry, LogLevel, Logger, MockAuthError, MockAuthProvider, MockLogger,
    MockRbacProvider, Permission, RbacProvider, UserIdentity,
};

pub use logger::{LogEntry, LogLevel, Logger, MockLogger};

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

    // Register HTTP client mock
    let http = http::MockHttpClient::new();
    let _ = http.register(&registry);

    // Register configuration mock
    let config = config::MockConfigurationProvider::new();
    let _ = config.register(&registry);

    // Register authentication mock
    let auth = auth::MockAuthProvider::new();
    let _ = auth.register(&registry);

    // Register RBAC mock
    let rbac = auth::MockRbacProvider::new();
    let _ = rbac.register(&registry);

    // Register logger mock
    let logger = logger::MockLogger::new();
    let _ = logger.register(&registry);

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

/// Trait for accessing a mock HTTP client in tests
pub trait HasMockHttpClient {
    /// Get the mock HTTP client
    fn http_client(&self) -> Arc<http::MockHttpClient>;
}

/// Trait for accessing a mock configuration provider in tests
pub trait HasMockConfiguration {
    /// Get the mock configuration provider
    fn configuration(&self) -> Arc<config::MockConfigurationProvider>;
}

/// Trait for accessing a mock authentication provider in tests
pub trait HasMockAuth {
    /// Get the mock authentication provider
    fn auth_provider(&self) -> Arc<auth::MockAuthProvider>;
}

/// Trait for accessing a mock RBAC provider in tests
pub trait HasMockRbac {
    /// Get the mock RBAC provider
    fn rbac_provider(&self) -> Arc<auth::MockRbacProvider>;
}

/// Trait for accessing a mock logger in tests
pub trait HasMockLogger {
    /// Get the mock logger
    fn logger(&self) -> Arc<logger::MockLogger>;
}

/// Trait for mock configurations that provide common mocks
pub trait CommonMocks:
    HasMockDatabase
    + HasMockFileSystem
    + HasMockHttpClient
    + HasMockConfiguration
    + HasMockAuth
    + HasMockRbac
    + HasMockLogger
{
}

/// A test fixture that provides common mocks
#[derive(Debug)]
pub struct MockFixture {
    /// The mock registry
    registry: Arc<crate::mock::MockRegistry>,

    /// The mock database client
    database: Arc<database::MockDatabaseClient>,

    /// The mock filesystem
    filesystem: Arc<filesystem::MockFileSystem>,

    /// The mock HTTP client
    http_client: Arc<http::MockHttpClient>,

    /// The mock configuration provider
    configuration: Arc<config::MockConfigurationProvider>,

    /// The mock authentication provider
    auth_provider: Arc<auth::MockAuthProvider>,

    /// The mock RBAC provider
    rbac_provider: Arc<auth::MockRbacProvider>,

    /// The mock logger
    logger: Arc<logger::MockLogger>,
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

        let http_client = registry
            .get::<dyn http::HttpClient, http::MockHttpClient>()
            .expect("Failed to get mock HTTP client");

        let configuration = registry
            .get::<dyn config::ConfigurationProvider, config::MockConfigurationProvider>()
            .expect("Failed to get mock configuration provider");

        let auth_provider = registry
            .get::<dyn auth::AuthProvider, auth::MockAuthProvider>()
            .expect("Failed to get mock authentication provider");

        let rbac_provider = registry
            .get::<dyn auth::RbacProvider, auth::MockRbacProvider>()
            .expect("Failed to get mock RBAC provider");

        let logger = registry
            .get::<dyn logger::Logger, logger::LoggerImpl>()
            .map(|l| l.mock_logger.clone())
            .expect("Failed to get mock logger");

        Self {
            registry,
            database,
            filesystem,
            http_client,
            configuration,
            auth_provider,
            rbac_provider,
            logger,
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

impl HasMockHttpClient for MockFixture {
    fn http_client(&self) -> Arc<http::MockHttpClient> {
        Arc::clone(&self.http_client)
    }
}

impl HasMockConfiguration for MockFixture {
    fn configuration(&self) -> Arc<config::MockConfigurationProvider> {
        Arc::clone(&self.configuration)
    }
}

impl HasMockAuth for MockFixture {
    fn auth_provider(&self) -> Arc<auth::MockAuthProvider> {
        Arc::clone(&self.auth_provider)
    }
}

impl HasMockRbac for MockFixture {
    fn rbac_provider(&self) -> Arc<auth::MockRbacProvider> {
        Arc::clone(&self.rbac_provider)
    }
}

impl HasMockLogger for MockFixture {
    fn logger(&self) -> Arc<logger::MockLogger> {
        Arc::clone(&self.logger)
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

        // Access the HTTP client mock
        let http = fixture.http_client();

        // Access the configuration mock
        let config = fixture.configuration();

        // Access the authentication mock
        let auth = fixture.auth_provider();

        // Access the RBAC mock
        let rbac = fixture.rbac_provider();

        // Access the logger mock
        let logger = fixture.logger();

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

        // Set up HTTP client expectations
        http.expect_get(
            "https://api.example.com/users/1",
            None,
            Ok(http::HttpResponse::ok().with_text("Success")),
        );

        // Set up configuration expectations
        config.expect_get_string("app.name", Ok("Test App".to_string()));

        // Set up authentication expectations
        let user = auth::UserIdentity::new("1", "testuser")
            .with_email("test@example.com")
            .with_role("user");
        let token = auth::AuthToken::new("test-token", "Bearer", 3600);
        auth.expect_authenticate("testuser", "password", Ok((user, token)));

        // Set up RBAC expectations
        rbac.expect_role_has_permission("user", "posts", "read", Ok(true));

        // Use the mocks
        let result = db.query("SELECT * FROM users WHERE id = ?").unwrap();
        assert_eq!(result.len(), 1);

        let file_content = fs.read_file("/test/data.txt").unwrap();
        assert_eq!(file_content, b"Test data");

        let http_response = http.get("https://api.example.com/users/1", None).unwrap();
        assert_eq!(http_response.as_text().unwrap(), "Success");

        let app_name = config.get_string("app.name").unwrap();
        assert_eq!(app_name, "Test App");

        let (auth_user, _) = auth.authenticate("testuser", "password").unwrap();
        assert_eq!(auth_user.username, "testuser");

        let has_permission = rbac.role_has_permission("user", "posts", "read").unwrap();
        assert!(has_permission);

        // Log some messages
        let logger_impl = fixture
            .registry
            .get::<dyn logger::Logger, logger::LoggerImpl>()
            .unwrap();
        logger_impl.info("Test info message", None);

        // Verify logging
        let logs = logger.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test info message");

        // Verify that all expectations have been met
        fixture.verify().unwrap();
    }
}
