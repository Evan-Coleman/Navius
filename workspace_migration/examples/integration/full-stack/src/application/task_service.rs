use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    Comment, Priority, Task, TaskStatus,
    events::{EventPublisher, TaskEvent, TaskEventType},
    repositories::TaskRepository,
};

#[derive(Debug, Clone)]
pub struct TaskServiceError {
    pub kind: TaskServiceErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum TaskServiceErrorKind {
    NotFound,
    ValidationError,
    PermissionDenied,
    RepositoryError,
    EventPublishError,
    InternalError,
}

impl TaskServiceError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: TaskServiceErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self {
            kind: TaskServiceErrorKind::ValidationError,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            kind: TaskServiceErrorKind::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn repository_error(message: impl Into<String>) -> Self {
        Self {
            kind: TaskServiceErrorKind::RepositoryError,
            message: message.into(),
        }
    }

    pub fn event_publish_error(message: impl Into<String>) -> Self {
        Self {
            kind: TaskServiceErrorKind::EventPublishError,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            kind: TaskServiceErrorKind::InternalError,
            message: message.into(),
        }
    }
}

pub type TaskResult<T> = Result<T, TaskServiceError>;

#[async_trait]
pub trait TaskService: Send + Sync {
    async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        priority: Priority,
        due_date: Option<DateTime<Utc>>,
        assigned_to: Option<Uuid>,
        created_by: Uuid,
        category_id: Option<Uuid>,
        tags: Vec<String>,
    ) -> TaskResult<Task>;

    async fn update_task(
        &self,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
        status: Option<TaskStatus>,
        priority: Option<Priority>,
        due_date: Option<DateTime<Utc>>,
        assigned_to: Option<Uuid>,
        category_id: Option<Uuid>,
        tags: Option<Vec<String>>,
        updated_by: Uuid,
    ) -> TaskResult<Task>;

    async fn delete_task(&self, id: Uuid, user_id: Uuid) -> TaskResult<()>;
    async fn get_task(&self, id: Uuid) -> TaskResult<Task>;
    async fn get_tasks(&self, filter: TaskFilter) -> TaskResult<Vec<Task>>;

    async fn add_comment(
        &self,
        task_id: Uuid,
        content: String,
        user_id: Uuid,
    ) -> TaskResult<Comment>;

    async fn update_comment(
        &self,
        comment_id: Uuid,
        content: String,
        user_id: Uuid,
    ) -> TaskResult<Comment>;

    async fn delete_comment(&self, comment_id: Uuid, user_id: Uuid) -> TaskResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub assigned_to: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub due_date_before: Option<DateTime<Utc>>,
    pub due_date_after: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

pub struct TaskServiceImpl {
    task_repository: Arc<dyn TaskRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl TaskServiceImpl {
    pub fn new(
        task_repository: Arc<dyn TaskRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            task_repository,
            event_publisher,
        }
    }

    async fn publish_task_event(
        &self,
        task_id: Uuid,
        event_type: TaskEventType,
        user_id: Uuid,
    ) -> Result<(), TaskServiceError> {
        let event = TaskEvent::new(task_id, event_type, user_id);
        self.event_publisher
            .publish_event(event)
            .await
            .map_err(|e| {
                TaskServiceError::event_publish_error(format!(
                    "Failed to publish task event: {}",
                    e
                ))
            })
    }
}

#[async_trait]
impl TaskService for TaskServiceImpl {
    async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        priority: Priority,
        due_date: Option<DateTime<Utc>>,
        assigned_to: Option<Uuid>,
        created_by: Uuid,
        category_id: Option<Uuid>,
        tags: Vec<String>,
    ) -> TaskResult<Task> {
        // Create a new task
        let task = Task::new(
            title,
            description,
            status,
            priority,
            due_date,
            assigned_to,
            created_by,
            category_id,
            tags,
        )
        .map_err(|e| TaskServiceError::validation_error(e))?;

        // Save to repository
        let created_task = self.task_repository.save(task).await.map_err(|e| {
            TaskServiceError::repository_error(format!("Failed to save task: {}", e))
        })?;

        // Publish task created event
        self.publish_task_event(created_task.id, TaskEventType::Created, created_by)
            .await?;

        Ok(created_task)
    }

    async fn update_task(
        &self,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
        status: Option<TaskStatus>,
        priority: Option<Priority>,
        due_date: Option<DateTime<Utc>>,
        assigned_to: Option<Uuid>,
        category_id: Option<Uuid>,
        tags: Option<Vec<String>>,
        updated_by: Uuid,
    ) -> TaskResult<Task> {
        // Get existing task
        let mut task = self.get_task(id).await?;

        // Verify user has permission to update
        if task.created_by != updated_by
            && (task.assigned_to.is_none() || task.assigned_to != Some(updated_by))
        {
            return Err(TaskServiceError::permission_denied(
                "You don't have permission to update this task",
            ));
        }

        // Update task fields
        if let Some(title) = title {
            task.update_title(title)
                .map_err(|e| TaskServiceError::validation_error(e))?;
        }

        if let Some(description) = description {
            task.update_description(Some(description));
        }

        if let Some(status) = status {
            task.update_status(status);
        }

        if let Some(priority) = priority {
            task.update_priority(priority);
        }

        if let Some(assigned_to) = assigned_to {
            task.update_assigned_to(Some(assigned_to));
        }

        if let Some(category_id) = category_id {
            task.update_category_id(Some(category_id));
        }

        if let Some(tags) = tags {
            task.update_tags(tags);
        }

        // Save updated task
        let updated_task = self.task_repository.save(task).await.map_err(|e| {
            TaskServiceError::repository_error(format!("Failed to update task: {}", e))
        })?;

        // Publish task updated event
        self.publish_task_event(updated_task.id, TaskEventType::Updated, updated_by)
            .await?;

        Ok(updated_task)
    }

