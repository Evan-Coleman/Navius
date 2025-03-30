// Basic example of using the navius-test framework

use navius_test::prelude::*;
use std::sync::Arc;

// Define a simple interface for testing
trait UserService: Send + Sync {
    fn get_user(&self, id: &str) -> Result<User, String>;
    fn create_user(&self, user: User) -> Result<(), String>;
}

// Define a simple user type
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Define a mock implementation of UserService
#[derive(Debug, Clone)]
struct MockUserService {
    users: Vec<User>,
}

impl MockUserService {
    fn new() -> Self {
        Self {
            users: vec![User {
                id: "1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            }],
        }
    }
}

impl UserService for MockUserService {
    fn get_user(&self, id: &str) -> Result<User, String> {
        self.users
            .iter()
            .find(|u| u.id == id)
            .cloned()
            .ok_or_else(|| format!("User not found: {}", id))
    }

    fn create_user(&self, _user: User) -> Result<(), String> {
        // In a real impl, we would insert the user
        // For the mock, we'll just return success
        Ok(())
    }
}

// A component that uses the UserService
struct UserManager<T: UserService> {
    service: T,
}

impl<T: UserService> UserManager<T> {
    fn new(service: T) -> Self {
        Self { service }
    }

    fn get_user(&self, id: &str) -> Result<User, String> {
        self.service.get_user(id)
    }

    fn create_user(&self, name: &str, email: &str) -> Result<User, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            name: name.to_string(),
            email: email.to_string(),
        };

        self.service.create_user(user.clone())?;
        Ok(user)
    }
}

fn main() -> TestResult<()> {
    // Create a test harness
    let mut harness = TestHarnessBuilder::new().with_runtime().build()?;

    // Run a synchronous test
    let result = harness.run(|fixture, mock_registry| {
        // Register a mock implementation of UserService
        let mock_service = MockUserService::new();
        mock_registry.register::<dyn UserService, MockUserService>(mock_service.clone())?;

        // Register the UserManager as a component
        let manager = UserManager::new(mock_service);
        fixture.register(manager)?;

        // Get the user manager from the fixture
        let manager = fixture.get::<UserManager<MockUserService>>()?;

        // Test getting a user
        let user = manager.get_user("1")?;
        assert_eq!(user.name, "John Doe");

        // Test creating a user
        let new_user = manager.create_user("Jane Smith", "jane@example.com")?;
        assert_eq!(new_user.name, "Jane Smith");

        Ok(())
    })?;

    println!("Synchronous test passed!");

    // Run an asynchronous test
    let async_result = harness.run_async(|fixture, mock_registry| {
        Box::pin(async move {
            // Register a mock implementation of UserService
            let mock_service = MockUserService::new();
            mock_registry.register::<dyn UserService, MockUserService>(mock_service.clone())?;

            // Register the UserManager as a component
            let manager = UserManager::new(mock_service);
            fixture.register(manager)?;

            // Get the user manager from the fixture
            let manager = fixture.get::<UserManager<MockUserService>>()?;

            // Test getting a user that doesn't exist
            let result = manager.get_user("999");
            assert!(result.is_err());

            // Test error message
            if let Err(msg) = result {
                assert!(msg.contains("User not found"));
            }

            Ok(())
        })
    })?;

    println!("Asynchronous test passed!");

    // Tear down the harness
    harness.tear_down()?;

    Ok(())
}
