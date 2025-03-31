use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::{TaskFilter, TaskService};
use crate::domain::{Priority, TaskStatus};
use crate::infrastructure::ServiceRegistry;

/// Task listing query parameters
#[derive(Debug, Deserialize)]
pub struct TaskListParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub created_by: Option<String>,
    pub category_id: Option<String>,
    pub tag_ids: Option<String>, // Comma-separated list of tag IDs
    pub due_before: Option<String>,
    pub due_after: Option<String>,
}

/// Task response
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub category_id: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Comment response
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: String,
    pub task_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Create task request
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub assigned_to: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Update task request
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub assigned_to: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Add comment request
#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub content: String,
}

/// Update comment request
#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: String,
}

/// Convert a domain Task to a TaskResponse
fn map_task_to_response(task: &crate::domain::Task) -> TaskResponse {
    TaskResponse {
        id: task.id.to_string(),
        title: task.title.clone(),
        description: task.description.clone(),
        status: task.status.to_string(),
        priority: task.priority.to_string(),
        due_date: task.due_date.map(|dt| dt.to_rfc3339()),
        assigned_to: task.assigned_to.map(|id| id.to_string()),
        created_by: task.created_by.to_string(),
        category_id: task.category_id.map(|id| id.to_string()),
        tags: task.tags.clone(),
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.to_rfc3339(),
    }
}

/// Convert a domain Comment to a CommentResponse
fn map_comment_to_response(task_id: Uuid, comment: &crate::domain::Comment) -> CommentResponse {
    CommentResponse {
        id: comment.id.to_string(),
        task_id: task_id.to_string(),
        user_id: comment.created_by.to_string(),
        content: comment.content.clone(),
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    }
}

/// Parse a priority string to the domain Priority enum
fn parse_priority(priority_str: &str) -> Result<Priority> {
    match priority_str.to_lowercase().as_str() {
        "high" => Ok(Priority::High),
        "medium" => Ok(Priority::Medium),
        "low" => Ok(Priority::Low),
        _ => Err(Error::validation_error(format!(
            "Invalid priority: {}",
            priority_str
        ))),
    }
}

/// Parse a status string to the domain TaskStatus enum
fn parse_status(status_str: &str) -> Result<TaskStatus> {
    match status_str.to_lowercase().as_str() {
        "todo" => Ok(TaskStatus::Todo),
        "in_progress" => Ok(TaskStatus::InProgress),
        "review" => Ok(TaskStatus::Review),
        "done" => Ok(TaskStatus::Done),
        _ => Err(Error::validation_error(format!(
            "Invalid status: {}",
            status_str
        ))),
    }
}

/// Parse a date string to DateTime<Utc>
fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::validation_error(format!("Invalid date format: {}", e)))
}

/// Get all tasks
pub async fn get_tasks(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<TaskListParams>,
) -> Result<Json<Vec<TaskResponse>>> {
    // Parse query parameters
    let task_service = registry.task_service();

    // Create filter
    let mut filter = TaskFilter::default();

    // Apply filters from query parameters
    if let Some(status_str) = &params.status {
        filter.status = Some(parse_status(status_str)?);
    }

    if let Some(priority_str) = &params.priority {
        filter.priority = Some(parse_priority(priority_str)?);
    }

    if let Some(assigned_to_str) = &params.assigned_to {
        filter.assigned_to = Some(
            Uuid::parse_str(assigned_to_str)
                .map_err(|_| Error::validation_error("Invalid UUID for assigned_to"))?,
        );
    }

    if let Some(created_by_str) = &params.created_by {
        filter.created_by = Some(
            Uuid::parse_str(created_by_str)
                .map_err(|_| Error::validation_error("Invalid UUID for created_by"))?,
        );
    }

    if let Some(category_id_str) = &params.category_id {
        filter.category_id = Some(
            Uuid::parse_str(category_id_str)
                .map_err(|_| Error::validation_error("Invalid UUID for category_id"))?,
        );
    }

    if let Some(due_before_str) = &params.due_before {
        filter.due_date_before = Some(parse_date(due_before_str)?);
    }

    if let Some(due_after_str) = &params.due_after {
        filter.due_date_after = Some(parse_date(due_after_str)?);
    }

    if let Some(tags_str) = &params.tag_ids {
        filter.tags = Some(tags_str.split(',').map(|s| s.trim().to_string()).collect());
    }

    // Get tasks with filter
    let tasks = task_service
        .get_tasks(filter)
        .await
        .map_err(|e| Error::internal_server_error(format!("Failed to get tasks: {}", e.message)))?;

    // Convert to response format
    let task_responses = tasks.iter().map(map_task_to_response).collect();

    Ok(Json(task_responses))
}

