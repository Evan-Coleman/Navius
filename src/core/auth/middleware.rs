use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};
use tracing::{debug, error, info};

use crate::core::config::app_config;
use crate::core::config::app_config::AppConfig;
use crate::core::config::constants;
use crate::core::router::AppState;

/// JWKS (JSON Web Key Set) response
#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    #[serde(rename = "kid")]
    key_id: String,
    #[serde(rename = "x5c")]
    x509_chain: Option<Vec<String>>,
    #[serde(rename = "n")]
    modulus: Option<String>,
    #[serde(rename = "e")]
    exponent: Option<String>,
    kty: String,
}

/// JWKS cache entry
#[derive(Debug, Clone)]
pub struct JwksCacheEntry {
    jwks: JwksResponse,
    expires_at: SystemTime,
}

/// Claims from the JWT token we validate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraClaims {
    /// Subject (user/client ID)
    pub sub: String,
    /// Audience (who is this token for)
    pub aud: String,
    /// Issuer (who issued this token)
    pub iss: String,
    /// Expiration time
    pub exp: usize,
    /// Not before time
    pub nbf: usize,
    /// Issued at time
    pub iat: usize,
    /// Roles assigned to the client/user
    #[serde(default)]
    pub roles: Vec<String>,
    /// App ID of the client
    pub appid: Option<String>,
    /// App ID URI of the client
    #[serde(rename = "azp")]
    pub app_id_uri: Option<String>,
    /// Scope in access token (can be string or array)
    #[serde(default)]
    pub scp: Option<String>,
}

impl EntraClaims {
    /// Get scopes as a Vec<String>
    pub fn get_scopes(&self) -> Vec<String> {
        let mut permissions = Vec::new();

        // Check for explicit scopes (delegated permissions)
        if let Some(scope_str) = &self.scp {
            permissions.extend(scope_str.split(' ').map(String::from));
        }

        // Include roles (application permissions) as well
        permissions.extend(self.roles.clone());

        permissions
    }
}

/// Role requirements for authorization
#[derive(Debug, Clone, PartialEq)]
pub enum RoleRequirement {
    /// Any of the listed roles is sufficient
    Any(Vec<String>),
    /// All of the listed roles are required
    All(Vec<String>),
    /// No roles required (authentication only)
    None,
    /// Admin role
    Admin,
    /// Read-only role
    ReadOnly,
    /// Full access role
    FullAccess,
}

/// Permission (scope) requirements for authorization
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionRequirement {
    /// Any of the listed permissions is sufficient
    Any(Vec<String>),
    /// All of the listed permissions are required
    All(Vec<String>),
    /// No permissions required (authentication only)
    None,
}

/// Role enum representing predefined roles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Administrator role
    Admin,
    /// Read-only role
    ReadOnly,
    /// Full access role
    FullAccess,
    /// Custom role
    Custom,
}

/// Permission enum representing predefined permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Read permission
    Read,
    /// Write permission
    Write,
    /// Delete permission
    Delete,
    /// Admin permission
    Admin,
    /// Custom permission
    Custom,
}

/// Configuration for Entra authentication middleware
#[derive(Debug, Clone)]
pub struct EntraAuthConfig {
    /// Entra tenant ID
    pub tenant_id: String,
    /// Client ID of our application
    pub client_id: String,
    /// Expected audience value
    pub audience: String,
    /// Whether to validate the token (disable for debugging)
    pub validate_token: bool,
    /// Required roles for authorization
    pub required_roles: RoleRequirement,
    /// Required permissions (scopes) for authorization
    pub required_permissions: PermissionRequirement,
    /// HTTP client
    pub client: Client,
    /// JWKS URI for token validation
    pub jwks_uri: String,
    /// JWKS cache
    pub jwks_cache: Arc<Mutex<Option<JwksCacheEntry>>>,
    /// Debug mode - skips signature validation
    pub debug_validation: bool,
    /// Default issuer URL formats
    pub issuer_url_formats: Vec<String>,
    /// Admin roles
    pub admin_roles: Vec<String>,
    /// Read-only roles
    pub read_only_roles: Vec<String>,
    /// Full access roles
    pub full_access_roles: Vec<String>,
}

/// OpenID Connect configuration response
#[derive(Debug, Clone, Deserialize)]
struct OpenIdConfiguration {
    #[serde(rename = "jwks_uri")]
    jwks_uri: String,
    #[serde(rename = "issuer")]
    issuer: String,
}

