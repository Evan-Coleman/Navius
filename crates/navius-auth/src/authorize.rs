//! Authorization system for Navius Auth.
//!
//! This module provides functionality for checking permissions and roles
//! for authenticated subjects.

use crate::error::{Error, Result};
use crate::types::{Permission, Role, Subject};
use std::collections::HashMap;
use tracing::{debug, instrument};

/// Authorization checker for validating access to resources.
#[derive(Debug, Clone)]
pub struct Authorizer {
    /// Role to permission mapping.
    role_permissions: HashMap<String, Vec<Permission>>,
}

impl Default for Authorizer {
    fn default() -> Self {
        Self {
            role_permissions: HashMap::new(),
        }
    }
}

impl Authorizer {
    /// Create a new authorizer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a role with permissions.
    pub fn add_role(&mut self, role: &str, permissions: Vec<Permission>) -> &mut Self {
        self.role_permissions.insert(role.to_string(), permissions);
        self
    }

    /// Check if a subject has a specific role.
    #[instrument(skip(self, subject))]
    pub fn has_role(&self, subject: &Subject, role: &str) -> bool {
        subject.roles.iter().any(|r| r.name == role)
    }

    /// Check if a subject has any of the specified roles.
    #[instrument(skip(self, subject, roles))]
    pub fn has_any_role(&self, subject: &Subject, roles: &[&str]) -> bool {
        roles.iter().any(|&role| self.has_role(subject, role))
    }

    /// Check if a subject has all of the specified roles.
    #[instrument(skip(self, subject, roles))]
    pub fn has_all_roles(&self, subject: &Subject, roles: &[&str]) -> bool {
        roles.iter().all(|&role| self.has_role(subject, role))
    }

    /// Get all permissions for a subject based on their roles.
    #[instrument(skip(self, subject))]
    pub fn get_permissions(&self, subject: &Subject) -> Vec<Permission> {
        let mut permissions = Vec::new();

        for role in &subject.roles {
            if let Some(role_permissions) = self.role_permissions.get(&role.name) {
                permissions.extend(role_permissions.clone());
            }
        }

        permissions
    }

    /// Check if a subject has permission to access a resource with a specific action.
    #[instrument(skip(self, subject))]
    pub fn can(&self, subject: &Subject, resource: &str, action: &str) -> bool {
        // Get all permissions for the subject
        let permissions = self.get_permissions(subject);

        // Check if any permission matches the resource and action
        permissions.iter().any(|p| {
            // Check if permission matches exactly
            if p.resource == resource && p.action == action {
                return true;
            }

            // Check wildcard permissions
            if p.resource == "*" && (p.action == "*" || p.action == action) {
                return true;
            }

            if p.resource == resource && p.action == "*" {
                return true;
            }

            false
        })
    }

    /// Authorize a subject to access a resource with a specific action.
    /// Returns an error if the subject is not authorized.
    #[instrument(skip(self, subject))]
    pub fn authorize(&self, subject: &Subject, resource: &str, action: &str) -> Result<()> {
        if self.can(subject, resource, action) {
            debug!(
                "Subject {} authorized for {} on {}",
                subject.id, action, resource
            );
            Ok(())
        } else {
            debug!(
                "Subject {} not authorized for {} on {}",
                subject.id, action, resource
            );
            Err(Error::authorization_failed(format!(
                "Subject does not have permission to {} on {}",
                action, resource
            )))
        }
    }

    /// Create a middleware that checks for specific permissions on resources.
    pub fn middleware_for_resource(&self, resource: &str, action: &str) -> ResourceAuthMiddleware {
        ResourceAuthMiddleware {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }
}

/// Middleware for checking resource permissions.
#[derive(Debug, Clone)]
pub struct ResourceAuthMiddleware {
    /// Resource to check.
    resource: String,
    /// Action to check.
    action: String,
}

impl ResourceAuthMiddleware {
    /// Create a new resource authentication middleware.
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }

    /// Get the resource being protected.
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// Get the action being protected.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Check if a subject is authorized for this resource and action.
    pub fn check_authorization(&self, subject: &Subject, authorizer: &Authorizer) -> Result<()> {
        authorizer.authorize(subject, &self.resource, &self.action)
    }
}

/// Extension trait for working with subjects.
pub trait SubjectExt {
    /// Check if the subject has a specific role.
    fn has_role(&self, role: &str) -> bool;

    /// Check if the subject has any of the specified roles.
    fn has_any_role(&self, roles: &[&str]) -> bool;

    /// Check if the subject has all of the specified roles.
    fn has_all_roles(&self, roles: &[&str]) -> bool;
}

impl SubjectExt for Subject {
    fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.name == role)
    }

    fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|&role| self.has_role(role))
    }

    fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|&role| self.has_role(role))
    }
}
