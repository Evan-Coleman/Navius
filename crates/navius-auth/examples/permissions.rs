use navius_auth::{
    basic::{BasicProvider, BasicProviderConfig, MockUser},
    middleware::AuthChecker,
    AuthProvider, Credentials, Error, Role, Subject,
};
use std::{collections::HashMap, sync::Arc};
use tracing::Level;

async fn main_func() -> Result<(), Error> {
    // Set up basic tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Create a mock provider for testing
    let provider = create_auth_provider();
    let provider_arc = Arc::new(provider) as Arc<dyn AuthProvider>;

    // Authenticate users
    let admin_identity = authenticate_user(Arc::clone(&provider_arc), "admin", "admin123").await?;
    let user_identity = authenticate_user(Arc::clone(&provider_arc), "user", "password").await?;
    let editor_identity =
        authenticate_user(Arc::clone(&provider_arc), "editor", "editor123").await?;

    println!("\nIdentities retrieved successfully!");

    // Display user roles and permissions
    println!("\n--- User Information ---");
    print_user_info("Regular User", &user_identity.roles);
    print_user_info("Editor User", &editor_identity.roles);
    print_user_info("Admin User", &admin_identity.roles);

    // Create an authorization checker based on roles
    let admin_checker = create_role_checker(Arc::clone(&provider_arc), vec!["admin"]);
    let editor_checker = create_role_checker(Arc::clone(&provider_arc), vec!["editor", "admin"]);
    let user_checker = create_role_checker(Arc::clone(&provider_arc), vec!["user"]);

    // Convert identities to subjects for authorization checks
    let admin_subject = create_subject(&admin_identity);
    let user_subject = create_subject(&user_identity);
    let editor_subject = create_subject(&editor_identity);

    // Check permissions
    println!("\n--- Permission Checks ---");

    // Simulate different action checks
    check_permission("view", "posts", &admin_subject, &admin_identity.roles);
    check_permission("edit", "posts", &admin_subject, &admin_identity.roles);
    check_permission("delete", "posts", &admin_subject, &admin_identity.roles);

    check_permission("view", "posts", &editor_subject, &editor_identity.roles);
    check_permission("edit", "posts", &editor_subject, &editor_identity.roles);
    check_permission("delete", "posts", &editor_subject, &editor_identity.roles);

    check_permission("view", "posts", &user_subject, &user_identity.roles);
    check_permission("edit", "posts", &user_subject, &user_identity.roles);
    check_permission("delete", "posts", &user_subject, &user_identity.roles);

    // Simulate role-based access control for different operations
    println!("\n--- Role-Based Access Control ---");

    // Admin operations (only admin should have access)
    println!("\nAdmin Operations:");
    let operations = ["View System Settings", "Manage Users", "Delete Content"];
    check_operations(&operations, &admin_checker, &admin_subject, "Admin");
    check_operations(&operations, &admin_checker, &editor_subject, "Editor");
    check_operations(&operations, &admin_checker, &user_subject, "Regular User");

    // Editor operations (admin and editor should have access)
    println!("\nEditor Operations:");
    let operations = ["Create Content", "Edit Content", "Publish Content"];
    check_operations(&operations, &editor_checker, &admin_subject, "Admin");
    check_operations(&operations, &editor_checker, &editor_subject, "Editor");
    check_operations(&operations, &editor_checker, &user_subject, "Regular User");

    // User operations (all users should have access)
    println!("\nRegular User Operations:");
    let operations = ["View Content", "Submit Comments", "Update Profile"];
    check_operations(&operations, &user_checker, &admin_subject, "Admin");
    check_operations(&operations, &user_checker, &editor_subject, "Editor");
    check_operations(&operations, &user_checker, &user_subject, "Regular User");

    Ok(())
}

