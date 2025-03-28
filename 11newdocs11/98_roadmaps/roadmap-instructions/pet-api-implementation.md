# Pet API Implementation Instructions

## Overview
These instructions provide a step-by-step guide for implementing the Pet API with database integration following clean architecture principles. We'll focus on proper separation of concerns by keeping database abstractions in the core layer while implementing minimal pet-specific logic in the app layer.

## Prerequisites
- Rust and Cargo installed
- PostgreSQL database set up
- Basic understanding of Axum web framework
- Familiarity with the existing codebase structure

## Implementation Steps

### 1. Core Database Abstractions

#### Create Generic Repository Trait
Create a new file in `src/core/database/repository.rs`:
```rust
use async_trait::async_trait;
use uuid::Uuid;
use std::error::Error;

/// Generic repository trait that provides standard CRUD operations
#[async_trait]
pub trait EntityRepository<T, E: Error + Send + Sync>: Send + Sync {
    /// Retrieve all entities
    async fn find_all(&self) -> Result<Vec<T>, E>;
    
    /// Find entity by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>, E>;
    
    /// Create a new entity
    async fn create(&self, entity: T) -> Result<T, E>;
    
    /// Update an existing entity
    async fn update(&self, entity: T) -> Result<T, E>;
    
    /// Delete an entity by ID
    async fn delete(&self, id: Uuid) -> Result<bool, E>;
}
```

#### Create Database Utilities
Create a new file in `src/core/database/utils.rs`:
```rust
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// Helper function to execute a database transaction
pub async fn with_transaction<F, T, E>(
    pool: &Pool<Postgres>,
    f: F,
) -> Result<T, E>
where
    F: FnOnce(&mut sqlx::Transaction<'_, Postgres>) -> Result<T, E> + Send,
    E: From<sqlx::Error> + Send,
    T: Send,
{
    let mut tx = pool.begin().await.map_err(|e| E::from(e))?;
    
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await.map_err(|e| E::from(e))?;
            Ok(result)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Generate a new UUID
pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}
```

#### Update Core Database Module
Update `src/core/database/mod.rs`:
```rust
// Generic database abstractions
pub mod repository;
pub mod utils;

// Re-export key items
pub use repository::EntityRepository;
pub use utils::{with_transaction, generate_uuid};
```

### 2. Pet Repository Implementation

#### Implement Pet Repository Using Core Abstractions
Update `src/app/database/repositories/pet_repository.rs`:
```rust
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::core::database::EntityRepository;
use crate::core::error::AppError;
use crate::core::database::utils::with_transaction;

/// Pet entity
#[derive(Debug, Clone)]
pub struct Pet {
    pub id: Uuid,
    pub name: String,
    pub species: String,
    pub age: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Repository for pet operations
pub struct PetRepository {
    db_pool: Arc<Pool<Postgres>>,
}

impl PetRepository {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self { db_pool }
    }
    
    /// Find pets by species
    pub async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, AppError> {
        let pets = sqlx::query_as!(
            Pet,
            r#"
            SELECT id, name, species, age, created_at, updated_at
            FROM pets
            WHERE species = $1
            "#,
            species
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(pets)
    }
}

#[async_trait]
impl EntityRepository<Pet, AppError> for PetRepository {
    async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
        let pets = sqlx::query_as!(
            Pet,
            r#"
            SELECT id, name, species, age, created_at, updated_at
            FROM pets
            "#
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(pets)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError> {
        let pet = sqlx::query_as!(
            Pet,
            r#"
            SELECT id, name, species, age, created_at, updated_at
            FROM pets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(pet)
    }
    
    async fn create(&self, pet: Pet) -> Result<Pet, AppError> {
        let created_pet = sqlx::query_as!(
            Pet,
            r#"
            INSERT INTO pets (id, name, species, age, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, species, age, created_at, updated_at
            "#,
            pet.id,
            pet.name,
            pet.species,
            pet.age,
            pet.created_at,
            pet.updated_at
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(created_pet)
    }
    
    async fn update(&self, pet: Pet) -> Result<Pet, AppError> {
        let updated_pet = sqlx::query_as!(
            Pet,
            r#"
            UPDATE pets
            SET name = $2, species = $3, age = $4, updated_at = $5
            WHERE id = $1
            RETURNING id, name, species, age, created_at, updated_at
            "#,
            pet.id,
            pet.name,
            pet.species,
            pet.age,
            Utc::now()
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(updated_pet)
    }
    
    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM pets
            WHERE id = $1
            "#,
            id
        )
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(result.rows_affected() > 0)
    }
}
```

