//! Basic example demonstrating dependency injection with navius-di
//!
//! This example shows:
//! 1. Component registration with different scopes
//! 2. Component lifecycle hooks
//! 3. Dependency resolution
//! 4. Qualifier usage

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use navius_di::{
    AsyncLifecycle, ComponentRegistry, ComponentScope, Lifecycle, LifecyclePhase, Result,
};

// Example service interfaces
trait DatabaseConnection: Send + Sync {
    fn connect(&self) -> Result<()>;
    fn execute(&self, query: &str) -> Result<String>;
    fn close(&self) -> Result<()>;
}

trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: &str) -> Result<User>;
    fn find_all(&self) -> Result<Vec<User>>;
}

trait UserService: Send + Sync {
    fn get_user(&self, id: &str) -> Result<User>;
    fn get_all_users(&self) -> Result<Vec<User>>;
}

// Data model
#[derive(Debug, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Implementation of database connection with lifecycle hooks
struct PostgresConnection {
    url: String,
    is_connected: bool,
    query_count: AtomicUsize,
}

impl PostgresConnection {
    fn new(url: String) -> Self {
        println!("Creating PostgresConnection to {}", url);
        Self {
            url,
            is_connected: false,
            query_count: AtomicUsize::new(0),
        }
    }
}

impl DatabaseConnection for PostgresConnection {
    fn connect(&self) -> Result<()> {
        println!("Connecting to {}", self.url);
        // In a real implementation, this would establish a connection
        // For demonstration, we'll just set a flag
        unsafe {
            let this = self as *const Self as *mut Self;
            (*this).is_connected = true;
        }
        Ok(())
    }

    fn execute(&self, query: &str) -> Result<String> {
        if !self.is_connected {
            return Err(navius_di::Error::Other("Not connected".to_string()));
        }

        println!("Executing query '{}' on {}", query, self.url);
        self.query_count.fetch_add(1, Ordering::SeqCst);

        Ok(format!(
            "Result of '{}' (query #{})",
            query,
            self.query_count.load(Ordering::SeqCst)
        ))
    }

    fn close(&self) -> Result<()> {
        println!("Closing connection to {}", self.url);
        // In a real implementation, this would close the connection
        unsafe {
            let this = self as *const Self as *mut Self;
            (*this).is_connected = false;
        }
        Ok(())
    }
}

impl Lifecycle for PostgresConnection {
    fn on_create(&self) -> Result<()> {
        println!("PostgresConnection created");
        Ok(())
    }

    fn on_initialize(&self) -> Result<()> {
        println!("PostgresConnection initialized, connecting...");
        self.connect()
    }

    fn on_destroy(&self) -> Result<()> {
        println!("PostgresConnection being destroyed, closing connection...");
        self.close()
    }
}

// Repository implementation using the database connection
struct PostgresUserRepository {
    db: Arc<PostgresConnection>,
}

impl PostgresUserRepository {
    fn new(db: Arc<PostgresConnection>) -> Self {
        println!("Creating PostgresUserRepository");
        Self { db }
    }
}

impl UserRepository for PostgresUserRepository {
    fn find_by_id(&self, id: &str) -> Result<User> {
        let query = format!("SELECT * FROM users WHERE id = '{}'", id);
        let _ = self.db.execute(&query)?;

        // For demonstration, we'll just return a dummy user
        Ok(User {
            id: id.to_string(),
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
        })
    }

    fn find_all(&self) -> Result<Vec<User>> {
        let query = "SELECT * FROM users";
        let _ = self.db.execute(query)?;

        // For demonstration, return dummy users
        Ok(vec![
            User {
                id: "1".to_string(),
                name: "User 1".to_string(),
                email: "user1@example.com".to_string(),
            },
            User {
                id: "2".to_string(),
                name: "User 2".to_string(),
                email: "user2@example.com".to_string(),
            },
        ])
    }
}

// Service implementation using the repository
struct DefaultUserService {
    repository: Arc<dyn UserRepository>,
}

impl DefaultUserService {
    fn new(repository: Arc<dyn UserRepository>) -> Self {
        println!("Creating DefaultUserService");
        Self { repository }
    }
}

impl UserService for DefaultUserService {
    fn get_user(&self, id: &str) -> Result<User> {
        println!("UserService: Getting user {}", id);
        self.repository.find_by_id(id)
    }

    fn get_all_users(&self) -> Result<Vec<User>> {
        println!("UserService: Getting all users");
        self.repository.find_all()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting DI example application");

    // Create the component registry
    let registry = ComponentRegistry::new();

    // Register components
    println!("\n--- Registering Components ---");

    // Register database connection as a singleton
    registry.register_with_factory(
        || {
            Arc::new(PostgresConnection::new(
                "jdbc:postgresql://localhost:5432/userdb".to_string(),
            ))
        },
        ComponentScope::Singleton,
    );

    // Register user repository as a singleton, using the database connection
    registry.register_with_factory(
        || {
            let db = registry.get::<Arc<PostgresConnection>>().unwrap();
            Arc::new(PostgresUserRepository::new(db.clone())) as Arc<dyn UserRepository>
        },
        ComponentScope::Singleton,
    );

    // Register user service as a singleton, using the repository
    registry.register_with_factory(
        || {
            let repo = registry.get::<Arc<dyn UserRepository>>().unwrap();
            Arc::new(DefaultUserService::new(repo.clone())) as Arc<dyn UserService>
        },
        ComponentScope::Singleton,
    );

    // Register a prototype-scoped component (new instance each time)
    registry.register_with_factory_and_qualifier(
        || {
            Arc::new(PostgresConnection::new(
                "jdbc:postgresql://localhost:5432/analyticsdb".to_string(),
            ))
        },
        ComponentScope::Prototype,
        "analytics-db",
    );

    // Use the components
    println!("\n--- Using Components ---");

    // Get the user service and use it
    let user_service = registry.get::<Arc<dyn UserService>>()?;

    // Get a user
    let user = user_service.get_user("42")?;
    println!("Found user: {:?}", user);

    // Get all users
    let users = user_service.get_all_users()?;
    println!("Found {} users", users.len());

    // Get analytics DB and use it (prototype scope creates a new instance)
    println!("\n--- Using Prototype-Scoped Component with Qualifier ---");
    let analytics_db1 = registry.get_by_qualifier::<Arc<PostgresConnection>>("analytics-db")?;
    let analytics_db2 = registry.get_by_qualifier::<Arc<PostgresConnection>>("analytics-db")?;

    analytics_db1.execute("SELECT count(*) FROM page_views")?;
    analytics_db2.execute("SELECT count(*) FROM user_clicks")?;

    // Different instances because of prototype scope
    println!(
        "analytics_db1 ptr: {:p}, analytics_db2 ptr: {:p}, same instance: {}",
        Arc::as_ptr(&analytics_db1),
        Arc::as_ptr(&analytics_db2),
        Arc::ptr_eq(&analytics_db1, &analytics_db2)
    );

    // Demonstrate shutdown with proper lifecycle hooks
    println!("\n--- Shutting Down ---");
    registry.shutdown()?;

    println!("DI example application completed successfully");
    Ok(())
}
