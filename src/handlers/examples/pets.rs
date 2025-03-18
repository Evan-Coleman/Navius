use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use metrics::counter;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::error::AppError;
use crate::{app::AppState, models::ApiResponse};

// Import the models from the correct location
use crate::generated_apis::petstore_api::models::{Category, Tag, Upet};

/// Handler for listing pets
pub async fn list_pets(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Upet>>, StatusCode> {
    info!("Listing all pets");

    // This is a placeholder - in a real app, you would fetch from a database or API
    let pets = vec![
        Upet {
            id: 1,
            name: "Fluffy".to_string(),
            category: Some(Category {
                id: 1,
                name: "Cat".to_string(),
            }),
            tags: vec![Tag {
                id: 1,
                name: "cute".to_string(),
            }],
            status: Some("available".to_string()),
        },
        Upet {
            id: 2,
            name: "Buddy".to_string(),
            category: Some(Category {
                id: 2,
                name: "Dog".to_string(),
            }),
            tags: vec![Tag {
                id: 2,
                name: "friendly".to_string(),
            }],
            status: Some("pending".to_string()),
        },
    ];

    // For list_pets, we might not cache the whole list but update cache with individual pets
    if let Some(cache) = &state.pet_cache {
        for pet in &pets {
            let id = pet.id;
            if cache.contains_key(&id) {
                info!("Pet {} already in cache", id);
                counter!("pet_cache_hits_total").increment(1);
            } else {
                cache.insert(pet.id, pet.clone()).await;
                info!("Added pet {} to cache", id);
                counter!("cache_entries_created").increment(1);
            }
        }
        info!("💾 Updated cache with {} pets", pets.len());
    }

    Ok(Json(pets))
}

/// Handler for getting a pet by ID
pub async fn get_pet(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Upet>, StatusCode> {
    info!("Getting pet with ID: {}", id);

    // This is a placeholder - in a real app, you would fetch from a database or API
    let pet = Upet {
        id,
        name: format!("Pet {}", id),
        category: Some(Category {
            id: 1,
            name: "Unknown".to_string(),
        }),
        tags: vec![],
        status: Some("available".to_string()),
    };

    Ok(Json(pet))
}

/// Handler for creating a pet
pub async fn create_pet(
    State(_state): State<Arc<AppState>>,
    Json(pet): Json<Upet>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Creating new pet: {:?}", pet);

    // This is a placeholder - in a real app, you would save to a database
    let response = ApiResponse {
        code: Some(200),
        r#type: Some("success".to_string()),
        message: Some(format!("Pet created with ID: {}", pet.id)),
    };

    Ok(Json(response))
}

/// Handler for deleting a pet
pub async fn delete_pet(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Deleting pet with ID: {}", id);

    // This is a placeholder - in a real app, you would delete from a database
    let response = ApiResponse {
        code: Some(200),
        r#type: Some("success".to_string()),
        message: Some(format!("Pet deleted with ID: {}", id)),
    };

    Ok(Json(response))
}
