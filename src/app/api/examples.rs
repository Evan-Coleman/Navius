//! # Examples module
//!
//! This module provides Spring Boot-like examples to demonstrate
//! how to use the Navius framework.

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::core::{error::AppError, router::app_router::AppState};

/// Example domain model (similar to Spring Boot entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// Create example request (similar to Spring Boot DTO)
#[derive(Debug, Deserialize)]
pub struct CreateExampleRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Example repository interface (similar to Spring Data repository)
pub trait ExampleRepository: Send + Sync {
    fn find_all(&self) -> Vec<Example>;
    fn find_by_id(&self, id: &str) -> Option<Example>;
    fn save(&self, example: Example) -> Example;
    fn delete(&self, id: &str) -> bool;
}

/// In-memory implementation of ExampleRepository
pub struct InMemoryExampleRepository {
    examples: RwLock<HashMap<String, Example>>,
}

impl InMemoryExampleRepository {
    pub fn new() -> Self {
        Self {
            examples: RwLock::new(HashMap::new()),
        }
    }
}

impl ExampleRepository for InMemoryExampleRepository {
    fn find_all(&self) -> Vec<Example> {
        let examples = self.examples.read().unwrap();
        examples.values().cloned().collect()
    }

    fn find_by_id(&self, id: &str) -> Option<Example> {
        let examples = self.examples.read().unwrap();
        examples.get(id).cloned()
    }

    fn save(&self, example: Example) -> Example {
        let mut examples = self.examples.write().unwrap();
        let example_clone = example.clone();
        examples.insert(example.id.clone(), example);
        example_clone
    }

    fn delete(&self, id: &str) -> bool {
        let mut examples = self.examples.write().unwrap();
        examples.remove(id).is_some()
    }
}

/// Example service (similar to Spring Boot service)
pub struct ExampleService {
    repository: Arc<dyn ExampleRepository>,
}

impl ExampleService {
    pub fn new(repository: Arc<dyn ExampleRepository>) -> Self {
        Self { repository }
    }

    pub fn find_all(&self) -> Vec<Example> {
        self.repository.find_all()
    }

    pub fn find_by_id(&self, id: &str) -> Option<Example> {
        self.repository.find_by_id(id)
    }

    pub fn create(&self, request: CreateExampleRequest) -> Example {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        let example = Example {
            id,
            name: request.name,
            description: request.description,
            created_at,
        };

        self.repository.save(example)
    }

    pub fn delete(&self, id: &str) -> bool {
        self.repository.delete(id)
    }
}

/// Example controller (similar to Spring Boot @RestController)
pub struct ExampleController {
    service: Arc<ExampleService>,
}

impl ExampleController {
    pub fn new(service: Arc<ExampleService>) -> Self {
        Self { service }
    }

    /// Get all examples
    async fn get_examples(&self) -> Result<Json<Vec<Example>>, AppError> {
        let examples = self.service.find_all();
        Ok(Json(examples))
    }

    /// Get example by ID
    async fn get_example(&self, Path(id): Path<String>) -> Result<Json<Example>, AppError> {
        match self.service.find_by_id(&id) {
            Some(example) => Ok(Json(example)),
            None => Err(AppError::not_found(&format!(
                "Example with ID {} not found",
                id
            ))),
        }
    }

    /// Create a new example
    async fn create_example(
        &self,
        Json(request): Json<CreateExampleRequest>,
    ) -> Result<(StatusCode, Json<Example>), AppError> {
        let example = self.service.create(request);
        Ok((StatusCode::CREATED, Json(example)))
    }

    /// Delete an example by ID
    async fn delete_example(&self, Path(id): Path<String>) -> Result<StatusCode, AppError> {
        if self.service.delete(&id) {
            Ok(StatusCode::NO_CONTENT)
        } else {
            Err(AppError::not_found(&format!(
                "Example with ID {} not found",
                id
            )))
        }
    }

    /// Register the controller's routes
    pub fn register_routes(
        self: Arc<Self>,
        router: Router<Arc<AppState>>,
    ) -> Router<Arc<AppState>> {
        router
            .route(
                "/examples",
                get(|state: State<Arc<AppState>>| async move {
                    let controller = state
                        .service_registry
                        .get::<Arc<ExampleController>>()
                        .expect("ExampleController not registered");
                    controller.get_examples().await
                }),
            )
            .route(
                "/examples/:id",
                get(
                    |state: State<Arc<AppState>>, path: Path<String>| async move {
                        let controller = state
                            .service_registry
                            .get::<Arc<ExampleController>>()
                            .expect("ExampleController not registered");
                        controller.get_example(path).await
                    },
                ),
            )
            .route(
                "/examples",
                post(
                    |state: State<Arc<AppState>>, json: Json<CreateExampleRequest>| async move {
                        let controller = state
                            .service_registry
                            .get::<Arc<ExampleController>>()
                            .expect("ExampleController not registered");
                        controller.create_example(json).await
                    },
                ),
            )
            .route(
                "/examples/:id",
                axum::routing::delete(
                    |state: State<Arc<AppState>>, path: Path<String>| async move {
                        let controller = state
                            .service_registry
                            .get::<Arc<ExampleController>>()
                            .expect("ExampleController not registered");
                        controller.delete_example(path).await
                    },
                ),
            )
    }
}

/// Configure routes for the examples module in a Spring Boot-like way
pub fn configure_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    // Create dependencies
    let repository = Arc::new(InMemoryExampleRepository::new()) as Arc<dyn ExampleRepository>;
    let service = Arc::new(ExampleService::new(repository));
    let controller = Arc::new(ExampleController::new(service));

    // Register routes
    controller.register_routes(router)
}

/// Register services for the examples module
pub fn register_services(
    builder: crate::core::router::app_router::RouterBuilder,
) -> crate::core::router::app_router::RouterBuilder {
    // Create dependencies
    let repository = Arc::new(InMemoryExampleRepository::new()) as Arc<dyn ExampleRepository>;
    let service = Arc::new(ExampleService::new(repository.clone()));
    let controller = Arc::new(ExampleController::new(service.clone()));

    // Register services
    builder
        .register_service(repository)
        .register_service(service)
        .register_service(controller)
}
