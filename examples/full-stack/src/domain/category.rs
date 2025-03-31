use chrono::{DateTime, Utc};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use validator::Validate;

/// Category represents a way to organize tasks into groups
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Category {
    pub id: Uuid,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Category name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(max = 500, message = "Description must be less than 500 characters"))]
    pub description: Option<String>,

    pub color: Option<String>,

    pub icon: Option<String>,

    pub created_by: Uuid,

    pub parent_id: Option<Uuid>,

    pub is_archived: bool,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

impl Category {
    /// Create a new category with minimal required information
    pub fn new(name: String, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            color: None,
            icon: None,
            created_by,
            parent_id: None,
            is_archived: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate the category data
    pub fn validate(&self) -> Result<()> {
        self.validate_with_message()
            .map_err(|e| Error::validation_error(&e.to_string()))?;

        // Additional validation logic
        if let Some(parent_id) = self.parent_id {
            if parent_id == self.id {
                return Err(Error::validation_error("Category cannot be its own parent"));
            }
        }

        Ok(())
    }

    /// Update name and description
    pub fn update_details(&mut self, name: String, description: Option<String>) -> Result<()> {
        if name.is_empty() || name.len() > 100 {
            return Err(Error::validation_error(
                "Category name must be between 1 and 100 characters",
            ));
        }

        if let Some(desc) = &description {
            if desc.len() > 500 {
                return Err(Error::validation_error(
                    "Description must be less than 500 characters",
                ));
            }
        }

        self.name = name;
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the visual properties of the category
    pub fn update_appearance(&mut self, color: Option<String>, icon: Option<String>) {
        self.color = color;
        self.icon = icon;
        self.updated_at = Utc::now();
    }

    /// Set or update the parent category
    pub fn set_parent(&mut self, parent_id: Option<Uuid>) -> Result<()> {
        if let Some(parent) = parent_id {
            if parent == self.id {
                return Err(Error::validation_error("Category cannot be its own parent"));
            }
        }

        self.parent_id = parent_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Archive or unarchive the category
    pub fn set_archived(&mut self, is_archived: bool) {
        self.is_archived = is_archived;
        self.updated_at = Utc::now();
    }

    /// Check if the category is a root category (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_category() {
        let user_id = Uuid::new_v4();
        let category = Category::new("Projects".to_string(), user_id);

        assert_eq!(category.name, "Projects");
        assert_eq!(category.created_by, user_id);
        assert!(!category.is_archived);
        assert!(category.is_root());
        assert!(category.description.is_none());
        assert!(category.color.is_none());
        assert!(category.icon.is_none());
    }

    #[test]
    fn test_update_category_details() {
        let user_id = Uuid::new_v4();
        let mut category = Category::new("Projects".to_string(), user_id);

        // Update details
        let result = category.update_details(
            "Work Projects".to_string(),
            Some("All work-related projects".to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(category.name, "Work Projects");
        assert_eq!(
            category.description,
            Some("All work-related projects".to_string())
        );

        // Test validation
        let result = category.update_details("".to_string(), None);
        assert!(result.is_err());

        // Test long description validation
        let long_desc = "a".repeat(501);
        let result = category.update_details("Valid Name".to_string(), Some(long_desc));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_appearance() {
        let user_id = Uuid::new_v4();
        let mut category = Category::new("Projects".to_string(), user_id);

        category.update_appearance(Some("#FF5733".to_string()), Some("folder".to_string()));

        assert_eq!(category.color, Some("#FF5733".to_string()));
        assert_eq!(category.icon, Some("folder".to_string()));
    }

    #[test]
    fn test_set_parent() {
        let user_id = Uuid::new_v4();
        let mut category = Category::new("Subtask".to_string(), user_id);
        let parent_id = Uuid::new_v4();

        // Set parent
        let result = category.set_parent(Some(parent_id));
        assert!(result.is_ok());
        assert_eq!(category.parent_id, Some(parent_id));
        assert!(!category.is_root());

        // Cannot set self as parent
        let result = category.set_parent(Some(category.id));
        assert!(result.is_err());

        // Clear parent
        let result = category.set_parent(None);
        assert!(result.is_ok());
        assert!(category.is_root());
    }

    #[test]
    fn test_archive_category() {
        let user_id = Uuid::new_v4();
        let mut category = Category::new("Projects".to_string(), user_id);

        assert!(!category.is_archived);

        category.set_archived(true);
        assert!(category.is_archived);

        category.set_archived(false);
        assert!(!category.is_archived);
    }
}