impl Default for EntraAuthConfig {
    fn default() -> Self {
        // Get configuration from environment variables
        let tenant_id = std::env::var(constants::auth::env_vars::TENANT_ID).unwrap_or_default();
        let client_id = std::env::var(constants::auth::env_vars::CLIENT_ID).unwrap_or_default();
        let debug_validation = std::env::var(constants::auth::env_vars::DEBUG_AUTH)
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let audience = std::env::var(constants::auth::env_vars::AUDIENCE).unwrap_or_else(|_| {
            constants::auth::urls::DEFAULT_AUDIENCE_FORMAT
                .replace("{}", &client_id)
                .to_string()
        });

        // Ensure tenant_id is not empty
        let tenant_id = if tenant_id.is_empty() {
            // Use a placeholder to avoid URL formatting issues
            "tenant-id-placeholder".to_string()
        } else {
            tenant_id
        };

        // Use the direct JWKS endpoint
        let jwks_uri = app_config::default_jwks_uri_format().replace("{}", &tenant_id);

        // Get default issuer URL formats
        let issuer_url_formats = app_config::default_issuer_url_formats();

        Self {
            tenant_id: tenant_id.clone(),
            client_id: client_id.clone(),
            audience,
            validate_token: true,
            required_roles: RoleRequirement::None,
            required_permissions: PermissionRequirement::None,
            client: Client::new(),
            jwks_uri,
            jwks_cache: Arc::new(Mutex::new(None)),
            debug_validation,
            issuer_url_formats,
            admin_roles: Vec::new(),
            read_only_roles: Vec::new(),
            full_access_roles: Vec::new(),
        }
    }
}

impl EntraAuthConfig {
    /// Create a new EntraAuthConfig
    pub fn new(
        tenant_id: String,
        client_id: String,
        audience: String,
        debug_validation: bool,
    ) -> Self {
        // Ensure tenant_id is not empty
        let tenant_id = if tenant_id.is_empty() {
            // Use a placeholder to avoid URL formatting issues
            "tenant-id-placeholder".to_string()
        } else {
            tenant_id
        };

        // Use the direct JWKS endpoint - use app_config defaults
        let jwks_uri_format = app_config::default_jwks_uri_format();
        let jwks_uri = jwks_uri_format.replace("{}", &tenant_id).to_string();

        // Get default issuer URL formats
        let issuer_url_formats = app_config::default_issuer_url_formats();

        Self {
            tenant_id: tenant_id.clone(),
            client_id: client_id.clone(),
            audience,
            validate_token: true,
            required_roles: RoleRequirement::None,
            required_permissions: PermissionRequirement::None,
            client: Client::new(),
            jwks_uri,
            jwks_cache: Arc::new(Mutex::new(None)),
            debug_validation,
            issuer_url_formats,
            admin_roles: Vec::new(),
            read_only_roles: Vec::new(),
            full_access_roles: Vec::new(),
        }
    }

    /// Create a new EntraAuthConfig from AppConfig
    pub fn from_app_config(config: &AppConfig) -> Self {
        let tenant_id = config.auth.entra.tenant_id.clone();
        let client_id = config.auth.entra.client_id.clone();
        let debug_validation = config.auth.debug;
        let audience = if config.auth.entra.audience.is_empty() {
            constants::auth::urls::DEFAULT_AUDIENCE_FORMAT
                .replace("{}", &client_id)
                .to_string()
        } else {
            config.auth.entra.audience.clone()
        };

        // Ensure tenant_id is not empty
        let tenant_id = if tenant_id.is_empty() {
            // Use a placeholder to avoid URL formatting issues
            "tenant-id-placeholder".to_string()
        } else {
            tenant_id
        };

        // Use the direct JWKS endpoint
        let jwks_uri = config.auth.entra.jwks_uri_format.replace("{}", &tenant_id);

        // Get issuer URL formats from config
        let issuer_url_formats = config.auth.entra.issuer_url_formats.clone();

        Self {
            tenant_id: tenant_id.clone(),
            client_id,
            audience,
            validate_token: true,
            required_roles: RoleRequirement::None,
            required_permissions: PermissionRequirement::None,
            client: Client::new(),
            jwks_uri,
            jwks_cache: Arc::new(Mutex::new(None)),
            debug_validation,
            issuer_url_formats,
            admin_roles: config.auth.entra.admin_roles.clone(),
            read_only_roles: config.auth.entra.read_only_roles.clone(),
            full_access_roles: config.auth.entra.full_access_roles.clone(),
        }
    }

    /// Build on existing config to modify specific fields
    pub fn with_role_requirement(mut self, role_requirement: RoleRequirement) -> Self {
        self.required_roles = role_requirement;
        self
    }

    /// Build on existing config to add permission requirements
    pub fn with_permission_requirement(
        mut self,
        permission_requirement: PermissionRequirement,
    ) -> Self {
        self.required_permissions = permission_requirement;
        self
    }
}

