//! Authentication and authorization data models.

/// JWT claims structure for authenticated tokens
#[derive(Debug, Clone)]
pub struct JwtClaims {
    /// Subject identifier (usually the user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Issued at time (Unix timestamp)
    pub iat: u64,
    /// Issuer identifier
    pub iss: String,
    /// Audience identifiers
    pub aud: Vec<String>,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
}

/// Token response from authentication service
#[derive(Debug, Clone)]
pub struct TokenResponse {
    /// JWT access token
    pub access_token: String,
    /// Token expiration time in seconds
    pub expires_in: u64,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Optional refresh token
    pub refresh_token: Option<String>,
}

/// User profile information
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// User ID
    pub id: String,
    /// User email address
    pub email: String,
    /// User display name
    pub name: String,
    /// Optional URL to user's profile picture
    pub picture: Option<String>,
}

/// Token claims for authentication
#[derive(Debug, Clone)]
pub struct TokenClaims {
    /// Subject (user identifier)
    pub sub: String,
    /// Token issuer
    pub iss: String,
    /// Token audience
    pub aud: String,
    /// Expiration time
    pub exp: i64,
    /// Not before time
    pub nbf: i64,
    /// Issued at time
    pub iat: i64,
    /// JWT ID
    pub jti: String,
    /// User roles
    pub roles: Vec<String>,
}
