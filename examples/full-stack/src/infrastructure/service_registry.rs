use std::sync::Arc;

use crate::application::{
    CategoryService, CategoryServiceImpl, NotificationService, NotificationServiceImpl,
    TaskService, TaskServiceImpl, UserService, UserServiceImpl,
};

use crate::domain::repositories::{
    CategoryRepository, NotificationRepository, TaskRepository, UserRepository,
};

use crate::domain::events::EventPublisher;
use crate::infrastructure::events::InMemoryEventPublisher;
use crate::infrastructure::repositories::{
    InMemoryCategoryRepository, InMemoryNotificationRepository, InMemoryTaskRepository,
    InMemoryUserRepository,
};

#[derive(Clone)]
pub struct ServiceRegistry {
    task_service: Arc<dyn TaskService>,
    user_service: Arc<dyn UserService>,
    notification_service: Arc<dyn NotificationService>,
    category_service: Arc<dyn CategoryService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        // Create repositories
        let task_repository: Arc<dyn TaskRepository> = Arc::new(InMemoryTaskRepository::new());
        let user_repository: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
        let notification_repository: Arc<dyn NotificationRepository> =
            Arc::new(InMemoryNotificationRepository::new());
        let category_repository: Arc<dyn CategoryRepository> =
            Arc::new(InMemoryCategoryRepository::new());

        // Create event publisher
        let event_publisher: Arc<dyn EventPublisher> = Arc::new(InMemoryEventPublisher::new());

        // Create services
        let task_service: Arc<dyn TaskService> = Arc::new(TaskServiceImpl::new(
            task_repository,
            event_publisher.clone(),
        ));

        let user_service: Arc<dyn UserService> = Arc::new(UserServiceImpl::new(
            user_repository,
            event_publisher.clone(),
        ));

        let notification_service: Arc<dyn NotificationService> = Arc::new(
            NotificationServiceImpl::new(notification_repository, event_publisher.clone()),
        );

        let category_service: Arc<dyn CategoryService> = Arc::new(CategoryServiceImpl::new(
            category_repository,
            event_publisher.clone(),
        ));

        Self {
            task_service,
            user_service,
            notification_service,
            category_service,
            event_publisher,
        }
    }

    // Getters for services
    pub fn task_service(&self) -> Arc<dyn TaskService> {
        self.task_service.clone()
    }

    pub fn user_service(&self) -> Arc<dyn UserService> {
        self.user_service.clone()
    }

    pub fn notification_service(&self) -> Arc<dyn NotificationService> {
        self.notification_service.clone()
    }

    pub fn category_service(&self) -> Arc<dyn CategoryService> {
        self.category_service.clone()
    }

    pub fn event_publisher(&self) -> Arc<dyn EventPublisher> {
        self.event_publisher.clone()
    }

    // For testing purposes
    #[cfg(test)]
    pub fn with_mock_task_service(&self, mock_service: Arc<dyn TaskService>) -> Self {
        Self {
            task_service: mock_service,
            user_service: self.user_service.clone(),
            notification_service: self.notification_service.clone(),
            category_service: self.category_service.clone(),
            event_publisher: self.event_publisher.clone(),
        }
    }

    #[cfg(test)]
    pub fn with_mock_user_service(&self, mock_service: Arc<dyn UserService>) -> Self {
        Self {
            task_service: self.task_service.clone(),
            user_service: mock_service,
            notification_service: self.notification_service.clone(),
            category_service: self.category_service.clone(),
            event_publisher: self.event_publisher.clone(),
        }
    }

    #[cfg(test)]
    pub fn with_mock_notification_service(
        &self,
        mock_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            task_service: self.task_service.clone(),
            user_service: self.user_service.clone(),
            notification_service: mock_service,
            category_service: self.category_service.clone(),
            event_publisher: self.event_publisher.clone(),
        }
    }

    #[cfg(test)]
    pub fn with_mock_category_service(&self, mock_service: Arc<dyn CategoryService>) -> Self {
        Self {
            task_service: self.task_service.clone(),
            user_service: self.user_service.clone(),
            notification_service: self.notification_service.clone(),
            category_service: mock_service,
            event_publisher: self.event_publisher.clone(),
        }
    }

    #[cfg(test)]
    pub fn with_mock_event_publisher(&self, mock_publisher: Arc<dyn EventPublisher>) -> Self {
        Self {
            task_service: self.task_service.clone(),
            user_service: self.user_service.clone(),
            notification_service: self.notification_service.clone(),
            category_service: self.category_service.clone(),
            event_publisher: mock_publisher,
        }
    }
}
