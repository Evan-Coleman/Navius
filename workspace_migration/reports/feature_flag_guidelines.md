# Feature Flag Organization Guidelines

**Date:** March 29, 2025  
**Status:** Draft  
**Priority:** Medium  
**Target Completion:** April 15, 2025

## Overview

This document outlines the guidelines for organizing and managing feature flags across the Navius ecosystem. Feature flags provide a powerful mechanism for controlling feature availability, enabling A/B testing, managing release cycles, and mitigating risk. These guidelines aim to establish consistent patterns for feature flag definition, management, and lifecycle.

## Core Principles

1. **Simplicity**: Feature flags should be simple to define, use, and understand
2. **Consistency**: Patterns for feature flag usage should be consistent across all crates
3. **Performance**: Feature flag evaluation should have minimal performance impact
4. **Testability**: Code using feature flags should be easily testable
5. **Auditability**: It should be easy to determine which flags are in use and their purpose

## Feature Flag Types

We define four types of feature flags based on their purpose and lifecycle:

1. **Release Flags**: Control the release of completed features to users
   - Typically short-lived (1-2 release cycles)
   - Binary (on/off) behavior
   - Example: `enable_new_authentication_flow`

2. **Experiment Flags**: Enable A/B testing of features or behavior changes
   - Medium-lived (until experiment conclusion)
   - May have multiple variants
   - Example: `new_ui_layout` with variants 'current', 'variant_a', 'variant_b'

3. **Operational Flags**: Control operational aspects of the system
   - Long-lived or permanent
   - Often used for performance tuning or system behavior
   - Example: `cache_expiration_strategy` with values 'time-based', 'lru', 'hybrid'

4. **Permission Flags**: Control access to features based on user permissions
   - Permanent
   - Tied to authentication/authorization system
   - Example: `enable_admin_features`

## Naming Conventions

Feature flags should follow a consistent naming pattern:

1. **Format**: `[category]_[feature]_[action/property]`
2. **Examples**:
   - `auth_mfa_enabled`
   - `ui_dashboard_layout`
   - `perf_query_cache_size`
   - `api_rate_limit_bypass`

3. **Categories**:
   - `auth`: Authentication-related features
   - `ui`: User interface features
   - `api`: API-related features
   - `perf`: Performance optimizations
   - `debug`: Debugging features
   - `core`: Core system features
   - `sec`: Security features
   - `exp`: Experimental features

## Flag Definition Structure

Feature flags should be defined in a central location with a consistent structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag<T> {
    /// Unique identifier for the flag
    pub key: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Description of the flag's purpose
    pub description: String,
    
    /// Default value if not explicitly set
    pub default_value: T,
    
    /// Flag type (release, experiment, operational, permission)
    pub flag_type: FlagType,
    
    /// When the flag was created
    pub created_at: DateTime<Utc>,
    
    /// When the flag is scheduled for removal (if temporary)
    pub expiry_date: Option<DateTime<Utc>>,
    
    /// Owner team/individual responsible for the flag
    pub owner: String,
    
    /// Whether the flag can be modified at runtime
    pub dynamic: bool,
    
    /// Additional metadata for the flag
    pub metadata: HashMap<String, Value>,
}

pub enum FlagType {
    Release,
    Experiment,
    Operational,
    Permission,
}
```

## Flag Registration

Feature flags should be registered in a central registry:

```rust
// In the configuration/bootstrap phase
let flag_registry = FeatureFlagRegistry::new();

// Register flags
flag_registry.register(
    FeatureFlag::new("auth_mfa_enabled")
        .with_description("Enables Multi-Factor Authentication for users")
        .with_default(false)
        .with_flag_type(FlagType::Release)
        .with_owner("auth-team")
        .with_expiry(Utc::now() + Duration::days(90))
);
```

## Accessing Feature Flags

Feature flags should be accessed through a consistent API:

```rust
// Simple boolean flags
if feature_manager.is_enabled("auth_mfa_enabled") {
    // Enable MFA flow
} else {
    // Use standard authentication flow
}

// Variant flags
match feature_manager.get_variant("ui_dashboard_layout") {
    "compact" => render_compact_layout(),
    "detailed" => render_detailed_layout(),
    _ => render_default_layout(),
}

// Typed values
let cache_size = feature_manager.get_value::<u64>("perf_query_cache_size").unwrap_or(1000);
```

## Contextual Evaluation

Feature flags should support contextual evaluation:

```rust
// Evaluate flag based on context
let context = FeatureContext::new()
    .with_user(current_user)
    .with_tenant(tenant_id)
    .with_environment(app_environment);

if feature_manager.is_enabled_with_context("premium_features", &context) {
    // Show premium features
}
```

## Flag Lifecycle Management

Feature flags should have a well-defined lifecycle:

1. **Creation**: Flags are created with clear purpose, owner, and expiry date (if temporary)
2. **Activation**: Flags are activated in specific environments (dev, staging, production)
3. **Monitoring**: Flag usage and impact are monitored
4. **Cleanup**: Temporary flags are removed after their purpose is fulfilled

Guidelines for cleanup:

```rust
// BAD: Flag check without cleanup plan
if feature_manager.is_enabled("new_feature") {
    use_new_implementation();
} else {
    use_old_implementation();
}