    async fn delete_task(&self, id: Uuid, user_id: Uuid) -> TaskResult<()> {
        // Get existing task
        let task = self.get_task(id).await?;

        // Verify user has permission to delete
        if task.created_by != user_id {
            return Err(TaskServiceError::permission_denied(
                "You don't have permission to delete this task",
            ));
        }

        // Delete task
        self.task_repository.delete(id).await.map_err(|e| {
            TaskServiceError::repository_error(format!("Failed to delete task: {}", e))
        })?;

        // Publish task deleted event
        self.publish_task_event(id, TaskEventType::Deleted, user_id)
            .await?;

        Ok(())
    }

    async fn get_task(&self, id: Uuid) -> TaskResult<Task> {
        self.task_repository
            .find_by_id(id)
            .await
            .map_err(|e| TaskServiceError::repository_error(format!("Failed to get task: {}", e)))?
            .ok_or_else(|| TaskServiceError::not_found(format!("Task with id {} not found", id)))
    }

    async fn get_tasks(&self, filter: TaskFilter) -> TaskResult<Vec<Task>> {
        self.task_repository
            .find(
                filter.status,
                filter.priority,
                filter.assigned_to,
                filter.created_by,
                filter.category_id,
                filter.due_date_before,
                filter.due_date_after,
                filter.tags,
            )
            .await
            .map_err(|e| TaskServiceError::repository_error(format!("Failed to get tasks: {}", e)))
    }

    async fn add_comment(
        &self,
        task_id: Uuid,
        content: String,
        user_id: Uuid,
    ) -> TaskResult<Comment> {
        // Get existing task
        let mut task = self.get_task(task_id).await?;

        // Create comment
        let comment =
            Comment::new(content, user_id).map_err(|e| TaskServiceError::validation_error(e))?;

        // Add comment to task
        task.add_comment(comment.clone());

        // Save updated task
        self.task_repository.save(task).await.map_err(|e| {
            TaskServiceError::repository_error(format!("Failed to add comment: {}", e))
        })?;

        // Publish comment added event
        self.publish_task_event(task_id, TaskEventType::CommentAdded, user_id)
            .await?;

        Ok(comment)
    }

    async fn update_comment(
        &self,
        comment_id: Uuid,
        content: String,
        user_id: Uuid,
    ) -> TaskResult<Comment> {
        // Find task containing the comment
        let task = self
            .task_repository
            .find_by_comment_id(comment_id)
            .await
            .map_err(|e| {
                TaskServiceError::repository_error(format!(
                    "Failed to find task by comment id: {}",
                    e
                ))
            })?
            .ok_or_else(|| {
                TaskServiceError::not_found(format!("Comment with id {} not found", comment_id))
            })?;

        // Get comment
        let mut task_mut = task.clone();
        let comment = task
            .comments
            .iter()
            .find(|c| c.id == comment_id)
            .ok_or_else(|| {
                TaskServiceError::not_found(format!("Comment with id {} not found", comment_id))
            })?;

        // Verify user has permission
        if comment.created_by != user_id {
            return Err(TaskServiceError::permission_denied(
                "You don't have permission to update this comment",
            ));
        }

        // Update comment
        let updated_comment = task_mut
            .update_comment(comment_id, content)
            .map_err(|e| TaskServiceError::validation_error(e))?;

        // Save updated task
        self.task_repository.save(task_mut).await.map_err(|e| {
            TaskServiceError::repository_error(format!("Failed to update comment: {}", e))
        })?;

        // Publish comment updated event
        self.publish_task_event(task.id, TaskEventType::CommentUpdated, user_id)
            .await?;

        Ok(updated_comment)
    }

    async fn delete_comment(&self, comment_id: Uuid, user_id: Uuid) -> TaskResult<()> {
        // Find task containing the comment
        let task = self
            .task_repository
            .find_by_comment_id(comment_id)
            .await
            .map_err(|e| {
                TaskServiceError::repository_error(format!(
                    "Failed to find task by comment id: {}",
                    e
                ))
            })?
            .ok_or_else(|| {
                TaskServiceError::not_found(format!("Comment with id {} not found", comment_id))
            })?;

        // Get comment
        let mut task_mut = task.clone();
        let comment = task
            .comments
            .iter()
            .find(|c| c.id == comment_id)
            .ok_or_else(|| {
                TaskServiceError::not_found(format!("Comment with id {} not found", comment_id))
            })?;

        // Verify user has permission (either comment creator or task creator)
        if comment.created_by != user_id && task.created_by != user_id {
            return Err(TaskServiceError::permission_denied(
                "You don't have permission to delete this comment",
            ));
        }

        // Delete comment
        task_mut.remove_comment(comment_id);

        // Save updated task
        self.task_repository.save(task_mut).await.map_err(|e| {
            TaskServiceError::repository_error(format!("Failed to delete comment: {}", e))
        })?;

        // Publish comment deleted event
        self.publish_task_event(task.id, TaskEventType::CommentDeleted, user_id)
            .await?;

        Ok(())
    }
}
