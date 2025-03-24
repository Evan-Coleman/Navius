use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::{database::models::Pet, error::AppError};

#[async_trait]
pub trait PetRepository: Send + Sync {
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
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct PgPetRepository {
    pool: Arc<PgPool>,
}

impl PgPetRepository {
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
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
        let id = Uuid::new_v4();

        let pet = sqlx::query_as!(
            Pet,
            r#"
            INSERT INTO pets (id, name, pet_type, breed, age, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            id,
            name,
            pet_type,
            breed,
            age,
            now,
            now
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
            SET name = $2, pet_type = $3, breed = $4, age = $5, updated_at = $6
            WHERE id = $1
            RETURNING id, name, pet_type, breed, age, created_at, updated_at
            "#,
            id,
            name,
            pet_type,
            breed,
            age,
            now
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(pet)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM pets
            WHERE id = $1
            "#,
            id
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Pet with id {} not found", id)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
            let pets = self.pets.lock().unwrap();
            Ok(pets.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError> {
            let pets = self.pets.lock().unwrap();
            Ok(pets.iter().find(|p| p.id == id).cloned())
        }

        async fn create(
            &self,
            name: String,
            pet_type: String,
            breed: Option<String>,
            age: Option<i32>,
        ) -> Result<Pet, AppError> {
            let mut pets = self.pets.lock().unwrap();
            let pet = Pet::new(name, pet_type, breed, age);
            pets.push(pet.clone());
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
            let mut pets = self.pets.lock().unwrap();
            if let Some(pet) = pets.iter_mut().find(|p| p.id == id) {
                pet.name = name;
                pet.pet_type = pet_type;
                pet.breed = breed;
                pet.age = age;
                pet.updated_at = Utc::now();
                Ok(Some(pet.clone()))
            } else {
                Ok(None)
            }
        }

        async fn delete(&self, id: Uuid) -> Result<(), AppError> {
            let mut pets = self.pets.lock().unwrap();
            if let Some(pos) = pets.iter().position(|p| p.id == id) {
                pets.remove(pos);
                Ok(())
            } else {
                Err(AppError::NotFound(format!("Pet with id {} not found", id)))
            }
        }
    }

    #[tokio::test]
    async fn test_mock_repository() {
        let repository = MockPetRepository::new(vec![]);

        // Test create
        let pet = repository
            .create(
                "Fluffy".to_string(),
                "Cat".to_string(),
                Some("Persian".to_string()),
                Some(3),
            )
            .await
            .unwrap();

        // Test find by id
        let found = repository.find_by_id(pet.id).await.unwrap().unwrap();
        assert_eq!(found.name, "Fluffy");

        // Test update
        let updated = repository
            .update(
                pet.id,
                "Fluffy Jr.".to_string(),
                "Cat".to_string(),
                Some("Persian".to_string()),
                Some(4),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Fluffy Jr.");
        assert_eq!(updated.age, Some(4));

        // Test delete
        repository.delete(pet.id).await.unwrap();
        assert!(repository.find_by_id(pet.id).await.unwrap().is_none());
    }
}
