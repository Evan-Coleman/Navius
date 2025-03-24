#[cfg(test)]
mod tests {
    use crate::{
        core::{database::repositories::Pet, router::AppState},
        services::{
            error::ServiceError,
            pet::{CreatePetDto, IPetService, UpdatePetDto},
        },
    };
    use axum::{
        Json, Router,
        body::Body,
        extract::{Path, State},
        http::{Request, StatusCode},
        response::IntoResponse,
        routing::get,
    };
    use chrono::{DateTime, Utc};
    use hyper;
    use mockall::predicate::*;
    use mockall::*;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::PetResponse;
    use super::map_service_error;

    // Mock the IPetService for testing
    mock! {
        PetService {}

        #[async_trait]
        impl IPetService for PetService {
            async fn get_all_pets(&self) -> Result<Vec<Pet>, ServiceError>;
            async fn get_pet_by_id(&self, id: Uuid) -> Result<Pet, ServiceError>;
            async fn create_pet(&self, dto: CreatePetDto) -> Result<Pet, ServiceError>;
            async fn update_pet(&self, id: Uuid, dto: UpdatePetDto) -> Result<Pet, ServiceError>;
            async fn delete_pet(&self, id: Uuid) -> Result<(), ServiceError>;
        }
    }

    // Helper to create a test app state with mock service
    fn create_test_state(service: MockPetService) -> Arc<AppState> {
        let mut state = AppState::default();
        state.set_data(
            "mock_pet_service",
            Arc::new(service) as Arc<dyn IPetService>,
        );
        Arc::new(state)
    }

    // Helper to create a test pet
    fn create_test_pet() -> Pet {
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

    // Test helper to get the pet service from state
    #[allow(dead_code)]
    fn get_test_pet_service(
        state: Arc<AppState>,
    ) -> Result<Arc<dyn IPetService>, (StatusCode, String)> {
        state
            .get_data::<dyn IPetService>("mock_pet_service")
            .map(|service| service.clone())
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Pet service not initialized".to_string(),
            ))
    }

    // Basic test for getting a pet by ID
    #[tokio::test]
    async fn test_get_pet_success() {
        // Arrange
        let pet_id = Uuid::new_v4();
        let test_pet = create_test_pet();

        let mut mock_service = MockPetService::new();
        mock_service
            .expect_get_pet_by_id()
            .with(eq(pet_id))
            .returning(move |_| Ok(test_pet.clone()));

        let state = create_test_state(mock_service);

        // Override the get_pet_service function just for testing
        let app = Router::new()
            .route(
                "/petdb/:id",
                get(
                    |Path(id): Path<String>, State(s): State<Arc<AppState>>| async move {
                        // Parse UUID
                        let id = match Uuid::parse_str(&id) {
                            Ok(uuid) => uuid,
                            Err(_) => {
                                return Err((
                                    StatusCode::BAD_REQUEST,
                                    "Invalid UUID format".to_string(),
                                ));
                            }
                        };

                        // Get mock pet service from state
                        let service = get_test_pet_service(s)?;

                        // Get pet by ID
                        let pet = service.get_pet_by_id(id).await.map_err(map_service_error)?;

                        // Convert to response format
                        let response = PetResponse::from(pet);

                        Ok::<_, (StatusCode, String)>(Json(response))
                    },
                ),
            )
            .with_state(state);

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/petdb/{}", pet_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
    }
}
