use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    app::services::pet_service::{Pet, PetService},
    core::{error::AppError, router::AppState},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePetRequest {
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePetRequest {
    pub name: Option<String>,
    pub pet_type: Option<String>,
    pub breed: Option<String>,
    pub age: Option<i32>,
}

pub async fn get_pets(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Pet>>, AppError> {
    let pets = state.service_registry.pet_service.find_all().await?;
    Ok(Json(pets))
}

pub async fn get_pet_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Pet>, AppError> {
    let pet = state.service_registry.pet_service.find_by_id(id).await?;
    Ok(Json(pet))
}

pub async fn create_pet(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePetRequest>,
) -> Result<(StatusCode, Json<Pet>), AppError> {
    let pet = Pet {
        id: Uuid::new_v4(),
        name: request.name,
        pet_type: request.pet_type,
        breed: request.breed,
        age: request.age,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_pet = state.service_registry.pet_service.create(pet).await?;
    Ok((StatusCode::CREATED, Json(created_pet)))
}

pub async fn update_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdatePetRequest>,
) -> Result<Json<Pet>, AppError> {
    let pet = state.service_registry.pet_service.find_by_id(id).await?;

    let updated_pet = Pet {
        id: pet.id,
        name: request.name.unwrap_or(pet.name),
        pet_type: request.pet_type.unwrap_or(pet.pet_type),
        breed: request.breed.or(pet.breed),
        age: request.age.unwrap_or(pet.age),
        created_at: pet.created_at,
        updated_at: chrono::Utc::now(),
    };

    let updated_pet = state
        .service_registry
        .pet_service
        .update(updated_pet)
        .await?;
    Ok(Json(updated_pet))
}

pub async fn delete_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.service_registry.pet_service.delete(id).await?;
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

    fn create_test_state() -> Arc<AppState> {
        let config = Arc::new(AppConfig::default());
        let metrics_handle = Arc::new(init_metrics());
        let cache_registry = Arc::new(CacheRegistry::new());
        let client = Arc::new(EntraTokenClient::from_config(&config));
        let mock_repository = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

        Arc::new(AppState {
            client,
            config,
            start_time: chrono::Utc::now(),
            cache_registry,
            metrics_handle,
            service_registry,
            db_pool: None,
        })
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
            age: 3,
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