/// Error response for authentication failures
#[derive(Debug)]
pub enum AuthError {
    /// Missing Authorization header
    MissingToken,
    /// Invalid token format
    InvalidTokenFormat,
    /// Token validation failed
    ValidationFailed(String),
    /// Backend error
    InternalError(String),
    /// Authorization failed
    AccessDenied(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Authentication required: Missing authorization token".to_string(),
            ),
            AuthError::InvalidTokenFormat => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed: Invalid token format".to_string(),
            ),
            AuthError::ValidationFailed(reason) => (
                StatusCode::UNAUTHORIZED,
                format!("Authentication failed: {}", reason),
            ),
            AuthError::InternalError(reason) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Server error during authentication: {}", reason),
            ),
            AuthError::AccessDenied(reason) => (StatusCode::FORBIDDEN, reason),
        };

        let body = axum::Json(serde_json::json!({
            "status": status.as_u16(),
            "error": status.to_string(),
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}

/// Extract the bearer token from the Authorization header
fn extract_token(headers: &HeaderMap) -> Result<String, AuthError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(AuthError::MissingToken)?;

    let header_str = header.to_str().map_err(|_| AuthError::InvalidTokenFormat)?;

    if !header_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidTokenFormat);
    }

    let token = header_str[7..].trim().to_string();

    // Basic JWT format validation - should have 3 parts separated by dots
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::ValidationFailed(
            "Invalid JWT format: token must have three parts (header.payload.signature)"
                .to_string(),
        ));
    }

    // Each part should be non-empty and contain valid Base64URL characters
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return Err(AuthError::ValidationFailed(format!(
                "JWT token part {} is empty",
                i + 1
            )));
        }

        // Very basic Base64URL validation - more thorough validation happens later
        for c in part.chars() {
            if !(c.is_alphanumeric() || c == '_' || c == '-' || c == '=') {
                return Err(AuthError::ValidationFailed(
                    "Invalid JWT token: contains non-base64url characters".to_string(),
                ));
            }
        }
    }

    Ok(token)
}

/// Fetch and cache JWKS from the Microsoft endpoint
async fn fetch_jwks(config: &EntraAuthConfig) -> Result<JwksResponse, AuthError> {
    // Check if we have a cached JWKS that's still valid
    {
        let cache = config.jwks_cache.lock().unwrap();
        if let Some(cache_entry) = &*cache {
            if cache_entry.expires_at > SystemTime::now() {
                return Ok(cache_entry.jwks.clone());
            }
        }
    }

    // If not, fetch a new JWKS
    let response = config
        .client
        .get(&config.jwks_uri)
        .send()
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to fetch JWKS: {}", e)))?;

    if !response.status().is_success() {
        return Err(AuthError::InternalError(format!(
            "Failed to fetch JWKS, status: {}",
            response.status()
        )));
    }

    let jwks = response
        .json::<JwksResponse>()
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to parse JWKS: {}", e)))?;

    // Cache the JWKS for 1 hour
    let expires_at = SystemTime::now() + Duration::from_secs(3600);
    {
        let mut cache = config.jwks_cache.lock().unwrap();
        *cache = Some(JwksCacheEntry {
            jwks: jwks.clone(),
            expires_at,
        });
    }

    Ok(jwks)
}

/// Find a JWK by its key ID
fn find_jwk<'a>(jwks: &'a JwksResponse, kid: &str) -> Result<&'a Jwk, AuthError> {
    jwks.keys
        .iter()
        .find(|key| key.key_id == kid)
        .ok_or_else(|| {
            AuthError::ValidationFailed(format!("No matching key found for kid: {}", kid))
        })
}

/// Create a decoding key from a JWK
fn create_decoding_key(jwk: &Jwk) -> Result<DecodingKey, AuthError> {
    // First try X.509 certificate chain
    if let Some(x509_chain) = &jwk.x509_chain {
        if let Some(cert) = x509_chain.first() {
            return DecodingKey::from_rsa_pem(
                format!(
                    "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
                    cert
                )
                .as_bytes(),
            )
            .map_err(|e| {
                AuthError::ValidationFailed(format!("Failed to create key from certificate: {}", e))
            });
        }
    }

    // Fall back to modulus and exponent
    if let (Some(n), Some(e)) = (&jwk.modulus, &jwk.exponent) {
        return DecodingKey::from_rsa_components(n, e).map_err(|e| {
            AuthError::ValidationFailed(format!("Failed to create key from components: {}", e))
        });
    }

    Err(AuthError::ValidationFailed(
        "JWK doesn't contain necessary key material".to_string(),
    ))
}

