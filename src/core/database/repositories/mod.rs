//! Database repository traits and implementations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::error::AppError;

/// Represents a pet in the database
#[derive(Debug, Clone)]
pub struct Pet {
    /// Unique identifier for the pet
    pub id: Uuid,
    /// Name of the pet
    pub name: String,
    /// Type of pet (e.g., dog, cat)
    pub pet_type: String,
    /// Pet's breed
    pub breed: Option<String>,
    /// Age of the pet in years
    pub age: Option<i32>,
    /// When the pet record was created
    pub created_at: DateTime<Utc>,
    /// When the pet record was last updated
    pub updated_at: DateTime<Utc>,
}

/// Interface for pet repository operations
#[async_trait]
pub trait PetRepository: Send + Sync + 'static {
    /// Find all pets in the database
    async fn find_all(&self) -> Result<Vec<Pet>, AppError>;

    /// Find a pet by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError>;

    /// Create a new pet
    async fn create(
        &self,
        name: String,
        pet_type: String,
        breed: Option<String>,
        age: Option<i32>,
    ) -> Result<Pet, AppError>;

    /// Update an existing pet
    async fn update(
        &self,
        id: Uuid,
        name: String,
        pet_type: String,
        breed: Option<String>,
        age: Option<i32>,
    ) -> Result<Option<Pet>, AppError>;

    /// Delete a pet by its ID
    async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
}

/// Implementation of the PetRepository trait using Postgres
pub struct PgPetRepository {
    pool: PgPool,
}

impl PgPetRepository {
    /// Create a new PgPetRepository with the given database pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PetRepository for PgPetRepository {
    async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
        let pets = sqlx::query_as!(
            Pet,
            r#"
            SELECT 
                id, 
                name, 
                pet_type, 
                breed, 
                age, 
                created_at, 
                updated_at
            FROM pets
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch pets: {}", e)))?;

        Ok(pets)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError> {
        let pet = sqlx::query_as!(
            Pet,
            r#"
            SELECT 
                id, 
                name, 
                pet_type, 
                breed, 
                age, 
                created_at, 
                updated_at
            FROM pets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch pet: {}", e)))?;

        Ok(pet)
    }

    async fn create(
        &self,
        name: String,
        pet_type: String,
        breed: Option<String>,
        age: Option<i32>,
    ) -> Result<Pet, AppError> {
        let pet = sqlx::query_as!(
            Pet,
            r#"
            INSERT INTO pets (name, pet_type, breed, age)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            name,
            pet_type,
            breed,
            age
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create pet: {}", e)))?;

        Ok(pet)
    }

    async fn update(
        &self,
        id: Uuid,
        name: String,
        pet_type: String,
        breed: Option<String>,
        age: Option<i32>,
    ) -> Result<Option<Pet>, AppError> {
        let pet = sqlx::query_as!(
            Pet,
            r#"
            UPDATE pets
            SET 
                name = $2,
                pet_type = $3,
                breed = $4,
                age = $5,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            id,
            name,
            pet_type,
            breed,
            age
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update pet: {}", e)))?;

        Ok(pet)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM pets
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete pet: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }
}
