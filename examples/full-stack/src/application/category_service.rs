use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    Category,
    events::{CategoryEvent, CategoryEventType, EventPublisher},
    repositories::CategoryRepository,
};

#[derive(Debug, Clone)]
pub struct CategoryServiceError {
    pub kind: CategoryServiceErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum CategoryServiceErrorKind {
    NotFound,
    ValidationError,
    PermissionDenied,
    RepositoryError,
    EventPublishError,
    InternalError,
    DuplicateCategory,
}

impl CategoryServiceError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::ValidationError,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn repository_error(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::RepositoryError,
            message: message.into(),
        }
    }

    pub fn event_publish_error(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::EventPublishError,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::InternalError,
            message: message.into(),
        }
    }

    pub fn duplicate_category(message: impl Into<String>) -> Self {
        Self {
            kind: CategoryServiceErrorKind::DuplicateCategory,
            message: message.into(),
        }
    }
}

pub type CategoryResult<T> = Result<T, CategoryServiceError>;

#[async_trait]
pub trait CategoryService: Send + Sync {
    async fn create_category(
        &self,
        name: String,
        description: Option<String>,
        created_by: Uuid,
        parent_id: Option<Uuid>,
        color: Option<String>,
    ) -> CategoryResult<Category>;

    async fn update_category(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        parent_id: Option<Uuid>,
        color: Option<String>,
        updated_by: Uuid,
    ) -> CategoryResult<Category>;

    async fn delete_category(&self, id: Uuid, user_id: Uuid) -> CategoryResult<()>;
    async fn get_category(&self, id: Uuid) -> CategoryResult<Category>;
    async fn get_categories(&self, parent_id: Option<Uuid>) -> CategoryResult<Vec<Category>>;
    async fn get_category_hierarchy(&self) -> CategoryResult<Vec<Category>>;
}

pub struct CategoryServiceImpl {
    category_repository: Arc<dyn CategoryRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CategoryServiceImpl {
    pub fn new(
        category_repository: Arc<dyn CategoryRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            category_repository,
            event_publisher,
        }
    }

    async fn publish_category_event(
        &self,
        category_id: Uuid,
        event_type: CategoryEventType,
        user_id: Uuid,
    ) -> Result<(), CategoryServiceError> {
        let event = CategoryEvent::new(category_id, event_type, user_id);
        self.event_publisher
            .publish_event(event)
            .await
            .map_err(|e| {
                CategoryServiceError::event_publish_error(format!(
                    "Failed to publish category event: {}",
                    e
                ))
            })
    }

