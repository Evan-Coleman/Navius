use navius_test::{
    error::TestResult,
    integration::{
        ConfigValue, CrossCrateTestBuilder, IntegrationContext, IntegrationTestConfig,
        LifecycleStage, ServiceConfig, TestData, create_cross_crate_test,
    },
    mock::MockRegistry,
    mocks::{
        cache::{CacheClient, MockCacheClient},
        database::{DatabaseClient, MockDatabaseClient, MockQueryResult, MockValue, Row},
        http::{HttpClient, HttpMethod, HttpResponse, MockHttpClient},
    },
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// Example entity for testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    active: bool,
}

// Example service that uses database, cache, and HTTP clients
struct UserService {
    db_client: Arc<dyn DatabaseClient>,
    cache_client: Arc<dyn CacheClient>,
    http_client: Arc<dyn HttpClient>,
}

impl UserService {
    fn new(
        db_client: Arc<dyn DatabaseClient>,
        cache_client: Arc<dyn CacheClient>,
        http_client: Arc<dyn HttpClient>,
    ) -> Self {
        Self {
            db_client,
            cache_client,
            http_client,
        }
    }

    fn get_user(&self, id: &str) -> TestResult<Option<User>> {
        // Check cache first
        if let Some(cached_user) = self.cache_client.get(&format!("user:{}", id))? {
            let user: User = serde_json::from_str(&cached_user)?;
            return Ok(Some(user));
        }

        // Query database
        let query = format!("SELECT * FROM users WHERE id = '{}'", id);
        let result = self.db_client.query(&query)?;

        if result.is_empty() {
            return Ok(None);
        }

        let row = result.row(0).unwrap();
        let user = User {
            id: row.get("id").unwrap().as_string().unwrap(),
            name: row.get("name").unwrap().as_string().unwrap(),
            email: row.get("email").unwrap().as_string().unwrap(),
            active: row.get("active").unwrap().as_boolean().unwrap(),
        };

        // Cache the user
        let user_json = serde_json::to_string(&user)?;
        self.cache_client
            .set(&format!("user:{}", id), &user_json, Some(300))?;

        Ok(Some(user))
    }

    fn activate_user(&self, id: &str) -> TestResult<User> {
        // Get current user
        let user = self.get_user(id)?.ok_or_else(|| {
            navius_test::error::TestError::missing_component(format!("User not found: {}", id))
        })?;

        if user.active {
            return Ok(user);
        }

        // Update user
        let update_query = format!("UPDATE users SET active = true WHERE id = '{}'", id);
        self.db_client.execute(&update_query)?;

        // Call external API to notify about activation
        let api_url = format!("https://notification-service/api/users/{}/activate", id);
        self.http_client
            .post(&api_url, None, Some(serde_json::to_string(&user)?))?;

        // Invalidate cache
        self.cache_client.delete(&format!("user:{}", id))?;

        // Return updated user
        let mut updated_user = user;
        updated_user.active = true;
        Ok(updated_user)
    }
}

