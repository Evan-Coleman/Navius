use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::{
    Category, Notification, NotificationStatus, NotificationType, Priority, Role, Task, TaskStatus,
    User, UserStatus,
    repositories::{
        CategoryRepository, NotificationRepository, RepositoryError, TaskRepository, UserRepository,
    },
};

// Task Repository Implementation
pub struct InMemoryTaskRepository {
    tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn save(&self, task: Task) -> Result<Task, RepositoryError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let task_clone = task.clone();
        tasks.insert(task.id, task);

        Ok(task_clone)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, RepositoryError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        Ok(tasks.get(&id).cloned())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        tasks.remove(&id);

        Ok(())
    }

    async fn find(
        &self,
        status: Option<TaskStatus>,
        priority: Option<Priority>,
        assigned_to: Option<Uuid>,
        created_by: Option<Uuid>,
        category_id: Option<Uuid>,
        due_date_before: Option<DateTime<Utc>>,
        due_date_after: Option<DateTime<Utc>>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<Task>, RepositoryError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let filtered_tasks = tasks
            .values()
            .filter(|task| {
                // Apply filters
                let status_match = status.as_ref().map_or(true, |s| &task.status == s);
                let priority_match = priority.as_ref().map_or(true, |p| &task.priority == p);
                let assigned_to_match = assigned_to.as_ref().map_or(true, |user_id| {
                    task.assigned_to.as_ref().map_or(false, |id| id == user_id)
                });
                let created_by_match = created_by
                    .as_ref()
                    .map_or(true, |user_id| &task.created_by == user_id);
                let category_match = category_id.as_ref().map_or(true, |cat_id| {
                    task.category_id.as_ref().map_or(false, |id| id == cat_id)
                });

                let before_match = due_date_before.as_ref().map_or(true, |before| {
                    task.due_date.as_ref().map_or(true, |date| date < before)
                });

                let after_match = due_date_after.as_ref().map_or(true, |after| {
                    task.due_date.as_ref().map_or(true, |date| date > after)
                });

                let tags_match = tags.as_ref().map_or(true, |tag_list| {
                    tag_list.iter().all(|tag| task.tags.contains(tag))
                });

                status_match
                    && priority_match
                    && assigned_to_match
                    && created_by_match
                    && category_match
                    && before_match
                    && after_match
                    && tags_match
            })
            .cloned()
            .collect();

        Ok(filtered_tasks)
    }

    async fn find_by_comment_id(&self, comment_id: Uuid) -> Result<Option<Task>, RepositoryError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let task = tasks
            .values()
            .find(|task| task.comments.iter().any(|c| c.id == comment_id))
            .cloned();

        Ok(task)
    }
}

// User Repository Implementation
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<Uuid, User>>>,
    email_index: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            email_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn save(&self, user: User) -> Result<User, RepositoryError> {
        let mut users = self
            .users
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let mut email_index = self
            .email_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        // Update email index
        email_index.insert(user.email.clone(), user.id);

        let user_clone = user.clone();
        users.insert(user.id, user);

        Ok(user_clone)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let users = self
            .users
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: String) -> Result<Option<User>, RepositoryError> {
        let email_index = self
            .email_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let user_id = email_index.get(&email).cloned();

        match user_id {
            Some(id) => self.find_by_id(id).await,
            None => Ok(None),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let mut users = self
            .users
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let mut email_index = self
            .email_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        // Remove from email index if exists
        if let Some(user) = users.get(&id) {
            email_index.remove(&user.email);
        }

        users.remove(&id);

        Ok(())
    }

    async fn find(
        &self,
        role: Option<Role>,
        status: Option<UserStatus>,
        email_contains: Option<String>,
        username_contains: Option<String>,
        created_after: Option<DateTime<Utc>>,
        created_before: Option<DateTime<Utc>>,
    ) -> Result<Vec<User>, RepositoryError> {
        let users = self
            .users
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let filtered_users = users
            .values()
            .filter(|user| {
                // Apply filters
                let role_match = role.as_ref().map_or(true, |r| &user.role == r);
                let status_match = status.as_ref().map_or(true, |s| &user.status == s);

                let email_match = email_contains.as_ref().map_or(true, |substring| {
                    user.email
                        .to_lowercase()
                        .contains(&substring.to_lowercase())
                });

                let username_match = username_contains.as_ref().map_or(true, |substring| {
                    user.username
                        .to_lowercase()
                        .contains(&substring.to_lowercase())
                });

                let after_match = created_after
                    .as_ref()
                    .map_or(true, |after| user.created_at > *after);

                let before_match = created_before
                    .as_ref()
                    .map_or(true, |before| user.created_at < *before);

                role_match
                    && status_match
                    && email_match
                    && username_match
                    && after_match
                    && before_match
            })
            .cloned()
            .collect();

        Ok(filtered_users)
    }
}

// Notification Repository Implementation
pub struct InMemoryNotificationRepository {
    notifications: Arc<Mutex<HashMap<Uuid, Notification>>>,
    user_index: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
}

