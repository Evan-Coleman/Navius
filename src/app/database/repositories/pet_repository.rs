use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::AppError;

/// Representation of a pet in the database
#[derive(Debug, Clone)]
pub struct Pet {
    /// Unique identifier
    pub id: Uuid,

    /// Name of the pet
    pub name: String,

    /// Type of animal (e.g., "Dog", "Cat")
    pub pet_type: String,

    /// Optional breed specification
    pub breed: Option<String>,

    /// Optional age of the pet
    pub age: Option<i32>,

    /// When the pet was created in the system
    pub created_at: DateTime<Utc>,

    /// When the pet was last updated in the system
    pub updated_at: DateTime<Utc>,
}

/// Interface for pet data access
#[async_trait]
pub trait PetRepository: Send + Sync + 'static {
    /// Find all pets
    async fn find_all(&self) -> Result<Vec<Pet>, AppError>;

    /// Find a pet by ID
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

    /// Delete a pet by ID
    async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
}

/// Postgres implementation of PetRepository
pub struct PgPetRepository {
    pool: Pool<Postgres>,
}

impl PgPetRepository {
    /// Create a new PgPetRepository with the given database pool
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PetRepository for PgPetRepository {
    async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
        let pets = sqlx::query_as!(
            Pet,
            r#"
            SELECT id, name, pet_type, breed, age, created_at, updated_at 
            FROM pets 
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(pets)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError> {
        let pet = sqlx::query_as!(
            Pet,
            r#"
            SELECT id, name, pet_type, breed, age, created_at, updated_at 
            FROM pets 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(pet)
    }

    async fn create(
        &self,
        name: String,
        pet_type: String,
        breed: Option<String>,
        age: Option<i32>,
    ) -> Result<Pet, AppError> {
        let now = Utc::now();

        let pet = sqlx::query_as!(
            Pet,
            r#"
            INSERT INTO pets (id, name, pet_type, breed, age, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            Uuid::new_v4(),
            name,
            pet_type,
            breed,
            age,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

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
        let now = Utc::now();

        let pet = sqlx::query_as!(
            Pet,
            r#"
            UPDATE pets
            SET name = $1, pet_type = $2, breed = $3, age = $4, updated_at = $5
            WHERE id = $6
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            name,
            pet_type,
            breed,
            age,
            now,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

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
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}