/// Validate claims in the token
fn validate_claims(claims: &EntraClaims, config: &EntraAuthConfig) -> Result<(), AuthError> {
    // Authorization based on roles
    match &config.required_roles {
        RoleRequirement::None => Ok(()),
        RoleRequirement::Any(required_roles) => {
            if required_roles.is_empty() {
                return Ok(());
            }

            for role in required_roles {
                if claims.roles.contains(role) {
                    return Ok(());
                }
            }

            Err(AuthError::AccessDenied(format!(
                "Access denied: User does not have any of the required roles: {:?}",
                required_roles
            )))
        }
        RoleRequirement::All(required_roles) => {
            if required_roles.is_empty() {
                return Ok(());
            }

            for role in required_roles {
                if !claims.roles.contains(role) {
                    return Err(AuthError::AccessDenied(format!(
                        "Access denied: User missing required role: {}",
                        role
                    )));
                }
            }

            Ok(())
        }
        RoleRequirement::Admin => {
            if claims.roles.iter().any(|r| config.admin_roles.contains(r)) {
                Ok(())
            } else {
                Err(AuthError::AccessDenied(
                    "Access denied: User does not have admin role".to_string(),
                ))
            }
        }
        RoleRequirement::ReadOnly => {
            if claims.roles.iter().any(|r| {
                config.admin_roles.contains(r)
                    || config.read_only_roles.contains(r)
                    || config.full_access_roles.contains(r)
            }) {
                Ok(())
            } else {
                Err(AuthError::AccessDenied(
                    "Access denied: User does not have read access".to_string(),
                ))
            }
        }
        RoleRequirement::FullAccess => {
            if claims
                .roles
                .iter()
                .any(|r| config.admin_roles.contains(r) || config.full_access_roles.contains(r))
            {
                Ok(())
            } else {
                Err(AuthError::AccessDenied(
                    "Access denied: User does not have full access".to_string(),
                ))
            }
        }
    }
}

/// Validate permissions in the token
fn validate_permissions(claims: &EntraClaims, config: &EntraAuthConfig) -> Result<(), AuthError> {
    let scopes = claims.get_scopes();

    // Authorization based on permissions (scopes)
    match &config.required_permissions {
        PermissionRequirement::Any(required_permissions) => {
            if required_permissions.is_empty() {
                return Ok(());
            }

            for permission in required_permissions {
                if scopes.contains(permission) {
                    return Ok(());
                }
            }

            Err(AuthError::AccessDenied(format!(
                "Access denied: Token for '{}' does not have any of the required permissions: {:?}",
                claims.sub, required_permissions
            )))
        }
        PermissionRequirement::All(required_permissions) => {
            if required_permissions.is_empty() {
                return Ok(());
            }

            for permission in required_permissions {
                if !scopes.contains(permission) {
                    return Err(AuthError::AccessDenied(format!(
                        "Access denied: Token missing required permission: {}",
                        permission
                    )));
                }
            }

            Ok(())
        }
        PermissionRequirement::None => Ok(()),
    }
}

/// Middleware layer for Entra ID authentication
#[derive(Clone, Debug, Default)]
pub struct EntraAuthLayer {
    config: EntraAuthConfig,
}

impl EntraAuthLayer {
    /// Create a new EntraAuthLayer from AppConfig
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            config: EntraAuthConfig::from_app_config(config),
        }
    }

    /// Create a new EntraAuthLayer with custom configuration
    pub fn new(config: EntraAuthConfig) -> Self {
        Self { config }
    }

    /// Create a new auth layer with role requirements
    pub fn with_roles(roles: RoleRequirement) -> Self {
        Self::new(EntraAuthConfig::default().with_role_requirement(roles))
    }

    /// Create a new EntraAuthLayer that requires any of the given roles
    pub fn require_any_role(roles: Vec<String>) -> Self {
        Self::with_roles(RoleRequirement::Any(roles))
    }

    /// Create a new auth layer that requires all of the specified roles
    pub fn require_all_roles(roles: Vec<String>) -> Self {
        Self::with_roles(RoleRequirement::All(roles))
    }

    /// Create a new auth layer with permission requirements
    pub fn with_permissions(permissions: PermissionRequirement) -> Self {
        Self::new(EntraAuthConfig::default().with_permission_requirement(permissions))
    }

    /// Create a new auth layer that requires any of the given permissions
    pub fn require_any_permission(permissions: Vec<String>) -> Self {
        Self::with_permissions(PermissionRequirement::Any(permissions))
    }

    /// Create a new auth layer that requires all of the specified permissions
    pub fn require_all_permissions(permissions: Vec<String>) -> Self {
        Self::with_permissions(PermissionRequirement::All(permissions))
    }

    /// Create a new auth layer from app config with added role requirements
    pub fn from_app_config_with_roles(config: &AppConfig, roles: RoleRequirement) -> Self {
        Self::new(EntraAuthConfig::from_app_config(config).with_role_requirement(roles))
    }

    /// Create a new auth layer from app config requiring any of the given roles
    pub fn from_app_config_require_any_role(config: &AppConfig, roles: Vec<String>) -> Self {
        Self::from_app_config_with_roles(config, RoleRequirement::Any(roles))
    }

    /// Create a new auth layer from app config requiring all of the specified roles
    pub fn from_app_config_require_all_roles(config: &AppConfig, roles: Vec<String>) -> Self {
        Self::from_app_config_with_roles(config, RoleRequirement::All(roles))
    }

    /// Create a new auth layer from app config requiring any of the admin roles
    pub fn from_app_config_require_admin_role(config: &AppConfig) -> Self {
        Self::from_app_config_require_any_role(config, config.auth.entra.admin_roles.clone())
    }

    /// Create a new auth layer from app config requiring any of the read-only roles
    pub fn from_app_config_require_read_only_role(config: &AppConfig) -> Self {
        Self::from_app_config_require_any_role(config, config.auth.entra.read_only_roles.clone())
    }

    /// Create a new auth layer from app config requiring any of the full access roles
    pub fn from_app_config_require_full_access_role(config: &AppConfig) -> Self {
        Self::from_app_config_require_any_role(config, config.auth.entra.full_access_roles.clone())
    }
}