// Example demonstrating enhanced integration testing features
#[tokio::main]
async fn main() -> TestResult<()> {
    println!("Enhanced Integration Test Example");
    println!("================================\n");

    // Create a temporary directory for test data
    let temp_dir = tempfile::tempdir()?;
    let test_data_path = temp_dir.path().join("test_data.json");

    // Create test data JSON
    let mut content = HashMap::new();
    content.insert(
        "name".to_string(),
        ConfigValue::String("Jane Doe".to_string()),
    );
    content.insert(
        "email".to_string(),
        ConfigValue::String("jane@example.com".to_string()),
    );
    content.insert("active".to_string(), ConfigValue::Boolean(false));

    let test_data = TestData {
        id: "test_user".to_string(),
        content,
    };

    // Write test data to file
    let json = serde_json::to_string_pretty(&test_data)?;
    std::fs::write(&test_data_path, json)?;

    // Create service configurations
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

    let mut service_configs = HashMap::new();
    service_configs.insert("database".to_string(), db_config);
    service_configs.insert("cache".to_string(), cache_config);

    // Create database setup scripts
    let db_setup_scripts = vec![
        "CREATE TABLE IF NOT EXISTS users (id VARCHAR(36) PRIMARY KEY, name VARCHAR(100), email VARCHAR(100), active BOOLEAN)".to_string(),
        "INSERT INTO users (id, name, email, active) VALUES ('user-123', 'John Doe', 'john@example.com', false)".to_string(),
    ];

    // Create lifecycle hooks
    let mut lifecycle_hooks = navius_test::integration::TestLifecycleHooks::default();
    lifecycle_hooks
        .before_setup
        .push("echo 'Setting up test environment'".to_string());
    lifecycle_hooks
        .after_test
        .push("echo 'Test completed, cleaning up'".to_string());

    // Method 1: Using the builder pattern
    println!("Method 1: Using CrossCrateTestBuilder\n");

    let cross_crate_test = CrossCrateTestBuilder::new("enhanced-integration-test")
        .with_crate("navius-core")
        .with_crate("navius-db")
        .with_crate("navius-cache")
        .with_crate("navius-http")
        .with_timeout(Duration::from_secs(30))
        .with_test_data_path(test_data_path.clone())
        .with_service_config("database", service_configs["database"].clone())
        .with_service_config("cache", service_configs["cache"].clone())
        .with_db_setup_script(db_setup_scripts[0].clone())
        .with_db_setup_script(db_setup_scripts[1].clone())
        .with_lifecycle_hook(LifecycleStage::BeforeSetup, "echo 'Before setup'")
        .with_lifecycle_hook(LifecycleStage::AfterTest, "echo 'After test'")
        .with_env_var("APP_ENV", "test")
        .build()?;

    // Run the test
    cross_crate_test.run(run_integration_test).await?;

    // Method 2: Using direct configuration
    println!("\nMethod 2: Using IntegrationTestConfig directly\n");

    let config = IntegrationTestConfig {
        name: "direct-config-test".to_string(),
        test_data_path: Some(test_data_path),
        service_configs,
        db_setup_scripts,
        lifecycle_hooks,
        ..Default::default()
    };

    let runner = navius_test::integration::IntegrationRunner::new(config)?;
    runner.run(run_integration_test).await?;

    // Method 3: Using the convenience function
    println!("\nMethod 3: Using convenience function\n");

    let convenience_test = create_cross_crate_test("convenience-test", |builder| {
        builder
            .with_crate("navius-core")
            .with_crate("navius-db")
            .with_env_var("APP_ENV", "test")
    })?;

    convenience_test.run(|_| Ok(true)).await?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

// Main test function used by different runner configurations
async fn run_integration_test(context: &IntegrationContext) -> TestResult<bool> {
    println!(
        "Running integration test with context: {}",
        context
            .get_env_var("APP_ENV")
            .unwrap_or(Some("unknown".to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    );

    // Load test data
    let user_data = match context.get_test_data("test_user") {
        Ok(data) => {
            println!("Loaded test data for user: {}", data.id);
            data
        }
        Err(e) => {
            println!("No test data found, using defaults: {}", e);
            let mut content = HashMap::new();
            content.insert(
                "name".to_string(),
                ConfigValue::String("Default User".to_string()),
            );
            content.insert(
                "email".to_string(),
                ConfigValue::String("default@example.com".to_string()),
            );
            content.insert("active".to_string(), ConfigValue::Boolean(false));

            TestData {
                id: "default_user".to_string(),
                content,
            }
        }
    };

    // Discover services if configured
    let _ = context.discover_services().map(|discovery| {
        println!(
            "Discovered {} services with {} dependency relationships",
            discovery.instances.len(),
            discovery
                .dependencies
                .values()
                .map(|deps| deps.len())
                .sum::<usize>()
        );
    });

    // Set up mock database client
    let db = context
        .registry()
        .get::<dyn DatabaseClient, MockDatabaseClient>()?;

    // Create a mock row for the user we want to return
    let user_id = "user-123";
    let user_name = user_data
        .content
        .get("name")
        .unwrap()
        .as_string()
        .unwrap_or_else(|| "John Doe".to_string());
    let user_email = user_data
        .content
        .get("email")
        .unwrap()
        .as_string()
        .unwrap_or_else(|| "john@example.com".to_string());
    let user_active = user_data
        .content
        .get("active")
        .unwrap()
        .as_boolean()
        .unwrap_or(false);

    // Set up database mock expectations
    db.expect_query(
        &format!("SELECT * FROM users WHERE id = '{}'", user_id),
        Ok(MockQueryResult::new().add_row({
            let mut row = HashMap::new();
            row.insert("id".to_string(), MockValue::String(user_id.to_string()));
            row.insert("name".to_string(), MockValue::String(user_name.clone()));
            row.insert("email".to_string(), MockValue::String(user_email.clone()));
            row.insert("active".to_string(), MockValue::Boolean(user_active));
            row
        })),
    );

    db.expect_execute(
        &format!("UPDATE users SET active = true WHERE id = '{}'", user_id),
        Ok(1),
    );

    // Set up mock cache client
    let cache = context
        .registry()
        .get::<dyn CacheClient, MockCacheClient>()?;

    // Initially the cache is empty (returns None)
    cache.expect_get(&format!("user:{}", user_id), Ok(None));

    // Expect the set call with the user data
    let user = User {
        id: user_id.to_string(),
        name: user_name,
        email: user_email,
        active: user_active,
    };
    let user_json = serde_json::to_string(&user)?;
    cache.expect_set(&format!("user:{}", user_id), &user_json, Some(300), Ok(()));

    // Expect the delete call when the user is activated
    cache.expect_delete(&format!("user:{}", user_id), Ok(1));

    // Set up mock HTTP client
    let http = context.registry().get::<dyn HttpClient, MockHttpClient>()?;

    // Expect the POST call to the notification service
    let activated_user = User {
        id: user_id.to_string(),
        name: user.name.clone(),
        email: user.email.clone(),
        active: true,
    };
    http.expect_post(
        &format!(
            "https://notification-service/api/users/{}/activate",
            user_id
        ),
        None,
        Some(serde_json::to_string(&user)?),
        Ok(HttpResponse::ok().with_json(r#"{"status":"success"}"#)),
    );

    // Create the service under test
    let user_service = UserService::new(db, cache, http);

    // Test the service methods
    println!("Testing UserService.get_user()");
    let get_result = user_service.get_user(user_id)?;
    assert!(get_result.is_some(), "User should be found");

    let found_user = get_result.unwrap();
    assert_eq!(found_user.id, user_id, "User ID should match");
    assert_eq!(
        found_user.active, user_active,
        "User active status should match"
    );

    println!("Testing UserService.activate_user()");
    let activate_result = user_service.activate_user(user_id)?;
    assert!(activate_result.active, "User should be activated");
    assert_eq!(activate_result.id, user_id, "User ID should match");

    println!("All assertions passed");
    Ok(true)
}
