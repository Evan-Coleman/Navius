use crate::error::{EntraError, EntraResult};
use crate::token::EntraTokenClaims;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a user authenticated through Microsoft Entra ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraUser {
    /// Unique identifier for the user
    pub id: String,

    /// User's display name
    pub display_name: Option<String>,

    /// User's email address
    pub email: Option<String>,

    /// User's preferred username
    pub preferred_username: Option<String>,

    /// Object ID in Microsoft Entra
    pub object_id: Option<String>,

    /// Tenant ID the user belongs to
    pub tenant_id: Option<String>,

    /// Roles assigned to the user
    pub roles: Vec<String>,

    /// Additional claims from the token
    pub additional_claims: HashMap<String, serde_json::Value>,
}

impl EntraUser {
    /// Create a new EntraUser from token claims
    pub fn from_claims(claims: &EntraTokenClaims) -> EntraResult<Self> {
        // Extract roles from both roles and groups claims
        let mut roles = Vec::new();

        if let Some(role_values) = &claims.roles {
            roles.extend(role_values.clone());
        }

        if let Some(group_values) = &claims.groups {
            roles.extend(group_values.clone());
        }

        // Deduplicate roles
        roles.sort();
        roles.dedup();

        Ok(Self {
            id: claims.sub.clone(),
            display_name: claims.name.clone(),
            email: claims.email.clone(),
            preferred_username: claims.preferred_username.clone(),
            object_id: claims.object_id.clone(),
            tenant_id: claims.tenant_id.clone(),
            roles,
            additional_claims: claims.additional_claims.clone(),
        })
    }

    /// Create a new EntraUser from raw token claims
    pub fn from_token_claims(claims_value: serde_json::Value) -> EntraResult<Self> {
        // Try to parse the claims
        let claims: EntraTokenClaims = serde_json::from_value(claims_value)
            .map_err(|e| EntraError::user_profile(format!("Failed to parse claims: {}", e)))?;

        Self::from_claims(&claims)
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if the user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|&r| self.has_role(r))
    }

    /// Check if the user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|&r| self.has_role(r))
    }

    /// Get a claim value from the additional claims
    pub fn get_claim<T: for<'de> Deserialize<'de>>(&self, name: &str) -> Option<T> {
        self.additional_claims
            .get(name)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a claim as a string
    pub fn get_claim_as_string(&self, name: &str) -> Option<String> {
        self.additional_claims.get(name).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            _ => v.as_str().map(|s| s.to_string()),
        })
    }
}