    async fn check_duplicate_name(
        &self,
        name: &str,
        parent_id: Option<Uuid>,
        exclude_id: Option<Uuid>,
    ) -> CategoryResult<()> {
        let categories = self
            .category_repository
            .find_by_parent_id(parent_id)
            .await
            .map_err(|e| {
                CategoryServiceError::repository_error(format!(
                    "Failed to check for duplicate name: {}",
                    e
                ))
            })?;

        let duplicate = categories.iter().any(|c| {
            c.name.to_lowercase() == name.to_lowercase() && exclude_id.map_or(true, |id| c.id != id)
        });

        if duplicate {
            return Err(CategoryServiceError::duplicate_category(format!(
                "Category with name '{}' already exists in this parent category",
                name
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl CategoryService for CategoryServiceImpl {
    async fn create_category(
        &self,
        name: String,
        description: Option<String>,
        created_by: Uuid,
        parent_id: Option<Uuid>,
        color: Option<String>,
    ) -> CategoryResult<Category> {
        // Check for duplicate name in the same parent
        self.check_duplicate_name(&name, parent_id, None).await?;

        // Check if parent exists if provided
        if let Some(parent_id) = parent_id {
            self.get_category(parent_id).await?;
        }

        // Create a new category
        let category = Category::new(name, description, created_by, parent_id, color)
            .map_err(|e| CategoryServiceError::validation_error(e))?;

        // Save to repository
        let created_category = self.category_repository.save(category).await.map_err(|e| {
            CategoryServiceError::repository_error(format!("Failed to save category: {}", e))
        })?;

        // Publish category created event
        self.publish_category_event(created_category.id, CategoryEventType::Created, created_by)
            .await?;

        Ok(created_category)
    }

    async fn update_category(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        parent_id: Option<Uuid>,
        color: Option<String>,
        updated_by: Uuid,
    ) -> CategoryResult<Category> {
        // Get existing category
        let mut category = self.get_category(id).await?;

        // Check if this would create a circular reference
        if let Some(parent_id) = parent_id {
            if parent_id == category.id {
                return Err(CategoryServiceError::validation_error(
                    "Category cannot be its own parent",
                ));
            }

            // Verify parent exists
            self.get_category(parent_id).await?;

            // Check for circular reference in the hierarchy
            let mut current_parent_id = Some(parent_id);
            while let Some(pid) = current_parent_id {
                let parent = self.get_category(pid).await?;
                if parent.id == category.id {
                    return Err(CategoryServiceError::validation_error(
                        "Circular reference detected in category hierarchy",
                    ));
                }
                current_parent_id = parent.parent_id;
            }
        }

        // Check for duplicate name if name is being updated
        if let Some(name) = &name {
            self.check_duplicate_name(name, parent_id.or(category.parent_id), Some(id))
                .await?;
        }

        // Update category fields
        if let Some(name) = name {
            category
                .update_name(name)
                .map_err(|e| CategoryServiceError::validation_error(e))?;
        }

        if let Some(description) = description {
            category.update_description(Some(description));
        }

        if let Some(parent_id) = parent_id {
            category.update_parent_id(Some(parent_id));
        }

        if let Some(color) = color {
            category
                .update_color(Some(color))
                .map_err(|e| CategoryServiceError::validation_error(e))?;
        }

        // Save updated category
        let updated_category = self.category_repository.save(category).await.map_err(|e| {
            CategoryServiceError::repository_error(format!("Failed to update category: {}", e))
        })?;

        // Publish category updated event
        self.publish_category_event(updated_category.id, CategoryEventType::Updated, updated_by)
            .await?;

        Ok(updated_category)
    }

    async fn delete_category(&self, id: Uuid, user_id: Uuid) -> CategoryResult<()> {
        // Check if category exists
        self.get_category(id).await?;

        // Check if category has children
        let children = self.get_categories(Some(id)).await?;
        if !children.is_empty() {
            return Err(CategoryServiceError::validation_error(
                "Cannot delete category with child categories",
            ));
        }

        // Delete category
        self.category_repository.delete(id).await.map_err(|e| {
            CategoryServiceError::repository_error(format!("Failed to delete category: {}", e))
        })?;

        // Publish category deleted event
        self.publish_category_event(id, CategoryEventType::Deleted, user_id)
            .await?;

        Ok(())
    }

    async fn get_category(&self, id: Uuid) -> CategoryResult<Category> {
        self.category_repository
            .find_by_id(id)
            .await
            .map_err(|e| {
                CategoryServiceError::repository_error(format!("Failed to get category: {}", e))
            })?
            .ok_or_else(|| {
                CategoryServiceError::not_found(format!("Category with id {} not found", id))
            })
    }

    async fn get_categories(&self, parent_id: Option<Uuid>) -> CategoryResult<Vec<Category>> {
        self.category_repository
            .find_by_parent_id(parent_id)
            .await
            .map_err(|e| {
                CategoryServiceError::repository_error(format!("Failed to get categories: {}", e))
            })
    }

    async fn get_category_hierarchy(&self) -> CategoryResult<Vec<Category>> {
        // Start with root-level categories (no parent)
        let root_categories = self.get_categories(None).await?;

        // Build the full hierarchy
        let mut result = Vec::new();

        for root_category in root_categories {
            result.push(root_category.clone());
            self.add_children_to_hierarchy(&mut result, root_category.id)
                .await?;
        }

        Ok(result)
    }
}

impl CategoryServiceImpl {
    async fn add_children_to_hierarchy(
        &self,
        result: &mut Vec<Category>,
        parent_id: Uuid,
    ) -> CategoryResult<()> {
        let children = self.get_categories(Some(parent_id)).await?;

        for child in children {
            result.push(child.clone());
            self.add_children_to_hierarchy(result, child.id).await?;
        }

        Ok(())
    }
}
