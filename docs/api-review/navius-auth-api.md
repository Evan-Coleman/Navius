# navius-auth API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 99%  
**Status:** ✅ Good

## Dependencies

- navius-core
- navius-http
- serde
- serde_json
- thiserror
- async-trait
- tower
- tower-http
- tracing
- chrono
- rand
- uuid
- hex
- sha2
- base64
- http
- tokio
- jsonwebtoken
- oauth2

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| AuthConfig | types.rs | 13 | ⚠️ Partial |
| Claims | types.rs | 54 | ⚠️ Partial |
| Identity | types.rs | 88 | ⚠️ Partial |
| Credentials | types.rs | 115 | ✅ Complete |
| Permission | types.rs | 124 | ✅ Complete |
| Subject | types.rs | 141 | ⚠️ Partial |
| Role | types.rs | 161 | ⚠️ Partial |
| OAuthConfig | types.rs | 175 | ✅ Complete |
| OAuthProviderConfig | types.rs | 186 | ✅ Complete |
| CorsConfig | types.rs | 205 | ✅ Complete |
| Authorizer | authorize.rs | 13 | ⚠️ Partial |
| ResourceAuthMiddleware | authorize.rs | 129 | ⚠️ Partial |
| ErrorResponse | error.rs | 67 | ❌ Missing |
| TokenProviderConfig | token.rs | 31 | ⚠️ Partial |
| MockUser | token.rs | 66 | ⚠️ Partial |
| JWTProvider | token.rs | 88 | ⚠️ Partial |
| Version; | lib.rs | 36 | ⚠️ Partial |
| AuthLayer | middleware.rs | 20 | ⚠️ Partial |
| AuthConfig | middleware.rs | 28 | ⚠️ Partial |
| AuthChecker | middleware.rs | 103 | ⚠️ Partial |
| ProviderConfig | providers.rs | 41 | ⚠️ Partial |
| ProviderFactory; | providers.rs | 77 | ✅ Complete |
| BasicProviderConfig | basic.rs | 18 | ⚠️ Partial |
| MockUser | basic.rs | 42 | ⚠️ Partial |
| BasicProvider | basic.rs | 62 | ✅ Complete |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Error | error.rs | 15 | ⚠️ Partial |
| ProviderType | providers.rs | 15 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| SubjectExt | authorize.rs | 162 | ⚠️ Partial |
| AuthProvider: | providers.rs | 53 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new() | authorize.rs | 28 | ⚠️ Partial |
| add_role(&mut | authorize.rs | 33 | ⚠️ Partial |
| has_role(&self, | authorize.rs | 40 | ⚠️ Partial |
| has_any_role(&self, | authorize.rs | 46 | ⚠️ Partial |
| has_all_roles(&self, | authorize.rs | 52 | ⚠️ Partial |
| get_permissions(&self, | authorize.rs | 58 | ⚠️ Partial |
| can(&self, | authorize.rs | 72 | ⚠️ Partial |
| authorize(&self, | authorize.rs | 99 | ⚠️ Partial |
| middleware_for_resource(&self, | authorize.rs | 119 | ⚠️ Partial |
| new(resource: | authorize.rs | 138 | ✅ Complete |
| resource(&self) | authorize.rs | 146 | ⚠️ Partial |
| action(&self) | authorize.rs | 151 | ⚠️ Partial |
| check_authorization(&self, | authorize.rs | 156 | ⚠️ Partial |
| status_code(&self) | error.rs | 76 | ⚠️ Partial |
| error_code(&self) | error.rs | 95 | ⚠️ Partial |
| to_response(&self) | error.rs | 114 | ⚠️ Partial |
| to_response_with_details(&self, | error.rs | 123 | ⚠️ Partial |
| authentication_failed<S: | error.rs | 132 | ⚠️ Partial |
| token_invalid<S: | error.rs | 137 | ⚠️ Partial |
| token_expired() | error.rs | 142 | ⚠️ Partial |
| token_missing() | error.rs | 147 | ⚠️ Partial |
| authorization_failed<S: | error.rs | 152 | ⚠️ Partial |
| configuration<S: | error.rs | 157 | ⚠️ Partial |
| provider<S: | error.rs | 162 | ⚠️ Partial |
| oauth<S: | error.rs | 168 | ⚠️ Partial |
| external_service<S: | error.rs | 173 | ⚠️ Partial |
| serialization<S: | error.rs | 178 | ⚠️ Partial |
| internal<S: | error.rs | 183 | ⚠️ Partial |
| core<S: | error.rs | 188 | ⚠️ Partial |
| is_authentication_error(&self) | error.rs | 193 | ⚠️ Partial |
| is_authorization_error(&self) | error.rs | 204 | ⚠️ Partial |
| is_client_error(&self) | error.rs | 209 | ⚠️ Partial |
| is_server_error(&self) | error.rs | 215 | ⚠️ Partial |
| new(name: | token.rs | 100 | ✅ Complete |
| user_to_identity(&self, | token.rs | 128 | ⚠️ Partial |
| user_to_subject(&self, | token.rs | 154 | ⚠️ Partial |
| current() | lib.rs | 40 | ⚠️ Partial |
| semver() | lib.rs | 45 | ⚠️ Partial |
| init() | lib.rs | 51 | ⚠️ Partial |
| new(provider: | middleware.rs | 57 | ⚠️ Partial |
| with_config(provider: | middleware.rs | 65 | ⚠️ Partial |
| optional(provider: | middleware.rs | 70 | ⚠️ Partial |
| extract_token<T>(headers: | middleware.rs | 83 | ⚠️ Partial |
| new(provider: | middleware.rs | 115 | ✅ Complete |
| require_roles(mut | middleware.rs | 124 | ⚠️ Partial |
| require_all_roles(mut | middleware.rs | 130 | ⚠️ Partial |
| check_roles(&self, | middleware.rs | 136 | ⚠️ Partial |
| fn | middleware.rs | 161 | ⚠️ Partial |
| create(config: | providers.rs | 81 | ✅ Complete |
| new(name: | basic.rs | 82 | ✅ Complete |
| user_to_identity(&self, | basic.rs | 183 | ⚠️ Partial |
| user_to_subject(&self, | basic.rs | 209 | ⚠️ Partial |

