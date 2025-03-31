use chrono::{DateTime, Utc};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use validator::Validate;

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
            Priority::Critical => write!(f, "Critical"),
        }
    }
}

impl Priority {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(Error::validation_error(&format!("Invalid priority: {}", s))),
        }
    }

    pub fn to_numeric(&self) -> u8 {
        match self {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
            Priority::Critical => 3,
        }
    }
}

/// Task status stages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Review,
    Done,
    Archived,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Todo
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Blocked => write!(f, "Blocked"),
            TaskStatus::Review => write!(f, "Review"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Archived => write!(f, "Archived"),
        }
    }
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().replace(' ', "").as_str() {
            "todo" => Ok(TaskStatus::Todo),
            "inprogress" => Ok(TaskStatus::InProgress),
            "blocked" => Ok(TaskStatus::Blocked),
            "inreview" => Ok(TaskStatus::Review),
            "done" => Ok(TaskStatus::Done),
            "cancelled" => Ok(TaskStatus::Archived),
            _ => Err(Error::validation_error(&format!(
                "Invalid task status: {}",
                s
            ))),
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Archived)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TaskStatus::Todo | TaskStatus::InProgress | TaskStatus::Review
        )
    }

    pub fn can_transition_to(&self, target: &TaskStatus) -> bool {
        use TaskStatus::*;

        match (self, target) {
            // Valid transitions from Todo
            (Todo, InProgress) => true,
            (Todo, Archived) => true,

            // Valid transitions from InProgress
            (InProgress, Blocked) => true,
            (InProgress, Review) => true,
            (InProgress, Archived) => true,

            // Valid transitions from Blocked
            (Blocked, InProgress) => true,
            (Blocked, Archived) => true,

            // Valid transitions from Review
            (Review, InProgress) => true,
            (Review, Done) => true,
            (Review, Archived) => true,

            // Valid transitions from Done
            (Done, InProgress) => true,

            // No valid transitions from Archived
            (Archived, _) => false,

            // Same status is not a transition
            (a, b) if a == b => false,

            // Any other transition is invalid
            _ => false,
        }
    }
}

/// Tag for categorizing tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Tag {
    pub id: Uuid,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Tag name must be between 1 and 50 characters"
    ))]
    pub name: String,

    pub color: Option<String>,

    pub created_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_color(name: String, color: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color: Some(color),
            created_at: Utc::now(),
        }
    }
}

/// Comment on a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Comment {
    pub id: Uuid,

    pub task_id: Uuid,

    pub user_id: Uuid,

    #[validate(length(
        min = 1,
        max = 2000,
        message = "Comment must be between 1 and 2000 characters"
    ))]
    pub content: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

