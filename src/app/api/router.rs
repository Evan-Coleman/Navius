use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::core::{
    database::{repositories::PetRepository, repositories::PgPetRepository},
    services::pet_service::PetService,
};

use super::handlers::pet_handler::{
    create_pet, delete_pet, get_all_pets, get_pet_by_id, update_pet,
};

pub fn pet_routes<R: PetRepository + 'static>(service: PetService<R>) -> Router {
    Router::new()
        .route("/pets", get(get_all_pets::<R>))
        .route("/pets", post(create_pet::<R>))
        .route("/pets/:id", get(get_pet_by_id::<R>))
        .route("/pets/:id", put(update_pet::<R>))
        .route("/pets/:id", delete(delete_pet::<R>))
        .with_state(service)
}

pub fn app_router(pool: Arc<PgPool>) -> Router {
    let pet_repository = Arc::new(PgPetRepository::new(pool));
    let pet_service = PetService::new(pet_repository);
    Router::new().nest("/api", pet_routes(pet_service))
}
