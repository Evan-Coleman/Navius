use navius_auth::{authorize::Authorizer, Permission, Role, Subject};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_authorizer_role_checking() {
    // Create a subject with roles
    let subject = Subject {
        id: Uuid::new_v4().to_string(),
        subject_type: "user".to_string(),
        name: "Test User".to_string(),
        roles: vec![
            Role {
                id: Uuid::new_v4().to_string(),
                name: "user".to_string(),
                description: None,
                permissions: None,
            },
            Role {
                id: Uuid::new_v4().to_string(),
                name: "editor".to_string(),
                description: None,
                permissions: None,
            },
        ],
        attributes: None,
    };

    // Create authorizer
    let authorizer = Authorizer::new();

    // Test role checking
    assert!(
        authorizer.has_role(&subject, "user"),
        "Subject should have user role"
    );
    assert!(
        authorizer.has_role(&subject, "editor"),
        "Subject should have editor role"
    );
    assert!(
        !authorizer.has_role(&subject, "admin"),
        "Subject should not have admin role"
    );

    // Test multiple role checking
    assert!(
        authorizer.has_any_role(&subject, &["user", "admin"]),
        "Subject should have at least one of the roles"
    );
    assert!(
        !authorizer.has_any_role(&subject, &["admin", "superuser"]),
        "Subject should not have any of the roles"
    );
    assert!(
        authorizer.has_all_roles(&subject, &["user", "editor"]),
        "Subject should have all the roles"
    );
    assert!(
        !authorizer.has_all_roles(&subject, &["user", "admin"]),
        "Subject should not have all the roles"
    );
}

#[test]
fn test_authorizer_permission_checking() {
    // Create an authorizer with roles and permissions
    let mut authorizer = Authorizer::new();

    // Add roles with permissions
    authorizer.add_role(
        "user",
        vec![
            Permission {
                id: Uuid::new_v4().to_string(),
                resource: "articles".to_string(),
                action: "read".to_string(),
                conditions: None,
            },
            Permission {
                id: Uuid::new_v4().to_string(),
                resource: "comments".to_string(),
                action: "create".to_string(),
                conditions: None,
            },
        ],
    );

    authorizer.add_role(
        "editor",
        vec![
            Permission {
                id: Uuid::new_v4().to_string(),
                resource: "articles".to_string(),
                action: "create".to_string(),
                conditions: None,
            },
            Permission {
                id: Uuid::new_v4().to_string(),
                resource: "articles".to_string(),
                action: "update".to_string(),
                conditions: None,
            },
        ],
    );

    authorizer.add_role(
        "admin",
        vec![Permission {
            id: Uuid::new_v4().to_string(),
            resource: "*".to_string(),
            action: "*".to_string(),
            conditions: None,
        }],
    );

    // Create subjects with different roles
    let user_subject = Subject {
        id: Uuid::new_v4().to_string(),
        subject_type: "user".to_string(),
        name: "Regular User".to_string(),
        roles: vec![Role {
            id: Uuid::new_v4().to_string(),
            name: "user".to_string(),
            description: None,
            permissions: None,
        }],
        attributes: Some(HashMap::new()),
    };

    let editor_subject = Subject {
        id: Uuid::new_v4().to_string(),
        subject_type: "user".to_string(),
        name: "Editor User".to_string(),
        roles: vec![
            Role {
                id: Uuid::new_v4().to_string(),
                name: "user".to_string(),
                description: None,
                permissions: None,
            },
            Role {
                id: Uuid::new_v4().to_string(),
                name: "editor".to_string(),
                description: None,
                permissions: None,
            },
        ],
        attributes: Some(HashMap::new()),
    };

    let admin_subject = Subject {
        id: Uuid::new_v4().to_string(),
        subject_type: "user".to_string(),
        name: "Admin User".to_string(),
        roles: vec![Role {
            id: Uuid::new_v4().to_string(),
            name: "admin".to_string(),
            description: None,
            permissions: None,
        }],
        attributes: Some(HashMap::new()),
    };

    // Test user permissions
    assert!(
        authorizer.can(&user_subject, "articles", "read"),
        "User should be able to read articles"
    );
    assert!(
        authorizer.can(&user_subject, "comments", "create"),
        "User should be able to create comments"
    );
    assert!(
        !authorizer.can(&user_subject, "articles", "update"),
        "User should not be able to update articles"
    );

    // Test editor permissions
    assert!(
        authorizer.can(&editor_subject, "articles", "read"),
        "Editor should be able to read articles"
    );
    assert!(
        authorizer.can(&editor_subject, "articles", "create"),
        "Editor should be able to create articles"
    );
    assert!(
        authorizer.can(&editor_subject, "articles", "update"),
        "Editor should be able to update articles"
    );
    assert!(
        !authorizer.can(&editor_subject, "articles", "delete"),
        "Editor should not be able to delete articles"
    );

    // Test admin permissions
    assert!(
        authorizer.can(&admin_subject, "articles", "read"),
        "Admin should be able to read articles"
    );
    assert!(
        authorizer.can(&admin_subject, "articles", "delete"),
        "Admin should be able to delete articles"
    );
    assert!(
        authorizer.can(&admin_subject, "users", "manage"),
        "Admin should be able to manage users"
    );

    // Test authorization method
    let read_result = authorizer.authorize(&user_subject, "articles", "read");
    assert!(
        read_result.is_ok(),
        "User should be authorized to read articles"
    );

    let delete_result = authorizer.authorize(&user_subject, "articles", "delete");
    assert!(
        delete_result.is_err(),
        "User should not be authorized to delete articles"
    );
}

#[test]
fn test_subject_ext_trait() {
    use navius_auth::authorize::SubjectExt;

    // Create a subject with roles
    let subject = Subject {
        id: Uuid::new_v4().to_string(),
        subject_type: "user".to_string(),
        name: "Test User".to_string(),
        roles: vec![
            Role {
                id: Uuid::new_v4().to_string(),
                name: "user".to_string(),
                description: None,
                permissions: None,
            },
            Role {
                id: Uuid::new_v4().to_string(),
                name: "editor".to_string(),
                description: None,
                permissions: None,
            },
        ],
        attributes: None,
    };

    // Test SubjectExt trait methods
    assert!(subject.has_role("user"), "Subject should have user role");
    assert!(
        subject.has_role("editor"),
        "Subject should have editor role"
    );
    assert!(
        !subject.has_role("admin"),
        "Subject should not have admin role"
    );

    assert!(
        subject.has_any_role(&["user", "admin"]),
        "Subject should have at least one of the roles"
    );
    assert!(
        !subject.has_any_role(&["admin", "superuser"]),
        "Subject should not have any of the roles"
    );

    assert!(
        subject.has_all_roles(&["user", "editor"]),
        "Subject should have all the roles"
    );
    assert!(
        !subject.has_all_roles(&["user", "admin"]),
        "Subject should not have all the roles"
    );
}
