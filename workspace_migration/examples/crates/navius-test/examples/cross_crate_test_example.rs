use navius_test::{
    error::{TestResult, assert_eq, assert_ok},
    fixture::TestFixture,
    integration::{IntegrationContext, IntegrationRunner, IntegrationTestConfig},
    mock::MockRegistry,
    mocks::{
        MockFixture,
        database::{DatabaseClient, MockDatabaseClient, MockQueryResult, MockValue},
        filesystem::{FileSystem, MockFileSystem},
    },
};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

// Simulate types from multiple crates
// These would normally be imported from different crates

// From navius-db
trait Repository {
    fn find_by_id(&self, id: &str) -> Result<Option<Entity>, RepositoryError>;
    fn save(&self, entity: &Entity) -> Result<(), RepositoryError>;
}

// From navius-core
#[derive(Debug, Clone, PartialEq)]
struct Entity {
    id: String,
    name: String,
    active: bool,
}

// From navius-core
#[derive(Debug, thiserror::Error)]
enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

// From navius-service
struct EntityService<R: Repository> {
    repository: R,
    file_service: FileService,
}

// From navius-service
impl<R: Repository> EntityService<R> {
    fn new(repository: R, file_service: FileService) -> Self {
        Self {
            repository,
            file_service,
        }
    }

    fn get_entity(&self, id: &str) -> Result<Option<Entity>, ServiceError> {
        self.repository
            .find_by_id(id)
            .map_err(|e| ServiceError::Repository(e.to_string()))
    }

    fn activate_entity(&self, id: &str) -> Result<Entity, ServiceError> {
        let entity = self
            .repository
            .find_by_id(id)
            .map_err(|e| ServiceError::Repository(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound(format!("Entity not found: {}", id)))?;

        let mut updated = entity.clone();
        updated.active = true;

        // Write to audit log
        self.file_service
            .write_log(
                "audit.log",
                &format!("Entity {} activated at {}", id, chrono::Utc::now()),
            )
            .map_err(|e| ServiceError::File(e.to_string()))?;

        self.repository
            .save(&updated)
            .map_err(|e| ServiceError::Repository(e.to_string()))?;

        Ok(updated)
    }
}

// From navius-filesystem
struct FileService {
    filesystem: Arc<dyn FileSystem>,
}

// From navius-filesystem
impl FileService {
    fn new(filesystem: Arc<dyn FileSystem>) -> Self {
        Self { filesystem }
    }

    fn write_log(&self, filename: &str, content: &str) -> Result<(), std::io::Error> {
        self.filesystem
            .write_file(filename, content.as_bytes())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

// From navius-service
#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("File error: {0}")]
    File(String),
}

// From navius-db
struct DatabaseRepository {
    db_client: Arc<dyn DatabaseClient>,
}

// From navius-db
impl DatabaseRepository {
    fn new(db_client: Arc<dyn DatabaseClient>) -> Self {
        Self { db_client }
    }
}

// From navius-db
impl Repository for DatabaseRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Entity>, RepositoryError> {
        let query = format!("SELECT * FROM entities WHERE id = '{}'", id);

        let result = self
            .db_client
            .query(&query)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.is_empty() {
            return Ok(None);
        }

        let row = result.row(0).unwrap();
        let id = row.get("id").unwrap().as_string().unwrap();
        let name = row.get("name").unwrap().as_string().unwrap();
        let active = row.get("active").unwrap().as_boolean().unwrap();

        Ok(Some(Entity { id, name, active }))
    }

    fn save(&self, entity: &Entity) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE entities SET name = '{}', active = {} WHERE id = '{}'",
            entity.name, entity.active, entity.id
        );

        self.db_client
            .execute(&query)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

// Integration test that tests interactions between components from different crates
#[tokio::main]
async fn main() -> TestResult<()> {
    println!("Running cross-crate integration test example");

    // Create integration test configuration
    let config = IntegrationTestConfig {
        name: "entity-service-integration-test".to_string(),
        verify_mocks: true,
        cleanup_resources: true,
        timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };

    // Create integration test runner
    let runner = IntegrationRunner::new(config)?;

    // Run the test with the integration context
    runner
        .run(|context| async move {
            // Set up test data
            let entity_id = "test-123";
            let entity = Entity {
                id: entity_id.to_string(),
                name: "Test Entity".to_string(),
                active: false,
            };

            // Set up mock database
            let db = context
                .registry()
                .get::<dyn DatabaseClient, MockDatabaseClient>()?;

            // Configure mock to return our test entity
            db.expect_query(
                &format!("SELECT * FROM entities WHERE id = '{}'", entity_id),
                Ok(MockQueryResult::new().add_row({
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), MockValue::String(entity_id.to_string()));
                    row.insert(
                        "name".to_string(),
                        MockValue::String("Test Entity".to_string()),
                    );
                    row.insert("active".to_string(), MockValue::Boolean(false));
                    row
                })),
            );

            // Configure mock to handle the update
            db.expect_execute(
                &format!(
                    "UPDATE entities SET name = '{}', active = {} WHERE id = '{}'",
                    entity.name, true, entity.id
                ),
                Ok(1),
            );

            // Set up mock filesystem
            let fs = context.registry().get::<dyn FileSystem, MockFileSystem>()?;

            // Configure mock to handle log write
            fs.add_file("audit.log", "").unwrap();

            // Create the components under test
            let repository = DatabaseRepository::new(db);
            let file_service = FileService::new(fs);
            let service = EntityService::new(repository, file_service);

            // Execute the test
            let result = service.activate_entity(entity_id);

            // Verify the result
            assert_ok(&result, "Activating entity should succeed")?;

            let updated_entity = result.unwrap();
            assert_eq(
                updated_entity.id,
                entity_id.to_string(),
                "Entity ID should match",
            )?;
            assert_eq(
                updated_entity.name,
                "Test Entity".to_string(),
                "Entity name should match",
            )?;
            assert_eq(updated_entity.active, true, "Entity should be activated")?;

            // Verify that the filesystem was used to write the log
            let fs = context.registry().get::<dyn FileSystem, MockFileSystem>()?;
            let log_content = fs.read_file("audit.log").unwrap();
            let log_str = String::from_utf8_lossy(&log_content);

            assert!(
                log_str.contains(&format!("Entity {} activated", entity_id)),
                "Audit log should contain activation message"
            );

            Ok(())
        })
        .await
}
