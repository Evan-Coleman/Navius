use crate::error::{EntraError, EntraResult};
use chrono::{DateTime, NaiveDateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Microsoft Entra ID token claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraTokenClaims {
    /// Issuer - identifies the token issuer
    pub iss: String,

    /// Subject - identifies the principal that is the subject of the token
    pub sub: String,

    /// Audience - identifies the recipients that the token is intended for
    #[serde(default)]
    pub aud: Value, // Can be a string or array of strings

    /// Expiration time - identifies the expiration time on or after which the token must not be accepted
    pub exp: i64,

    /// Not before - identifies the time before which the token must not be accepted
    #[serde(default)]
    pub nbf: Option<i64>,

    /// Issued at - identifies the time at which the token was issued
    pub iat: i64,

    /// JWT ID - provides a unique identifier for the token
    #[serde(default)]
    pub jti: Option<String>,

    /// Name - user's full name
    #[serde(default)]
    pub name: Option<String>,

    /// Preferred username
    #[serde(default)]
    pub preferred_username: Option<String>,

    /// Email address
    #[serde(default)]
    pub email: Option<String>,

    /// Object ID - unique identifier for the user in the tenant
    #[serde(rename = "oid", default)]
    pub object_id: Option<String>,

    /// Tenant ID
    #[serde(rename = "tid", default)]
    pub tenant_id: Option<String>,

    /// Roles assigned to the user
    #[serde(default)]
    pub roles: Option<Vec<String>>,

    /// Groups the user is a member of
    #[serde(default)]
    pub groups: Option<Vec<String>>,

    /// Version of the token
    #[serde(rename = "ver", default)]
    pub version: Option<String>,

    /// Application ID of the client
    #[serde(rename = "azp", default)]
    pub authorized_party: Option<String>,

    /// App ID of the application
    #[serde(rename = "appid", default)]
    pub app_id: Option<String>,

    /// Any additional claims not explicitly defined
    #[serde(flatten)]
    pub additional_claims: Map<String, Value>,
}

impl EntraTokenClaims {
    /// Get the expiration time as a DateTime
    pub fn expiration_time(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(self.exp, 0).unwrap_or_default(),
            Utc,
        )
    }

    /// Get the issuance time as a DateTime
    pub fn issued_at(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(self.iat, 0).unwrap_or_default(),
            Utc,
        )
    }

    /// Get the not before time as a DateTime if available
    pub fn not_before(&self) -> Option<DateTime<Utc>> {
        self.nbf.map(|nbf| {
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(nbf, 0).unwrap_or_default(),
                Utc,
            )
        })
    }

    /// Check if the token is expired, taking into account the clock skew
    pub fn is_expired(&self, clock_skew_seconds: u64) -> bool {
        let now = Utc::now();
        let exp = self.expiration_time() + chrono::Duration::seconds(clock_skew_seconds as i64);
        now > exp
    }

    /// Check if the token is valid for use, taking into account the not before claim and clock skew
    pub fn is_valid_for_use(&self, clock_skew_seconds: u64) -> bool {
        let now = Utc::now();

        // Check if the token is not yet valid
        if let Some(nbf) = self.not_before() {
            let valid_from = nbf - chrono::Duration::seconds(clock_skew_seconds as i64);
            if now < valid_from {
                return false;
            }
        }

        // Check if the token has expired
        !self.is_expired(clock_skew_seconds)
    }

    /// Get all roles from the token (from both roles and groups claims)
    pub fn get_all_roles(&self) -> HashSet<String> {
        let mut roles = HashSet::new();

        // Add roles from the roles claim
        if let Some(role_values) = &self.roles {
            for role in role_values {
                roles.insert(role.clone());
            }
        }

        // Add roles from the groups claim
        if let Some(group_values) = &self.groups {
            for group in group_values {
                roles.insert(group.clone());
            }
        }

        roles
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        // Check roles claim
        if let Some(roles) = &self.roles {
            if roles.iter().any(|r| r == role) {
                return true;
            }
        }

        // Check groups claim
        if let Some(groups) = &self.groups {
            if groups.iter().any(|g| g == role) {
                return true;
            }
        }

        false
    }

    /// Get the audience as a string or the first audience if it's an array
    pub fn audience(&self) -> Option<String> {
        match &self.aud {
            Value::String(s) => Some(s.clone()),
            Value::Array(arr) => arr.first().and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    /// Get all audiences as a set of strings
    pub fn all_audiences(&self) -> HashSet<String> {
        let mut audiences = HashSet::new();

        match &self.aud {
            Value::String(s) => {
                audiences.insert(s.clone());
            }
            Value::Array(arr) => {
                for value in arr {
                    if let Value::String(s) = value {
                        audiences.insert(s.clone());
                    }
                }
            }
            _ => {}
        }

        audiences
    }

    /// Get a claim value by name
    pub fn get_claim<T: for<'de> Deserialize<'de>>(&self, name: &str) -> Option<T> {
        self.additional_claims
            .get(name)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a claim as a string
    pub fn get_claim_as_string(&self, name: &str) -> Option<String> {
        self.additional_claims.get(name).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => v.as_str().map(|s| s.to_string()),
        })
    }

    /// Check if a specific claim exists
    pub fn has_claim(&self, name: &str) -> bool {
        self.additional_claims.contains_key(name)
    }
}

