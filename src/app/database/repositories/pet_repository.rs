use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Error as SqlxError, PgPool};
use std::sync::Arc;
use std::sync::Mutex;
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
    pub age: Option<i32>,

    /// When the pet was created in the system
    pub created_at: DateTime<Utc>,

    /// When the pet was last updated in the system
    pub updated_at: DateTime<Utc>,
}

impl Pet {
    pub fn new(
        id: Uuid,
        name: String,
        pet_type: String,
        breed: Option<String>,
        age: Option<i32>,
    ) -> Self {
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
    async fn get_pets(&self) -> Result<Vec<Pet>, AppError>;

    /// Find a pet by ID
    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, AppError>;

    /// Create a new pet
    async fn create_pet(&self, pet: Pet) -> Result<Pet, AppError>;

    /// Update an existing pet
    async fn update_pet(&self, id: Uuid, pet: Pet) -> Result<Pet, AppError>;

    /// Delete a pet by ID
    async fn delete_pet(&self, id: Uuid) -> Result<(), AppError>;
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
    async fn get_pets(&self) -> Result<Vec<Pet>, AppError> {
        // Use query! instead of query_as! to avoid type checking issues
        let rows = sqlx::query!(
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
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Map the rows to Pet structs manually
        let pets = rows
            .into_iter()
            .map(|row| Pet {
                id: row.id,
                name: row.name,
                pet_type: row.pet_type,
                breed: row.breed,
                age: row.age,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(pets)
    }

    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, AppError> {
        // Use query! instead of query_as! to avoid type checking issues
        let row = sqlx::query!(
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
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Pet with id {} not found", id)))?;

        // Map the row to a Pet struct manually
        let pet = Pet {
            id: row.id,
            name: row.name,
            pet_type: row.pet_type,
            breed: row.breed,
            age: row.age,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(pet)
    }

    async fn create_pet(&self, pet: Pet) -> Result<Pet, AppError> {
        // Use query! instead of query_as! to avoid type checking issues
        let row = sqlx::query!(
            r#"
            INSERT INTO pets (id, name, pet_type, breed, age, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id, 
                name, 
                pet_type, 
                breed, 
                age,
                created_at, 
                updated_at
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
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Map the row to a Pet struct manually
        let created_pet = Pet {
            id: row.id,
            name: row.name,
            pet_type: row.pet_type,
            breed: row.breed,
            age: row.age,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(created_pet)
    }

    async fn update_pet(&self, id: Uuid, pet: Pet) -> Result<Pet, AppError> {
        // Use query! instead of query_as! to avoid type checking issues
        let row = sqlx::query!(
            r#"
            UPDATE pets
            SET 
                name = $1,
                pet_type = $2,
                breed = $3,
                age = $4,
                updated_at = $5
            WHERE id = $6
            RETURNING 
                id, 
                name, 
                pet_type, 
                breed, 
                age,
                created_at, 
                updated_at
            "#,
            pet.name,
            pet.pet_type,
            pet.breed,
            pet.age,
            pet.updated_at,
            id
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Map the row to a Pet struct manually
        let updated_pet = Pet {
            id: row.id,
            name: row.name,
            pet_type: row.pet_type,
            breed: row.breed,
            age: row.age,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(updated_pet)
    }

    async fn delete_pet(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM pets
            WHERE id = $1
            "#,
            id
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Mutex;

    pub struct MockPetRepository {
        pets: Arc<Mutex<Vec<Pet>>>,
    }

    impl MockPetRepository {
        pub fn new(pets: Vec<Pet>) -> Self {
            Self {
                pets: Arc::new(Mutex::new(pets)),
            }
        }
    }

    #[async_trait]
    impl PetRepository for MockPetRepository {
        async fn get_pets(&self) -> Result<Vec<Pet>, AppError> {
            Ok(self.pets.lock().unwrap().clone())
        }

        async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, AppError> {
            self.pets
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("Pet with id {} not found", id)))
        }

        async fn create_pet(&self, mut pet: Pet) -> Result<Pet, AppError> {
            let mut pets = self.pets.lock().unwrap();
            pet.id = Uuid::new_v4();
            pets.push(pet.clone());
            Ok(pet)
        }

        async fn update_pet(&self, id: Uuid, pet: Pet) -> Result<Pet, AppError> {
            let mut pets = self.pets.lock().unwrap();
            if let Some(existing_pet) = pets.iter_mut().find(|p| p.id == id) {
                *existing_pet = pet;
                Ok(existing_pet.clone())
            } else {
                Err(AppError::NotFound(format!("Pet with id {} not found", id)))
            }
        }

        async fn delete_pet(&self, id: Uuid) -> Result<(), AppError> {
            let mut pets = self.pets.lock().unwrap();
            if let Some(index) = pets.iter().position(|p| p.id == id) {
                pets.remove(index);
                Ok(())
            } else {
                Err(AppError::NotFound(format!("Pet with id {} not found", id)))
            }
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
            age: Some(3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test create
        let created_pet = repository.create_pet(pet.clone()).await.unwrap();
        assert_eq!(created_pet.name, pet.name);

        // Test get_pet_by_id
        let found_pet = repository.get_pet_by_id(created_pet.id).await.unwrap();
        assert_eq!(found_pet.id, created_pet.id);

        // Test get_pets
        let all_pets = repository.get_pets().await.unwrap();
        assert!(all_pets.len() >= 1);

        // Test update_pet
        let mut updated_pet = created_pet.clone();
        updated_pet.name = "UpdatedTestPet".to_string();
        let result = repository
            .update_pet(updated_pet.id, updated_pet.clone())
            .await
            .unwrap();
        assert_eq!(result.name, "UpdatedTestPet");

        // Test delete_pet
        repository.delete_pet(created_pet.id).await.unwrap();

        // Verify it's gone
        let result = repository.get_pet_by_id(created_pet.id).await;
        assert!(result.is_err());
    }
}
