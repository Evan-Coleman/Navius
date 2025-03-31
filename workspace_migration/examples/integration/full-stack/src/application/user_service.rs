use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand_core::OsRng;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    Role, User, UserProfile, UserStatus,
    events::{EventPublisher, UserEvent, UserEventType},
    repositories::UserRepository,
};

#[derive(Debug, Clone)]
pub struct UserServiceError {
    pub kind: UserServiceErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum UserServiceErrorKind {
    NotFound,
    ValidationError,
    AuthenticationError,
    PermissionDenied,
    RepositoryError,
    EventPublishError,
    InternalError,
    DuplicateUser,
}

impl UserServiceError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::ValidationError,
            message: message.into(),
        }
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::AuthenticationError,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn repository_error(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::RepositoryError,
            message: message.into(),
        }
    }

    pub fn event_publish_error(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::EventPublishError,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::InternalError,
            message: message.into(),
        }
    }

    pub fn duplicate_user(message: impl Into<String>) -> Self {
        Self {
            kind: UserServiceErrorKind::DuplicateUser,
            message: message.into(),
        }
    }
}

pub type UserResult<T> = Result<T, UserServiceError>;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
        role: Option<Role>,
    ) -> UserResult<User>;

    async fn authenticate_user(&self, email: String, password: String) -> UserResult<User>;

    async fn get_user(&self, id: Uuid) -> UserResult<User>;
    async fn get_user_by_email(&self, email: String) -> UserResult<User>;
    async fn get_users(&self, filter: UserFilter) -> UserResult<Vec<User>>;

    async fn update_user(
        &self,
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
        status: Option<UserStatus>,
        role: Option<Role>,
    ) -> UserResult<User>;

    async fn update_password(
        &self,
        id: Uuid,
        current_password: String,
        new_password: String,
    ) -> UserResult<()>;

    async fn update_profile(&self, id: Uuid, profile: UserProfile) -> UserResult<User>;

    async fn delete_user(&self, id: Uuid) -> UserResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    pub role: Option<Role>,
    pub status: Option<UserStatus>,
    pub email_contains: Option<String>,
    pub username_contains: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

