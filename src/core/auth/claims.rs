use serde::{Deserialize, Serialize};

/// Standard OAuth2 claims that all providers must support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardClaims {
    /// Subject (user/client ID)
    pub sub: String,
    /// Audience
    pub aud: String,
    /// Expiration time
    pub exp: usize,
    /// Issued at
    pub iat: usize,
    /// Issuer
    pub iss: String,
    /// Scopes (space-separated string)
    #[serde(default)]
    pub scope: Option<String>,
}

impl StandardClaims {
    /// Get scopes as Vec<String>
    pub fn scopes(&self) -> Vec<String> {
        self.scope
            .as_deref()
            .unwrap_or_default()
            .split(' ')
            .map(String::from)
            .collect()
    }
}
