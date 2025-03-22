//! User service
//!
//! This module provides business logic for user-related operations.

use std::sync::Arc;
use uuid::Uuid;

use super::{ServiceError, ServiceResult};
use crate::core::error::AppError;
use crate::repository::{User, UserRepository, models::UserRole};

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

/// User service for user-related business logic
pub struct UserService {
    /// User repository instance
    user_repository: Arc<UserRepository>,
}

impl UserService {
    /// Create a new user service
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        Self { user_repository }
    }

    /// Get a user by ID
    pub async fn get_user_by_id(&self, id: Uuid) -> ServiceResult<User> {
        let user = self
            .user_repository
            .find_by_id(id)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?
            .ok_or(ServiceError::UserNotFound)?;

        Ok(user)
    }

    /// Get a user by username
    pub async fn get_user_by_username(&self, username: &str) -> ServiceResult<User> {
        let user = self
            .user_repository
            .find_by_username(username)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?
            .ok_or(ServiceError::UserNotFound)?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create_user(&self, data: CreateUserDto) -> ServiceResult<User> {
        // Check if username already exists
        if let Some(_) = self
            .user_repository
            .find_by_username(&data.username)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?
        {
            return Err(ServiceError::UsernameExists);
        }

        // Check if email already exists
        if let Some(_) = self
            .user_repository
            .find_by_email(&data.email)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?
        {
            return Err(ServiceError::EmailExists);
        }

        // Validate username and email format
        self.validate_username(&data.username)?;
        self.validate_email(&data.email)?;

        // Create user entity
        let mut user = User::new(data.username, data.email);

        // Set optional fields
        if let Some(full_name) = data.full_name {
            user.full_name = Some(full_name);
        }

        if let Some(role) = data.role {
            user.role = role;
        }

        // Save user to repository
        let created_user = self
            .user_repository
            .save(user)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?;

        Ok(created_user)
    }

    /// Update a user
    pub async fn update_user(&self, id: Uuid, data: UpdateUserDto) -> ServiceResult<User> {
        // Get existing user
        let mut user = self.get_user_by_id(id).await?;

        // Update email if provided
        if let Some(email) = data.email {
            // Check if email already exists for another user
            if let Some(existing) = self
                .user_repository
                .find_by_email(&email)
                .await
                .map_err(|e| ServiceError::Repository(e.to_string()))?
            {
                if existing.id != id {
                    return Err(ServiceError::EmailExists);
                }
            }

            // Validate email format
            self.validate_email(&email)?;

            user.email = email;
        }

        // Update full name if provided
        if let Some(full_name) = data.full_name {
            user.full_name = Some(full_name);
        }

        // Update active status if provided
        if let Some(is_active) = data.is_active {
            user.is_active = is_active;
        }

        // Update role if provided
        if let Some(role) = data.role {
            user.role = role;
        }

        // Save updated user to repository
        let updated_user = self
            .user_repository
            .save(user)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?;

        Ok(updated_user)
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> ServiceResult<bool> {
        // Check if user exists
        let _ = self.get_user_by_id(id).await?;

        // Delete user from repository
        let deleted = self
            .user_repository
            .delete(id)
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?;

        Ok(deleted)
    }

    /// Get all users
    pub async fn get_all_users(&self) -> ServiceResult<Vec<User>> {
        let users = self
            .user_repository
            .find_all()
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?;

        Ok(users)
    }

    /// Count users
    pub async fn count_users(&self) -> ServiceResult<usize> {
        let count = self
            .user_repository
            .count()
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?;

        Ok(count)
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
