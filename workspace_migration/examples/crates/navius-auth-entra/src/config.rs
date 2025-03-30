use crate::error::{EntraError, EntraResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use url::Url;

/// Authentication flow types supported by the Entra provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthFlow {
    /// Authorization Code flow - for web applications
    AuthorizationCode,
    /// Implicit flow - for single-page applications
    Implicit,
    /// Client Credentials flow - for server-to-server applications
    ClientCredentials,
    /// Resource Owner Password Credentials flow - for trusted applications
    Password,
    /// Device Code flow - for devices with limited input capabilities
    DeviceCode,
}

impl Default for AuthFlow {
    fn default() -> Self {
        AuthFlow::AuthorizationCode
    }
}

impl fmt::Display for AuthFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthFlow::AuthorizationCode => write!(f, "authorization_code"),
            AuthFlow::Implicit => write!(f, "implicit"),
            AuthFlow::ClientCredentials => write!(f, "client_credentials"),
            AuthFlow::Password => write!(f, "password"),
            AuthFlow::DeviceCode => write!(f, "device_code"),
        }
    }
}

/// JSON Web Key Set cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksCacheConfig {
    /// Whether to cache JWKS keys
    #[serde(default = "default_jwks_cache_enabled")]
    pub enabled: bool,

    /// How long to cache JWKS keys (in seconds)
    #[serde(default = "default_jwks_cache_ttl")]
    pub ttl_seconds: u64,

    /// Maximum size of the JWKS cache
    #[serde(default = "default_jwks_cache_size")]
    pub max_size: usize,
}

fn default_jwks_cache_enabled() -> bool {
    true
}

fn default_jwks_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_jwks_cache_size() -> usize {
    100
}

impl Default for JwksCacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_jwks_cache_enabled(),
            ttl_seconds: default_jwks_cache_ttl(),
            max_size: default_jwks_cache_size(),
        }
    }
}

/// Circuit breaker configuration for external API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether to enable the circuit breaker
    #[serde(default = "default_circuit_breaker_enabled")]
    pub enabled: bool,

    /// Time window to track failures (in seconds)
    #[serde(default = "default_circuit_breaker_window")]
    pub window_seconds: u64,

    /// Number of failures before opening the circuit
    #[serde(default = "default_circuit_breaker_threshold")]
    pub failure_threshold: u32,

    /// Time to keep the circuit open before attempting to half-open (in seconds)
    #[serde(default = "default_circuit_breaker_timeout")]
    pub timeout_seconds: u64,
}

fn default_circuit_breaker_enabled() -> bool {
    true
}

fn default_circuit_breaker_window() -> u64 {
    60 // 1 minute
}

fn default_circuit_breaker_threshold() -> u32 {
    5
}

fn default_circuit_breaker_timeout() -> u64 {
    30 // 30 seconds
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: default_circuit_breaker_enabled(),
            window_seconds: default_circuit_breaker_window(),
            failure_threshold: default_circuit_breaker_threshold(),
            timeout_seconds: default_circuit_breaker_timeout(),
        }
    }
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Timeout for HTTP requests (in seconds)
    #[serde(default = "default_http_timeout")]
    pub timeout_seconds: u64,

    /// User agent to use for HTTP requests
    #[serde(default = "default_http_user_agent")]
    pub user_agent: String,

    /// Whether to enable connection pooling
    #[serde(default = "default_http_connection_pooling")]
    pub connection_pooling: bool,

    /// Maximum number of redirects to follow
    #[serde(default = "default_http_max_redirects")]
    pub max_redirects: u32,
}

fn default_http_timeout() -> u64 {
    30 // 30 seconds
}

fn default_http_user_agent() -> String {
    format!("navius-auth-entra/{}", crate::VERSION)
}

fn default_http_connection_pooling() -> bool {
    true
}

fn default_http_max_redirects() -> u32 {
    10
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_http_timeout(),
            user_agent: default_http_user_agent(),
            connection_pooling: default_http_connection_pooling(),
            max_redirects: default_http_max_redirects(),
        }
    }
}

