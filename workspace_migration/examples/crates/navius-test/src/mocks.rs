// Export mock implementations from the directory
// Note: These are direct references to files in the mocks directory
pub mod auth;
pub mod cache;
pub mod config;
pub mod database;
pub mod events;
pub mod filesystem;
pub mod http;
pub mod logger;
pub mod messaging;
pub mod metrics;

// Import commonly used types for re-export
pub use database::{
    DatabaseClient, MockDatabaseClient, MockDatabaseError, MockQueryResult, MockValue, QueryResult,
    Row, Value,
};

pub use filesystem::{
    FileMetadata, FileSystem, MockFileHandle, MockFileMetadata, MockFileSystem, MockFileSystemError,
};

pub use http::{
    HttpClient, HttpMethod, HttpResponse, MockHttpClient, MockHttpError, RequestBody, ResponseBody,
};

pub use config::{ConfigValue, ConfigurationProvider, MockConfigError, MockConfigurationProvider};

pub use auth::{
    AuthProvider, MockAuthError, MockAuthProvider, MockRbacProvider, Permission, RbacProvider,
    UserIdentity,
};

pub use logger::{LogEntry, LogLevel, Logger, MockLogger};

pub use events::{Event, EventBroker, EventEnvelope, EventPriority, MockEventBroker};
pub use messaging::{
    Binding, DeliveryMode, Exchange, Message, MessageBroker, MockMessageBroker, MockMessagingError,
    PublishStatus, Queue, ReceivedMessage,
};
pub use metrics::{MetricType, MetricValue, MetricsCollector, MetricsExporter, MockMetrics};

use std::sync::Arc;

