//! User service
//!
//! This module provides business logic for user-related operations.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    core::{
        error::AppError,
        repository::{Repository, models::UserRole},
    },
    core_service_error::{ServiceError, ServiceResult},
    models::User,
};

/// User creation data transfer object
#[derive(Debug, Clone)]
pub struct CreateUserDto {
    /// Username
    pub username: String,

    /// Email
    pub email: String,

    /// Full name (optional)
    pub full_name: Option<String>,

    /// Role (defaults to User if not specified)
    pub role: Option<UserRole>,
}

/// User update data transfer object
#[derive(Debug, Clone)]
pub struct UpdateUserDto {
    /// Email (optional)
    pub email: Option<String>,

    /// Full name (optional)
    pub full_name: Option<String>,

    /// Whether the user is active (optional)
    pub is_active: Option<bool>,

    /// Role (optional)
    pub role: Option<UserRole>,
}

#[async_trait]
pub trait IUserService: Send + Sync {
    async fn get_all_users(&self) -> Result<Vec<User>, ServiceError>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<User, ServiceError>;
    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError>;
    async fn find_by_email(&self, email: &str) -> Result<User, ServiceError>;
    async fn create_user(&self, user: CreateUserDto) -> Result<User, ServiceError>;
    async fn update_user(&self, id: Uuid, user: UpdateUserDto) -> Result<User, ServiceError>;
    async fn delete_user(&self, id: Uuid) -> Result<(), ServiceError>;
}

pub struct UserService<R>
where
    R: Repository<User, Uuid> + Send + Sync + 'static,
{
    repository: Arc<R>,
}

impl<R> UserService<R>
where
    R: Repository<User, Uuid> + Send + Sync + 'static,
{
    /// Create a new user service
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Get all users
    pub async fn get_all_users(&self) -> ServiceResult<Vec<User>> {
        self.repository.find_all().await.map_err(ServiceError::from)
    }

    /// Get a user by ID
    pub async fn get_user_by_id(&self, id: Uuid) -> ServiceResult<User> {
        match self.repository.find_by_id(id).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(ServiceError::UserNotFound),
            Err(e) => Err(ServiceError::from(e)),
        }
    }

    /// Get a user by username
    pub async fn get_user_by_username(&self, username: &str) -> ServiceResult<User> {
        // Since we don't have a direct find_by_username method in the Repository trait,
        // we need to find all users and filter
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        users
            .into_iter()
            .find(|u| u.username == username)
            .ok_or(ServiceError::UserNotFound)
    }

    /// Find a user by email
    pub async fn find_by_email(&self, email: &str) -> ServiceResult<User> {
        // Since we don't have a direct find_by_email method in the Repository trait,
        // we need to find all users and filter
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        users
            .into_iter()
            .find(|u| u.email == email)
            .ok_or(ServiceError::UserNotFound)
    }

    /// Get the number of users in the system
    pub async fn count_users(&self) -> ServiceResult<usize> {
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        Ok(users.len())
    }

    /// Create a new user
    pub async fn create_user(&self, user: CreateUserDto) -> ServiceResult<User> {
        // Validate username and email format
        self.validate_username(&user.username)?;
        self.validate_email(&user.email)?;

        // Check if username exists
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        if users.iter().any(|u| u.username == user.username) {
            return Err(ServiceError::UsernameExists);
        }

        // Check if email exists
        if users.iter().any(|u| u.email == user.email) {
            return Err(ServiceError::EmailExists);
        }

        // Create new user
        let user = User::new(
            user.username,
            user.email,
            user.full_name,
            user.role.unwrap_or(UserRole::User),
        );

        self.repository.save(user).await.map_err(ServiceError::from)
    }

    /// Update an existing user
    pub async fn update_user(&self, id: Uuid, user: UpdateUserDto) -> ServiceResult<User> {
        let mut existing = match self.repository.find_by_id(id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(ServiceError::UserNotFound),
            Err(e) => return Err(ServiceError::from(e)),
        };

        if let Some(email) = &user.email {
            let users = self
                .repository
                .find_all()
                .await
                .map_err(ServiceError::from)?;
            if users.iter().any(|u| u.email == *email && u.id != id) {
                return Err(ServiceError::EmailExists);
            }
            existing.email = email.clone();
        }

        if let Some(full_name) = user.full_name {
            existing.full_name = Some(full_name);
        }

        if let Some(is_active) = user.is_active {
            existing.is_active = is_active;
        }

        if let Some(role) = user.role {
            existing.role = role;
        }

        existing.updated_at = chrono::Utc::now();

        self.repository
            .save(existing)
            .await
            .map_err(ServiceError::from)
    }

    /// Delete a user by ID
    pub async fn delete_user(&self, id: Uuid) -> ServiceResult<()> {
        match self.repository.find_by_id(id).await {
            Ok(Some(_)) => {
                self.repository
                    .delete(id)
                    .await
                    .map_err(ServiceError::from)?;
                Ok(())
            }
            Ok(None) => Err(ServiceError::UserNotFound),
            Err(e) => Err(ServiceError::from(e)),
        }
    }

    // Helper methods

    /// Validate username format
    fn validate_username(&self, username: &str) -> ServiceResult<()> {
        // Username must be at least 3 characters
        if username.len() < 3 {
            return Err(ServiceError::Validation(
                "Username must be at least 3 characters".into(),
            ));
        }

        // Username must be alphanumeric or underscore
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ServiceError::Validation(
                "Username must contain only alphanumeric characters or underscores".into(),
            ));
        }

        Ok(())
    }

    /// Validate email format
    fn validate_email(&self, email: &str) -> ServiceResult<()> {
        // Simple email validation - in a real app, would use a proper email validation library
        if !email.contains('@') || !email.contains('.') {
            return Err(ServiceError::Validation("Invalid email format".into()));
        }

        Ok(())
    }
}