// GOOD: Flag with cleanup comments
// FLAG: new_feature
// PURPOSE: Enables the new implementation of feature X
// CREATED: 2025-03-15
// OWNER: team-a
// CLEANUP: When flag is fully rolled out, remove this conditional and the old implementation
if feature_manager.is_enabled("new_feature") {
    use_new_implementation();
} else {
    use_old_implementation();
}
```

## Flag Configuration Sources

Feature flags should support multiple configuration sources:

1. **Static Configuration**: Defined in configuration files (YAML, JSON, TOML)
2. **Environment Variables**: Overrides through environment variables
3. **Database**: Dynamic configuration stored in a database
4. **Remote Configuration**: Retrieved from a remote service
5. **In-Memory Overrides**: Overrides for testing purposes

Example configuration file:

```yaml
feature_flags:
  auth_mfa_enabled:
    enabled: true
    environments:
      production: true
      staging: true
      development: true
  ui_dashboard_layout:
    value: "compact"
    environments:
      production: "standard"
      staging: "compact"
      development: "compact"
  perf_query_cache_size:
    value: 5000
    environments:
      production: 10000
      staging: 5000
      development: 1000
```

## Testing with Feature Flags

Code using feature flags should be easily testable:

```rust
#[test]
fn test_with_feature_enabled() {
    let mut feature_manager = MockFeatureManager::new();
    feature_manager.set_enabled("auth_mfa_enabled", true);
    
    let service = AuthService::new(feature_manager);
    assert!(service.requires_mfa(&user));
}

#[test]
fn test_with_feature_disabled() {
    let mut feature_manager = MockFeatureManager::new();
    feature_manager.set_enabled("auth_mfa_enabled", false);
    
    let service = AuthService::new(feature_manager);
    assert!(!service.requires_mfa(&user));
}
```

## Flag Management and Monitoring

Feature flags should be monitored and managed:

1. **Flag Inventory**: Maintain a central inventory of all feature flags
2. **Usage Metrics**: Track how often each flag is evaluated
3. **Impact Analysis**: Measure the impact of feature flags on system behavior
4. **Stale Flag Detection**: Identify flags that are no longer needed
5. **Dependency Tracking**: Track which parts of the code depend on each flag

Example dashboard metrics:

```
Flag: auth_mfa_enabled
- Status: Enabled in production
- Evaluations: 1.2M/day
- Last Changed: 2025-03-15
- Created: 2025-01-10
- Owner: auth-team
- Dependencies: 5 modules
```

## Implementation in Navius

The feature flag system will be implemented as a new crate `navius-feature-flags` with the following components:

1. **FeatureFlag**: Core structures for flag definition
2. **FeatureFlagRegistry**: Central registry for flag registration
3. **FeatureManager**: Interface for flag evaluation
4. **FeatureContext**: Context for contextual flag evaluation
5. **ConfigurationSource**: Interface for configuration sources
6. **FeatureFlagMiddleware**: HTTP middleware for flag evaluation
7. **FeatureFlagMetrics**: Metrics collection for flag usage

Example implementation (simplified):

```rust
// Core trait for feature management
pub trait FeatureManager: Send + Sync {
    fn is_enabled(&self, key: &str) -> bool;
    fn is_enabled_with_context(&self, key: &str, context: &FeatureContext) -> bool;
    fn get_variant(&self, key: &str) -> Option<String>;
    fn get_variant_with_context(&self, key: &str, context: &FeatureContext) -> Option<String>;
    fn get_value<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    fn get_value_with_context<T: DeserializeOwned>(&self, key: &str, context: &FeatureContext) -> Option<T>;
}

// Default implementation
pub struct DefaultFeatureManager {
    registry: Arc<FeatureFlagRegistry>,
    sources: Vec<Box<dyn ConfigurationSource>>,
    cache: Cache<String, Value>,
}

impl DefaultFeatureManager {
    pub fn new(registry: FeatureFlagRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
            sources: Vec::new(),
            cache: Cache::new(),
        }
    }
    
    pub fn with_source(mut self, source: Box<dyn ConfigurationSource>) -> Self {
        self.sources.push(source);
        self
    }
}

impl FeatureManager for DefaultFeatureManager {
    // Implementation details
}
```

## Best Practices

1. **Minimize Flag Count**: Avoid creating too many flags; consolidate related flags when possible
2. **Set Expiry Dates**: Always set an expiry date for temporary flags
3. **Clear Ownership**: Each flag should have a clear owner responsible for its lifecycle
4. **Document Purpose**: Clearly document the purpose and expected behavior of each flag
5. **Test Both Paths**: Always test both enabled and disabled paths
6. **Regular Cleanup**: Regularly review and clean up unnecessary flags
7. **Consistent Evaluation**: Use the same evaluation logic across the system
8. **Avoid Deep Nesting**: Avoid deeply nested flag checks
9. **Performance Awareness**: Be aware of performance implications of flag evaluation
10. **Feature Isolation**: Keep flagged features isolated to minimize technical debt

## Anti-Patterns to Avoid

1. **Flag Creep**: Creating too many flags without cleanup
2. **Nested Flags**: Deeply nested flag evaluations creating complex logic paths
3. **Inconsistent Naming**: Inconsistent naming making it hard to understand flag purpose
4. **Permanent "Temporary" Flags**: Temporary flags that become permanent
5. **Orphaned Flags**: Flags without clear ownership
6. **Flag Logic Duplication**: Duplicating flag evaluation logic
7. **Hard-Coded Defaults**: Hard-coding default values in multiple places

## Conclusion

These feature flag organization guidelines provide a consistent approach to defining, managing, and using feature flags across the Navius ecosystem. By following these guidelines, teams can effectively use feature flags to manage feature releases, conduct experiments, and optimize system behavior while maintaining code quality and system performance. 