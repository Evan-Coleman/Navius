//! Constants used throughout the application configuration
//! This centralizes common values to reduce duplication

/// Authentication-related constants
pub mod auth {
    /// URL formats
    pub mod urls {
        /// Format for JWKS URI (Microsoft Entra ID)
        pub const ENTRA_JWKS_URI_FORMAT: &str =
            "https://login.microsoftonline.com/{}/discovery/v2.0/keys";

        /// Format for Microsoft Entra ID authorize endpoint
        pub const ENTRA_AUTH_URL_FORMAT: &str =
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize";

        /// Format for Microsoft Entra ID token endpoint
        pub const ENTRA_TOKEN_URL_FORMAT: &str =
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token";

        /// Format for default audience (when not specified)
        pub const DEFAULT_AUDIENCE_FORMAT: &str = "api://{}";
    }

    /// Environment variable names
    pub mod env_vars {
        /// Tenant ID environment variable
        pub const TENANT_ID: &str = "RUST_BACKEND_TENANT_ID";

        /// Client ID environment variable
        pub const CLIENT_ID: &str = "RUST_BACKEND_CLIENT_ID";

        /// Client secret environment variable
        pub const CLIENT_SECRET: &str = "RUST_BACKEND_SECRET";

        /// Audience environment variable
        pub const AUDIENCE: &str = "RUST_BACKEND_AUDIENCE";

        /// Scope environment variable
        pub const SCOPE: &str = "RUST_BACKEND_SCOPE";

        /// Token URL environment variable
        pub const TOKEN_URL: &str = "RUST_BACKEND_TOKEN_URL";

        /// Debug authentication environment variable
        pub const DEBUG_AUTH: &str = "DEBUG_AUTH";
    }
}

/// Future timestamp values
pub mod timestamps {
    /// Year 2100 timestamp for long expiry tokens (in seconds since Unix epoch)
    pub const YEAR_2100: usize = 4102444800;

    /// January 1, 2021 timestamp (in seconds since Unix epoch)
    pub const JAN_1_2021: usize = 1609459200;
}
