use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    app::api::pet_core::{CreatePetRequest, PetResponse, UpdatePetRequest},
    app::services::dto::{CreatePetDto, UpdatePetDto},
    core::{error::AppError, services::ServiceRegistry},
};

/// Get all pets
pub async fn get_pets(
    State(state): State<Arc<ServiceRegistry>>,
) -> Result<Json<Vec<PetResponse>>, AppError> {
    let pet_service = state.get_pet_service()?;
    let pets = pet_service.get_all_pets().await?;
    Ok(Json(pets.into_iter().map(PetResponse::from).collect()))
}

/// Get a pet by ID
pub async fn get_pet_by_id(
    State(state): State<Arc<ServiceRegistry>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PetResponse>, AppError> {
    let pet_service = state.get_pet_service()?;
    let pet = pet_service.get_pet_by_id(id).await?;
    Ok(Json(PetResponse::from(pet)))
}

/// Create a pet
pub async fn create_pet(
    State(state): State<Arc<ServiceRegistry>>,
    Json(pet): Json<CreatePetRequest>,
) -> Result<Json<PetResponse>, AppError> {
    let pet_service = state.get_pet_service()?;
    let dto: CreatePetDto = pet.into();
    let pet = pet_service.create_pet(dto).await?;
    Ok(Json(PetResponse::from(pet)))
}

/// Update a pet
pub async fn update_pet(
    State(state): State<Arc<ServiceRegistry>>,
    Path(id): Path<Uuid>,
    Json(pet): Json<UpdatePetRequest>,
) -> Result<Json<PetResponse>, AppError> {
    let pet_service = state.get_pet_service()?;
    let dto: UpdatePetDto = pet.into();
    let pet = pet_service.update_pet(id, dto).await?;
    Ok(Json(PetResponse::from(pet)))
}

/// Delete a pet
pub async fn delete_pet(
    State(state): State<Arc<ServiceRegistry>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pet_service = state.get_pet_service()?;
    pet_service.delete_pet(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::pet_service::{MockPetRepository, PetService};
    use crate::core::auth::EntraTokenClient;
    use crate::core::cache::CacheRegistry;
    use crate::core::config::AppConfig;
    use crate::core::metrics::init_metrics;

    fn create_test_state() -> Arc<ServiceRegistry> {
        let config = Arc::new(AppConfig::default());
        let metrics_handle = Arc::new(init_metrics());
        let cache_registry = Arc::new(CacheRegistry::new());
        let client = Arc::new(EntraTokenClient::from_config(&config));
        let mock_repository = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

        service_registry
    }

    #[tokio::test]
    async fn test_get_pets() {
        let state = create_test_state();
        let result = get_pets(State(state)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_pet_by_id() {
        let state = create_test_state();
        let id = Uuid::new_v4();
        let result = get_pet_by_id(State(state), Path(id)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_pet() {
        let state = create_test_state();
        let request = CreatePetRequest {
            name: "Fluffy".to_string(),
            pet_type: "Cat".to_string(),
            breed: Some("Persian".to_string()),
            age: Some(3),
        };

        let result = create_pet(State(state), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_pet() {
        let state = create_test_state();
        let id = Uuid::new_v4();
        let request = UpdatePetRequest {
            name: Some("Fluffy Jr.".to_string()),
            pet_type: Some("Cat".to_string()),
            breed: Some("Persian".to_string()),
            age: Some(4),
        };

        let result = update_pet(State(state), Path(id), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_pet() {
        let state = create_test_state();
        let id = Uuid::new_v4();
        let result = delete_pet(State(state), Path(id)).await;
        assert!(result.is_ok());
    }
}