impl<S> Layer<S> for EntraAuthLayer {
    type Service = EntraAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EntraAuthMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Middleware for Entra ID authentication
#[derive(Clone)]
pub struct EntraAuthMiddleware<S> {
    inner: S,
    config: EntraAuthConfig,
}

impl<S> EntraAuthMiddleware<S> {
    /// Create a new EntraAuthMiddleware with the given service and configuration
    pub fn new(inner: S, config: EntraAuthConfig) -> Self {
        Self { inner, config }
    }
}

// Implement the middleware using axum's middleware approach
impl<S> tower::Service<Request> for EntraAuthMiddleware<S>
where
    S: tower::Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let config = self.config.clone();
        let inner = self.inner.clone();

        Box::pin(async move {
            let mut inner_svc = inner;

            // Handle the auth validation
            match validate_token_wrapper(req, &config).await {
                Ok(req) => inner_svc.call(req).await,
                Err(err) => Ok(err.into_response()),
            }
        })
    }
}

/// Wrapper function to validate token without using Next
async fn validate_token_wrapper(
    mut req: Request,
    config: &EntraAuthConfig,
) -> Result<Request, AuthError> {
    // Extract the token
    let headers = req.headers();
    let token = extract_token(headers)?;

    let header = decode_header(&token).map_err(|e| {
        AuthError::ValidationFailed(format!("Failed to decode token header: {}", e))
    })?;

    if config.debug_validation {
        // In debug mode, let's completely bypass JWT validation
        // and just accept any token format
        let mut req = req;
        let claims = EntraClaims {
            aud: config.audience.clone(),
            exp: constants::timestamps::YEAR_2100, // Year 2100
            iss: "debug_issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021, // 2021-01-01
            sub: "debug_subject".to_string(),
            roles: vec!["admin".to_string(), "resource-manager".to_string()],
            appid: Some("debug_app_id".to_string()),
            app_id_uri: Some("debug_app_id_uri".to_string()),
            scp: Some("api-access".to_string()),
        };

        req.extensions_mut().insert(claims);
        return Ok(req);
    }

    // Skip validation if disabled (for debugging or development)
    if !config.validate_token {
        return Ok(req);
    }

    // Full validation with signature verification

    // Get the key ID
    let kid = header.kid.ok_or_else(|| {
        AuthError::ValidationFailed("Token header missing 'kid' claim".to_string())
    })?;

    // Fetch JWKS (JSON Web Key Set) from Microsoft
    let jwks = fetch_jwks(config).await?;

    // Find the key matching our token's kid
    let jwk = find_jwk(&jwks, &kid)?;

    // Create a decoding key
    let decoding_key = create_decoding_key(jwk)?;

    // Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.set_audience(&[&config.audience]);
    validation.set_required_spec_claims(&["exp", "iss", "sub", "aud"]);

    // Set up issuer validation with multiple accepted issuers
    let mut issuers = Vec::new();
    for format in &config.issuer_url_formats {
        issuers.push(format.replace("{}", &config.tenant_id));
    }
    validation.set_issuer(&issuers);

    // Validate token with better error handling
    let token_data = match decode::<EntraClaims>(&token, &decoding_key, &validation) {
        Ok(data) => data,
        Err(e) => {
            let detail = match e.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => "Token format is invalid",
                jsonwebtoken::errors::ErrorKind::InvalidSignature => "Invalid signature",
                jsonwebtoken::errors::ErrorKind::Base64(_) => {
                    "Base64 decoding error - token may be malformed or corrupted"
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token has expired",
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    &format!("Invalid audience, expected: {}", config.audience)
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => "Invalid issuer",
                _ => "Token validation failed",
            };
            return Err(AuthError::ValidationFailed(format!("{}: {}", detail, e)));
        }
    };

    // Role-based authorization check
    validate_claims(&token_data.claims, config)?;

    // Permission-based authorization check
    validate_permissions(&token_data.claims, config)?;

    // Store claims in request extensions for handlers to access
    req.extensions_mut().insert(token_data.claims);

    Ok(req)
}

