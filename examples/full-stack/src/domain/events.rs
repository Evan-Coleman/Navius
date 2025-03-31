use chrono::{DateTime, Utc};
use navius_core::events::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::domain::{Priority, TaskStatus};

/// Event triggered when a new user is registered
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub metadata: EventMetadata,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
}

impl DomainEvent for UserRegisteredEvent {
    fn event_type(&self) -> String {
        "user.registered".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl UserRegisteredEvent {
    pub fn new(user_id: Uuid, username: String, email: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            user_id,
            username,
            email,
        }
    }
}

/// Event triggered when a user logs in
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserLoggedInEvent {
    pub metadata: EventMetadata,
    pub user_id: Uuid,
    pub username: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl DomainEvent for UserLoggedInEvent {
    fn event_type(&self) -> String {
        "user.logged_in".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl UserLoggedInEvent {
    pub fn new(
        user_id: Uuid,
        username: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            user_id,
            username,
            ip_address,
            user_agent,
        }
    }
}

/// Event triggered when a task is created
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskCreatedEvent {
    pub metadata: EventMetadata,
    pub task_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub tags: HashSet<Uuid>,
}

impl DomainEvent for TaskCreatedEvent {
    fn event_type(&self) -> String {
        "task.created".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl TaskCreatedEvent {
    pub fn new(
        task_id: Uuid,
        title: String,
        description: Option<String>,
        priority: Priority,
        due_date: Option<DateTime<Utc>>,
        created_by: Uuid,
        assigned_to: Option<Uuid>,
        category_id: Option<Uuid>,
        tags: HashSet<Uuid>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            task_id,
            title,
            description,
            priority,
            due_date,
            created_by,
            assigned_to,
            category_id,
            tags,
        }
    }
}

/// Event triggered when a task status changes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskStatusChangedEvent {
    pub metadata: EventMetadata,
    pub task_id: Uuid,
    pub title: String,
    pub previous_status: TaskStatus,
    pub new_status: TaskStatus,
    pub changed_by: Uuid,
}

impl DomainEvent for TaskStatusChangedEvent {
    fn event_type(&self) -> String {
        "task.status_changed".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl TaskStatusChangedEvent {
    pub fn new(
        task_id: Uuid,
        title: String,
        previous_status: TaskStatus,
        new_status: TaskStatus,
        changed_by: Uuid,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            task_id,
            title,
            previous_status,
            new_status,
            changed_by,
        }
    }
}

/// Event triggered when a task is assigned to a user
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskAssignedEvent {
    pub metadata: EventMetadata,
    pub task_id: Uuid,
    pub title: String,
    pub previous_assignee: Option<Uuid>,
    pub new_assignee: Uuid,
    pub assigned_by: Uuid,
    pub due_date: Option<DateTime<Utc>>,
}

impl DomainEvent for TaskAssignedEvent {
    fn event_type(&self) -> String {
        "task.assigned".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl TaskAssignedEvent {
    pub fn new(
        task_id: Uuid,
        title: String,
        previous_assignee: Option<Uuid>,
        new_assignee: Uuid,
        assigned_by: Uuid,
        due_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            task_id,
            title,
            previous_assignee,
            new_assignee,
            assigned_by,
            due_date,
        }
    }
}

/// Event triggered when a comment is added to a task
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskCommentAddedEvent {
    pub metadata: EventMetadata,
    pub comment_id: Uuid,
    pub task_id: Uuid,
    pub task_title: String,
    pub user_id: Uuid,
    pub content: String,
    pub mentions: Vec<Uuid>,
}

impl DomainEvent for TaskCommentAddedEvent {
    fn event_type(&self) -> String {
        "task.comment_added".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl TaskCommentAddedEvent {
    pub fn new(
        comment_id: Uuid,
        task_id: Uuid,
        task_title: String,
        user_id: Uuid,
        content: String,
        mentions: Vec<Uuid>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            comment_id,
            task_id,
            task_title,
            user_id,
            content,
            mentions,
        }
    }
}

/// Event triggered when a task is due soon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskDueSoonEvent {
    pub metadata: EventMetadata,
    pub task_id: Uuid,
    pub title: String,
    pub due_date: DateTime<Utc>,
    pub assigned_to: Uuid,
    pub created_by: Uuid,
    pub hours_remaining: i64,
}

impl DomainEvent for TaskDueSoonEvent {
    fn event_type(&self) -> String {
        "task.due_soon".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl TaskDueSoonEvent {
    pub fn new(
        task_id: Uuid,
        title: String,
        due_date: DateTime<Utc>,
        assigned_to: Uuid,
        created_by: Uuid,
        hours_remaining: i64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            task_id,
            title,
            due_date,
            assigned_to,
            created_by,
            hours_remaining,
        }
    }
}

/// Event triggered when a task becomes overdue
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskOverdueEvent {
    pub metadata: EventMetadata,
    pub task_id: Uuid,
    pub title: String,
    pub due_date: DateTime<Utc>,
    pub assigned_to: Uuid,
    pub created_by: Uuid,
}

impl DomainEvent for TaskOverdueEvent {
    fn event_type(&self) -> String {
        "task.overdue".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl TaskOverdueEvent {
    pub fn new(
        task_id: Uuid,
        title: String,
        due_date: DateTime<Utc>,
        assigned_to: Uuid,
        created_by: Uuid,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            task_id,
            title,
            due_date,
            assigned_to,
            created_by,
        }
    }
}

/// Event triggered when a notification is created
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationCreatedEvent {
    pub metadata: EventMetadata,
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub notification_type: String,
    pub related_entity_id: Option<Uuid>,
    pub related_entity_type: Option<String>,
}

impl DomainEvent for NotificationCreatedEvent {
    fn event_type(&self) -> String {
        "notification.created".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.notification_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl NotificationCreatedEvent {
    pub fn new(
        notification_id: Uuid,
        user_id: Uuid,
        title: String,
        notification_type: String,
        related_entity_id: Option<Uuid>,
        related_entity_type: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(),
            notification_id,
            user_id,
            title,
            notification_type,
            related_entity_id,
            related_entity_type,
        }
    }
}

/// Event triggered when a notification is delivered
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationDeliveredEvent {
    pub metadata: EventMetadata,
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub delivery_method: String,
}

impl DomainEvent for NotificationDeliveredEvent {
    fn event_type(&self) -> String {
        "notification.delivered".to_string()
    }

    fn aggregate_id(&self) -> String {
        self.notification_id.to_string()
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }
}

impl NotificationDeliveredEvent {
    pub fn new(notification_id: Uuid, user_id: Uuid, delivery_method: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            notification_id,
            user_id,
            delivery_method,
        }
    }
}
