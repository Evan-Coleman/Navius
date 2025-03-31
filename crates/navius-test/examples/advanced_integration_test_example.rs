use navius_test::{
    error::TestResult,
    integration::{
        CIEnvironment, CIReportConfig, ConfigValue, CrossCrateTestBuilder, IntegrationContext,
        IntegrationTestConfig, LifecycleStage, ReportFormat, ServiceConfig, TestData,
        TestDataBuilder,
    },
    mock::MockRegistry,
    mocks::{
        cache::{CacheClient, MockCacheClient},
        database::{DatabaseClient, MockDatabaseClient, MockQueryResult, MockValue},
        http::{HttpClient, HttpResponse, MockHttpClient},
        messaging::{MessagingClient, MockMessagingClient},
        storage::{MockStorageClient, StorageClient},
    },
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Example demonstrating advanced integration testing features
#[tokio::main]
async fn main() -> TestResult<()> {
    println!("Advanced Integration Test Example");
    println!("================================\n");

    // Detect CI environment (if any)
    if let Some(ci) = CIEnvironment::detect() {
        println!(
            "Running in CI environment: {} ({})",
            ci.name,
            ci.build_id.unwrap_or_default()
        );
        println!("Branch: {}", ci.branch.unwrap_or_default());
        println!("Commit: {}", ci.commit.unwrap_or_default());
        println!("Is PR: {}\n", ci.is_pull_request);
    } else {
        println!("Running in local environment\n");
    }

    // Demonstration 1: Advanced Service Dependency Management
    println!("Demonstration 1: Advanced Service Dependency Management");
    println!("-----------------------------------------------------");
    run_dependency_management_example().await?;

    // Demonstration 2: Test Data Generation
    println!("\nDemonstration 2: Test Data Generation");
    println!("-------------------------------------");
    run_test_data_generation_example().await?;

    // Demonstration 3: CI/CD Integration
    println!("\nDemonstration 3: CI/CD Integration");
    println!("----------------------------------");
    run_cicd_integration_example().await?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// Example demonstrating service dependency management
async fn run_dependency_management_example() -> TestResult<()> {
    // Create service configurations with complex dependencies
    let mut service_configs = HashMap::new();

    // 1. Database service (no dependencies)
    let mut db_properties = HashMap::new();
    db_properties.insert(
        "url".to_string(),
        ConfigValue::String("postgres://localhost/test".to_string()),
    );
    let db_config = ServiceConfig {
        name: "database".to_string(),
        service_type: "PostgresDatabase".to_string(),
        properties: db_properties,
        dependencies: Vec::new(),
    };
    service_configs.insert("database".to_string(), db_config);

    // 2. Cache service (depends on database)
    let mut cache_properties = HashMap::new();
    cache_properties.insert(
        "url".to_string(),
        ConfigValue::String("redis://localhost:6379".to_string()),
    );
    let cache_config = ServiceConfig {
        name: "cache".to_string(),
        service_type: "RedisCache".to_string(),
        properties: cache_properties,
        dependencies: vec!["database".to_string()],
    };
    service_configs.insert("cache".to_string(), cache_config);

    // 3. Storage service (no dependencies)
    let mut storage_properties = HashMap::new();
    storage_properties.insert(
        "path".to_string(),
        ConfigValue::String("/var/data".to_string()),
    );
    let storage_config = ServiceConfig {
        name: "storage".to_string(),
        service_type: "FileStorage".to_string(),
        properties: storage_properties,
        dependencies: Vec::new(),
    };
    service_configs.insert("storage".to_string(), storage_config);

    // 4. Messaging service (depends on database)
    let mut messaging_properties = HashMap::new();
    messaging_properties.insert(
        "broker_url".to_string(),
        ConfigValue::String("amqp://localhost:5672".to_string()),
    );
    let messaging_config = ServiceConfig {
        name: "messaging".to_string(),
        service_type: "RabbitMQ".to_string(),
        properties: messaging_properties,
        dependencies: vec!["database".to_string()],
    };
    service_configs.insert("messaging".to_string(), messaging_config);

    // 5. User service (depends on database, cache, and messaging)
    let user_config = ServiceConfig {
        name: "user-service".to_string(),
        service_type: "ApplicationService".to_string(),
        properties: HashMap::new(),
        dependencies: vec![
            "database".to_string(),
            "cache".to_string(),
            "messaging".to_string(),
        ],
    };
    service_configs.insert("user-service".to_string(), user_config);

    // 6. Notification service (depends on messaging and storage)
    let notification_config = ServiceConfig {
        name: "notification-service".to_string(),
        service_type: "ApplicationService".to_string(),
        properties: HashMap::new(),
        dependencies: vec!["messaging".to_string(), "storage".to_string()],
    };
    service_configs.insert("notification-service".to_string(), notification_config);

    // 7. API service (depends on user-service and notification-service)
    let api_config = ServiceConfig {
        name: "api-service".to_string(),
        service_type: "HttpService".to_string(),
        properties: HashMap::new(),
        dependencies: vec![
            "user-service".to_string(),
            "notification-service".to_string(),
        ],
    };
    service_configs.insert("api-service".to_string(), api_config);

    // Create test config with service configurations
    let mut config = IntegrationTestConfig::default();
    config.name = "advanced-dependency-management".to_string();
    config.service_configs = service_configs;

    // Create context and discover services
    let context = IntegrationContext::new(config)?;
    let discovery_result = context.discover_services()?;

    // Display discovery results
    println!(
        "Discovered {} services with the following dependencies:",
        discovery_result.instances.len()
    );
    for (name, deps) in &discovery_result.dependencies {
        println!("- {}: depends on [{}]", name, deps.join(", "));
    }

    // Verify that services were discovered in the correct order
    // (i.e., dependencies were resolved before dependents)
    let service_names: Vec<&String> = discovery_result.instances.keys().collect();
    println!("Service initialization order:");
    for (i, name) in service_names.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    Ok(())
}

/// Example demonstrating test data generation
async fn run_test_data_generation_example() -> TestResult<()> {
    // Create a temporary directory for test data
    let temp_dir = tempfile::tempdir()?;
    let test_data_path = temp_dir.path().join("test_data.json");

    // Create test configuration
    let mut config = IntegrationTestConfig::default();
    config.name = "test-data-generation".to_string();
    config.test_data_path = Some(test_data_path.clone());

    // Create context
    let context = IntegrationContext::new(config)?;

    // Method 1: Generate test data using builder pattern
    let test_data_builder = context.create_test_data_builder();

    let builder = test_data_builder
        .with_user(
            "user-1",
            "Alice Smith",
            Some("alice@example.com"),
            Some(vec!["admin", "user"]),
        )
        .with_user(
            "user-2",
            "Bob Jones",
            Some("bob@example.com"),
            Some(vec!["user"]),
        )
        .with_entity("product-1", Some("Product One"), Some(true))
        .with_entity("product-2", Some("Product Two"), Some(false));

    // Export the data to a file
    builder.clone().export_to_file(&test_data_path)?;
    println!("Test data exported to {}", test_data_path.display());

    // Register the data with the context
    builder.register()?;

    // Method 2: Generate test data using the functional API
    context.generate_test_data(|builder| {
        builder
            .with_config("app", {
                let mut props = HashMap::new();
                props.insert(
                    "app_name".to_string(),
                    ConfigValue::String("Test App".to_string()),
                );
                props.insert(
                    "version".to_string(),
                    ConfigValue::String("1.0.0".to_string()),
                );
                props.insert("debug".to_string(), ConfigValue::Boolean(true));
                props
            })
            .with_data(
                TestData::new("order-1")
                    .with_string("id", "order-1")
                    .with_string("customer_id", "user-1")
                    .with_array(
                        "items",
                        vec![
                            ConfigValue::String("product-1".to_string()),
                            ConfigValue::String("product-2".to_string()),
                        ],
                    )
                    .with_float("total", 123.45),
            )
    })?;

    // Retrieve and display the generated test data
    println!("Generated test data:");

    let user1 = context.get_test_data("user-1")?;
    println!(
        "- User 1: {}",
        user1.content.get("name").unwrap().as_string().unwrap()
    );

    let product1 = context.get_test_data("entity-product-1")?;
    println!(
        "- Product 1: {}",
        product1.content.get("name").unwrap().as_string().unwrap()
    );

    let order1 = context.get_test_data("order-1")?;
    println!(
        "- Order 1: Customer ID: {}",
        order1
            .content
            .get("customer_id")
            .unwrap()
            .as_string()
            .unwrap()
    );

    Ok(())
}

/// Example demonstrating CI/CD integration
async fn run_cicd_integration_example() -> TestResult<()> {
    println!("Creating test configured for CI/CD integration");

    // Create a test config
    let mut config = IntegrationTestConfig::default();
    config.name = "cicd-integration-test".to_string();

    // Configure for CI environment
    config.configure_for_ci()?;

    // Create the runner with CI configuration
    let runner = navius_test::integration::IntegrationRunner::new(config)?;

    // Run a simple test
    let result = runner
        .run(|context| {
            println!("Running test within CI-configured context");

            // Access CI environment variables
            for (key, value) in &context.config.env_vars {
                if key.starts_with("CI_") {
                    println!("  CI ENV: {} = {}", key, value);
                }
            }

            // Return success
            Ok(true)
        })
        .await?;

    assert!(result, "Test should succeed");

    // Generate test reports in multiple formats
    let report_dir = tempfile::tempdir()?;

    // JUnit XML report
    let junit_config = CIReportConfig::new()
        .with_format(ReportFormat::JUnit)
        .with_output_dir(report_dir.path())
        .with_include_environment(true);

    let junit_path = runner.generate_report(junit_config)?;
    println!("Generated JUnit XML report at: {}", junit_path.display());

    // JSON report
    let json_config = CIReportConfig::new()
        .with_format(ReportFormat::JSON)
        .with_output_dir(report_dir.path())
        .with_include_test_data(true)
        .with_include_environment(true);

    let json_path = runner.generate_report(json_config)?;
    println!("Generated JSON report at: {}", json_path.display());

    // Example using builder with CI configuration
    println!("Using builder pattern with CI configuration");
    let ci_test = CrossCrateTestBuilder::new("ci-builder-test")
        .with_crate("navius-core")
        .with_crate("navius-db")
        .for_ci()?
        .build()?;

    ci_test.run(|_| Ok(true)).await?;

    Ok(())
}
