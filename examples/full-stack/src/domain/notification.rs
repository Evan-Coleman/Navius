use chrono::{DateTime, Utc};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use validator::Validate;

/// Types of notifications in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    TaskAssigned,
    TaskDueSoon,
    TaskOverdue,
    TaskStatusChanged,
    TaskCommentAdded,
    MentionedInComment,
    SystemAlert,
    InvitationReceived,
    AccountSecurity,
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationType::TaskAssigned => write!(f, "Task Assigned"),
            NotificationType::TaskDueSoon => write!(f, "Task Due Soon"),
            NotificationType::TaskOverdue => write!(f, "Task Overdue"),
            NotificationType::TaskStatusChanged => write!(f, "Task Status Changed"),
            NotificationType::TaskCommentAdded => write!(f, "Task Comment Added"),
            NotificationType::MentionedInComment => write!(f, "Mentioned in Comment"),
            NotificationType::SystemAlert => write!(f, "System Alert"),
            NotificationType::InvitationReceived => write!(f, "Invitation Received"),
            NotificationType::AccountSecurity => write!(f, "Account Security"),
        }
    }
}

/// Notification priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Normal
    }
}

impl fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationPriority::Low => write!(f, "Low"),
            NotificationPriority::Normal => write!(f, "Normal"),
            NotificationPriority::High => write!(f, "High"),
            NotificationPriority::Urgent => write!(f, "Urgent"),
        }
    }
}

/// Notification delivery methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMethod {
    InApp,
    Email,
    Push,
    SMS,
}

impl fmt::Display for DeliveryMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryMethod::InApp => write!(f, "In-App"),
            DeliveryMethod::Email => write!(f, "Email"),
            DeliveryMethod::Push => write!(f, "Push Notification"),
            DeliveryMethod::SMS => write!(f, "SMS"),
        }
    }
}

/// Notification delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Cancelled,
}

impl Default for DeliveryStatus {
    fn default() -> Self {
        DeliveryStatus::Pending
    }
}

impl fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryStatus::Pending => write!(f, "Pending"),
            DeliveryStatus::Delivered => write!(f, "Delivered"),
            DeliveryStatus::Failed => write!(f, "Failed"),
            DeliveryStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Notification delivery attempt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub timestamp: DateTime<Utc>,
    pub status: DeliveryStatus,
    pub method: DeliveryMethod,
    pub error_message: Option<String>,
}

impl DeliveryAttempt {
    pub fn new(method: DeliveryMethod) -> Self {
        Self {
            timestamp: Utc::now(),
            status: DeliveryStatus::Pending,
            method,
            error_message: None,
        }
    }

    pub fn mark_delivered(&mut self) {
        self.status = DeliveryStatus::Delivered;
        self.timestamp = Utc::now();
    }

    pub fn mark_failed(&mut self, error_message: String) {
        self.status = DeliveryStatus::Failed;
        self.error_message = Some(error_message);
        self.timestamp = Utc::now();
    }

    pub fn mark_cancelled(&mut self) {
        self.status = DeliveryStatus::Cancelled;
        self.timestamp = Utc::now();
    }
}