/// Microsoft Entra ID token
#[derive(Debug, Clone)]
pub struct EntraToken {
    /// The raw JWT token
    pub raw_token: String,

    /// The decoded token header
    pub header: jsonwebtoken::Header,

    /// The decoded token claims
    pub claims: EntraTokenClaims,
}

impl EntraToken {
    /// Parse and validate a JWT token without signature validation
    pub fn parse_without_validation(token: &str) -> EntraResult<Self> {
        // Parse the token header
        let header = decode_header(token)
            .map_err(|e| EntraError::jwt(format!("Failed to decode token header: {}", e)))?;

        // Split the token to get the payload
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(EntraError::jwt("Invalid JWT token format"));
        }

        // Decode the payload
        let payload = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
            .map_err(|e| EntraError::jwt(format!("Failed to decode token payload: {}", e)))?;

        // Parse the claims
        let claims: EntraTokenClaims = serde_json::from_slice(&payload)
            .map_err(|e| EntraError::jwt(format!("Failed to parse token claims: {}", e)))?;

        Ok(EntraToken {
            raw_token: token.to_string(),
            header,
            claims,
        })
    }

    /// Validate a JWT token with the provided key
    pub fn validate(
        token: &str,
        decoding_key: &DecodingKey,
        validation: &Validation,
    ) -> EntraResult<Self> {
        // Decode and validate the token
        let token_data: TokenData<EntraTokenClaims> = decode(token, decoding_key, validation)
            .map_err(|e| EntraError::token_validation(format!("Token validation failed: {}", e)))?;

        Ok(EntraToken {
            raw_token: token.to_string(),
            header: token_data.header,
            claims: token_data.claims,
        })
    }

    /// Get the algorithm from the token header
    pub fn algorithm(&self) -> Option<Algorithm> {
        Some(self.header.alg)
    }

    /// Get the key ID from the token header
    pub fn key_id(&self) -> Option<&str> {
        self.header.kid.as_deref()
    }

    /// Check if the token is expired, taking into account the clock skew
    pub fn is_expired(&self, clock_skew_seconds: u64) -> bool {
        self.claims.is_expired(clock_skew_seconds)
    }

    /// Check if the token is valid for use, taking into account nbf and exp
    pub fn is_valid_for_use(&self, clock_skew_seconds: u64) -> bool {
        self.claims.is_valid_for_use(clock_skew_seconds)
    }

    /// Create a validation object for this token
    pub fn create_validation(
        audience: Option<&str>,
        issuer: Option<&str>,
        validate_exp: bool,
        validate_nbf: bool,
    ) -> Validation {
        let mut validation = Validation::new(Algorithm::RS256);

        if let Some(aud) = audience {
            validation.set_audience(&[aud]);
        }

        if let Some(iss) = issuer {
            validation.set_issuer(&[iss]);
        }

        validation.validate_exp = validate_exp;
        validation.validate_nbf = validate_nbf;
        validation.leeway = 0; // We handle clock skew separately

        validation
    }
}