### 3. Service Layer

#### Create Pet Service
Create a new file in `src/services/pet.rs`:
```rust
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::app::database::repositories::{Pet, PetRepository};
use crate::core::database::EntityRepository;
use crate::services::error::ServiceError;

/// DTOs for pets (kept minimal)
#[derive(Debug)]
pub struct CreatePetDto {
    pub name: String,
    pub species: String,
    pub age: i32,
}

#[derive(Debug)]
pub struct UpdatePetDto {
    pub name: Option<String>,
    pub species: Option<String>,
    pub age: Option<i32>,
}

/// Pet service for handling business logic
pub struct PetService {
    pet_repository: Arc<PetRepository>,
}

impl PetService {
    pub fn new(pet_repository: Arc<PetRepository>) -> Self {
        Self { pet_repository }
    }
    
    /// Get all pets
    pub async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError> {
        self.pet_repository.find_all().await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
    
    /// Get pet by ID
    pub async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError> {
        let pet = self.pet_repository.find_by_id(id).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        match pet {
            Some(pet) => Ok(pet),
            None => Err(ServiceError::PetNotFound),
        }
    }
    
    /// Find pets by species
    pub async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, ServiceError> {
        self.pet_repository.find_by_species(species).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
    
    /// Create a new pet
    pub async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError> {
        // Validate input
        if dto.name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if dto.species.trim().is_empty() {
            return Err(ServiceError::ValidationError("Species cannot be empty".to_string()));
        }
        
        if dto.age < 0 {
            return Err(ServiceError::ValidationError("Age cannot be negative".to_string()));
        }
        
        // Create new pet entity
        let new_pet = Pet {
            id: Uuid::new_v4(),
            name: dto.name,
            species: dto.species,
            age: dto.age,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Save to database
        self.pet_repository.create(new_pet).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
    
    /// Update an existing pet
    pub async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError> {
        // Get existing pet
        let existing_pet = self.get_pet_by_id(id).await?;
        
        // Validate input
        if let Some(ref name) = dto.name {
            if name.trim().is_empty() {
                return Err(ServiceError::ValidationError("Name cannot be empty".to_string()));
            }
        }
        
        if let Some(ref species) = dto.species {
            if species.trim().is_empty() {
                return Err(ServiceError::ValidationError("Species cannot be empty".to_string()));
            }
        }
        
        if let Some(age) = dto.age {
            if age < 0 {
                return Err(ServiceError::ValidationError("Age cannot be negative".to_string()));
            }
        }
        
        // Update fields
        let updated_pet = Pet {
            id: existing_pet.id,
            name: dto.name.unwrap_or(existing_pet.name),
            species: dto.species.unwrap_or(existing_pet.species),
            age: dto.age.unwrap_or(existing_pet.age),
            created_at: existing_pet.created_at,
            updated_at: Utc::now(),
        };
        
        // Save to database
        self.pet_repository.update(updated_pet).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
    
    /// Delete a pet
    pub async fn delete_pet(&self, id: Uuid) -> Result<(), ServiceError> {
        // Check if pet exists
        let _ = self.get_pet_by_id(id).await?;
        
        // Delete pet
        let deleted = self.pet_repository.delete(id).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::PetNotFound)
        }
    }
}
```

### 4. API Endpoints