impl Comment {
    pub fn new(task_id: Uuid, user_id: Uuid, content: String) -> Result<Self> {
        if content.trim().is_empty() {
            return Err(Error::Validation(
                "Comment content cannot be empty".to_string(),
            ));
        }

        if content.len() > 1000 {
            return Err(Error::Validation(
                "Comment is too long (max 1000 characters)".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            task_id,
            user_id,
            content,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_content(&mut self, content: String) -> Result<()> {
        if content.is_empty() || content.len() > 2000 {
            return Err(Error::validation_error(
                "Comment must be between 1 and 2000 characters",
            ));
        }

        self.content = content;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Task representing a unit of work in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Task {
    pub id: Uuid,

    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(max = 5000, message = "Description must be less than 5000 characters"))]
    pub description: Option<String>,

    pub status: TaskStatus,

    pub priority: Priority,

    pub due_date: Option<DateTime<Utc>>,

    pub assigned_to: Option<Uuid>,

    pub created_by: Uuid,

    pub category_id: Option<Uuid>,

    pub tags: HashSet<Uuid>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with minimal required information
    pub fn new(title: String, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: TaskStatus::default(),
            priority: Priority::default(),
            due_date: None,
            assigned_to: None,
            created_by,
            category_id: None,
            tags: HashSet::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Validate the task data
    pub fn validate(&self) -> Result<()> {
        self.validate_with_message()
            .map_err(|e| Error::validation_error(&e.to_string()))?;

        // Additional validation logic
        if let Some(due_date) = self.due_date {
            if due_date < Utc::now() && !self.status.is_completed() {
                return Err(Error::validation_error(
                    "Due date cannot be in the past for active tasks",
                ));
            }
        }

        Ok(())
    }

    /// Assign the task to a user
    pub fn assign(&mut self, user_id: Uuid) {
        self.assigned_to = Some(user_id);
        self.updated_at = Utc::now();
    }

    /// Unassign the task
    pub fn unassign(&mut self) {
        self.assigned_to = None;
        self.updated_at = Utc::now();
    }

    /// Update the task status
    pub fn update_status(&mut self, new_status: TaskStatus) -> Result<()> {
        if !self.status.can_transition_to(&new_status) {
            return Err(Error::validation_error(&format!(
                "Cannot transition from {:?} to {:?}",
                self.status, new_status
            )));
        }

        self.status = new_status;
        self.updated_at = Utc::now();

        // If task is completed, set completed_at timestamp
        if new_status.is_completed() {
            self.completed_at = Some(Utc::now());
        } else {
            // If task was completed before and is now active again, clear completed_at
            self.completed_at = None;
        }

        Ok(())
    }

    /// Update task priority
    pub fn update_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }

    /// Set the due date
    pub fn set_due_date(&mut self, due_date: Option<DateTime<Utc>>) -> Result<()> {
        if let Some(date) = due_date {
            if date < Utc::now() && !self.status.is_completed() {
                return Err(Error::validation_error(
                    "Due date cannot be in the past for active tasks",
                ));
            }
        }

        self.due_date = due_date;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add a tag to the task
    pub fn add_tag(&mut self, tag_id: Uuid) -> bool {
        let result = self.tags.insert(tag_id);
        if result {
            self.updated_at = Utc::now();
        }
        result
    }

    /// Remove a tag from the task
    pub fn remove_tag(&mut self, tag_id: &Uuid) -> bool {
        let result = self.tags.remove(tag_id);
        if result {
            self.updated_at = Utc::now();
        }
        result
    }

    /// Set the category
    pub fn set_category(&mut self, category_id: Option<Uuid>) {
        self.category_id = category_id;
        self.updated_at = Utc::now();
    }

    /// Update the task title and description
    pub fn update_details(&mut self, title: String, description: Option<String>) -> Result<()> {
        if title.is_empty() || title.len() > 200 {
            return Err(Error::validation_error(
                "Title must be between 1 and 200 characters",
            ));
        }

        if let Some(desc) = &description {
            if desc.len() > 5000 {
                return Err(Error::validation_error(
                    "Description must be less than 5000 characters",
                ));
            }
        }

        self.title = title;
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if the task is overdue
    pub fn is_overdue(&self) -> bool {
        if self.status.is_completed() {
            return false;
        }

        if let Some(due_date) = self.due_date {
            due_date < Utc::now()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_task() {
        let user_id = Uuid::new_v4();
        let task = Task::new("Test Task".to_string(), user_id);

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.created_by, user_id);
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::Medium);
        assert!(task.description.is_none());
        assert!(task.assigned_to.is_none());
        assert!(task.tags.is_empty());
    }

    #[test]
    fn test_task_status_transitions() {
        let user_id = Uuid::new_v4();
        let mut task = Task::new("Test Task".to_string(), user_id);

        // Valid transitions
        assert!(task.update_status(TaskStatus::InProgress).is_ok());
        assert_eq!(task.status, TaskStatus::InProgress);

        assert!(task.update_status(TaskStatus::Blocked).is_ok());
        assert_eq!(task.status, TaskStatus::Blocked);

        assert!(task.update_status(TaskStatus::InProgress).is_ok());
        assert!(task.update_status(TaskStatus::InReview).is_ok());
        assert!(task.update_status(TaskStatus::Done).is_ok());

        // Completed timestamp should be set
        assert!(task.completed_at.is_some());

        // Invalid transitions
        let mut completed_task = task.clone();
        assert!(completed_task.update_status(TaskStatus::Cancelled).is_err());

        // Can reopen a completed task
        assert!(task.update_status(TaskStatus::InProgress).is_ok());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_tags() {
        let user_id = Uuid::new_v4();
        let mut task = Task::new("Test Task".to_string(), user_id);

        let tag1 = Uuid::new_v4();
        let tag2 = Uuid::new_v4();

        assert!(task.tags.is_empty());

        // Add tags
        assert!(task.add_tag(tag1));
        assert!(task.add_tag(tag2));
        assert_eq!(task.tags.len(), 2);

        // Adding same tag again returns false
        assert!(!task.add_tag(tag1));
        assert_eq!(task.tags.len(), 2);

        // Remove tag
        assert!(task.remove_tag(&tag1));
        assert_eq!(task.tags.len(), 1);
        assert!(!task.tags.contains(&tag1));
        assert!(task.tags.contains(&tag2));

        // Removing non-existent tag returns false
        assert!(!task.remove_tag(&Uuid::new_v4()));
    }

    #[test]
    fn test_due_date_validation() {
        let user_id = Uuid::new_v4();
        let mut task = Task::new("Test Task".to_string(), user_id);

        // Past date should be rejected for active tasks
        let past_date = Utc::now() - Duration::hours(24);
        assert!(task.set_due_date(Some(past_date)).is_err());

        // Future date should be accepted
        let future_date = Utc::now() + Duration::hours(24);
        assert!(task.set_due_date(Some(future_date)).is_ok());
        assert_eq!(task.due_date, Some(future_date));

        // Completing the task should allow past due dates
        task.update_status(TaskStatus::InProgress).unwrap();
        task.update_status(TaskStatus::InReview).unwrap();
        task.update_status(TaskStatus::Done).unwrap();
        assert!(task.set_due_date(Some(past_date)).is_ok());
    }

    #[test]
    fn test_is_overdue() {
        let user_id = Uuid::new_v4();
        let mut task = Task::new("Test Task".to_string(), user_id);

        // No due date - not overdue
        assert!(!task.is_overdue());

        // Future due date - not overdue
        let future_date = Utc::now() + Duration::hours(24);
        task.set_due_date(Some(future_date)).unwrap();
        assert!(!task.is_overdue());

        // To test overdue, we need to set a due date and manipulate the task directly
        // since validation prevents setting past due dates
        let past_date = Utc::now() - Duration::hours(1);
        task.due_date = Some(past_date);
        assert!(task.is_overdue());

        // Completed tasks are never overdue
        task.update_status(TaskStatus::InProgress).unwrap();
        task.update_status(TaskStatus::InReview).unwrap();
        task.update_status(TaskStatus::Done).unwrap();
        assert!(!task.is_overdue());
    }
}
