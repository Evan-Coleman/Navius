use navius_test::{
    ErrorContext, ErrorInjection, ErrorPropagationTracker, ErrorVerifier, TestHarness, assert_err,
    assert_injected_error, assert_ok, verify_error_context, verify_error_path,
};
use std::fmt;

// Define a simple error type for a user service
#[derive(Debug)]
enum UserServiceError {
    NotFound(String),
    Unauthorized(String),
    DatabaseError(String),
    ValidationError(String),
}

impl fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserServiceError::NotFound(msg) => write!(f, "User not found: {}", msg),
            UserServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            UserServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            UserServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for UserServiceError {}

impl From<String> for UserServiceError {
    fn from(s: String) -> Self {
        UserServiceError::DatabaseError(s)
    }
}

// Define a simple user struct
struct User {
    id: String,
    name: String,
    email: String,
}

// Define mock database interface
trait UserDatabase {
    fn get_user(&self, id: &str) -> Result<User, UserServiceError>;
    fn save_user(&self, user: &User) -> Result<(), UserServiceError>;
}

// Define user service that uses the database
struct UserService<DB: UserDatabase> {
    db: DB,
}

impl<DB: UserDatabase> UserService<DB> {
    fn new(db: DB) -> Self {
        Self { db }
    }

    fn get_user(&self, id: &str) -> Result<User, UserServiceError> {
        // Create error injection point
        let mut injection =
            ErrorInjection::new("get_user").with_message("Simulated database error");

        // Create error tracker
        let mut tracker = ErrorPropagationTracker::new();
        tracker.add_component("UserService");

        // Execute operation with potential error injection
        match injection.check(|| self.db.get_user(id)) {
            Ok(user) => Ok(user),
            Err(err) => {
                // Add context to error and track propagation
                let mut context = ErrorContext::new("UserService", "get_user");
                context.add_context("user_id", id);
                tracker.add_component("Database").add_context(context);

                // Log error details for debugging
                eprintln!("Error getting user: {}", err);
                eprintln!("Error path: {:?}", tracker.path);
                eprintln!("Error context: {:?}", tracker.contexts);

                Err(err)
            }
        }
    }

    fn create_user(&self, name: &str, email: &str) -> Result<User, UserServiceError> {
        // Validate inputs
        if name.is_empty() {
            return Err(UserServiceError::ValidationError(
                "Name cannot be empty".into(),
            ));
        }
        if email.is_empty() || !email.contains('@') {
            return Err(UserServiceError::ValidationError("Invalid email".into()));
        }

        // Create user
        let user = User {
            id: format!("user_{}", rand::random::<u32>()),
            name: name.to_string(),
            email: email.to_string(),
        };

        // Save user
        self.db.save_user(&user)?;

        Ok(user)
    }
}

// Mock implementation of UserDatabase for testing
struct MockUserDatabase {
    should_fail: bool,
    error_message: String,
}

impl MockUserDatabase {
    fn new() -> Self {
        Self {
            should_fail: false,
            error_message: "Database error".into(),
        }
    }

    fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.error_message = message.to_string();
        self
    }
}

impl UserDatabase for MockUserDatabase {
    fn get_user(&self, id: &str) -> Result<User, UserServiceError> {
        if self.should_fail {
            return Err(UserServiceError::DatabaseError(self.error_message.clone()));
        }

        if id == "not_found" {
            return Err(UserServiceError::NotFound(format!("User {} not found", id)));
        }

        Ok(User {
            id: id.to_string(),
            name: "Test User".into(),
            email: "test@example.com".into(),
        })
    }

    fn save_user(&self, user: &User) -> Result<(), UserServiceError> {
        if self.should_fail {
            return Err(UserServiceError::DatabaseError(self.error_message.clone()));
        }

        println!("User saved: {} ({}, {})", user.name, user.id, user.email);
        Ok(())
    }
}

fn main() {
    // Create test harness
    let harness = TestHarness::new();

    // Test successful user retrieval
    harness.run_test(|| {
        let db = MockUserDatabase::new();
        let service = UserService::new(db);

        let user = assert_ok!(service.get_user("test123"));
        assert_eq!(user.id, "test123");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");

        Ok(())
    });

    // Test user not found
    harness.run_test(|| {
        let db = MockUserDatabase::new();
        let service = UserService::new(db);

        let err = assert_err!(service.get_user("not_found"));
        match err {
            UserServiceError::NotFound(msg) => {
                assert!(msg.contains("not_found"));
            }
            _ => panic!("Expected NotFound error, got: {:?}", err),
        }

        Ok(())
    });

    // Test database error with error injection
    harness.run_test(|| {
        let db = MockUserDatabase::new().with_failure("Connection timeout");
        let service = UserService::new(db);

        let err = assert_err!(service.get_user("test123"));
        match err {
            UserServiceError::DatabaseError(msg) => {
                assert_eq!(msg, "Connection timeout");
            }
            _ => panic!("Expected DatabaseError, got: {:?}", err),
        }

        Ok(())
    });

    // Test error injection
    harness.run_test(|| {
        let db = MockUserDatabase::new();
        let service = UserService::new(db);

        // Monkey patch the ErrorInjection in get_user to always inject errors
        let original_new = ErrorInjection::new;
        let _guard = scopeguard::guard((), |_| {
            // This would restore the original function in a real test
        });

        // Override ErrorInjection::new to return an injected error
        // In a real test, you'd use a mocking framework instead of this demonstration
        let injection = ErrorInjection::new("get_user")
            .inject()
            .with_message("Injected database failure");

        // Simulate the get_user call with our injected error
        let result = injection.check(|| {
            Ok(User {
                id: "test123".into(),
                name: "Test User".into(),
                email: "test@example.com".into(),
            })
        });

        // Verify the injected error
        assert_injected_error!(result, "Injected database failure");

        Ok(())
    });

    // Test error propagation tracking
    harness.run_test(|| {
        // Create tracker and error context
        let mut tracker = ErrorPropagationTracker::new();
        tracker.add_component("Controller");

        let mut context = ErrorContext::new("Controller", "handle_request");
        context.add_context("user_id", "test123");
        tracker.add_context(context);

        // Simulate service call
        tracker.add_component("UserService");

        let mut context = ErrorContext::new("UserService", "get_user");
        context.add_context("user_id", "test123");
        tracker.add_context(context);

        // Simulate database error
        tracker.add_component("Database");

        let mut context = ErrorContext::new("Database", "execute_query");
        context.add_context("query", "SELECT * FROM users WHERE id = ?");
        context.add_context("user_id", "test123");
        tracker.add_context(context);

        // Verify error propagation path
        verify_error_path!(tracker, "Controller", "UserService", "Database");

        // Verify error context
        verify_error_context!(tracker, "user_id", "test123");
        verify_error_context!(tracker, "query");

        Ok(())
    });

    // Test error verification
    harness.run_test(|| {
        // Create tracker and simulate error path
        let mut tracker = ErrorPropagationTracker::new();
        tracker
            .add_component("Controller")
            .add_component("UserService")
            .add_component("Database");

        let mut context = ErrorContext::new("Database", "execute_query");
        context.add_context("user_id", "test123");
        tracker.add_context(context);

        // Create error verifier
        let verifier = ErrorVerifier::new()
            .expect_message("not found")
            .expect_component("UserService")
            .expect_context("user_id");

        // Verify against an error
        let error = UserServiceError::NotFound("User test123 not found".into());
        assert_ok!(verifier.verify(&error, &tracker));

        Ok(())
    });

    println!("All error testing examples completed successfully!");
}