/// Get task by ID
pub async fn get_task(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
) -> Result<Json<TaskResponse>> {
    let task_service = registry.task_service();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Get task
    let task = task_service.get_task(id).await.map_err(|e| match e.kind {
        crate::application::TaskServiceErrorKind::NotFound => Error::not_found("Task not found"),
        _ => Error::internal_server_error(format!("Failed to get task: {}", e.message)),
    })?;

    Ok(Json(map_task_to_response(&task)))
}

/// Create task
pub async fn create_task(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<CreateTaskRequest>,
    // CurrentUser information would be extracted from auth middleware
) -> Result<Json<TaskResponse>> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse priority
    let priority = match &request.priority {
        Some(p) => parse_priority(p)?,
        None => Priority::Medium,
    };

    // Parse due date if provided
    let due_date = match &request.due_date {
        Some(date_str) => Some(parse_date(date_str)?),
        None => None,
    };

    // Parse assigned_to if provided
    let assigned_to = match &request.assigned_to {
        Some(user_id) => Some(
            Uuid::parse_str(user_id)
                .map_err(|_| Error::validation_error("Invalid UUID for assigned_to"))?,
        ),
        None => None,
    };

    // Parse category_id if provided
    let category_id = match &request.category_id {
        Some(cat_id) => Some(
            Uuid::parse_str(cat_id)
                .map_err(|_| Error::validation_error("Invalid UUID for category_id"))?,
        ),
        None => None,
    };

    // Create task
    let task = task_service
        .create_task(
            request.title,
            request.description,
            TaskStatus::Todo, // New tasks always start as Todo
            priority,
            due_date,
            assigned_to,
            current_user_id,
            category_id,
            request.tags.unwrap_or_default(),
        )
        .await
        .map_err(|e| {
            Error::internal_server_error(format!("Failed to create task: {}", e.message))
        })?;

    Ok(Json(map_task_to_response(&task)))
}

/// Update task
pub async fn update_task(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdateTaskRequest>,
    // CurrentUser would be extracted from auth middleware
) -> Result<Json<TaskResponse>> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Parse status if provided
    let status = match &request.status {
        Some(s) => Some(parse_status(s)?),
        None => None,
    };

    // Parse priority if provided
    let priority = match &request.priority {
        Some(p) => Some(parse_priority(p)?),
        None => None,
    };

    // Parse due date if provided
    let due_date = match &request.due_date {
        Some(date_str) => Some(parse_date(date_str)?),
        None => None,
    };

    // Parse assigned_to if provided
    let assigned_to = match &request.assigned_to {
        Some(user_id) => Some(
            Uuid::parse_str(user_id)
                .map_err(|_| Error::validation_error("Invalid UUID for assigned_to"))?,
        ),
        None => None,
    };

    // Parse category_id if provided
    let category_id = match &request.category_id {
        Some(cat_id) => Some(
            Uuid::parse_str(cat_id)
                .map_err(|_| Error::validation_error("Invalid UUID for category_id"))?,
        ),
        None => None,
    };

    // Update task
    let task = task_service
        .update_task(
            id,
            request.title,
            request.description,
            status,
            priority,
            due_date,
            assigned_to,
            category_id,
            request.tags,
            current_user_id,
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Task not found")
            }
            crate::application::TaskServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to update task: {}", e.message)),
        })?;

    Ok(Json(map_task_to_response(&task)))
}

/// Delete task
pub async fn delete_task(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    // CurrentUser would be extracted from auth middleware
) -> Result<StatusCode> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Delete task
    task_service
        .delete_task(id, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Task not found")
            }
            crate::application::TaskServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to delete task: {}", e.message)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Assign task to user
