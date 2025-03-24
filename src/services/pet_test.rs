#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::database::repositories::Pet;
    use crate::core::database::PgPool;
    use crate::core::error::AppError;
    use chrono::Utc;
    use mockall::predicate::*;
    use mockall::*;
    use std::sync::Arc;
    use uuid::Uuid;

    mock! {
        PetRepository {}

        impl PetRepository {
            pub fn new(db_pool: Arc<Box<dyn PgPool>>) -> Self;

            pub async fn find_all(&self) -> Result<Vec<Pet>, AppError>;
            pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, AppError>;
            pub async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, AppError>;
            pub async fn create(&self, pet: Pet) -> Result<Pet, AppError>;
            pub async fn update(&self, pet: Pet) -> Result<Pet, AppError>;
            pub async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
        }
    }

    // Helper to create a test pet
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
        let test_pet_clone = test_pet.clone();

        mock_repo
            .expect_find_by_id()
            .with(eq(pet_id))
            .returning(move |_| Ok(Some(test_pet_clone.clone())));

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
        let test_pet_clone = test_pet.clone();

        mock_repo
            .expect_create()
            .returning(move |pet| Ok(test_pet_clone.clone()));

        let service = PetService::new(Arc::new(mock_repo));

        // Act
        let create_dto = CreatePetDto {
            name: "Fluffy".to_string(),
            species: "Cat".to_string(),
            age: 3,
        };

        let result = service.create_pet(create_dto).await;

        // Assert
        assert!(result.is_ok());
        let pet = result.unwrap();
        assert_eq!(pet.name, "Fluffy");
        assert_eq!(pet.species, "Cat");
        assert_eq!(pet.age, 3);
    }
}