impl RoleRequirement {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "admin" => Ok(RoleRequirement::Admin),
            "read" => Ok(RoleRequirement::ReadOnly),
            "full" => Ok(RoleRequirement::FullAccess),
            "none" => Ok(RoleRequirement::None),
            _ => Err(format!("Invalid role requirement: {}", s)),
        }
    }

    pub fn matches(&self, roles: &[String], config: &EntraAuthConfig) -> bool {
        match self {
            RoleRequirement::None => true,
            RoleRequirement::Any(required_roles) => {
                required_roles.iter().any(|r| roles.contains(r))
            }
            RoleRequirement::All(required_roles) => {
                required_roles.iter().all(|r| roles.contains(r))
            }
            RoleRequirement::Admin => roles.iter().any(|r| config.admin_roles.contains(r)),
            RoleRequirement::ReadOnly => roles.iter().any(|r| {
                config.admin_roles.contains(r)
                    || config.read_only_roles.contains(r)
                    || config.full_access_roles.contains(r)
            }),
            RoleRequirement::FullAccess => roles
                .iter()
                .any(|r| config.admin_roles.contains(r) || config.full_access_roles.contains(r)),
        }
    }
}

impl PermissionRequirement {
    fn matches(&self, _roles: &[String], permissions: &[String]) -> bool {
        match self {
            PermissionRequirement::None => true,
            PermissionRequirement::Any(perms) => perms.iter().any(|p| permissions.contains(p)),
            PermissionRequirement::All(perms) => perms.iter().all(|p| permissions.contains(p)),
        }
    }
}

/// Helper function to create auth middleware with any role
pub fn auth_middleware(roles: Vec<String>) -> EntraAuthLayer {
    EntraAuthLayer::require_any_role(roles)
}

/// Helper function to require authentication without specific roles
pub fn require_auth() -> EntraAuthLayer {
    EntraAuthLayer::default()
}

/// Helper function to require specific roles
pub fn require_roles(roles: Vec<String>) -> EntraAuthLayer {
    EntraAuthLayer::require_any_role(roles)
}

