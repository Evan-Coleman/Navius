use axum::Json;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use chrono::Utc;
use navius_core::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification listing query parameters
#[derive(Debug, Deserialize)]
pub struct NotificationListParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub unread_only: Option<bool>,
    pub from_date: Option<String>,
}

/// Notification type enum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    TaskAssigned,
    TaskUpdated,
    CommentAdded,
    UserMention,
    SystemAlert,
}

/// Notification response
#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub link: Option<String>,
    pub created_at: String,
}

/// Create notification request
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
}

/// Get user notifications
pub async fn get_notifications(
    Query(params): Query<NotificationListParams>,
) -> Result<Json<Vec<NotificationResponse>>> {
    // Implementation will be added later
    Ok(Json(vec![
        NotificationResponse {
            id: Uuid::new_v4().to_string(),
            notification_type: NotificationType::TaskAssigned,
            title: "Task Assigned".to_string(),
            message: "You have been assigned a new task".to_string(),
            is_read: false,
            link: Some("/tasks/123".to_string()),
            created_at: Utc::now().to_rfc3339(),
        },
        NotificationResponse {
            id: Uuid::new_v4().to_string(),
            notification_type: NotificationType::CommentAdded,
            title: "New Comment".to_string(),
            message: "A new comment was added to a task you're following".to_string(),
            is_read: true,
            link: Some("/tasks/456/comments".to_string()),
            created_at: Utc::now().to_rfc3339(),
        },
    ]))
}

/// Mark notification as read
pub async fn mark_as_read(Path(id): Path<String>) -> Result<StatusCode> {
    // Implementation will be added later
    Ok(StatusCode::OK)
}

/// Mark all notifications as read
pub async fn mark_all_as_read() -> Result<StatusCode> {
    // Implementation will be added later
    Ok(StatusCode::OK)
}

/// Delete notification
pub async fn delete_notification(Path(id): Path<String>) -> Result<StatusCode> {
    // Implementation will be added later
    Ok(StatusCode::NO_CONTENT)
}

/// Send notification (admin only)
pub async fn send_notification(
    Json(request): Json<CreateNotificationRequest>,
) -> Result<Json<NotificationResponse>> {
    // Implementation will be added later
    let id = Uuid::new_v4().to_string();
    Ok(Json(NotificationResponse {
        id,
        notification_type: request.notification_type,
        title: request.title,
        message: request.message,
        is_read: false,
        link: request.link,
        created_at: Utc::now().to_rfc3339(),
    }))
}
