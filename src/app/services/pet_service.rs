use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::app::database::repositories::pet_repository::{Pet, PetRepository};
use crate::app::services::ServiceError;

/// Data transfer object for creating a new pet
#[derive(Debug, Clone)]
pub struct CreatePetDto {
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

/// Data transfer object for updating an existing pet
#[derive(Debug, Clone)]
pub struct UpdatePetDto {
    pub name: Option<String>,
    pub pet_type: Option<String>,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

/// Interface for pet service operations
#[async_trait]
pub trait IPetService: Send + Sync + 'static {
    /// Get all pets
    async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError>;

    /// Get a pet by ID
    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError>;

    /// Create a new pet
    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError>;

    /// Update an existing pet
    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError>;

    /// Delete a pet by ID
    async fn delete_pet(&self, id: Uuid) -> Result<bool, ServiceError>;

    /// Find pets by species
    async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, ServiceError>;
}

/// Implementation of the pet service
pub struct PetService {
    repository: Arc<dyn PetRepository>,
}

impl PetService {
    /// Create a new PetService with the given repository
    pub fn new(repository: Arc<dyn PetRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl IPetService for PetService {
    async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError> {
        self.repository
            .find_all()
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError> {
        match self.repository.find_by_id(id).await {
            Ok(Some(pet)) => Ok(pet),
            Ok(None) => Err(ServiceError::PetNotFound),
            Err(e) => Err(ServiceError::DatabaseError(e.to_string())),
        }
    }

    async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError> {
        // Validate input
        if dto.name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Name cannot be empty".into()));
        }

        if dto.pet_type.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Pet type cannot be empty".into(),
            ));
        }

        // Create pet
        self.repository
            .create(dto.name, dto.pet_type, dto.breed, dto.age)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError> {
        // Check if pet exists
        let existing = self.get_pet_by_id(id).await?;

        // Update with new values or keep existing ones
        let name = dto.name.unwrap_or(existing.name);
        let pet_type = dto.pet_type.unwrap_or(existing.pet_type);
        let breed = dto.breed.or(existing.breed);
        let age = dto.age.or(existing.age);

        // Validate input
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Name cannot be empty".into()));
        }