pub async fn assign_task(
    State(registry): State<Arc<ServiceRegistry>>,
    Path((id_str, user_id_str)): Path<(String, String)>,
    // CurrentUser would be extracted from auth middleware
) -> Result<StatusCode> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Parse user ID to assign to
    let user_id = Uuid::parse_str(&user_id_str)
        .map_err(|_| Error::validation_error("Invalid user ID format"))?;

    // Get the task first
    let task = task_service.get_task(id).await.map_err(|e| match e.kind {
        crate::application::TaskServiceErrorKind::NotFound => Error::not_found("Task not found"),
        _ => Error::internal_server_error(format!("Failed to get task: {}", e.message)),
    })?;

    // Update the task with the new assigned_to value
    task_service
        .update_task(
            id,
            None,
            None,
            None,
            None,
            None,
            Some(user_id),
            None,
            None,
            current_user_id,
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Task not found")
            }
            crate::application::TaskServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to assign task: {}", e.message)),
        })?;

    Ok(StatusCode::OK)
}

/// Unassign task
pub async fn unassign_task(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    // CurrentUser would be extracted from auth middleware
) -> Result<StatusCode> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Get the task first
    let task = task_service.get_task(id).await.map_err(|e| match e.kind {
        crate::application::TaskServiceErrorKind::NotFound => Error::not_found("Task not found"),
        _ => Error::internal_server_error(format!("Failed to get task: {}", e.message)),
    })?;

    // Update the task with assigned_to set to None
    task_service
        .update_task(
            id,
            None,
            None,
            None,
            None,
            None,
            Some(Uuid::nil()), // Using Uuid::nil() to indicate removal of assignment
            None,
            None,
            current_user_id,
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Task not found")
            }
            crate::application::TaskServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to unassign task: {}", e.message)),
        })?;

    Ok(StatusCode::OK)
}

/// Get task comments
pub async fn get_task_comments(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
) -> Result<Json<Vec<CommentResponse>>> {
    let task_service = registry.task_service();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Get task with comments
    let task = task_service.get_task(id).await.map_err(|e| match e.kind {
        crate::application::TaskServiceErrorKind::NotFound => Error::not_found("Task not found"),
        _ => Error::internal_server_error(format!("Failed to get task: {}", e.message)),
    })?;

    // Map comments to response format
    let comments = task
        .comments
        .iter()
        .map(|comment| map_comment_to_response(task.id, comment))
        .collect();

    Ok(Json(comments))
}

/// Add comment to task
pub async fn add_comment(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    Json(request): Json<AddCommentRequest>,
    // CurrentUser would be extracted from auth middleware
) -> Result<Json<CommentResponse>> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse task ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid task ID format"))?;

    // Add comment
    let comment = task_service
        .add_comment(id, request.content, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Task not found")
            }
            crate::application::TaskServiceErrorKind::ValidationError => {
                Error::validation_error(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to add comment: {}", e.message)),
        })?;

    Ok(Json(map_comment_to_response(id, &comment)))
}

/// Update comment
pub async fn update_comment(
    State(registry): State<Arc<ServiceRegistry>>,
    Path((task_id_str, comment_id_str)): Path<(String, String)>,
    Json(request): Json<UpdateCommentRequest>,
    // CurrentUser would be extracted from auth middleware
) -> Result<Json<CommentResponse>> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse task ID and comment ID
    let task_id = Uuid::parse_str(&task_id_str)
        .map_err(|_| Error::validation_error("Invalid task ID format"))?;

    let comment_id = Uuid::parse_str(&comment_id_str)
        .map_err(|_| Error::validation_error("Invalid comment ID format"))?;

    // Update comment
    let comment = task_service
        .update_comment(comment_id, request.content, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Comment not found")
            }
            crate::application::TaskServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            crate::application::TaskServiceErrorKind::ValidationError => {
                Error::validation_error(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to update comment: {}", e.message)),
        })?;

    Ok(Json(map_comment_to_response(task_id, &comment)))
}

/// Delete comment
pub async fn delete_comment(
    State(registry): State<Arc<ServiceRegistry>>,
    Path((task_id_str, comment_id_str)): Path<(String, String)>,
    // CurrentUser would be extracted from auth middleware
) -> Result<StatusCode> {
    let task_service = registry.task_service();

    // Mock user ID for now (would come from authentication)
    let current_user_id = Uuid::new_v4();

    // Parse comment ID
    let comment_id = Uuid::parse_str(&comment_id_str)
        .map_err(|_| Error::validation_error("Invalid comment ID format"))?;

    // Delete comment
    task_service
        .delete_comment(comment_id, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::TaskServiceErrorKind::NotFound => {
                Error::not_found("Comment not found")
            }
            crate::application::TaskServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to delete comment: {}", e.message)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