pub struct UserServiceImpl {
    user_repository: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UserServiceImpl {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            user_repository,
            event_publisher,
        }
    }

    async fn publish_user_event(
        &self,
        user_id: Uuid,
        event_type: UserEventType,
    ) -> Result<(), UserServiceError> {
        let event = UserEvent::new(user_id, event_type);
        self.event_publisher
            .publish_event(event)
            .await
            .map_err(|e| {
                UserServiceError::event_publish_error(format!(
                    "Failed to publish user event: {}",
                    e
                ))
            })
    }

    fn hash_password(&self, password: &str) -> Result<String, UserServiceError> {
        // Generate a random salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash the password with Argon2
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                UserServiceError::internal_error(format!("Password hashing failed: {}", e))
            })?
            .to_string();

        Ok(hash)
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, UserServiceError> {
        // Parse the stored password hash
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            UserServiceError::internal_error(format!("Failed to parse password hash: {}", e))
        })?;

        // Verify the password
        let argon2 = Argon2::default();
        let result = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        Ok(result)
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
        role: Option<Role>,
    ) -> UserResult<User> {
        // Check if user with email already exists
        if let Ok(_) = self.get_user_by_email(email.clone()).await {
            return Err(UserServiceError::duplicate_user(format!(
                "User with email {} already exists",
                email
            )));
        }

        // Hash the password
        let password_hash = self.hash_password(&password)?;

        // Create a new user
        let user = User::new(username, email, password_hash, role.unwrap_or(Role::User))
            .map_err(|e| UserServiceError::validation_error(e))?;

        // Save to repository
        let created_user = self.user_repository.save(user).await.map_err(|e| {
            UserServiceError::repository_error(format!("Failed to save user: {}", e))
        })?;

        // Publish user created event
        self.publish_user_event(created_user.id, UserEventType::Registered)
            .await?;

        Ok(created_user)
    }

    async fn authenticate_user(&self, email: String, password: String) -> UserResult<User> {
        // Find user by email
        let user = self.get_user_by_email(email).await?;

        // Verify password
        let is_valid = self.verify_password(&password, &user.password_hash)?;

        if !is_valid {
            return Err(UserServiceError::authentication_error(
                "Invalid credentials",
            ));
        }

        // Check if user is active
        if user.status != UserStatus::Active {
            return Err(UserServiceError::authentication_error(
                "User account is not active",
            ));
        }

        // Publish login event
        self.publish_user_event(user.id, UserEventType::LoggedIn)
            .await?;

        Ok(user)
    }

    async fn get_user(&self, id: Uuid) -> UserResult<User> {
        self.user_repository
            .find_by_id(id)
            .await
            .map_err(|e| UserServiceError::repository_error(format!("Failed to get user: {}", e)))?
            .ok_or_else(|| UserServiceError::not_found(format!("User with id {} not found", id)))
    }

    async fn get_user_by_email(&self, email: String) -> UserResult<User> {
        self.user_repository
            .find_by_email(email.clone())
            .await
            .map_err(|e| {
                UserServiceError::repository_error(format!("Failed to get user by email: {}", e))
            })?
            .ok_or_else(|| {
                UserServiceError::not_found(format!("User with email {} not found", email))
            })
    }

    async fn get_users(&self, filter: UserFilter) -> UserResult<Vec<User>> {
        self.user_repository
            .find(
                filter.role,
                filter.status,
                filter.email_contains,
                filter.username_contains,
                filter.created_after,
                filter.created_before,
            )
            .await
            .map_err(|e| UserServiceError::repository_error(format!("Failed to get users: {}", e)))
    }

    async fn update_user(
        &self,
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
        status: Option<UserStatus>,
        role: Option<Role>,
    ) -> UserResult<User> {
        // Get existing user
        let mut user = self.get_user(id).await?;

        // Update user fields
        if let Some(username) = username {
            user.update_username(username)
                .map_err(|e| UserServiceError::validation_error(e))?;
        }

        if let Some(email) = email {
            // Check if email is already in use by another user
            if let Ok(found_user) = self.get_user_by_email(email.clone()).await {
                if found_user.id != id {
                    return Err(UserServiceError::duplicate_user(format!(
                        "Email {} is already in use",
                        email
                    )));
                }
            }

            user.update_email(email)
                .map_err(|e| UserServiceError::validation_error(e))?;
        }

        if let Some(status) = status {
            user.update_status(status);
        }

        if let Some(role) = role {
            user.update_role(role);
        }

        // Save updated user
        let updated_user = self.user_repository.save(user).await.map_err(|e| {
            UserServiceError::repository_error(format!("Failed to update user: {}", e))
        })?;

        // Publish user updated event
        self.publish_user_event(updated_user.id, UserEventType::Updated)
            .await?;

        Ok(updated_user)
    }

    async fn update_password(
        &self,
        id: Uuid,
        current_password: String,
        new_password: String,
    ) -> UserResult<()> {
        // Get existing user
        let mut user = self.get_user(id).await?;

        // Verify current password
        let is_valid = self.verify_password(&current_password, &user.password_hash)?;

        if !is_valid {
            return Err(UserServiceError::authentication_error(
                "Current password is incorrect",
            ));
        }

        // Hash the new password
        let new_password_hash = self.hash_password(&new_password)?;

        // Update password
        user.update_password_hash(new_password_hash);

        // Save updated user
        self.user_repository.save(user).await.map_err(|e| {
            UserServiceError::repository_error(format!("Failed to update password: {}", e))
        })?;

        // Publish password changed event
        self.publish_user_event(id, UserEventType::PasswordChanged)
            .await?;

        Ok(())
    }

    async fn update_profile(&self, id: Uuid, profile: UserProfile) -> UserResult<User> {
        // Get existing user
        let mut user = self.get_user(id).await?;

        // Update profile
        user.update_profile(profile);

        // Save updated user
        let updated_user = self.user_repository.save(user).await.map_err(|e| {
            UserServiceError::repository_error(format!("Failed to update profile: {}", e))
        })?;

        // Publish profile updated event
        self.publish_user_event(updated_user.id, UserEventType::ProfileUpdated)
            .await?;

        Ok(updated_user)
    }

    async fn delete_user(&self, id: Uuid) -> UserResult<()> {
        // Check if user exists
        self.get_user(id).await?;

        // Delete user
        self.user_repository.delete(id).await.map_err(|e| {
            UserServiceError::repository_error(format!("Failed to delete user: {}", e))
        })?;

        // Publish user deleted event
        self.publish_user_event(id, UserEventType::Deleted).await?;

        Ok(())
    }
}
