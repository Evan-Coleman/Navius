//! # User Controller Example
//!
//! This example demonstrates how to use the Spring Boot-like annotation macros
//! to create a REST controller.

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use crate::core::utils::api_resource;
use crate::core::{error::AppError, router::core_app_router::AppState};

/// Example user model
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

impl api_resource::ApiResource for User {
    type Id = Uuid;

    fn resource_type() -> &'static str {
        "user"
    }

    fn api_name() -> &'static str {
        "UserService"
    }
}

/// Create user request DTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

/// Update user request DTO
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// User service trait
pub trait UserService: Send + Sync {
    fn find_all(&self) -> Vec<User>;
    fn find_by_id(&self, id: Uuid) -> Option<User>;
    fn create(&self, request: CreateUserRequest) -> User;
    fn update(&self, id: Uuid, request: UpdateUserRequest) -> Option<User>;
    fn delete(&self, id: Uuid) -> bool;
}

/// In-memory implementation of UserService
pub struct InMemoryUserService {
    users: Mutex<HashMap<Uuid, User>>,
}

impl InMemoryUserService {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
        }
    }
}

impl UserService for InMemoryUserService {
    fn find_all(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values().cloned().collect()
    }

    fn find_by_id(&self, id: Uuid) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.get(&id).cloned()
    }

    fn create(&self, request: CreateUserRequest) -> User {
        let user = User {
            id: Uuid::new_v4(),
            name: request.name,
            email: request.email,
        };

        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        user
    }

    fn update(&self, id: Uuid, request: UpdateUserRequest) -> Option<User> {
        let mut users = self.users.lock().unwrap();

        if let Some(user) = users.get_mut(&id) {
            if let Some(name) = request.name {
                user.name = name;
            }

            if let Some(email) = request.email {
                user.email = email;
            }

            return Some(user.clone());
        }

        None
    }

    fn delete(&self, id: Uuid) -> bool {
        let mut users = self.users.lock().unwrap();
        users.remove(&id).is_some()
    }
}

/// User controller with Spring Boot-like annotations
pub struct UserController {
    service: Arc<dyn UserService>,
}

impl UserController {
    pub fn new(service: Arc<dyn UserService>) -> Self {
        Self { service }
    }

    /// Get all users
    async fn get_all_users(&self, _state: State<Arc<AppState>>) -> Json<Vec<User>> {
        Json(self.service.find_all())
    }

    /// Get user by ID
    async fn get_user_by_id(
        &self,
        _state: State<Arc<AppState>>,
        Path(id): Path<Uuid>,
    ) -> Result<Json<User>, AppError> {
        match self.service.find_by_id(id) {
            Some(user) => Ok(Json(user)),
            None => Err(AppError::not_found(&format!(
                "User with ID {} not found",
                id
            ))),
        }
    }

    /// Create a new user
    async fn create_user(
        &self,
        _state: State<Arc<AppState>>,
        Json(request): Json<CreateUserRequest>,
    ) -> Result<(StatusCode, Json<User>), AppError> {
        let user = self.service.create(request);
        Ok((StatusCode::CREATED, Json(user)))
    }

    /// Update an existing user
    async fn update_user(
        &self,
        _state: State<Arc<AppState>>,
        Path(id): Path<Uuid>,
        Json(request): Json<UpdateUserRequest>,
    ) -> Result<Json<User>, AppError> {
        match self.service.update(id, request) {
            Some(user) => Ok(Json(user)),
            None => Err(AppError::not_found(&format!(
                "User with ID {} not found",
                id
            ))),
        }
    }

    /// Delete a user
    async fn delete_user(
        &self,
        _state: State<Arc<AppState>>,
        Path(id): Path<Uuid>,
    ) -> Result<StatusCode, AppError> {
        if self.service.delete(id) {
            Ok(StatusCode::NO_CONTENT)
        } else {
            Err(AppError::not_found(&format!(
                "User with ID {} not found",
                id
            )))
        }
    }

    /// Register the controller's routes with the router
    /// Note: This is just an example to demonstrate the concepts.
    /// In a real application, this would need to be integrated with Axum's router.
    pub fn register_routes(&self, _router: &mut Router) {
        // Define the base path for the controller
        let base_path = RequestMapping::new("/api/users");

        // Define the endpoint mappings
        let get_all = EndpointMapping::get("/");
        let get_by_id = EndpointMapping::get("/{id}");
        let create = EndpointMapping::post("/");
        let update = EndpointMapping::put("/{id}");
        let delete = EndpointMapping::delete("/{id}");

        // In a real implementation, this would register the routes with the router
        println!(
            "Registering routes for UserController at base path: {}",
            base_path.path
        );
        println!("- GET {} -> get_all_users", get_all.path);
        println!(
            "- GET {}/{} -> get_user_by_id",
            base_path.path, get_by_id.path
        );
        println!("- POST {} -> create_user", create.path);
        println!("- PUT {}/{} -> update_user", base_path.path, update.path);
        println!("- DELETE {}/{} -> delete_user", base_path.path, delete.path);
    }
}

impl Clone for UserController {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
}

// Spring Boot-like annotations (will be implemented in a real framework)

/// Marker trait for controllers (similar to @Controller in Spring Boot)
pub trait Controller {}

/// Marker trait for REST controllers (similar to @RestController in Spring Boot)
pub trait RestController: Controller {}

/// Base path configuration (similar to @RequestMapping in Spring Boot)
pub struct RequestMapping {
    pub path: String,
}

impl RequestMapping {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

/// HTTP method-specific endpoint mapping (similar to @GetMapping, @PostMapping, etc. in Spring Boot)
pub struct EndpointMapping {
    pub method: String,
    pub path: String,
}

impl EndpointMapping {
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.to_string(),
        }
    }

    pub fn post(path: &str) -> Self {
        Self {
            method: "POST".to_string(),
            path: path.to_string(),
        }
    }

    pub fn put(path: &str) -> Self {
        Self {
            method: "PUT".to_string(),
            path: path.to_string(),
        }
    }

    pub fn delete(path: &str) -> Self {
        Self {
            method: "DELETE".to_string(),
            path: path.to_string(),
        }
    }
}

impl Controller for UserController {}
impl RestController for UserController {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::router::core_app_router::ServiceRegistry;

    #[test]
    fn test_user_controller() {
        // Arrange
        let service = Arc::new(InMemoryUserService::new());
        let controller = UserController::new(service.clone());

        // Act & Assert
        assert!(controller.service.find_all().is_empty());

        // Test create user
        let request = CreateUserRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        let user = controller.service.create(request);

        // Test find all
        let users = controller.service.find_all();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "Test User");

        // Test find by id
        let found_user = controller.service.find_by_id(user.id);
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().name, "Test User");

        // Test update
        let update_request = UpdateUserRequest {
            name: Some("Updated User".to_string()),
            email: None,
        };
        let updated_user = controller.service.update(user.id, update_request);
        assert!(updated_user.is_some());
        assert_eq!(updated_user.unwrap().name, "Updated User");

        // Test delete
        assert!(controller.service.delete(user.id));
        assert!(controller.service.find_all().is_empty());
    }
}
