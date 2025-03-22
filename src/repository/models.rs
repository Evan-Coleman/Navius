//! Repository data models
//!
//! This module defines the data models used by repositories.
//! These models represent database entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    pub id: Uuid,

    /// Username (unique)
    pub username: String,

    /// Email address (unique)
    pub email: String,

    /// User's full name
    pub full_name: Option<String>,

    /// Whether the user is active
    pub is_active: bool,

    /// User's role (admin, user, etc.)
    pub role: UserRole,

    /// When the user was created
    pub created_at: DateTime<Utc>,

    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with default values
    pub fn new(username: String, email: String) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            username,
            email,
            full_name: None,
            is_active: true,
            role: UserRole::User,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the user's last updated timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// Administrator with full access
    Admin,

    /// Regular user with limited access
    User,

    /// Read-only user
    ReadOnly,
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::ReadOnly => "readonly".to_string(),
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}