/// Token validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationConfig {
    /// Whether to validate the issuer claim
    #[serde(default = "default_validate_issuer")]
    pub validate_issuer: bool,

    /// Whether to validate the audience claim
    #[serde(default = "default_validate_audience")]
    pub validate_audience: bool,

    /// Whether to validate the expiration time claim
    #[serde(default = "default_validate_exp")]
    pub validate_exp: bool,

    /// Whether to validate the not before claim
    #[serde(default = "default_validate_nbf")]
    pub validate_nbf: bool,

    /// Clock skew allowance (in seconds)
    #[serde(default = "default_clock_skew")]
    pub clock_skew_seconds: u64,

    /// Required claims that must be present in the token
    #[serde(default)]
    pub required_claims: Vec<String>,
}

fn default_validate_issuer() -> bool {
    true
}

fn default_validate_audience() -> bool {
    true
}

fn default_validate_exp() -> bool {
    true
}

fn default_validate_nbf() -> bool {
    true
}

fn default_clock_skew() -> u64 {
    300 // 5 minutes
}

impl Default for TokenValidationConfig {
    fn default() -> Self {
        Self {
            validate_issuer: default_validate_issuer(),
            validate_audience: default_validate_audience(),
            validate_exp: default_validate_exp(),
            validate_nbf: default_validate_nbf(),
            clock_skew_seconds: default_clock_skew(),
            required_claims: vec![],
        }
    }
}

/// Role mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleMappingConfig {
    /// Whether to enable role mapping
    #[serde(default = "default_role_mapping_enabled")]
    pub enabled: bool,

    /// Source claim to use for roles (typically "roles" or "groups")
    #[serde(default = "default_role_source_claim")]
    pub source_claim: String,

    /// Map from Entra roles/groups to application roles
    #[serde(default)]
    pub role_map: HashMap<String, String>,

    /// Default role to assign if no roles are found
    #[serde(default)]
    pub default_role: Option<String>,
}

fn default_role_mapping_enabled() -> bool {
    true
}

fn default_role_source_claim() -> String {
    "roles".to_string()
}

impl Default for RoleMappingConfig {
    fn default() -> Self {
        Self {
            enabled: default_role_mapping_enabled(),
            source_claim: default_role_source_claim(),
            role_map: HashMap::new(),
            default_role: None,
        }
    }
}

/// Configuration for the Microsoft Entra authentication provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraConfig {
    /// Microsoft Entra Tenant ID
    pub tenant_id: String,

    /// Application (Client) ID registered in Microsoft Entra
    pub client_id: String,

    /// Client Secret (if using confidential client flows)
    #[serde(default)]
    pub client_secret: Option<String>,

    /// Application authentication flow
    #[serde(default)]
    pub auth_flow: AuthFlow,

    /// Redirect URI for authorization code flow
    #[serde(default)]
    pub redirect_uri: Option<String>,

    /// Scopes to request
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,

    /// Authority URL (defaults to Microsoft Entra global endpoint)
    #[serde(default = "default_authority")]
    pub authority: String,

    /// JWKS cache configuration
    #[serde(default)]
    pub jwks_cache: JwksCacheConfig,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// HTTP client configuration
    #[serde(default)]
    pub http_client: HttpClientConfig,

    /// Token validation configuration
    #[serde(default)]
    pub token_validation: TokenValidationConfig,

    /// Role mapping configuration
    #[serde(default)]
    pub role_mapping: RoleMappingConfig,

    /// Whether to enable metrics
    #[serde(default = "default_metrics_enabled")]
    pub metrics_enabled: bool,

    /// Additional configuration options
    #[serde(default)]
    pub additional_options: HashMap<String, String>,
}

fn default_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
    ]
}

fn default_authority() -> String {
    "https://login.microsoftonline.com/".to_string()
}

fn default_metrics_enabled() -> bool {
    true
}

impl EntraConfig {
    /// Validate the configuration
    pub fn validate(&self) -> EntraResult<()> {
        // Validate tenant ID
        if self.tenant_id.is_empty() {
            return Err(EntraError::config("Tenant ID is required"));
        }

        // Validate client ID
        if self.client_id.is_empty() {
            return Err(EntraError::config("Client ID is required"));
        }

        // Validate client secret for certain flows
        match self.auth_flow {
            AuthFlow::AuthorizationCode | AuthFlow::ClientCredentials => {
                if self.client_secret.is_none() || self.client_secret.as_ref().unwrap().is_empty() {
                    return Err(EntraError::config(format!(
                        "Client secret is required for {} flow",
                        self.auth_flow
                    )));
                }
            }
            _ => {}
        }

        // Validate redirect URI for certain flows
        match self.auth_flow {
            AuthFlow::AuthorizationCode | AuthFlow::Implicit => {
                if self.redirect_uri.is_none() || self.redirect_uri.as_ref().unwrap().is_empty() {
                    return Err(EntraError::config(format!(
                        "Redirect URI is required for {} flow",
                        self.auth_flow
                    )));
                }

                // Validate redirect URI format
                if let Some(redirect_uri) = &self.redirect_uri {
                    if let Err(e) = Url::parse(redirect_uri) {
                        return Err(EntraError::config(format!("Invalid redirect URI: {}", e)));
                    }
                }
            }
            _ => {}
        }

        // Validate authority URL
        if let Err(e) = Url::parse(&self.authority) {
            return Err(EntraError::config(format!("Invalid authority URL: {}", e)));
        }

        Ok(())
    }