#### Create Pet API Module
Create a new file in `src/app/api/pet.rs`:
```rust
use axum::{
    Router,
    extract::{Json, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::app::database::repositories::{Pet, PetRepository};
use crate::core::router::AppState;
use crate::services::{
    error::ServiceError,
    pet::{CreatePetDto, PetService, UpdatePetDto},
};

// API response for pet operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PetResponse {
    pub id: String,
    pub name: String,
    pub species: String,
    pub age: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Pet> for PetResponse {
    fn from(pet: Pet) -> Self {
        Self {
            id: pet.id.to_string(),
            name: pet.name,
            species: pet.species,
            age: pet.age,
            created_at: pet.created_at.to_rfc3339(),
            updated_at: pet.updated_at.to_rfc3339(),
        }
    }
}

// Create pet request
#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePetRequest {
    pub name: String,
    pub species: String,
    pub age: i32,
}

// Update pet request
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePetRequest {
    pub name: Option<String>,
    pub species: Option<String>,
    pub age: Option<i32>,
}

// Configure pet routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/petdb", get(get_all_pets))
        .route("/petdb", post(create_pet))
        .route("/petdb/:id", get(get_pet))
        .route("/petdb/:id", put(update_pet))
        .route("/petdb/:id", delete(delete_pet))
}

// Map service errors to HTTP status codes
fn map_service_error(err: ServiceError) -> (StatusCode, String) {
    match err {
        ServiceError::PetNotFound => (StatusCode::NOT_FOUND, "Pet not found".to_string()),
        ServiceError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        ServiceError::DatabaseError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", msg),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR, 
            "Internal server error".to_string()
        ),
    }
}

// Get all pets
async fn get_all_pets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<PetResponse>>, (StatusCode, String)> {
    // Get pet service
    let pet_service = get_pet_service(state)?;
    
    // Check for species query parameter
    if let Some(species) = params.get("species") {
        // Find by species
        let pets = pet_service
            .find_by_species(species)
            .await
            .map_err(map_service_error)?;
            
        // Convert to response format
        let responses = pets.into_iter().map(PetResponse::from).collect();
        return Ok(Json(responses));
    }
    
    // Get all pets
    let pets = pet_service
        .get_all_pets()
        .await
        .map_err(map_service_error)?;
        
    // Convert to response format
    let responses = pets.into_iter().map(PetResponse::from).collect();
    
    Ok(Json(responses))
}

// Get a pet by ID
async fn get_pet(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Result<Json<PetResponse>, (StatusCode, String)> {
    // Parse UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())),
    };
    
    // Get pet service
    let pet_service = get_pet_service(state)?;
    
    // Get pet by ID
    let pet = pet_service
        .get_pet_by_id(id)
        .await
        .map_err(map_service_error)?;
        
    // Convert to response format
    let response = PetResponse::from(pet);
    
    Ok(Json(response))
}

// Create a new pet
async fn create_pet(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePetRequest>,
) -> Result<(StatusCode, Json<PetResponse>), (StatusCode, String)> {
    // Get pet service
    let pet_service = get_pet_service(state)?;
    
    // Create DTO
    let create_dto = CreatePetDto {
        name: request.name,
        species: request.species,
        age: request.age,
    };
    
    // Create pet
    let pet = pet_service
        .create_pet(create_dto)
        .await
        .map_err(map_service_error)?;
        
    // Convert to response format
    let response = PetResponse::from(pet);
    
    Ok((StatusCode::CREATED, Json(response)))
}

// Update a pet
async fn update_pet(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdatePetRequest>,
) -> Result<Json<PetResponse>, (StatusCode, String)> {
    // Parse UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())),
    };
    
    // Get pet service
    let pet_service = get_pet_service(state)?;
    
    // Create DTO
    let update_dto = UpdatePetDto {
        name: request.name,
        species: request.species,
        age: request.age,
    };
    
    // Update pet
    let pet = pet_service
        .update_pet(id, update_dto)
        .await
        .map_err(map_service_error)?;
        
    // Convert to response format
    let response = PetResponse::from(pet);
    
    Ok(Json(response))
}

// Delete a pet
async fn delete_pet(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Parse UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())),
    };
    
    // Get pet service
    let pet_service = get_pet_service(state)?;
    
    // Delete pet
    pet_service
        .delete_pet(id)
        .await
        .map_err(map_service_error)?;
        
    Ok(StatusCode::NO_CONTENT)
}

// Helper function to get pet service
fn get_pet_service(state: Arc<AppState>) -> Result<PetService, (StatusCode, String)> {
    // Get database pool
    let db_pool = match state.db_pool {
        Some(ref pool) => pool.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database pool not initialized".to_string(),
            ));
        }
    };
    
    // Create repository
    let pet_repo = Arc::new(PetRepository::new(db_pool));
    
    // Create service
    let pet_service = PetService::new(pet_repo);
    
    Ok(pet_service)
}
```