/// Notification entity representing a system message for a user
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Notification {
    pub id: Uuid,

    pub user_id: Uuid,

    pub notification_type: NotificationType,

    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 2000,
        message = "Content must be between 1 and 2000 characters"
    ))]
    pub content: String,

    pub priority: NotificationPriority,

    pub read: bool,

    pub dismissed: bool,

    pub action_url: Option<String>,

    pub related_entity_id: Option<Uuid>,

    pub related_entity_type: Option<String>,

    pub delivery_attempts: Vec<DeliveryAttempt>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    pub expires_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        user_id: Uuid,
        notification_type: NotificationType,
        title: String,
        content: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            notification_type,
            title,
            content,
            priority: NotificationPriority::default(),
            read: false,
            dismissed: false,
            action_url: None,
            related_entity_id: None,
            related_entity_type: None,
            delivery_attempts: Vec::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    /// Validate the notification data
    pub fn validate(&self) -> Result<()> {
        self.validate_with_message()
            .map_err(|e| Error::validation_error(&e.to_string()))?;

        // Additional validation
        if let Some(expires_at) = self.expires_at {
            if expires_at < self.created_at {
                return Err(Error::validation_error(
                    "Expiration date cannot be before creation date",
                ));
            }
        }

        Ok(())
    }

    /// Mark notification as read
    pub fn mark_read(&mut self) {
        if !self.read {
            self.read = true;
            self.updated_at = Utc::now();
        }
    }

    /// Mark notification as unread
    pub fn mark_unread(&mut self) {
        if self.read {
            self.read = false;
            self.updated_at = Utc::now();
        }
    }

    /// Dismiss the notification
    pub fn dismiss(&mut self) {
        if !self.dismissed {
            self.dismissed = true;
            self.updated_at = Utc::now();
        }
    }

    /// Set the notification priority
    pub fn set_priority(&mut self, priority: NotificationPriority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }

    /// Set expiration date for the notification
    pub fn set_expiration(&mut self, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        if let Some(date) = expires_at {
            if date < self.created_at {
                return Err(Error::validation_error(
                    "Expiration date cannot be before creation date",
                ));
            }
        }

        self.expires_at = expires_at;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set action URL for the notification
    pub fn set_action(&mut self, action_url: Option<String>) {
        self.action_url = action_url;
        self.updated_at = Utc::now();
    }

    /// Set related entity information
    pub fn set_related_entity(&mut self, entity_id: Uuid, entity_type: String) {
        self.related_entity_id = Some(entity_id);
        self.related_entity_type = Some(entity_type);
        self.updated_at = Utc::now();
    }

    /// Add a delivery attempt
    pub fn add_delivery_attempt(&mut self, method: DeliveryMethod) {
        let attempt = DeliveryAttempt::new(method);
        self.delivery_attempts.push(attempt);
        self.updated_at = Utc::now();
    }

    /// Update the latest delivery attempt status
    pub fn update_latest_delivery_status(
        &mut self,
        status: DeliveryStatus,
        error_message: Option<String>,
    ) -> Result<()> {
        if let Some(attempt) = self.delivery_attempts.last_mut() {
            match status {
                DeliveryStatus::Delivered => attempt.mark_delivered(),
                DeliveryStatus::Failed => {
                    if let Some(msg) = error_message {
                        attempt.mark_failed(msg);
                    } else {
                        attempt.mark_failed("Unknown error".to_string());
                    }
                }
                DeliveryStatus::Cancelled => attempt.mark_cancelled(),
                DeliveryStatus::Pending => (),
            }
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(Error::validation_error("No delivery attempts to update"))
        }
    }

    /// Check if the notification has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    /// Get delivery status for a specific method
    pub fn get_delivery_status(&self, method: DeliveryMethod) -> Option<DeliveryStatus> {
        self.delivery_attempts
            .iter()
            .filter(|attempt| attempt.method == method)
            .last()
            .map(|attempt| attempt.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_notification() {
        let user_id = Uuid::new_v4();
        let notification = Notification::new(
            user_id,
            NotificationType::TaskAssigned,
            "Task Assigned".to_string(),
            "You have been assigned to a new task".to_string(),
        );

        assert_eq!(notification.user_id, user_id);
        assert_eq!(
            notification.notification_type,
            NotificationType::TaskAssigned
        );
        assert_eq!(notification.title, "Task Assigned");
        assert_eq!(notification.content, "You have been assigned to a new task");
        assert_eq!(notification.priority, NotificationPriority::Normal);
        assert!(!notification.read);
        assert!(!notification.dismissed);
        assert!(notification.action_url.is_none());
        assert!(notification.related_entity_id.is_none());
        assert!(notification.related_entity_type.is_none());
        assert!(notification.delivery_attempts.is_empty());
    }

    #[test]
    fn test_notification_status() {
        let user_id = Uuid::new_v4();
        let mut notification = Notification::new(
            user_id,
            NotificationType::TaskDueSoon,
            "Task Due Soon".to_string(),
            "Your task is due in 24 hours".to_string(),
        );

        // Initially unread and not dismissed
        assert!(!notification.read);
        assert!(!notification.dismissed);

        // Mark as read
        notification.mark_read();
        assert!(notification.read);

        // Mark as unread
        notification.mark_unread();
        assert!(!notification.read);

        // Dismiss notification
        notification.dismiss();
        assert!(notification.dismissed);
    }

    #[test]
    fn test_notification_expiration() {
        let user_id = Uuid::new_v4();
        let mut notification = Notification::new(
            user_id,
            NotificationType::TaskDueSoon,
            "Task Due Soon".to_string(),
            "Your task is due in 24 hours".to_string(),
        );

        // No expiration initially
        assert!(!notification.is_expired());

        // Set future expiration
        let future = Utc::now() + Duration::days(1);
        assert!(notification.set_expiration(Some(future)).is_ok());
        assert!(!notification.is_expired());

        // Invalid expiration date (before creation)
        let past = Utc::now() - Duration::days(1);
        assert!(notification.set_expiration(Some(past)).is_err());

        // Set expiration to current time to test expiration
        let now = Utc::now();
        assert!(notification.set_expiration(Some(now)).is_ok());
        // Sleep briefly to ensure now is in the past
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(notification.is_expired());

        // Clear expiration
        assert!(notification.set_expiration(None).is_ok());
        assert!(!notification.is_expired());
    }

    #[test]
    fn test_delivery_attempts() {
        let user_id = Uuid::new_v4();
        let mut notification = Notification::new(
            user_id,
            NotificationType::SystemAlert,
            "System Maintenance".to_string(),
            "The system will be down for maintenance tonight".to_string(),
        );

        // No delivery attempts initially
        assert_eq!(notification.delivery_attempts.len(), 0);
        assert!(
            notification
                .get_delivery_status(DeliveryMethod::InApp)
                .is_none()
        );

        // Add a delivery attempt
        notification.add_delivery_attempt(DeliveryMethod::InApp);
        assert_eq!(notification.delivery_attempts.len(), 1);
        assert_eq!(
            notification.get_delivery_status(DeliveryMethod::InApp),
            Some(DeliveryStatus::Pending)
        );

        // Update delivery status to delivered
        assert!(
            notification
                .update_latest_delivery_status(DeliveryStatus::Delivered, None)
                .is_ok()
        );
        assert_eq!(
            notification.get_delivery_status(DeliveryMethod::InApp),
            Some(DeliveryStatus::Delivered)
        );

        // Add email attempt and mark as failed
        notification.add_delivery_attempt(DeliveryMethod::Email);
        assert!(
            notification
                .update_latest_delivery_status(
                    DeliveryStatus::Failed,
                    Some("Invalid email address".to_string())
                )
                .is_ok()
        );
        assert_eq!(
            notification.get_delivery_status(DeliveryMethod::Email),
            Some(DeliveryStatus::Failed)
        );

        // Original in-app notification should still be delivered
        assert_eq!(
            notification.get_delivery_status(DeliveryMethod::InApp),
            Some(DeliveryStatus::Delivered)
        );
    }

    #[test]
    fn test_related_entity() {
        let user_id = Uuid::new_v4();
        let mut notification = Notification::new(
            user_id,
            NotificationType::TaskStatusChanged,
            "Task Status Changed".to_string(),
            "The status of your task has changed to Done".to_string(),
        );

        // No related entity initially
        assert!(notification.related_entity_id.is_none());
        assert!(notification.related_entity_type.is_none());

        // Set related entity
        let task_id = Uuid::new_v4();
        notification.set_related_entity(task_id, "Task".to_string());

        assert_eq!(notification.related_entity_id, Some(task_id));
        assert_eq!(notification.related_entity_type, Some("Task".to_string()));
    }
}