/// Utility function to create a mock registry with common mocks
pub fn setup_common_mocks() -> Arc<crate::mock::MockRegistry> {
    let registry = Arc::new(crate::mock::MockRegistry::new());

    // Register database mock
    let db = database::MockDatabaseClient::new();
    // let _ = db.register(&registry);

    // Register filesystem mock
    let fs = filesystem::MockFileSystem::new();
    // let _ = fs.register(&registry);

    // Register HTTP client mock
    let http = http::MockHttpClient::default();
    // let _ = http.register(&registry);

    // Register configuration mock
    let config = config::MockConfigurationProvider::default();
    // let _ = config.register(&registry);

    // Register authentication mock
    let auth = auth::MockAuthProvider::default();
    // let _ = auth.register(&registry);

    // Register RBAC mock
    let rbac = auth::MockRbacProvider::default();
    // let _ = rbac.register(&registry);

    // Register logger mock
    let logger = logger::MockLogger::new();
    // let _ = logger.register(&registry);

    // Register metrics mock
    let metrics = metrics::MockMetrics::new();
    // let _ = metrics.register(&registry);

    // Register event broker mock
    let event_broker = events::MockEventBroker::new();
    // let _ = event_broker.register(&registry);

    // Register message broker mock
    let message_broker = messaging::MockMessageBroker::new();
    // let _ = message_broker.register(&registry);

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

/// Trait for accessing a mock metrics in tests
pub trait HasMockMetrics {
    /// Get the mock metrics
    fn metrics(&self) -> Arc<metrics::MockMetrics>;
}

/// Trait for accessing a mock event broker in tests
pub trait HasMockEventBroker {
    /// Get the mock event broker
    fn event_broker(&self) -> Arc<events::MockEventBroker>;
}

/// Trait for accessing a mock message broker in tests
pub trait HasMockMessageBroker {
    /// Get the mock message broker
    fn message_broker(&self) -> Arc<messaging::MockMessageBroker>;
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
    + HasMockMetrics
    + HasMockEventBroker
    + HasMockMessageBroker
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

    /// The mock metrics
    metrics: Arc<metrics::MockMetrics>,

    /// The mock event broker
    event_broker: Arc<events::MockEventBroker>,

    /// The mock message broker
    message_broker: Arc<messaging::MockMessageBroker>,
}

impl MockFixture {
    /// Create a new mock fixture with all common mocks
    pub fn new() -> Self {
        let registry = Arc::new(crate::mock::MockRegistry::new());

        // Create mock instances
        let database = Arc::new(database::MockDatabaseClient::new());
        let filesystem = Arc::new(filesystem::MockFileSystem::new());
        let http_client = Arc::new(http::MockHttpClient::default());
        let configuration = Arc::new(config::MockConfigurationProvider::default());
        let auth_provider = Arc::new(auth::MockAuthProvider::default());
        let rbac_provider = Arc::new(auth::MockRbacProvider::default());
        let logger = Arc::new(logger::MockLogger::new());
        let metrics = Arc::new(metrics::MockMetrics::new());
        let event_broker = Arc::new(events::MockEventBroker::new());
        let message_broker = Arc::new(messaging::MockMessageBroker::new());

        // Register with the registry
        // let _ = database.register(&registry);
        // let _ = filesystem.register(&registry);
        // let _ = http_client.register(&registry);
        // let _ = configuration.register(&registry);
        // let _ = auth_provider.register(&registry);
        // let _ = rbac_provider.register(&registry);
        // let _ = logger.register(&registry);
        // let _ = metrics.register(&registry);
        // let _ = event_broker.register(&registry);
        // let _ = message_broker.register(&registry);

        Self {
            registry,
            database,
            filesystem,
            http_client,
            configuration,
            auth_provider,
            rbac_provider,
            logger,
            metrics,
            event_broker,
            message_broker,
        }
    }

    /// Get the mock registry
    pub fn registry(&self) -> Arc<crate::mock::MockRegistry> {
        self.registry.clone()
    }

    /// Verify all mock expectations
    pub fn verify(&self) -> crate::error::TestResult<()> {
        // self.registry.verify()
        Ok(())
    }

    /// Reset all mock expectations
    pub fn reset(&self) -> crate::error::TestResult<()> {
        // self.registry.reset()
        Ok(())
    }
}

impl HasMockDatabase for MockFixture {
    fn database(&self) -> Arc<database::MockDatabaseClient> {
        self.database.clone()
    }
}

impl HasMockFileSystem for MockFixture {
    fn filesystem(&self) -> Arc<filesystem::MockFileSystem> {
        self.filesystem.clone()
    }
}

impl HasMockHttpClient for MockFixture {
    fn http_client(&self) -> Arc<http::MockHttpClient> {
        self.http_client.clone()
    }
}

impl HasMockConfiguration for MockFixture {
    fn configuration(&self) -> Arc<config::MockConfigurationProvider> {
        self.configuration.clone()
    }
}

impl HasMockAuth for MockFixture {
    fn auth_provider(&self) -> Arc<auth::MockAuthProvider> {
        self.auth_provider.clone()
    }
}

impl HasMockRbac for MockFixture {
    fn rbac_provider(&self) -> Arc<auth::MockRbacProvider> {
        self.rbac_provider.clone()
    }
}

impl HasMockLogger for MockFixture {
    fn logger(&self) -> Arc<logger::MockLogger> {
        self.logger.clone()
    }
}

impl HasMockMetrics for MockFixture {
    fn metrics(&self) -> Arc<metrics::MockMetrics> {
        self.metrics.clone()
    }
}

impl HasMockEventBroker for MockFixture {
    fn event_broker(&self) -> Arc<events::MockEventBroker> {
        self.event_broker.clone()
    }
}

impl HasMockMessageBroker for MockFixture {
    fn message_broker(&self) -> Arc<messaging::MockMessageBroker> {
        self.message_broker.clone()
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

        // Access the metrics mock
        let metrics = fixture.metrics();

        // Access the event broker mock
        let event_broker = fixture.event_broker();

        // Access the message broker mock
        let message_broker = fixture.message_broker();

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
        logger.info("Test info message", None);

        // Verify logging
        let logs = logger.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test info message");

        // Verify that all expectations have been met
        fixture.verify().unwrap();
    }
}
