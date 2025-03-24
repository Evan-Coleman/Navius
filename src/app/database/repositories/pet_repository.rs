use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Error as SqlxError, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::AppError;

/// Representation of a pet in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
    pub age: i32,

    /// When the pet was created in the system
    pub created_at: DateTime<Utc>,

    /// When the pet was last updated in the system
    pub updated_at: DateTime<Utc>,
}

impl Pet {
    pub fn new(id: Uuid, name: String, pet_type: String, breed: Option<String>, age: i32) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            pet_type,
            breed,
            age,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<SqlxError> for AppError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::RowNotFound => AppError::NotFound("Pet not found".to_string()),
            _ => AppError::DatabaseError(error.to_string()),
        }
    }
}

/// Interface for pet data access
#[async_trait]
pub trait PetRepository: Send + Sync {
    /// Find all pets
    async fn find_all(&self) -> Result<Vec<Pet>, AppError>;

    /// Find a pet by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Pet, AppError>;

    /// Create a new pet
    async fn create(&self, pet: Pet) -> Result<Pet, AppError>;

    /// Update an existing pet
    async fn update(&self, pet: Pet) -> Result<Pet, AppError>;

    /// Delete a pet by ID
    async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
}

/// Postgres implementation of PetRepository
pub struct PgPetRepository {
    pool: Arc<PgPool>,
}

impl PgPetRepository {
    /// Create a new PgPetRepository with the given database pool
    pub fn new(pool: Arc<PgPool>) -> Self {
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
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(pets)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Pet, AppError> {
        let pet = sqlx::query_as!(
            Pet,
            r#"
            SELECT id, name, pet_type, breed, age, created_at, updated_at
            FROM pets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pet with id {} not found", id)))?;

        Ok(pet)
    }

    async fn create(&self, pet: Pet) -> Result<Pet, AppError> {
        let created_pet = sqlx::query_as!(
            Pet,
            r#"
            INSERT INTO pets (id, name, pet_type, breed, age, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            pet.id,
            pet.name,
            pet.pet_type,
            pet.breed,
            pet.age,
            pet.created_at,
            pet.updated_at
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(created_pet)
    }

    async fn update(&self, pet: Pet) -> Result<Pet, AppError> {
        let updated_pet = sqlx::query_as!(
            Pet,
            r#"
            UPDATE pets
            SET name = $2, pet_type = $3, breed = $4, age = $5, updated_at = $6
            WHERE id = $1
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            pet.id,
            pet.name,
            pet.pet_type,
            pet.breed,
            pet.age,
            pet.updated_at
        )
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pet with id {} not found", pet.id)))?;

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
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
pub struct MockPetRepository {
    pets: std::sync::Mutex<Vec<Pet>>,
}

#[cfg(test)]
impl MockPetRepository {
    pub fn new() -> Self {
        Self {
            pets: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl PetRepository for MockPetRepository {
    async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
        let pets = self.pets.lock().unwrap();
        Ok(pets.iter().cloned().collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Pet, AppError> {
        let pets = self.pets.lock().unwrap();
        pets.iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Pet with id {} not found", id)))
    }

    async fn create(&self, pet: Pet) -> Result<Pet, AppError> {
        let mut pets = self.pets.lock().unwrap();
        pets.push(pet.clone());
        Ok(pet)
    }

    async fn update(&self, pet: Pet) -> Result<Pet, AppError> {
        let mut pets = self.pets.lock().unwrap();
        if let Some(pos) = pets.iter().position(|p| p.id == pet.id) {
            pets[pos] = pet;
            Ok(pet)
        } else {
            Err(AppError::NotFound(format!(
                "Pet with id {} not found",
                pet.id
            )))
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let mut pets = self.pets.lock().unwrap();
        if let Some(index) = pets.iter().position(|p| p.id == id) {
            pets.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn create_test_pool() -> PgPool {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database pool")
    }

    #[tokio::test]
    async fn test_pet_repository() {
        let pool = Arc::new(create_test_pool().await);
        let repository = PgPetRepository::new(pool);

        // Create a test pet
        let pet = Pet {
            id: Uuid::new_v4(),
            name: "TestPet".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("TestBreed".to_string()),
            age: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test create
        let created_pet = repository.create(pet.clone()).await.unwrap();
        assert_eq!(created_pet.name, pet.name);

        // Test find_by_id
        let found_pet = repository.find_by_id(created_pet.id).await.unwrap();
        assert_eq!(found_pet.id, created_pet.id);

        // Test find_all
        let all_pets = repository.find_all().await.unwrap();
        assert!(all_pets.len() >= 1);

        // Test update
        let mut updated_pet = created_pet.clone();
        updated_pet.name = "UpdatedTestPet".to_string();
        let result = repository.update(updated_pet.clone()).await.unwrap();
        assert_eq!(result.name, "UpdatedTestPet");

        // Test delete
        let deleted = repository.delete(created_pet.id).await.unwrap();
        assert!(deleted);
    }
}