### 5. Update Router Configuration

#### Update App Router
Edit `src/app/api/mod.rs` to include the pet routes:
```rust
pub mod pet;

// ... existing code ...

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(pet::configure())
        // Other routes here
}
```

### 6. Testing

#### Create Core Repository Tests
Create a test file in `src/core/database/repository_test.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;
    use std::error::Error;
    use std::fmt;
    
    // Define a simple test entity
    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        id: Uuid,
        name: String,
    }
    
    // Define a simple error type
    #[derive(Debug)]
    struct TestError(String);
    
    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error: {}", self.0)
        }
    }
    
    impl Error for TestError {}
    
    // Create a mock repository implementation
    struct MockRepository {
        entities: Vec<TestEntity>,
    }
    
    impl MockRepository {
        fn new() -> Self {
            Self { entities: Vec::new() }
        }
    }
    
    #[async_trait]
    impl EntityRepository<TestEntity, TestError> for MockRepository {
        async fn find_all(&self) -> Result<Vec<TestEntity>, TestError> {
            Ok(self.entities.clone())
        }
        
        async fn find_by_id(&self, id: Uuid) -> Result<Option<TestEntity>, TestError> {
            let entity = self.entities.iter()
                .find(|e| e.id == id)
                .cloned();
                
            Ok(entity)
        }
        
        async fn create(&self, entity: TestEntity) -> Result<TestEntity, TestError> {
            // In a real implementation, this would add to self.entities
            // but since self is immutable in this method, we just return the entity
            Ok(entity)
        }
        
        async fn update(&self, entity: TestEntity) -> Result<TestEntity, TestError> {
            // Similar to create, just return the entity
            Ok(entity)
        }
        
        async fn delete(&self, id: Uuid) -> Result<bool, TestError> {
            // Check if entity exists
            let exists = self.entities.iter().any(|e| e.id == id);
            Ok(exists)
        }
    }
    
    #[tokio::test]
    async fn test_repository_operations() {
        // Create a mock repository
        let repo = MockRepository::new();
        
        // Test creating an entity
        let id = Uuid::new_v4();
        let entity = TestEntity { id, name: "Test".to_string() };
        let created = repo.create(entity.clone()).await.unwrap();
        assert_eq!(created.id, id);
        assert_eq!(created.name, "Test");
        
        // More tests for other operations...
    }
}
```

