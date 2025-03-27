use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::app::models::example_user_entity::{User, UserRole};
use crate::app::repositories::example_user_repository::UserRepository;
use crate::core::services::error::ServiceError;
use crate::core::services::repository_service::RepositoryService;
use crate::core::services::{Lifecycle, Service};

/// Input for creating a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    /// Username for login
    pub username: String,

    /// Email address
    pub email: String,

    /// Display name
    pub display_name: String,

    /// Optional user role (defaults to User)
    pub role: Option<UserRole>,

    /// Whether the user is active (defaults to true)
    pub active: Option<bool>,
}

/// Input for updating a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserInput {
    /// Updated email address
    pub email: Option<String>,

    /// Updated display name
    pub display_name: Option<String>,

    /// Updated role
    pub role: Option<UserRole>,

    /// Updated active status
    pub active: Option<bool>,
}

/// Output representing a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOutput {
    /// User ID
    pub id: Uuid,

    /// Username
    pub username: String,

    /// Email address
    pub email: String,

    /// Display name
    pub display_name: String,

    /// User role
    pub role: UserRole,

    /// Account active status
    pub active: bool,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserOutput {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
            active: user.active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Example service for managing users
pub struct UserService {
    /// User repository
    repository: Arc<UserRepository>,
}

impl Service for UserService {}

#[async_trait]
impl Lifecycle for UserService {
    async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Initializing example user service");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Shutting down example user service");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Check if repository is accessible by counting users
        self.repository.count().await.map_err(|e| {
            Box::new(ServiceError::unavailable(format!(
                "User repository unavailable: {}",
                e
            )))
        })?;
        Ok(())
    }
}

impl UserService {
    /// Create a new user service
    pub async fn new(repository_service: &RepositoryService) -> Result<Self, ServiceError> {
        let repository = UserRepository::new(repository_service).await?;
        Ok(Self {
            repository: Arc::new(repository),
        })
    }

    /// Find a user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserOutput>, ServiceError> {
        let user = self.repository.find_by_id(&id).await?;
        Ok(user.map(UserOutput::from))
    }

    /// Find all users
    pub async fn find_all(&self) -> Result<Vec<UserOutput>, ServiceError> {
        let users = self.repository.find_all().await?;
        Ok(users.into_iter().map(UserOutput::from).collect())
    }

    /// Create a new user
    pub async fn create_user(&self, input: CreateUserInput) -> Result<UserOutput, ServiceError> {
        // Check if username is already taken
        let existing_user = self.repository.find_by_username(&input.username).await?;
        if existing_user.is_some() {
            return Err(ServiceError::conflict(format!(
                "Username '{}' is already taken",
                input.username
            )));
        }

        // Create user entity
        let mut user = User::new(input.username, input.email, input.display_name);

        // Set optional fields
        if let Some(role) = input.role {
            user = user.with_role(role);
        }
        if let Some(active) = input.active {
            user = user.with_active(active);
        }

        // Save user
        let saved_user = self.repository.save(&user).await?;

        Ok(UserOutput::from(saved_user))
    }

    /// Update a user
    pub async fn update_user(
        &self,
        id: Uuid,
        input: UpdateUserInput,
    ) -> Result<UserOutput, ServiceError> {
        // Find existing user
        let user = self
            .repository
            .find_by_id(&id)
            .await?
            .ok_or_else(|| ServiceError::not_found(format!("User with ID {} not found", id)))?;

        // Update fields
        let mut updated_user = user.clone();

        if let Some(email) = input.email {
            updated_user.email = email;
        }

        if let Some(display_name) = input.display_name {
            updated_user.display_name = display_name;
        }

        if let Some(role) = input.role {
            updated_user.role = role;
        }

        if let Some(active) = input.active {
            updated_user.active = active;
        }

        // Update timestamp
        updated_user.update_timestamps();

        // Save updated user
        let saved_user = self.repository.save(&updated_user).await?;

        Ok(UserOutput::from(saved_user))
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> Result<bool, ServiceError> {
        // Check if user exists
        if !self.repository.exists(&id).await? {
            return Err(ServiceError::not_found(format!(
                "User with ID {} not found",
                id
            )));
        }

        // Delete user
        self.repository.delete(&id).await
    }

    /// Find users by role
    pub async fn find_by_role(&self, role: UserRole) -> Result<Vec<UserOutput>, ServiceError> {
        let role_users = self.repository.find_by_role(role).await?;
        Ok(role_users.into_iter().map(UserOutput::from).collect())
    }

    /// Find active users
    pub async fn find_active_users(&self) -> Result<Vec<UserOutput>, ServiceError> {
        let active_users = self.repository.find_active_users().await?;
        Ok(active_users.into_iter().map(UserOutput::from).collect())
    }

    /// Count users
    pub async fn count_users(&self) -> Result<usize, ServiceError> {
        self.repository.count().await
    }

    /// Find user by username
    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserOutput>, ServiceError> {
        let user = self.repository.find_by_username(username).await?;
        Ok(user.map(UserOutput::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    async fn create_test_service() -> UserService {
        let repo_service = RepositoryService::new();
        UserService::new(&repo_service).await.unwrap()
    }

    #[test]
    async fn test_create_user() {
        let service = create_test_service().await;

        let input = CreateUserInput {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            role: Some(UserRole::Admin),
            active: Some(true),
        };

        let user = service.create_user(input).await.unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.role, UserRole::Admin);
        assert!(user.active);
    }

    #[test]
    async fn test_update_user() {
        let service = create_test_service().await;

        // Create a user first
        let input = CreateUserInput {
            username: "updateuser".to_string(),
            email: "update@example.com".to_string(),
            display_name: "Update User".to_string(),
            role: None,
            active: None,
        };

        let created_user = service.create_user(input).await.unwrap();

        // Update the user
        let update_input = UpdateUserInput {
            email: Some("updated@example.com".to_string()),
            display_name: Some("Updated Name".to_string()),
            role: Some(UserRole::Editor),
            active: Some(false),
        };

        let updated_user = service
            .update_user(created_user.id, update_input)
            .await
            .unwrap();

        assert_eq!(updated_user.id, created_user.id);
        assert_eq!(updated_user.username, "updateuser"); // Username shouldn't change
        assert_eq!(updated_user.email, "updated@example.com");
        assert_eq!(updated_user.display_name, "Updated Name");
        assert_eq!(updated_user.role, UserRole::Editor);
        assert!(!updated_user.active);
    }

    #[test]
    async fn test_find_by_role() {
        let service = create_test_service().await;

        // Create users with different roles
        service
            .create_user(CreateUserInput {
                username: "user1".to_string(),
                email: "user1@example.com".to_string(),
                display_name: "User One".to_string(),
                role: Some(UserRole::User),
                active: None,
            })
            .await
            .unwrap();

        service
            .create_user(CreateUserInput {
                username: "editor1".to_string(),
                email: "editor1@example.com".to_string(),
                display_name: "Editor One".to_string(),
                role: Some(UserRole::Editor),
                active: None,
            })
            .await
            .unwrap();

        // Find by role
        let editors = service.find_by_role(UserRole::Editor).await.unwrap();
        assert_eq!(editors.len(), 1);
        assert_eq!(editors[0].username, "editor1");
    }
}
