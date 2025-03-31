use navius_db::{DatabaseConnectionManager, DatabaseError, DatabaseResult, PgPool, PoolOptions};
use navius_test::{
    error::{TestResult, assert_eq, assert_true},
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
async fn create_test_fixture() -> MockFixture {
    let fixture = MockFixture::new();

    // Configure the mock database client with default responses
    let db = fixture.database();

    // Set up schema-related query expectations
    db.expect_execute(
        "CREATE TABLE IF NOT EXISTS schema_test (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
        Ok(0),
    );

    db.expect_execute(
        "ALTER TABLE schema_test ADD COLUMN IF NOT EXISTS description TEXT",
        Ok(0),
    );

    db.expect_execute(
        "CREATE INDEX IF NOT EXISTS idx_schema_test_name ON schema_test (name)",
        Ok(0),
    );

    db.expect_query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
        Ok(MockQueryResult::new().add_row({
            let mut row = HashMap::new();
            row.insert(
                "table_name".to_string(),
                MockValue::String("schema_test".to_string()),
            );
            row
        })),
    );

    db.expect_query(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'schema_test'",
        Ok(MockQueryResult::new()
            .add_row({
                let mut row = HashMap::new();
                row.insert("column_name".to_string(), MockValue::String("id".to_string()));
                row.insert("data_type".to_string(), MockValue::String("integer".to_string()));
                row
            })
            .add_row({
                let mut row = HashMap::new();
                row.insert("column_name".to_string(), MockValue::String("name".to_string()));
                row.insert("data_type".to_string(), MockValue::String("text".to_string()));
                row
            })
            .add_row({
                let mut row = HashMap::new();
                row.insert("column_name".to_string(), MockValue::String("description".to_string()));
                row.insert("data_type".to_string(), MockValue::String("text".to_string()));
                row
            })),
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
async fn test_create_table() -> TestResult<()> {
    // Arrange
    let fixture = create_test_fixture().await;
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
            // Test creating a table
            let result = db
                .connection()
                .await?
                .execute(
                    "CREATE TABLE IF NOT EXISTS schema_test (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
                    &[],
                )
                .await;

            assert_true(
                result.is_ok(),
                "Table creation should succeed",
            )?;

            // Verify all mock expectations were met
            fixture.verify()?;

            Ok(())
        })
        .await
}

#[test]
async fn test_alter_table() -> TestResult<()> {
    // Arrange
    let fixture = create_test_fixture().await;
    let db = create_test_db(&fixture).await;

    // Act & Assert
    let result = db
        .connection()
        .await?
        .execute(
            "ALTER TABLE schema_test ADD COLUMN IF NOT EXISTS description TEXT",
            &[],
        )
        .await;

    assert_true(result.is_ok(), "Table alteration should succeed")?;

    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
}

#[test]
async fn test_create_index() -> TestResult<()> {
    // Arrange
    let fixture = create_test_fixture().await;
    let db = create_test_db(&fixture).await;

    // Act & Assert
    let result = db
        .connection()
        .await?
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_schema_test_name ON schema_test (name)",
            &[],
        )
        .await;

    assert_true(result.is_ok(), "Index creation should succeed")?;

    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
}

#[test]
async fn test_query_schema_info() -> TestResult<()> {
    // Arrange
    let fixture = create_test_fixture().await;
    let db = create_test_db(&fixture).await;

    // Act
    let conn = db.connection().await?;

    // Get table list
    let tables_result = conn
        .query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
            &[],
        )
        .await;

    // Assert tables query succeeded
    assert_true(tables_result.is_ok(), "Table schema query should succeed")?;

    let tables = tables_result?;
    assert_eq(tables.len(), 1, "Should find exactly one table")?;

    let table_name = tables[0].get::<_, String>("table_name");
    assert_eq(
        table_name,
        "schema_test",
        "Should find the schema_test table",
    )?;

    // Get column info
    let columns_result = conn
        .query(
            "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'schema_test'",
            &[],
        )
        .await;

    // Assert columns query succeeded
    assert_true(columns_result.is_ok(), "Column schema query should succeed")?;

    let columns = columns_result?;
    assert_eq(columns.len(), 3, "Schema test table should have 3 columns")?;

    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
}

// This test would be ignored in normal runs as it requires an actual database
#[test]
#[ignore]
async fn test_schema_migrations() -> TestResult<()> {
    // This would be a full integration test with a real database
    // In a real test, we would:
    // 1. Apply migrations
    // 2. Verify schema changes
    // 3. Test rollbacks

    // For mock testing, we'll just verify the infrastructure works
    let fixture = create_test_fixture().await;
    let db = create_test_db(&fixture).await;

    // Verify we can call the database
    let conn = db.connection().await?;
    assert_true(conn.is_valid().await?, "Connection should be valid")?;

    // Verify all mock expectations were met
    fixture.verify()?;

    Ok(())
}