fn create_auth_provider() -> BasicProvider {
    let config = BasicProviderConfig {
        secret_key: "example-secret-key".to_string(),
        token_expiry: 3600,    // 1 hour
        hash_passwords: false, // No hashing for simplicity in examples
        mock_users: vec![
            MockUser {
                id: "user-1".to_string(),
                username: "user".to_string(),
                password: "password".to_string(),
                display_name: Some("Regular User".to_string()),
                email: Some("user@example.com".to_string()),
                roles: vec!["user".to_string()],
                permissions: vec!["posts:view".to_string()],
            },
            MockUser {
                id: "editor-1".to_string(),
                username: "editor".to_string(),
                password: "editor123".to_string(),
                display_name: Some("Editor User".to_string()),
                email: Some("editor@example.com".to_string()),
                roles: vec!["editor".to_string(), "user".to_string()],
                permissions: vec!["posts:view".to_string(), "posts:edit".to_string()],
            },
            MockUser {
                id: "admin-1".to_string(),
                username: "admin".to_string(),
                password: "admin123".to_string(),
                display_name: Some("Admin User".to_string()),
                email: Some("admin@example.com".to_string()),
                roles: vec![
                    "admin".to_string(),
                    "editor".to_string(),
                    "user".to_string(),
                ],
                permissions: vec![
                    "posts:view".to_string(),
                    "posts:edit".to_string(),
                    "posts:delete".to_string(),
                    "users:manage".to_string(),
                    "system:configure".to_string(),
                ],
            },
        ],
    };

    BasicProvider::new("example-provider".to_string(), config)
}

async fn authenticate_user(
    provider: Arc<dyn AuthProvider>,
    username: &str,
    password: &str,
) -> Result<navius_auth::Identity, Error> {
    let credentials = Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };

    // Authenticate and return identity
    provider.authenticate(&credentials).await
}

fn create_subject(identity: &navius_auth::Identity) -> Subject {
    Subject {
        id: identity.id.clone(),
        subject_type: "user".to_string(),
        name: identity
            .display_name
            .clone()
            .unwrap_or_else(|| identity.username.clone()),
        roles: identity.roles.clone(),
        attributes: None,
    }
}

fn create_role_checker(provider: Arc<dyn AuthProvider>, roles: Vec<&str>) -> AuthChecker {
    let roles_string = roles.iter().map(|r| r.to_string()).collect();
    AuthChecker::new(provider)
        .require_roles(roles_string)
        .require_all_roles(false) // Any of the roles is sufficient
}

fn print_user_info(user_type: &str, roles: &[Role]) {
    println!("\n{} has roles:", user_type);
    for role in roles {
        println!("  - {}", role.name);
    }

    println!("  Permissions:");
    let permissions = get_permissions_for_roles(roles);
    for (resource, actions) in &permissions {
        for action in actions {
            println!("  - {}:{}", resource, action);
        }
    }
}

fn check_permission(action: &str, resource: &str, subject: &Subject, roles: &[Role]) {
    let permissions = get_permissions_for_roles(roles);
    let has_permission = permissions
        .get(resource)
        .map(|actions| actions.contains(&action.to_string()))
        .unwrap_or(false);

    println!(
        "{} {}:{} permission: {}",
        subject.name,
        action,
        resource,
        if has_permission { "✅" } else { "❌" }
    );
}

fn check_operations(
    operations: &[&str],
    checker: &AuthChecker,
    subject: &Subject,
    user_type: &str,
) {
    for operation in operations {
        let access = checker.check_roles(subject);
        println!(
            "  {} - '{}': {}",
            user_type,
            operation,
            if access {
                "✅ Access Granted"
            } else {
                "❌ Access Denied"
            }
        );
    }
}

// Helper function to extract permissions from roles
fn get_permissions_for_roles(roles: &[Role]) -> HashMap<String, Vec<String>> {
    let mut resource_permissions: HashMap<String, Vec<String>> = HashMap::new();

    for role in roles {
        if let Some(permissions) = &role.permissions {
            for permission in permissions {
                if let Some((resource, action)) = permission.split_once(':') {
                    resource_permissions
                        .entry(resource.to_string())
                        .or_default()
                        .push(action.to_string());
                }
            }
        }
    }

    resource_permissions
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(err) = main_func().await {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

/*
 * To run this example:
 *
 * ```
 * cargo run --example permissions --features basic
 * ```
 */
