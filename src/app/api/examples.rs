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
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::core::{error::AppError, router::core_app_router::AppState};

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
                "/examples/{id}",
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

/// Example of a Spring Boot-like health indicator
///
/// In Spring Boot, you can create custom health indicators by extending the
/// HealthIndicator interface. This example shows how to create a similar pattern
/// in Navius.
pub trait HealthIndicator {
    /// Get the name of the health indicator
    fn name(&self) -> String;

    /// Check health and return status and details
    fn health(&self) -> (String, Option<String>);
}

/// Example database health indicator
pub struct DatabaseHealthIndicator {
    connection_string: String,
}

impl DatabaseHealthIndicator {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
        }
    }

    fn check_connection(&self) -> bool {
        // In a real implementation, this would check the database connection
        !self.connection_string.is_empty()
    }
}

impl HealthIndicator for DatabaseHealthIndicator {
    fn name(&self) -> String {
        "database".to_string()
    }

    fn health(&self) -> (String, Option<String>) {
        if self.check_connection() {
            (
                "UP".to_string(),
                Some(format!("Connected to {}", self.connection_string)),
            )
        } else {
            (
                "DOWN".to_string(),
                Some("Unable to connect to database".to_string()),
            )
        }
    }
}

/// Example API health indicator
pub struct ApiHealthIndicator {
    api_url: String,
}

impl ApiHealthIndicator {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
        }
    }

    fn check_api(&self) -> bool {
        // In a real implementation, this would check the API status
        !self.api_url.is_empty()
    }
}

impl HealthIndicator for ApiHealthIndicator {
    fn name(&self) -> String {
        "api".to_string()
    }

    fn health(&self) -> (String, Option<String>) {
        if self.check_api() {
            (
                "UP".to_string(),
                Some(format!("API available at {}", self.api_url)),
            )
        } else {
            ("DOWN".to_string(), Some("API unavailable".to_string()))
        }
    }
}

/// Custom health service that aggregates health indicators
pub struct HealthService {
    indicators: Vec<Box<dyn HealthIndicator + Send + Sync>>,
}

impl HealthService {
    pub fn new() -> Self {
        Self {
            indicators: Vec::new(),
        }
    }

    pub fn add_indicator<I: HealthIndicator + Send + Sync + 'static>(&mut self, indicator: I) {
        self.indicators.push(Box::new(indicator));
    }

    pub fn check_health(&self) -> Value {
        let mut status = "UP".to_string();
        let mut components = HashMap::new();

        for indicator in &self.indicators {
            let (indicator_status, details) = indicator.health();

            // If any component is down, the overall status is down
            if indicator_status != "UP" {
                status = "DOWN".to_string();
            }

            // Add the component status to the response
            let component_details = if let Some(details) = details {
                json!({
                    "status": indicator_status,
                    "details": {
                        "message": details
                    }
                })
            } else {
                json!({
                    "status": indicator_status
                })
            };

            components.insert(indicator.name(), component_details);
        }

        // Create a Spring Boot-style health response
        json!({
            "status": status,
            "components": components
        })
    }
}

/// Example of a custom health endpoint handler that extends the simple /health endpoint
///
/// This shows how a user can extend the simple /health endpoint with custom checks
/// in a Spring Boot-like way.
pub async fn custom_health_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    // Create a health service with custom indicators
    let mut health_service = HealthService::new();

    // Add health indicators
    health_service.add_indicator(DatabaseHealthIndicator::new(
        "postgres://localhost:5432/navius",
    ));
    health_service.add_indicator(ApiHealthIndicator::new("https://api.example.com"));

    // Get the health status
    let health = health_service.check_health();

    Json(health)
}

/// Example of how to register a custom health endpoint in your application
pub fn register_custom_health_endpoint(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.route("/health/custom", get(custom_health_handler))
}

/// Configure routes for the examples module in a Spring Boot-like way
pub fn configure_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    // Create dependencies
    let repository = Arc::new(InMemoryExampleRepository::new()) as Arc<dyn ExampleRepository>;
    let service = Arc::new(ExampleService::new(repository));
    let controller = Arc::new(ExampleController::new(service));

    // Register routes including the custom health endpoint
    let router = controller.register_routes(router);
    register_custom_health_endpoint(router)
}

/// Register services needed by the example controller in the application registry
pub fn register_services(
    builder: crate::core::router::core_app_router::RouterBuilder,
) -> crate::core::router::core_app_router::RouterBuilder {
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
