use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::core::models::Entity;
use crate::core::services::error::ServiceError;

/// Represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    /// Unique identifier for the user
    pub id: Uuid,

    /// Username for login (must be unique)
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,

    /// User's email address
    #[validate(email(message = "Email must be a valid email address"))]
    pub email: String,

    /// User's display name
    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,

    /// Whether the user account is active
    pub active: bool,

    /// User's role in the system
    pub role: UserRole,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// User roles in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    /// Standard user with basic permissions
    User,

    /// Editor with content management permissions
    Editor,

    /// Administrator with full system access
    Admin,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

impl Entity for User {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn collection_name() -> String {
        "users".to_string()
    }

    fn validate(&self) -> Result<(), ServiceError> {
        // Use the validator crate's validation
        if let Err(validation_errors) = Validate::validate(self) {
            let error_messages: Vec<String> = validation_errors
                .field_errors()
                .iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(|error| {
                        format!(
                            "{}: {}",
                            field,
                            error.message.as_ref().unwrap_or(&"Invalid".into())
                        )
                    })
                })
                .collect();

            return Err(ServiceError::validation(error_messages.join(", ")));
        }

        Ok(())
    }
}

impl User {
    /// Create a new user with default values
    pub fn new(username: String, email: String, display_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            display_name,
            active: true,
            role: UserRole::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Create a new user with specific ID (for migrations or testing)
    pub fn with_id(id: Uuid, username: String, email: String, display_name: String) -> Self {
        Self {
            id,
            username,
            email,
            display_name,
            active: true,
            role: UserRole::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Set the user's role
    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }

    /// Set the user's active status
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Update the user's timestamps
    pub fn update_timestamps(&mut self) {
        self.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.role, UserRole::User);
        assert!(user.active);
    }

    #[test]
    fn test_user_validation_success() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
        );

        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_validation_failure() {
        // Test invalid username (too short)
        let user = User::new(
            "te".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
        );
        assert!(user.validate().is_err());

        // Test invalid email
        let user = User::new(
            "testuser".to_string(),
            "invalid-email".to_string(),
            "Test User".to_string(),
        );
        assert!(user.validate().is_err());

        // Test empty display name
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "".to_string(),
        );
        assert!(user.validate().is_err());
    }

    #[test]
    fn test_with_methods() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
        )
        .with_role(UserRole::Admin)
        .with_active(false);

        assert_eq!(user.role, UserRole::Admin);
        assert!(!user.active);
    }
}
