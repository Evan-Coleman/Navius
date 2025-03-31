use navius_db::{DatabaseConnectionManager, DatabaseError, DatabaseResult, PgPool, PoolOptions};
use navius_test::{
    error::{TestResult, assert_eq, assert_true, assert_matches},
    fixture::TestFixture,
    harness::{TestHarness, TestOptions},
    mock::MockRegistry,
    mocks::{
        MockFixture,
        database::{DatabaseClient, MockDatabaseClient, MockQueryResult, MockValue},
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::test;

// Helper function to create a test fixture with a configured MockDatabaseClient
async fn create_migration_fixture() -> MockFixture {
    let fixture = MockFixture::new();

    // Configure the mock database client with default responses
    let db = fixture.database();
    
    // Set up migration table check query
    db.expect_query(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '_migrations')",
        Ok(MockQueryResult::new().add_row({
            let mut row = HashMap::new();
            row.insert("exists".to_string(), MockValue::Boolean(false));
            row
        })),
    );
    
    // Set up migration table creation
    db.expect_execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id SERIAL PRIMARY KEY,
            version TEXT NOT NULL,
            name TEXT NOT NULL,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            checksum TEXT NOT NULL,
            execution_time_ms INTEGER NOT NULL
        )",
        Ok(1),
    );
    
    // Set up applied migrations query
    db.expect_query(
        "SELECT version, name, checksum FROM _migrations ORDER BY id",
        Ok(MockQueryResult::new().add_row({
            let mut row = HashMap::new();
            row.insert("version".to_string(), MockValue::String("00001".to_string()));
            row.insert("name".to_string(), MockValue::String("initial_schema".to_string()));
            row.insert("checksum".to_string(), MockValue::String("abcdef1234567890".to_string()));
            row
        })),
    );
    
    // Set up migration application
    db.expect_execute(
        "INSERT INTO _migrations (version, name, checksum, execution_time_ms) VALUES ($1, $2, $3, $4)",
        Ok(1),
    );
    
    // Set up migration content execution
    db.expect_execute(
        "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )",
        Ok(1),
    );
    
    fixture
}

// Helper function to create a test database pool using the mock client
async fn create_test_pool(fixture: &MockFixture) -> PgPool {
    let options = PoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(3));

    // Use the mock database client from the fixture
    PgPool::new_with_client(fixture.database(), options)
}

// Helper to create a test database connection manager
async fn create_test_db(fixture: &MockFixture) -> Arc<DatabaseConnectionManager> {
    let pool = create_test_pool(fixture).await;
    Arc::new(DatabaseConnectionManager::new(pool))
}

#[test]
async fn test_migration_table_creation() -> TestResult<()> {
    // Arrange
    let fixture = create_migration_fixture().await;
    let db = create_test_db(&fixture).await;

    // Create a test harness with appropriate options
    let options = TestOptions {
        verify_mocks: true,
        cleanup_resources: true,
        timeout: Some(Duration::from_secs(5)),
    };

    let harness = TestHarness::new().with_options(options).with_subject(db)?;

    // Act & Assert using the test harness
    harness
        .run(|db| async move {
            let conn = db.connection().await?;
            
            // Check if migration table exists
            let result = conn
                .query(
                    "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '_migrations')",
                    &[],
                )
                .await?;
            
            let table_exists = result[0].get::<_, bool>("exists");
            assert_eq(
                table_exists,
                false,
                "Migration table should not exist initially",
            )?;
            
            // Create migration table
            let create_result = conn
                .execute(
                    "CREATE TABLE IF NOT EXISTS _migrations (
                        id SERIAL PRIMARY KEY,
                        version TEXT NOT NULL,
                        name TEXT NOT NULL,
                        applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                        checksum TEXT NOT NULL,
                        execution_time_ms INTEGER NOT NULL
                    )",
                    &[],
                )
                .await;
                
            assert_true(
                create_result.is_ok(),
                "Creating migration table should succeed",
            )?;
            
            // Verify all mock expectations were met
            fixture.verify()?;

            Ok(())
        })
        .await
}

#[test]
async fn test_get_applied_migrations() -> TestResult<()> {
    // Arrange
    let fixture = create_migration_fixture().await;
    let db = create_test_db(&fixture).await;

    // Act & Assert
    let conn = db.connection().await?;
    
    // Query applied migrations
    let result = conn
        .query(
            "SELECT version, name, checksum FROM _migrations ORDER BY id",
            &[],
        )
        .await?;
    
    // Check that we got the expected migration
    assert_eq(
        result.len(),
        1,
        "Should have one applied migration",
    )?;
    
    let version = result[0].get::<_, String>("version");
    let name = result[0].get::<_, String>("name");
    let checksum = result[0].get::<_, String>("checksum");
    
    assert_eq(
        version,
        "00001",
        "Migration version should be 00001",
    )?;
    
    assert_eq(
        name,
        "initial_schema",
        "Migration name should be initial_schema",
    )?;
    
    assert_eq(
        checksum,
        "abcdef1234567890",
        "Migration checksum should match",
    )?;
    
    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
}

#[test]
async fn test_apply_migration() -> TestResult<()> {
    // Arrange
    let fixture = create_migration_fixture().await;
    let db = create_test_db(&fixture).await;

    // Act & Assert
    let conn = db.connection().await?;
    
    // Execute migration SQL
    let execute_result = conn
        .execute(
            "CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await;
    
    assert_true(
        execute_result.is_ok(),
        "Executing migration SQL should succeed",
    )?;
    
    // Record migration
    let record_result = conn
        .execute(
            "INSERT INTO _migrations (version, name, checksum, execution_time_ms) VALUES ($1, $2, $3, $4)",
            &["00002", "create_users", "0123456789abcdef", &50],
        )
        .await;
    
    assert_true(
        record_result.is_ok(),
        "Recording migration should succeed",
    )?;
    
    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
}

// This test would typically be ignored in normal runs as it would need a real database
#[test]
#[ignore]
async fn test_migration_integration() -> TestResult<()> {
    // This would be a full integration test with a real database
    // In a real test environment, we'd:
    // 1. Create a test database
    // 2. Apply migrations from a test directory
    // 3. Verify the schema matches expectations
    // 4. Test rolling back migrations
    
    // For now, we'll just verify our test infrastructure works
    let fixture = create_migration_fixture().await;
    let db = create_test_db(&fixture).await;
    
    // Verify connection works
    let conn = db.connection().await?;
    assert_true(conn.is_valid().await?, "Connection should be valid")?;
    
    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
} 