#### Create Pet Service Tests
Create a test file in `src/services/pet_test.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mockall::predicate::*;
    use mockall::*;
    use uuid::Uuid;
    use chrono::Utc;
    
    use crate::app::database::repositories::Pet;
    use crate::core::error::AppError;
    
    // Create a mock repository for testing
    mock! {
        PetRepository {}
        
        #[async_trait]
        impl EntityRepository<Pet, AppError> for PetRepository {
            async fn find_all(&self) -> Result<Vec<Pet>, AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError>;
            async fn create(&self, pet: Pet) -> Result<Pet, AppError>;
            async fn update(&self, pet: Pet) -> Result<Pet, AppError>;
            async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
        }
        
        impl PetRepository {
            fn new(_: Arc<Pool<Postgres>>) -> Self;
            async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, AppError>;
        }
    }
    
    // Helper function to create a test pet
    fn create_test_pet() -> Pet {
        Pet {
            id: Uuid::new_v4(),
            name: "Fluffy".to_string(),
            species: "Cat".to_string(),
            age: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    #[tokio::test]
    async fn test_get_pet_by_id() {
        // Arrange
        let mut mock_repo = MockPetRepository::new();
        let pet_id = Uuid::new_v4();
        let test_pet = create_test_pet();
        
        mock_repo
            .expect_find_by_id()
            .with(eq(pet_id))
            .returning(move |_| Ok(Some(test_pet.clone())));
            
        let service = PetService::new(Arc::new(mock_repo));
        
        // Act
        let result = service.get_pet_by_id(pet_id).await;
        
        // Assert
        assert!(result.is_ok());
        let pet = result.unwrap();
        assert_eq!(pet.id, test_pet.id);
        assert_eq!(pet.name, "Fluffy");
        assert_eq!(pet.species, "Cat");
        assert_eq!(pet.age, 3);
    }
    
    #[tokio::test]
    async fn test_get_pet_by_id_not_found() {
        // Arrange
        let mut mock_repo = MockPetRepository::new();
        let pet_id = Uuid::new_v4();
        
        mock_repo
            .expect_find_by_id()
            .with(eq(pet_id))
            .returning(|_| Ok(None));
            
        let service = PetService::new(Arc::new(mock_repo));
        
        // Act
        let result = service.get_pet_by_id(pet_id).await;
        
        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::PetNotFound) => (),
            _ => panic!("Expected PetNotFound error"),
        }
    }
    
    #[tokio::test]
    async fn test_create_pet() {
        // Arrange
        let mut mock_repo = MockPetRepository::new();
        let test_pet = create_test_pet();
        
        mock_repo
            .expect_create()
            .returning(move |pet| Ok(pet));
            
        let service = PetService::new(Arc::new(mock_repo));
        
        let dto = CreatePetDto {
            name: "Fluffy".to_string(),
            species: "Cat".to_string(),
            age: 3,
        };
        
        // Act
        let result = service.create_pet(dto).await;
        
        // Assert
        assert!(result.is_ok());
        let pet = result.unwrap();
        assert_eq!(pet.name, "Fluffy");
        assert_eq!(pet.species, "Cat");
        assert_eq!(pet.age, 3);
    }
    
    // Add more tests as needed
}
```

## Implementation Checklist

- [ ] Create core database abstractions
  - [ ] Generic Repository trait
  - [ ] Database utilities
  - [ ] Update module exports

- [ ] Implement Pet Repository
  - [ ] Map to EntityRepository trait
  - [ ] Implement specific methods like find_by_species

- [ ] Create Pet Service
  - [ ] Basic CRUD operations
  - [ ] Input validation
  - [ ] Error handling

- [ ] Set up API endpoints
  - [ ] CRUD endpoints for /petdb
  - [ ] Request validation
  - [ ] Error mapping

- [ ] Configure Router
  - [ ] Include pet routes in app router

- [ ] Add Tests
  - [ ] Core abstraction tests
  - [ ] Service layer tests
  - [ ] API endpoint tests

## Testing the Implementation

```bash
# Run tests for core database abstractions
cargo test core::database

# Run tests for pet service
cargo test services::pet

# Run tests for pet API
cargo test app::api::pet

# Start the server
./run_dev.sh

# Test API endpoints
curl http://localhost:8080/petdb
curl http://localhost:8080/petdb/[UUID]
curl -X POST -H "Content-Type: application/json" -d '{"name":"Fluffy","species":"Cat","age":3}' http://localhost:8080/petdb
curl -X PUT -H "Content-Type: application/json" -d '{"name":"Fluffy Updated"}' http://localhost:8080/petdb/[UUID]
curl -X DELETE http://localhost:8080/petdb/[UUID]
``` 