impl InMemoryNotificationRepository {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(HashMap::new())),
            user_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl NotificationRepository for InMemoryNotificationRepository {
    async fn save(&self, notification: Notification) -> Result<Notification, RepositoryError> {
        let mut notifications = self
            .notifications
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let mut user_index = self
            .user_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        // Update user index
        let user_notifications = user_index
            .entry(notification.user_id)
            .or_insert_with(Vec::new);
        if !user_notifications.contains(&notification.id) {
            user_notifications.push(notification.id);
        }

        let notification_clone = notification.clone();
        notifications.insert(notification.id, notification);

        Ok(notification_clone)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Notification>, RepositoryError> {
        let notifications = self
            .notifications
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        Ok(notifications.get(&id).cloned())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let mut notifications = self
            .notifications
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let mut user_index = self
            .user_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        // Remove from user index if exists
        if let Some(notification) = notifications.get(&id) {
            if let Some(user_notifications) = user_index.get_mut(&notification.user_id) {
                if let Some(pos) = user_notifications.iter().position(|&x| x == id) {
                    user_notifications.remove(pos);
                }
            }
        }

        notifications.remove(&id);

        Ok(())
    }

    async fn find_by_user_id(
        &self,
        user_id: Uuid,
        status: Option<NotificationStatus>,
        notification_type: Option<NotificationType>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        related_entity_id: Option<Uuid>,
    ) -> Result<Vec<Notification>, RepositoryError> {
        let notifications = self
            .notifications
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let user_index = self
            .user_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let user_notification_ids = user_index.get(&user_id).cloned().unwrap_or_default();

        let filtered_notifications = user_notification_ids
            .iter()
            .filter_map(|id| notifications.get(id))
            .filter(|notification| {
                // Apply filters
                let status_match = status.as_ref().map_or(true, |s| &notification.status == s);

                let type_match = notification_type
                    .as_ref()
                    .map_or(true, |t| &notification.notification_type == t);

                let from_match = from_date
                    .as_ref()
                    .map_or(true, |from| notification.created_at >= *from);

                let to_match = to_date
                    .as_ref()
                    .map_or(true, |to| notification.created_at <= *to);

                let entity_match = related_entity_id.as_ref().map_or(true, |entity_id| {
                    notification
                        .related_entity_id
                        .as_ref()
                        .map_or(false, |id| id == entity_id)
                });

                status_match && type_match && from_match && to_match && entity_match
            })
            .cloned()
            .collect();

        Ok(filtered_notifications)
    }
}

// Category Repository Implementation
pub struct InMemoryCategoryRepository {
    categories: Arc<Mutex<HashMap<Uuid, Category>>>,
    parent_index: Arc<Mutex<HashMap<Option<Uuid>, Vec<Uuid>>>>,
}

impl InMemoryCategoryRepository {
    pub fn new() -> Self {
        Self {
            categories: Arc::new(Mutex::new(HashMap::new())),
            parent_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepository {
    async fn save(&self, category: Category) -> Result<Category, RepositoryError> {
        let mut categories = self
            .categories
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let mut parent_index = self
            .parent_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        // Update parent index
        let parent_categories = parent_index
            .entry(category.parent_id)
            .or_insert_with(Vec::new);
        if !parent_categories.contains(&category.id) {
            parent_categories.push(category.id);
        }

        // If parent ID has changed, update the index accordingly
        if let Some(existing_category) = categories.get(&category.id) {
            if existing_category.parent_id != category.parent_id {
                if let Some(old_parent_categories) =
                    parent_index.get_mut(&existing_category.parent_id)
                {
                    if let Some(pos) = old_parent_categories.iter().position(|&x| x == category.id)
                    {
                        old_parent_categories.remove(pos);
                    }
                }
            }
        }

        let category_clone = category.clone();
        categories.insert(category.id, category);

        Ok(category_clone)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Category>, RepositoryError> {
        let categories = self
            .categories
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        Ok(categories.get(&id).cloned())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let mut categories = self
            .categories
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let mut parent_index = self
            .parent_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        // Remove from parent index if exists
        if let Some(category) = categories.get(&id) {
            if let Some(parent_categories) = parent_index.get_mut(&category.parent_id) {
                if let Some(pos) = parent_categories.iter().position(|&x| x == id) {
                    parent_categories.remove(pos);
                }
            }
        }

        categories.remove(&id);

        Ok(())
    }

    async fn find_by_parent_id(
        &self,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<Category>, RepositoryError> {
        let categories = self
            .categories
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let parent_index = self
            .parent_index
            .lock()
            .map_err(|e| RepositoryError::internal(format!("Failed to acquire lock: {}", e)))?;

        let category_ids = parent_index.get(&parent_id).cloned().unwrap_or_default();

        let filtered_categories = category_ids
            .iter()
            .filter_map(|id| categories.get(id))
            .cloned()
            .collect();

        Ok(filtered_categories)
    }
}