        if pet_type.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Pet type cannot be empty".into(),
            ));
        }

        // Update pet
        match self.repository.update(id, name, pet_type, breed, age).await {
            Ok(Some(pet)) => Ok(pet),
            Ok(None) => Err(ServiceError::PetNotFound),
            Err(e) => Err(ServiceError::DatabaseError(e.to_string())),
        }
    }

    async fn delete_pet(&self, id: Uuid) -> Result<bool, ServiceError> {
        // Check if pet exists
        self.get_pet_by_id(id).await?;

        // Delete pet
        match self.repository.delete(id).await {
            Ok(true) => Ok(true),
            Ok(false) => Err(ServiceError::PetNotFound),
            Err(e) => Err(ServiceError::DatabaseError(e.to_string())),
        }
    }

    async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, ServiceError> {
        // In a real implementation, this would call a repository method
        // For now, we'll get all pets and filter by species/type
        let all_pets = self.get_all_pets().await?;

        let filtered = all_pets
            .into_iter()
            .filter(|pet| pet.pet_type.to_lowercase() == species.to_lowercase())
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::database::repositories::pet_repository::PetRepository;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::predicate::*;
    use mockall::*;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock PetRepository for testing
    mock! {
        PetRepository {}

        #[async_trait]
        impl PetRepository for PetRepository {
            async fn find_all(&self) -> Result<Vec<Pet>, crate::core::error::AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, crate::core::error::AppError>;
            async fn create(
                &self,
                name: String,
                pet_type: String,
                breed: Option<String>,
                age: Option<i32>,
            ) -> Result<Pet, crate::core::error::AppError>;
            async fn update(
                &self,
                id: Uuid,
                name: String,
                pet_type: String,
                breed: Option<String>,
                age: Option<i32>,
            ) -> Result<Option<Pet>, crate::core::error::AppError>;
            async fn delete(&self, id: Uuid) -> Result<bool, crate::core::error::AppError>;
        }
    }

    // Helper function to create a sample pet
    fn create_sample_pet() -> Pet {
        Pet {
            id: Uuid::new_v4(),
            name: "Fluffy".to_string(),
            pet_type: "Cat".to_string(),
            breed: Some("Persian".to_string()),
            age: Some(3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_all_pets() {
        // Arrange
        let mut repo = MockPetRepository::new();
        let pets = vec![create_sample_pet(), create_sample_pet()];

        repo.expect_find_all()
            .times(1)
            .returning(move || Ok(pets.clone()));

        let service = PetService::new(Arc::new(repo));

        // Act
        let result = service.get_all_pets().await;

        // Assert
        assert!(result.is_ok());
        let pets = result.unwrap();
        assert_eq!(pets.len(), 2);
    }

    #[tokio::test]
    async fn test_get_pet_by_id() {
        // Arrange
        let mut repo = MockPetRepository::new();
        let pet = create_sample_pet();
        let id = pet.id;

        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(pet.clone())));

        let service = PetService::new(Arc::new(repo));

        // Act
        let result = service.get_pet_by_id(id).await;

        // Assert
        assert!(result.is_ok());
        let found_pet = result.unwrap();
        assert_eq!(found_pet.id, id);
    }

    #[tokio::test]
    async fn test_get_pet_by_id_not_found() {
        // Arrange
        let mut repo = MockPetRepository::new();
        let id = Uuid::new_v4();

        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(None));

        let service = PetService::new(Arc::new(repo));

        // Act
        let result = service.get_pet_by_id(id).await;

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
        let mut repo = MockPetRepository::new();
        let pet = create_sample_pet();

        repo.expect_create()
            .with(
                eq("Fluffy".to_string()),
                eq("Dog".to_string()),
                eq(Some("Labrador".to_string())),
                eq(Some(2)),
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(pet.clone()));

        let service = PetService::new(Arc::new(repo));

        let dto = CreatePetDto {
            name: "Fluffy".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Labrador".to_string()),
            age: Some(2),
        };

        // Act
        let result = service.create_pet(dto).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_pet() {
        // Arrange
        let mut repo = MockPetRepository::new();
        let pet = create_sample_pet();
        let id = pet.id;
        let updated_pet = Pet {
            name: "Fluffy Updated".to_string(),
            ..pet.clone()
        };

        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(pet.clone())));

        repo.expect_update()
            .with(
                eq(id),
                eq("Fluffy Updated".to_string()),
                eq("Cat".to_string()),
                eq(Some("Persian".to_string())),
                eq(Some(3)),
            )
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(updated_pet.clone())));

        let service = PetService::new(Arc::new(repo));

        let dto = UpdatePetDto {
            name: Some("Fluffy Updated".to_string()),
            pet_type: None,
            breed: None,
            age: None,
        };

        // Act
        let result = service.update_pet(id, dto).await;

        // Assert
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Fluffy Updated");
    }

    #[tokio::test]
    async fn test_delete_pet() {
        // Arrange
        let mut repo = MockPetRepository::new();
        let pet = create_sample_pet();
        let id = pet.id;

        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(pet.clone())));

        repo.expect_delete()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(true));

        let service = PetService::new(Arc::new(repo));

        // Act
        let result = service.delete_pet(id).await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_find_by_species() {
        // Arrange
        let mut repo = MockPetRepository::new();
        let cat1 = Pet {
            pet_type: "Cat".to_string(),
            ..create_sample_pet()
        };
        let cat2 = Pet {
            pet_type: "Cat".to_string(),
            ..create_sample_pet()
        };
        let dog = Pet {
            pet_type: "Dog".to_string(),
            ..create_sample_pet()
        };

        let pets = vec![cat1, cat2, dog];

        repo.expect_find_all()
            .times(1)
            .returning(move || Ok(pets.clone()));

        let service = PetService::new(Arc::new(repo));

        // Act
        let result = service.find_by_species("cat").await;

        // Assert
        assert!(result.is_ok());
        let cats = result.unwrap();
        assert_eq!(cats.len(), 2);
        assert!(cats.iter().all(|p| p.pet_type == "Cat"));
    }
}