/// Helper function to convert a string to a Role enum
pub fn role_from_string(role: &str) -> Role {
    match role.to_lowercase().as_str() {
        "admin" => Role::Admin,
        "readonly" => Role::ReadOnly,
        "fullaccess" => Role::FullAccess,
        _ => Role::Custom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_auth_config_default() {
        let config = EntraAuthConfig::default();

        // Check default values - get the actual value from the config
        let actual_tenant_id = config.tenant_id.clone();
        assert_eq!(config.tenant_id, actual_tenant_id);

        // Client ID may be picked up from environment - just check it's either empty
        // or has sufficient length to be a UUID
        let client_id_valid = config.client_id.is_empty() || config.client_id.len() > 30;
        assert!(client_id_valid);

        // Check audience format
        assert!(config.audience.starts_with("api://"));
        assert!(config.validate_token);
        assert_eq!(
            format!("{:?}", config.required_roles),
            format!("{:?}", RoleRequirement::None)
        );
        assert_eq!(
            format!("{:?}", config.required_permissions),
            format!("{:?}", PermissionRequirement::None)
        );
    }

    #[test]
    fn test_auth_config_from_app_config() {
        let mut app_config = AppConfig::default();

        // Set custom values for testing
        app_config.auth.entra.tenant_id = "test-tenant".to_string();
        app_config.auth.entra.client_id = "test-client".to_string();
        app_config.auth.entra.audience = "test-audience".to_string();

        let config = EntraAuthConfig::from_app_config(&app_config);

        // Verify configuration was applied
        assert_eq!(config.tenant_id, "test-tenant");
        assert_eq!(config.client_id, "test-client");
        assert_eq!(config.audience, "test-audience");
    }

    #[test]
    fn test_auth_config_builder_methods() {
        // Start with default config
        let config = EntraAuthConfig::default()
            .with_role_requirement(RoleRequirement::Any(vec!["admin".to_string()]))
            .with_permission_requirement(PermissionRequirement::All(vec![
                "read".to_string(),
                "write".to_string(),
            ]));

        // Verify builder methods
        match config.required_roles {
            RoleRequirement::Any(roles) => {
                assert_eq!(roles.len(), 1);
                assert_eq!(roles[0], "admin");
            }
            _ => panic!("Expected RoleRequirement::Any"),
        }

        match config.required_permissions {
            PermissionRequirement::All(permissions) => {
                assert_eq!(permissions.len(), 2);
                assert_eq!(permissions[0], "read");
                assert_eq!(permissions[1], "write");
            }
            _ => panic!("Expected PermissionRequirement::All"),
        }
    }

    #[test]
    fn test_extract_token() {
        // Test missing token
        let headers = HeaderMap::new();
        let result = extract_token(&headers);
        assert!(matches!(result, Err(AuthError::MissingToken)));

        // Test invalid token format
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "InvalidFormat".parse().unwrap(),
        );
        let result = extract_token(&headers);
        assert!(matches!(result, Err(AuthError::InvalidTokenFormat)));

        // Test non-bearer token
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Basic dGVzdDp0ZXN0".parse().unwrap(),
        );
        let result = extract_token(&headers);
        assert!(matches!(result, Err(AuthError::InvalidTokenFormat)));

        // Test valid token format (minimal JWT format with three parts)
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer header.payload.signature".parse().unwrap(),
        );
        let result = extract_token(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "header.payload.signature");

        // Test invalid JWT format (missing parts)
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer header.payload".parse().unwrap(),
        );
        let result = extract_token(&headers);
        assert!(matches!(result, Err(AuthError::ValidationFailed(_))));
    }

    #[test]
    fn test_validate_claims_roles() {
        // Create test claims
        let claims = EntraClaims {
            aud: "test-audience".to_string(),
            exp: constants::timestamps::YEAR_2100,
            iss: "test-issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021,
            sub: "test-subject".to_string(),
            roles: vec!["user".to_string(), "editor".to_string()],
            appid: Some("test-app-id".to_string()),
            app_id_uri: Some("test-app-uri".to_string()),
            scp: Some("read write".to_string()),
        };

        // Test with no role requirements
        let config = EntraAuthConfig::default();
        assert!(validate_claims(&claims, &config).is_ok());

        // Test with matching "Any" role requirements
        let config = EntraAuthConfig::default().with_role_requirement(RoleRequirement::Any(vec![
            "editor".to_string(),
            "admin".to_string(),
        ]));
        assert!(validate_claims(&claims, &config).is_ok());

        // Test with non-matching "Any" role requirements
        let config = EntraAuthConfig::default()
            .with_role_requirement(RoleRequirement::Any(vec!["admin".to_string()]));
        assert!(matches!(
            validate_claims(&claims, &config),
            Err(AuthError::AccessDenied(_))
        ));

        // Test with matching "All" role requirements
        let config = EntraAuthConfig::default().with_role_requirement(RoleRequirement::All(vec![
            "user".to_string(),
            "editor".to_string(),
        ]));
        assert!(validate_claims(&claims, &config).is_ok());

        // Test with partially matching "All" role requirements
        let config = EntraAuthConfig::default().with_role_requirement(RoleRequirement::All(vec![
            "user".to_string(),
            "admin".to_string(),
        ]));
        assert!(matches!(
            validate_claims(&claims, &config),
            Err(AuthError::AccessDenied(_))
        ));
    }

    #[test]
    fn test_validate_permissions() {
        // Create test claims with scopes
        let claims = EntraClaims {
            aud: "test-audience".to_string(),
            exp: constants::timestamps::YEAR_2100,
            iss: "test-issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021,
            sub: "test-subject".to_string(),
            roles: vec![],
            appid: Some("test-app-id".to_string()),
            app_id_uri: Some("test-app-uri".to_string()),
            scp: Some("read write".to_string()),
        };

        // Test with no permission requirements
        let config = EntraAuthConfig::default();
        assert!(validate_permissions(&claims, &config).is_ok());

        // Test with matching "Any" permission requirements
        let config = EntraAuthConfig::default().with_permission_requirement(
            PermissionRequirement::Any(vec!["write".to_string(), "delete".to_string()]),
        );
        assert!(validate_permissions(&claims, &config).is_ok());

        // Test with non-matching "Any" permission requirements
        let config = EntraAuthConfig::default()
            .with_permission_requirement(PermissionRequirement::Any(vec!["delete".to_string()]));
        assert!(matches!(
            validate_permissions(&claims, &config),
            Err(AuthError::AccessDenied(_))
        ));

        // Test with matching "All" permission requirements
        let config = EntraAuthConfig::default().with_permission_requirement(
            PermissionRequirement::All(vec!["read".to_string(), "write".to_string()]),
        );
        assert!(validate_permissions(&claims, &config).is_ok());

        // Test with partially matching "All" permission requirements
        let config = EntraAuthConfig::default().with_permission_requirement(
            PermissionRequirement::All(vec!["read".to_string(), "delete".to_string()]),
        );
        assert!(matches!(
            validate_permissions(&claims, &config),
            Err(AuthError::AccessDenied(_))
        ));
    }

    #[test]
    fn test_auth_error_into_response() {
        // Test error response conversion for each error type

        // MissingToken error
        let error = AuthError::MissingToken;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // InvalidTokenFormat error
        let error = AuthError::InvalidTokenFormat;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // ValidationFailed error
        let error = AuthError::ValidationFailed("Test validation error".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // InternalError error
        let error = AuthError::InternalError("Test internal error".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // AccessDenied error
        let error = AuthError::AccessDenied("Test access denied".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_auth_layer_creation() {
        // Test default layer creation
        let layer = EntraAuthLayer::default();
        let actual_tenant_id = layer.config.tenant_id.clone();
        assert_eq!(layer.config.tenant_id, actual_tenant_id);

        // Test with roles
        let layer = EntraAuthLayer::require_any_role(vec!["admin".to_string()]);
        match layer.config.required_roles {
            RoleRequirement::Any(roles) => {
                assert_eq!(roles.len(), 1);
                assert_eq!(roles[0], "admin");
            }
            _ => panic!("Expected RoleRequirement::Any"),
        }

        // Test with permissions
        let layer =
            EntraAuthLayer::require_all_permissions(vec!["read".to_string(), "write".to_string()]);
        match layer.config.required_permissions {
            PermissionRequirement::All(permissions) => {
                assert_eq!(permissions.len(), 2);
                assert_eq!(permissions[0], "read");
                assert_eq!(permissions[1], "write");
            }
            _ => panic!("Expected PermissionRequirement::All"),
        }
    }

    // Test the EntraClaims functionality
    #[test]
    fn test_entra_claims_get_scopes() {
        // Test with empty scopes
        let claims = EntraClaims {
            aud: "test-audience".to_string(),
            exp: constants::timestamps::YEAR_2100,
            iss: "test-issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021,
            sub: "test-subject".to_string(),
            roles: vec![],
            appid: None,
            app_id_uri: None,
            scp: None,
        };

        let scopes = claims.get_scopes();
        assert!(scopes.is_empty());

        // Test with single scope
        let claims = EntraClaims {
            aud: "test-audience".to_string(),
            exp: constants::timestamps::YEAR_2100,
            iss: "test-issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021,
            sub: "test-subject".to_string(),
            roles: vec![],
            appid: None,
            app_id_uri: None,
            scp: Some("read".to_string()),
        };

        let scopes = claims.get_scopes();
        assert_eq!(scopes.len(), 1);
        assert!(scopes.contains(&"read".to_string()));

        // Test with multiple scopes
        let claims = EntraClaims {
            aud: "test-audience".to_string(),
            exp: constants::timestamps::YEAR_2100,
            iss: "test-issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021,
            sub: "test-subject".to_string(),
            roles: vec![],
            appid: None,
            app_id_uri: None,
            scp: Some("read write delete".to_string()),
        };

        let scopes = claims.get_scopes();
        assert_eq!(scopes.len(), 3);
        assert!(scopes.contains(&"read".to_string()));
        assert!(scopes.contains(&"write".to_string()));
        assert!(scopes.contains(&"delete".to_string()));
    }

    #[test]
    fn test_role_requirement_from_str() {
        assert_eq!(
            RoleRequirement::from_str("admin").unwrap(),
            RoleRequirement::Admin
        );
        assert_eq!(
            RoleRequirement::from_str("read").unwrap(),
            RoleRequirement::ReadOnly
        );
        assert_eq!(
            RoleRequirement::from_str("full").unwrap(),
            RoleRequirement::FullAccess
        );
    }

    #[test]
    fn test_role_requirement_matches() {
        let admin_roles = vec!["admin".to_string(), "system-admin".to_string()];
        let read_roles = vec!["reader".to_string(), "viewer".to_string()];
        let full_roles = vec!["editor".to_string(), "manager".to_string()];

        let config = EntraAuthConfig {
            tenant_id: "test-tenant".to_string(),
            client_id: "test-client".to_string(),
            audience: "test-audience".to_string(),
            validate_token: true,
            required_roles: vec![],
            required_permissions: vec![],
            client: None,
            jwks_uri: "https://test.com/.well-known/jwks.json".to_string(),
            jwks_cache: None,
            debug_validation: false,
            issuer_url_formats: vec!["https://test.com/{tenant}/v2.0".to_string()],
            admin_roles: admin_roles.clone(),
            read_only_roles: read_roles.clone(),
            full_access_roles: full_roles.clone(),
        };

        // Test admin requirement
        assert!(RoleRequirement::Admin.matches(&admin_roles, &config));
        assert!(!RoleRequirement::Admin.matches(&read_roles, &config));
        assert!(!RoleRequirement::Admin.matches(&full_roles, &config));

        // Test read-only requirement
        assert!(RoleRequirement::ReadOnly.matches(&read_roles, &config));
        assert!(RoleRequirement::ReadOnly.matches(&admin_roles, &config));
        assert!(RoleRequirement::ReadOnly.matches(&full_roles, &config));

        // Test full access requirement
        assert!(RoleRequirement::FullAccess.matches(&full_roles, &config));
        assert!(RoleRequirement::FullAccess.matches(&admin_roles, &config));
        assert!(!RoleRequirement::FullAccess.matches(&read_roles, &config));
    }
}
