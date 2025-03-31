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

    // Create and register database mock
    let db = Arc::new(database::MockDatabaseClient::new());
    let _ = registry.register::<dyn database::DatabaseClient, _>(db);

    // Create and register filesystem mock
    let fs = Arc::new(filesystem::MockFileSystem::new());
    let _ = registry.register::<dyn filesystem::FileSystem, _>(fs);

    // Create and register HTTP client mock
    let http = Arc::new(http::MockHttpClient::default());
    let _ = registry.register::<dyn http::HttpClient, _>(http);

    // Create and register configuration mock
    let config = Arc::new(config::MockConfigurationProvider::default());
    let _ = registry.register::<dyn config::ConfigurationProvider, _>(config);

    // Create and register authentication mock
    let auth = Arc::new(auth::MockAuthProvider::default());
    let _ = registry.register::<dyn auth::AuthProvider, _>(auth);

    // Create and register RBAC mock
    let rbac = Arc::new(auth::MockRbacProvider::default());
    let _ = registry.register::<dyn auth::RbacProvider, _>(rbac);

    // Create and register logger mock
    let logger = Arc::new(logger::MockLogger::new());
    let _ = registry.register::<dyn logger::Logger, _>(logger);

    // Create and register metrics mock
    let metrics = Arc::new(metrics::MockMetrics::new());
    let _ = registry.register::<dyn metrics::MetricsCollector, _>(metrics);

    // Create and register event broker mock
    let event_broker = Arc::new(events::MockEventBroker::new());
    let _ = registry.register::<dyn events::EventBroker, _>(event_broker);

    // Create and register message broker mock
    let message_broker = Arc::new(messaging::MockMessageBroker::new());
    let _ = registry.register::<dyn messaging::MessageBroker, _>(message_broker);

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
    /// Create a new mock fixture
    pub fn new() -> Self {
        let registry = Arc::new(crate::mock::MockRegistry::new());

        // Create all mock components
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

        // Register components with the registry
        registry
            .register::<dyn database::DatabaseClient, _>(database.clone())
            .expect("Failed to register database client");

        registry
            .register::<dyn filesystem::FileSystem, _>(filesystem.clone())
            .expect("Failed to register filesystem");

        registry
            .register::<dyn http::HttpClient, _>(http_client.clone())
            .expect("Failed to register HTTP client");

        registry
            .register::<dyn config::ConfigurationProvider, _>(configuration.clone())
            .expect("Failed to register configuration provider");

        registry
            .register::<dyn auth::AuthProvider, _>(auth_provider.clone())
            .expect("Failed to register auth provider");

        registry
            .register::<dyn auth::RbacProvider, _>(rbac_provider.clone())
            .expect("Failed to register RBAC provider");

        registry
            .register::<dyn logger::Logger, _>(logger.clone())
            .expect("Failed to register logger");

        registry
            .register::<dyn metrics::MetricsCollector, _>(metrics.clone())
            .expect("Failed to register metrics");

        registry
            .register::<dyn events::EventBroker, _>(event_broker.clone())
            .expect("Failed to register event broker");

        registry
            .register::<dyn messaging::MessageBroker, _>(message_broker.clone())
            .expect("Failed to register message broker");

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
        self.registry.verify()
    }

    /// Reset all mock expectations
    pub fn reset(&self) -> crate::error::TestResult<()> {
        self.registry.reset()
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

pub fn register_default_mocks(
    registry: &mut crate::mock::MockRegistry,
) -> crate::error::TestResult<()> {
    let event_broker = Arc::new(MockEventBroker::new());
    let message_broker = Arc::new(MockMessageBroker::new());
    let cache = Arc::new(MockCacheConnection::new());
    let db = Arc::new(MockDatabaseClient::new());
    let fs = Arc::new(MockFileSystem::new());
    let logger = Arc::new(MockLogger::new());
    let metrics = Arc::new(MockMetrics::new());

    let _ = registry.register::<dyn events::EventPublisher, _>(event_broker.clone());
    let _ = registry.register::<dyn events::EventSubscriber, _>(event_broker.clone());
    let _ = registry.register::<dyn events::EventBroker, _>(event_broker);

    let _ = registry.register::<dyn messaging::MessagePublisher, _>(message_broker.clone());
    let _ = registry.register::<dyn messaging::MessageConsumer, _>(message_broker.clone());
    let _ = registry.register::<dyn messaging::MessageBroker, _>(message_broker);

    let _ = registry.register::<dyn cache::Cache, _>(cache);
    let _ = registry.register::<dyn database::DatabaseClient, _>(db);
    let _ = registry.register::<dyn filesystem::FileSystem, _>(fs);
    let _ = registry.register::<dyn logger::Logger, _>(logger);
    let _ = registry.register::<dyn metrics::Metrics, _>(metrics);

    Ok(())
}

pub fn register_default_mocks_with_config(
    registry: &mut crate::mock::MockRegistry,
    config: &TestConfig,
) -> crate::error::TestResult<()> {
    let event_broker = Arc::new(MockEventBroker::new());
    let message_broker = Arc::new(MockMessageBroker::new());
    let cache = Arc::new(MockCacheConnection::new());
    let db = Arc::new(MockDatabaseClient::new());
    let fs = Arc::new(MockFileSystem::new());
    let logger = Arc::new(MockLogger::new());
    let metrics = Arc::new(MockMetrics::new());

    registry
        .register::<dyn events::EventPublisher, _>(event_broker.clone())
        .register::<dyn events::EventSubscriber, _>(event_broker.clone())
        .register::<dyn events::EventBroker, _>(event_broker.clone())
        .register::<dyn messaging::MessagePublisher, _>(message_broker.clone())
        .register::<dyn messaging::MessageConsumer, _>(message_broker.clone())
        .register::<dyn messaging::MessageBroker, _>(message_broker.clone())
        .register::<dyn cache::Cache, _>(cache)
        .register::<dyn database::DatabaseClient, _>(db)
        .register::<dyn filesystem::FileSystem, _>(fs)
        .register::<dyn logger::Logger, _>(logger)
        .register::<dyn metrics::Metrics, _>(metrics);

    Ok(())
}
