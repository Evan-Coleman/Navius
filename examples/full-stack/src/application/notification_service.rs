use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    Notification, NotificationStatus, NotificationType,
    events::{EventPublisher, NotificationEvent, NotificationEventType},
    repositories::NotificationRepository,
};

#[derive(Debug, Clone)]
pub struct NotificationServiceError {
    pub kind: NotificationServiceErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NotificationServiceErrorKind {
    NotFound,
    ValidationError,
    PermissionDenied,
    RepositoryError,
    EventPublishError,
    InternalError,
}

impl NotificationServiceError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationServiceErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationServiceErrorKind::ValidationError,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationServiceErrorKind::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn repository_error(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationServiceErrorKind::RepositoryError,
            message: message.into(),
        }
    }

    pub fn event_publish_error(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationServiceErrorKind::EventPublishError,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationServiceErrorKind::InternalError,
            message: message.into(),
        }
    }
}

pub type NotificationResult<T> = Result<T, NotificationServiceError>;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn create_notification(
        &self,
        user_id: Uuid,
        title: String,
        content: String,
        notification_type: NotificationType,
        related_entity_id: Option<Uuid>,
    ) -> NotificationResult<Notification>;

    async fn mark_as_read(&self, id: Uuid, user_id: Uuid) -> NotificationResult<Notification>;
    async fn mark_all_as_read(&self, user_id: Uuid) -> NotificationResult<Vec<Notification>>;

    async fn get_notification(&self, id: Uuid) -> NotificationResult<Notification>;
    async fn get_user_notifications(
        &self,
        user_id: Uuid,
        filter: NotificationFilter,
    ) -> NotificationResult<Vec<Notification>>;

    async fn delete_notification(&self, id: Uuid, user_id: Uuid) -> NotificationResult<()>;
    async fn delete_all_notifications(&self, user_id: Uuid) -> NotificationResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct NotificationFilter {
    pub status: Option<NotificationStatus>,
    pub notification_type: Option<NotificationType>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub related_entity_id: Option<Uuid>,
}

pub struct NotificationServiceImpl {
    notification_repository: Arc<dyn NotificationRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl NotificationServiceImpl {
    pub fn new(
        notification_repository: Arc<dyn NotificationRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            notification_repository,
            event_publisher,
        }
    }

    async fn publish_notification_event(
        &self,
        notification_id: Uuid,
        event_type: NotificationEventType,
        user_id: Uuid,
    ) -> Result<(), NotificationServiceError> {
        let event = NotificationEvent::new(notification_id, event_type, user_id);
        self.event_publisher
            .publish_event(event)
            .await
            .map_err(|e| {
                NotificationServiceError::event_publish_error(format!(
                    "Failed to publish notification event: {}",
                    e
                ))
            })
    }
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn create_notification(
        &self,
        user_id: Uuid,
        title: String,
        content: String,
        notification_type: NotificationType,
        related_entity_id: Option<Uuid>,
    ) -> NotificationResult<Notification> {
        // Create a new notification
        let notification = Notification::new(
            user_id,
            title,
            content,
            notification_type,
            related_entity_id,
        )
        .map_err(|e| NotificationServiceError::validation_error(e))?;

        // Save to repository
        let created_notification = self
            .notification_repository
            .save(notification)
            .await
            .map_err(|e| {
                NotificationServiceError::repository_error(format!(
                    "Failed to save notification: {}",
                    e
                ))
            })?;

        // Publish notification created event
        self.publish_notification_event(
            created_notification.id,
            NotificationEventType::Created,
            user_id,
        )
        .await?;

        Ok(created_notification)
    }

    async fn mark_as_read(&self, id: Uuid, user_id: Uuid) -> NotificationResult<Notification> {
        // Get existing notification
        let notification = self.get_notification(id).await?;

        // Check if notification belongs to user
        if notification.user_id != user_id {
            return Err(NotificationServiceError::permission_denied(
                "You don't have permission to access this notification",
            ));
        }

        // Update status if not already read
        if notification.status == NotificationStatus::Read {
            return Ok(notification);
        }

        // Create updated notification
        let mut updated_notification = notification.clone();
        updated_notification.mark_as_read();

        // Save updated notification
        let saved_notification = self
            .notification_repository
            .save(updated_notification)
            .await
            .map_err(|e| {
                NotificationServiceError::repository_error(format!(
                    "Failed to update notification: {}",
                    e
                ))
            })?;

        // Publish notification read event
        self.publish_notification_event(
            saved_notification.id,
            NotificationEventType::Read,
            user_id,
        )
        .await?;

        Ok(saved_notification)
    }

    async fn mark_all_as_read(&self, user_id: Uuid) -> NotificationResult<Vec<Notification>> {
        // Get all unread notifications for user
        let filter = NotificationFilter {
            status: Some(NotificationStatus::Unread),
            ..Default::default()
        };

        let notifications = self.get_user_notifications(user_id, filter).await?;
        let mut updated_notifications = Vec::new();

        // Mark each notification as read
        for notification in notifications {
            let result = self.mark_as_read(notification.id, user_id).await?;
            updated_notifications.push(result);
        }

        Ok(updated_notifications)
    }

    async fn get_notification(&self, id: Uuid) -> NotificationResult<Notification> {
        self.notification_repository
            .find_by_id(id)
            .await
            .map_err(|e| {
                NotificationServiceError::repository_error(format!(
                    "Failed to get notification: {}",
                    e
                ))
            })?
            .ok_or_else(|| {
                NotificationServiceError::not_found(format!(
                    "Notification with id {} not found",
                    id
                ))
            })
    }

    async fn get_user_notifications(
        &self,
        user_id: Uuid,
        filter: NotificationFilter,
    ) -> NotificationResult<Vec<Notification>> {
        self.notification_repository
            .find_by_user_id(
                user_id,
                filter.status,
                filter.notification_type,
                filter.from_date,
                filter.to_date,
                filter.related_entity_id,
            )
            .await
            .map_err(|e| {
                NotificationServiceError::repository_error(format!(
                    "Failed to get user notifications: {}",
                    e
                ))
            })
    }

    async fn delete_notification(&self, id: Uuid, user_id: Uuid) -> NotificationResult<()> {
        // Get existing notification
        let notification = self.get_notification(id).await?;

        // Check if notification belongs to user
        if notification.user_id != user_id {
            return Err(NotificationServiceError::permission_denied(
                "You don't have permission to delete this notification",
            ));
        }

        // Delete notification
        self.notification_repository.delete(id).await.map_err(|e| {
            NotificationServiceError::repository_error(format!(
                "Failed to delete notification: {}",
                e
            ))
        })?;

        // Publish notification deleted event
        self.publish_notification_event(id, NotificationEventType::Deleted, user_id)
            .await?;

        Ok(())
    }

    async fn delete_all_notifications(&self, user_id: Uuid) -> NotificationResult<()> {
        // Get all notifications for user
        let filter = NotificationFilter::default();
        let notifications = self.get_user_notifications(user_id, filter).await?;

        // Delete each notification
        for notification in notifications {
            self.delete_notification(notification.id, user_id).await?;
        }

        Ok(())
    }
}
