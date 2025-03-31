use async_trait::async_trait;
use chrono::{DateTime, Utc};
use navius_core::error::Result;
use uuid::Uuid;

use crate::domain::{
    Category, Notification, NotificationType, Priority, Role, Task, TaskStatus, User, UserStatus,
};

/// Repository interface for User entity
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    /// Find a user by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;

    /// Find a user by username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Find a user by email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Find all users with optional filtering
    async fn find_all(
        &self,
        role: Option<Role>,
        status: Option<UserStatus>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<User>>;

    /// Create a new user
    async fn create(&self, user: &User) -> Result<User>;

    /// Update an existing user
    async fn update(&self, user: &User) -> Result<User>;

    /// Delete a user
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Count users with optional filtering
    async fn count(&self, role: Option<Role>, status: Option<UserStatus>) -> Result<usize>;

    /// Find users with a specific role
    async fn find_by_role(&self, role: Role) -> Result<Vec<User>>;
}

/// Repository interface for Task entity
#[async_trait]
pub trait TaskRepository: Send + Sync + 'static {
    /// Find a task by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>>;

    /// Find all tasks with optional filtering
    async fn find_all(
        &self,
        status: Option<TaskStatus>,
        priority: Option<Priority>,
        assigned_to: Option<Uuid>,
        created_by: Option<Uuid>,
        category_id: Option<Uuid>,
        tag_ids: Option<Vec<Uuid>>,
        due_before: Option<DateTime<Utc>>,
        due_after: Option<DateTime<Utc>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Task>>;

    /// Create a new task
    async fn create(&self, task: &Task) -> Result<Task>;

    /// Update an existing task
    async fn update(&self, task: &Task) -> Result<Task>;

    /// Delete a task
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Count tasks with optional filtering
    async fn count(
        &self,
        status: Option<TaskStatus>,
        priority: Option<Priority>,
        assigned_to: Option<Uuid>,
        created_by: Option<Uuid>,
        category_id: Option<Uuid>,
    ) -> Result<usize>;

    /// Find tasks by status
    async fn find_by_status(&self, status: TaskStatus) -> Result<Vec<Task>>;

    /// Find tasks by assignee
    async fn find_by_assignee(&self, user_id: Uuid) -> Result<Vec<Task>>;

    /// Find tasks by creator
    async fn find_by_creator(&self, user_id: Uuid) -> Result<Vec<Task>>;

    /// Find tasks by category
    async fn find_by_category(&self, category_id: Uuid) -> Result<Vec<Task>>;

    /// Find tasks by tag
    async fn find_by_tag(&self, tag_id: Uuid) -> Result<Vec<Task>>;

    /// Find overdue tasks
    async fn find_overdue(&self) -> Result<Vec<Task>>;

    /// Find tasks due soon (within the specified hours)
    async fn find_due_soon(&self, hours: i64) -> Result<Vec<Task>>;
}

/// Repository interface for Category entity
#[async_trait]
pub trait CategoryRepository: Send + Sync + 'static {
    /// Find a category by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Category>>;

    /// Find all categories with optional filtering
    async fn find_all(
        &self,
        parent_id: Option<Uuid>,
        created_by: Option<Uuid>,
        include_archived: bool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Category>>;

    /// Create a new category
    async fn create(&self, category: &Category) -> Result<Category>;

    /// Update an existing category
    async fn update(&self, category: &Category) -> Result<Category>;

    /// Delete a category
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Find root categories (those without a parent)
    async fn find_roots(&self, include_archived: bool) -> Result<Vec<Category>>;

    /// Find child categories of a parent
    async fn find_children(&self, parent_id: Uuid, include_archived: bool)
    -> Result<Vec<Category>>;

    /// Count categories with optional filtering
    async fn count(
        &self,
        parent_id: Option<Uuid>,
        created_by: Option<Uuid>,
        include_archived: bool,
    ) -> Result<usize>;
}

/// Repository interface for Notification entity
#[async_trait]
pub trait NotificationRepository: Send + Sync + 'static {
    /// Find a notification by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Notification>>;

    /// Find all notifications for a user with optional filtering
    async fn find_by_user(
        &self,
        user_id: Uuid,
        notification_type: Option<NotificationType>,
        read: Option<bool>,
        dismissed: Option<bool>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Notification>>;

    /// Create a new notification
    async fn create(&self, notification: &Notification) -> Result<Notification>;

    /// Update an existing notification
    async fn update(&self, notification: &Notification) -> Result<Notification>;

    /// Delete a notification
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Mark notifications as read for a user
    async fn mark_as_read(&self, user_id: Uuid, notification_ids: Vec<Uuid>) -> Result<usize>;

    /// Mark notifications as dismissed for a user
    async fn dismiss(&self, user_id: Uuid, notification_ids: Vec<Uuid>) -> Result<usize>;

    /// Count unread notifications for a user
    async fn count_unread(&self, user_id: Uuid) -> Result<usize>;

    /// Find expired notifications
    async fn find_expired(&self) -> Result<Vec<Notification>>;

    /// Delete expired notifications
    async fn delete_expired(&self) -> Result<usize>;
}

/// Repository interface for Comment entity
#[async_trait]
pub trait CommentRepository: Send + Sync + 'static {
    /// Find a comment by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<crate::domain::Comment>>;

    /// Find all comments for a task
    async fn find_by_task(
        &self,
        task_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<crate::domain::Comment>>;

    /// Create a new comment
    async fn create(&self, comment: &crate::domain::Comment) -> Result<crate::domain::Comment>;

    /// Update an existing comment
    async fn update(&self, comment: &crate::domain::Comment) -> Result<crate::domain::Comment>;

    /// Delete a comment
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Count comments for a task
    async fn count_by_task(&self, task_id: Uuid) -> Result<usize>;
}

/// Repository interface for Tag entity
#[async_trait]
pub trait TagRepository: Send + Sync + 'static {
    /// Find a tag by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<crate::domain::Tag>>;

    /// Find a tag by name
    async fn find_by_name(&self, name: &str) -> Result<Option<crate::domain::Tag>>;

    /// Find all tags
    async fn find_all(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<crate::domain::Tag>>;

    /// Create a new tag
    async fn create(&self, tag: &crate::domain::Tag) -> Result<crate::domain::Tag>;

    /// Update an existing tag
    async fn update(&self, tag: &crate::domain::Tag) -> Result<crate::domain::Tag>;

    /// Delete a tag
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Find all tags for a task
    async fn find_by_task(&self, task_id: Uuid) -> Result<Vec<crate::domain::Tag>>;
}