    /// Get the authorization endpoint URL
    pub fn authorization_endpoint(&self) -> EntraResult<String> {
        let authority = if self.authority.ends_with('/') {
            self.authority.clone()
        } else {
            format!("{}/", self.authority)
        };

        Ok(format!(
            "{}{}oauth2/v2.0/authorize",
            authority, self.tenant_id
        ))
    }

    /// Get the token endpoint URL
    pub fn token_endpoint(&self) -> EntraResult<String> {
        let authority = if self.authority.ends_with('/') {
            self.authority.clone()
        } else {
            format!("{}/", self.authority)
        };

        Ok(format!("{}{}oauth2/v2.0/token", authority, self.tenant_id))
    }

    /// Get the logout endpoint URL
    pub fn logout_endpoint(&self) -> EntraResult<String> {
        let authority = if self.authority.ends_with('/') {
            self.authority.clone()
        } else {
            format!("{}/", self.authority)
        };

        Ok(format!("{}{}oauth2/v2.0/logout", authority, self.tenant_id))
    }

    /// Get the JWKS endpoint URL
    pub fn jwks_endpoint(&self) -> EntraResult<String> {
        let authority = if self.authority.ends_with('/') {
            self.authority.clone()
        } else {
            format!("{}/", self.authority)
        };

        Ok(format!(
            "{}{}v2.0/.well-known/jwks.json",
            authority, self.tenant_id
        ))
    }

    /// Get the OpenID configuration endpoint URL
    pub fn openid_configuration_endpoint(&self) -> EntraResult<String> {
        let authority = if self.authority.ends_with('/') {
            self.authority.clone()
        } else {
            format!("{}/", self.authority)
        };

        Ok(format!(
            "{}{}v2.0/.well-known/openid-configuration",
            authority, self.tenant_id
        ))
    }

    /// Get the HTTP client timeout as a Duration
    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(self.http_client.timeout_seconds)
    }

    /// Get the JWKS cache TTL as a Duration
    pub fn jwks_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.jwks_cache.ttl_seconds)
    }

    /// Get the circuit breaker window as a Duration
    pub fn circuit_breaker_window(&self) -> Duration {
        Duration::from_secs(self.circuit_breaker.window_seconds)
    }

    /// Get the circuit breaker timeout as a Duration
    pub fn circuit_breaker_timeout(&self) -> Duration {
        Duration::from_secs(self.circuit_breaker.timeout_seconds)
    }

    /// Get the token validation clock skew as a Duration
    pub fn token_validation_clock_skew(&self) -> Duration {
        Duration::from_secs(self.token_validation.clock_skew_seconds)
    }

    /// Get the issuer URI (used for token validation)
    pub fn issuer(&self) -> String {
        let authority = if self.authority.ends_with('/') {
            self.authority.clone()
        } else {
            format!("{}/", self.authority)
        };

        format!("{}{}v2.0", authority, self.tenant_id)
    }
}

impl Default for EntraConfig {
    fn default() -> Self {
        Self {
            tenant_id: "common".to_string(), // Common endpoint for multi-tenant apps
            client_id: String::new(),
            client_secret: None,
            auth_flow: AuthFlow::default(),
            redirect_uri: None,
            scopes: default_scopes(),
            authority: default_authority(),
            jwks_cache: JwksCacheConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            http_client: HttpClientConfig::default(),
            token_validation: TokenValidationConfig::default(),
            role_mapping: RoleMappingConfig::default(),
            metrics_enabled: default_metrics_enabled(),
            additional_options: HashMap::new(),
        }
    }
}