#[async_trait]
impl<R> IUserService for UserService<R>
where
    R: Repository<User, Uuid> + Send + Sync + 'static,
{
    async fn get_all_users(&self) -> Result<Vec<User>, ServiceError> {
        self.repository.find_all().await.map_err(ServiceError::from)
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<User, ServiceError> {
        match self.repository.find_by_id(id).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(ServiceError::UserNotFound),
            Err(e) => Err(ServiceError::from(e)),
        }
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError> {
        // Since we don't have a direct find_by_username method in the Repository trait,
        // we need to find all users and filter
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        users
            .into_iter()
            .find(|u| u.username == username)
            .ok_or(ServiceError::UserNotFound)
    }

    async fn find_by_email(&self, email: &str) -> Result<User, ServiceError> {
        // Since we don't have a direct find_by_email method in the Repository trait,
        // we need to find all users and filter
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        users
            .into_iter()
            .find(|u| u.email == email)
            .ok_or(ServiceError::UserNotFound)
    }

    async fn create_user(&self, user: CreateUserDto) -> Result<User, ServiceError> {
        // Validate username and email format
        self.validate_username(&user.username)?;
        self.validate_email(&user.email)?;

        // Check if username exists
        let users = self
            .repository
            .find_all()
            .await
            .map_err(ServiceError::from)?;
        if users.iter().any(|u| u.username == user.username) {
            return Err(ServiceError::UsernameExists);
        }

        // Check if email exists
        if users.iter().any(|u| u.email == user.email) {
            return Err(ServiceError::EmailExists);
        }

        // Create new user
        let user = User::new(
            user.username,
            user.email,
            user.full_name,
            user.role.unwrap_or(UserRole::User),
        );

        self.repository.save(user).await.map_err(ServiceError::from)
    }

    async fn update_user(&self, id: Uuid, user: UpdateUserDto) -> Result<User, ServiceError> {
        let mut existing = match self.repository.find_by_id(id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(ServiceError::UserNotFound),
            Err(e) => return Err(ServiceError::from(e)),
        };

        if let Some(email) = &user.email {
            let users = self
                .repository
                .find_all()
                .await
                .map_err(ServiceError::from)?;
            if users.iter().any(|u| u.email == *email && u.id != id) {
                return Err(ServiceError::EmailExists);
            }
            existing.email = email.clone();
        }

        if let Some(full_name) = user.full_name {
            existing.full_name = Some(full_name);
        }

        if let Some(is_active) = user.is_active {
            existing.is_active = is_active;
        }

        if let Some(role) = user.role {
            existing.role = role;
        }

        existing.updated_at = chrono::Utc::now();

        self.repository
            .save(existing)
            .await
            .map_err(ServiceError::from)
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), ServiceError> {
        match self.repository.find_by_id(id).await {
            Ok(Some(_)) => {
                self.repository
                    .delete(id)
                    .await
                    .map_err(ServiceError::from)?;
                Ok(())
            }
            Ok(None) => Err(ServiceError::UserNotFound),
            Err(e) => Err(ServiceError::from(e)),
        }
    }
}
