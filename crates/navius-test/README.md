# Navius Testing Framework

A comprehensive testing framework for Navius applications, providing tools for unit testing, integration testing, and cross-crate testing.

## Features

### Mock Interfaces and Registry

- **Mock Registry**: Central registry for managing mocks, expectations, and verifications
- **Method Expectations**: Specify expected method calls with argument matching
- **Return Values**: Configure return values or errors for mock methods
- **Call Verification**: Verify calls were made with expected arguments and frequency
- **Pre-built Mocks**: Ready-to-use mocks for databases, HTTP clients, caches, messaging, storage, etc.

### Integration Test Utilities

- **Test Configuration**: Comprehensive configuration options for test environment
- **Service Discovery**: Automatic service discovery and dependency resolution
- **Service Configuration**: Manage service configurations and properties
- **Test Data Management**: Generate and manage test data for integration tests
- **Database Setup**: Helpers for setting up and tearing down test databases
- **Test Lifecycle Hooks**: Customize test lifecycle with hooks at various stages
- **CI/CD Integration**: Special support for tests running in CI/CD environments
- **Test Reports**: Generate test reports in various formats (JUnit XML, JSON, Text)

### Cross-Crate Testing

- **Cross-Crate Test Builder**: Fluent API for building tests that span multiple crates
- **Dependency Management**: Advanced service dependency management across crates
- **Resource Sharing**: Share resources across test boundaries

## Getting Started

### Basic Usage

```rust
use navius_test::{
    integration::{IntegrationTestConfig, IntegrationContext},
    error::TestResult,
};

#[tokio::main]
async fn main() -> TestResult<()> {
    // Create a test configuration
    let config = IntegrationTestConfig::default()
        .with_name("my-test")
        .with_timeout(Duration::from_secs(30));
    
    // Create a test context
    let context = IntegrationContext::new(config)?;
    
    // Run your test
    let result = context.run(|ctx| {
        // Test code here
        Ok(true)
    }).await?;
    
    assert!(result, "Test should succeed");
    Ok(())
}
```

### Using Mocks

```rust
use navius_test::{
    mock::MockRegistry,
    mocks::database::{MockDatabaseClient, DatabaseClient},
};

fn test_with_mocks() {
    // Create a mock registry
    let registry = MockRegistry::new();
    
    // Create a mock database client
    let db_mock = MockDatabaseClient::new(&registry);
    
    // Set expectations
    db_mock.expect_query()
        .with(|query| query.contains("SELECT"))
        .times(1)
        .returns(Ok(vec![/* mock data */]));
    
    // Use the mock in your code
    let client: Box<dyn DatabaseClient> = Box::new(db_mock);
    
    // Your code that uses the client...
    
    // Verify all expectations were met
    registry.verify();
}
```

### Advanced Service Dependencies

```rust
use navius_test::{
    integration::{IntegrationTestConfig, IntegrationContext, ServiceConfig},
    error::TestResult,
};
use std::collections::HashMap;

async fn test_with_dependencies() -> TestResult<()> {
    // Define service configurations with dependencies
    let mut service_configs = HashMap::new();
    
    // Add database service (no dependencies)
    service_configs.insert("database".to_string(), ServiceConfig {
        name: "database".to_string(),
        service_type: "PostgresDatabase".to_string(),
        properties: HashMap::new(),
        dependencies: Vec::new(),
    });
    
    // Add cache service (depends on database)
    service_configs.insert("cache".to_string(), ServiceConfig {
        name: "cache".to_string(),
        service_type: "RedisCache".to_string(),
        properties: HashMap::new(),
        dependencies: vec!["database".to_string()],
    });
    
    // Create config with service configurations
    let mut config = IntegrationTestConfig::default();
    config.name = "dependency-test".to_string();
    config.service_configs = service_configs;
    
    // Create context and discover services
    let context = IntegrationContext::new(config)?;
    let discovery_result = context.discover_services()?;
    
    // Services are now available in the context
    // and were initialized in the correct dependency order
    
    Ok(())
}
```

### Test Data Generation

```rust
use navius_test::{
    integration::{IntegrationTestConfig, IntegrationContext},
    error::TestResult,
};

async fn test_with_data_generation() -> TestResult<()> {
    // Create test configuration
    let mut config = IntegrationTestConfig::default();
    config.name = "data-generation-test".to_string();
    
    // Create context
    let context = IntegrationContext::new(config)?;
    
    // Generate test data
    context.generate_test_data(|builder| {
        builder
            .with_user("user-1", "Test User", Some("test@example.com"), Some(vec!["admin"]))
            .with_entity("entity-1", Some("Test Entity"), Some(true))
    })?;
    
    // Retrieve generated test data
    let user = context.get_test_data("user-1")?;
    let entity = context.get_test_data("entity-entity-1")?;
    
    // Use the test data in your tests
    
    Ok(())
}
```

### CI/CD Integration

```rust
use navius_test::{
    integration::{IntegrationTestConfig, CIReportConfig, ReportFormat},
    error::TestResult,
};

async fn test_in_ci() -> TestResult<()> {
    // Create test configuration
    let mut config = IntegrationTestConfig::default();
    config.name = "ci-test".to_string();
    
    // Configure for CI environment
    config.configure_for_ci()?;
    
    // Create runner and run test
    let runner = navius_test::integration::IntegrationRunner::new(config)?;
    runner.run(|_| Ok(true)).await?;
    
    // Generate test report
    let report_config = CIReportConfig::new()
        .with_format(ReportFormat::JUnit)
        .with_include_environment(true);
    
    runner.generate_report(report_config)?;
    
    Ok(())
}
```

## Examples

Check out the examples directory for complete working examples:

- `basic_test.rs` - Basic testing example
- `mock_registry_example.rs` - Using the mock registry
- `integration_test_example.rs` - Basic integration test
- `cross_crate_test_example.rs` - Testing across crates
- `enhanced_integration_test_example.rs` - Advanced integration testing features
- `advanced_integration_test_example.rs` - Advanced service dependency management, test data generation, and CI/CD integration

## License

Copyright © Navius Technologies, Inc. All rights reserved. 