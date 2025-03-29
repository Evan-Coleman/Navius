---
title: "Feature Selection Pattern"
description: "Design and implementation of the feature selection pattern in Navius"
category: patterns
tags:
  - patterns
  - feature-flags
  - configuration
  - architecture
related:
  - reference/patterns/cache-provider-pattern.md
  - examples/configuration-example.md
last_updated: March 27, 2025
version: 1.0
---

# Feature Selection Pattern

## Overview

The Feature Selection Pattern in Navius provides a mechanism for dynamically enabling or disabling specific features or functionalities without changing the codebase. This pattern allows for feature toggling, A/B testing, progressive rollouts, and conditional feature access based on user roles, environments, or other criteria.

## Problem Statement

Modern application development faces several challenges related to feature deployment and management:

1. **Continuous Deployment**: How to deploy new features without affecting current users?
2. **Progressive Rollout**: How to release features to a subset of users for testing?
3. **A/B Testing**: How to compare different implementations of the same feature?
4. **Environment-Specific Behavior**: How to enable features only in specific environments?
5. **User-Specific Access**: How to restrict features to certain user roles or subscription levels?
6. **Performance Optimization**: How to selectively enable resource-intensive features?

## Solution: Feature Selection Pattern

The Feature Selection Pattern in Navius addresses these challenges through a unified feature flag system:

1. **Feature Flags**: Named boolean switches that control feature availability
2. **Flag Sources**: Multiple sources for flag values (config files, database, remote services)
3. **Evaluation Context**: Context-aware feature resolution (user data, environment, etc.)
4. **Override Hierarchy**: Clear precedence rules for conflicting flag values
5. **Runtime Toggles**: Ability to change flag values without application restart

### Pattern Structure

```
┌───────────────────┐
│  FeatureService   │
└─────────┬─────────┘
          │
          │ uses
          ▼
┌───────────────────┐     consults     ┌───────────────────┐
│  FeatureRegistry  │────────────────► │  Configuration    │
└───────────────────┘                  └───────────────────┘
          │                                     ▲
          │ contains                            │
          ▼                                     │ provides
┌───────────────────┐                  ┌───────────────────┐
│   FeatureFlag     │                  │  Flag Providers   │
└───────────────────┘                  └───────────────────┘
```

## Implementation

### Feature Flag Definition

A feature flag is defined by its name, description, default value, and optional context-dependent rules:

```rust
pub struct FeatureFlag {
    /// Unique identifier for the feature
    pub name: String,
    
    /// Human-readable description of the feature
    pub description: String,
    
    /// Default value if no other rules match
    pub default_value: bool,
    
    /// Optional rules for contextual evaluation
    pub rules: Vec<FeatureRule>,
}

pub struct FeatureRule {
    /// Condition that must be satisfied for this rule to apply
    pub condition: Box<dyn FeatureCondition>,
    
    /// Value to use if the condition is met
    pub value: bool,
}

/// Trait for implementing feature flag conditions
pub trait FeatureCondition: Send + Sync {
    /// Evaluate whether this condition applies in the given context
    fn evaluate(&self, context: &FeatureContext) -> bool;
}
```

### Feature Registry

The feature registry maintains the collection of available feature flags:

```rust
pub struct FeatureRegistry {
    features: RwLock<HashMap<String, FeatureFlag>>,
}

impl FeatureRegistry {
    pub fn new() -> Self {
        Self {
            features: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn register(&self, feature: FeatureFlag) -> Result<(), AppError> {
        let mut features = self.features.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock on feature registry")
        })?;
        
        features.insert(feature.name.clone(), feature);
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Result<FeatureFlag, AppError> {
        let features = self.features.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock on feature registry")
        })?;
        
        features.get(name)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("Feature flag '{}' not found", name)))
    }
}
```

### Feature Service

The feature service provides the primary API for checking feature flags:

```rust
pub struct FeatureService {
    registry: Arc<FeatureRegistry>,
    providers: Vec<Box<dyn FeatureFlagProvider>>,
}

impl FeatureService {
    pub fn new(registry: Arc<FeatureRegistry>, providers: Vec<Box<dyn FeatureFlagProvider>>) -> Self {
        Self {
            registry,
            providers,
        }
    }
    
    /// Check if a feature is enabled in the given context
    pub fn is_enabled(&self, name: &str, context: &FeatureContext) -> Result<bool, AppError> {
        // First check if any provider has an override
        for provider in &self.providers {
            if let Some(value) = provider.get_flag_value(name, context) {
                return Ok(value);
            }
        }
        
        // If no provider has an override, evaluate the feature flag
        let feature = self.registry.get(name)?;
        
        // Evaluate rules in order, using the first matching rule
        for rule in &feature.rules {
            if rule.condition.evaluate(context) {
                return Ok(rule.value);
            }
        }
        
        // If no rules match, use the default value
        Ok(feature.default_value)
    }
    
    /// Shorthand for checking if a feature is enabled with empty context
    pub fn is_feature_enabled(&self, name: &str) -> Result<bool, AppError> {
        self.is_enabled(name, &FeatureContext::default())
    }
}
```

### Feature Context

The context provides information for contextual feature evaluation:

```rust
pub struct FeatureContext {
    /// Current environment (dev, test, prod)
    pub environment: String,
    
    /// Current user information (if available)
    pub user: Option<UserContext>,
    
    /// Custom attributes for application-specific conditions
    pub attributes: HashMap<String, Value>,
}

pub struct UserContext {
    pub id: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

impl FeatureContext {
    pub fn default() -> Self {
        Self {
            environment: "development".to_string(),
            user: None,
            attributes: HashMap::new(),
        }
    }
    
    pub fn for_environment(environment: &str) -> Self {
        let mut context = Self::default();
        context.environment = environment.to_string();
        context
    }
    
    pub fn for_user(user_id: &str, roles: Vec<String>, groups: Vec<String>) -> Self {
        let mut context = Self::default();
        context.user = Some(UserContext {
            id: user_id.to_string(),
            roles,
            groups,
        });
        context
    }
}
```

### Feature Flag Providers

Providers supply feature flag values from different sources:

```rust
/// Trait for implementing feature flag providers
pub trait FeatureFlagProvider: Send + Sync {
    /// Get the value for a feature flag in the given context
    /// Returns None if this provider doesn't have a value for the flag
    fn get_flag_value(&self, name: &str, context: &FeatureContext) -> Option<bool>;
}

/// Configuration-based provider that reads flags from the application config
pub struct ConfigFeatureFlagProvider {
    config: Arc<AppConfig>,
}

impl FeatureFlagProvider for ConfigFeatureFlagProvider {
    fn get_flag_value(&self, name: &str, _context: &FeatureContext) -> Option<bool> {
        self.config.features.get(name).copied()
    }
}

/// Remote provider that fetches flags from a remote service
pub struct RemoteFeatureFlagProvider {
    client: FeatureFlagClient,
    cache: Arc<dyn Cache>,
    cache_ttl: Duration,
}

impl FeatureFlagProvider for RemoteFeatureFlagProvider {
    fn get_flag_value(&self, name: &str, context: &FeatureContext) -> Option<bool> {
        // Check cache first
        if let Some(value) = self.cache.get::<bool>(name) {
            return Some(value);
        }
        
        // If not in cache, fetch from remote
        if let Ok(value) = self.client.get_flag_value(name, context) {
            // Update cache
            let _ = self.cache.set(name, value, self.cache_ttl);
            return Some(value);
        }
        
        None
    }
}
```

## Usage Examples

### Basic Feature Checking

```rust
// Check if a feature is enabled
let analytics_enabled = feature_service.is_feature_enabled("enable_analytics")?;

if analytics_enabled {
    analytics_service.track_event("page_view", &event_data);
}
```

### Contextual Feature Checking

```rust
// Create context with user information
let context = FeatureContext::for_user(
    &user.id,
    user.roles.clone(),
    user.groups.clone(),
);

// Check if premium feature is available for this user
let premium_enabled = feature_service.is_enabled("premium_features", &context)?;

if premium_enabled {
    return Ok(Json(premium_content));
} else {
    return Err(AppError::forbidden("Premium subscription required"));
}
```

### Environment-Based Features

```rust
// Create environment-specific context
let context = FeatureContext::for_environment("production");

// Check if feature is enabled in this environment
let beta_feature = feature_service.is_enabled("new_ui", &context)?;

if beta_feature {
    // Use new UI components
    html_response.render_new_ui()
} else {
    // Use classic UI components
    html_response.render_classic_ui()
}
```

### Percentage Rollout

```rust
// Define a percentage rollout condition
let percentage_condition = PercentageRolloutCondition::new("new_checkout", 25);

// Create a feature flag with this condition
let feature = FeatureFlag {
    name: "new_checkout".to_string(),
    description: "New checkout experience".to_string(),
    default_value: false,
    rules: vec![
        FeatureRule {
            condition: Box::new(percentage_condition),
            value: true,
        },
    ],
};

// Register the feature
feature_registry.register(feature)?;
```

### Feature Flag Configuration

```yaml
# config/default.yaml
features:
  enable_analytics: true
  premium_features: false
  new_ui: false
  experimental_api: false
  
# config/production.yaml
features:
  enable_analytics: true
  premium_features: true
  new_ui: false
  experimental_api: false
```

## Benefits

1. **Continuous Deployment**: Deploy code with disabled features for later activation
2. **Risk Mitigation**: Quickly disable problematic features without deployment
3. **Progressive Rollout**: Release features to a subset of users for feedback
4. **A/B Testing**: Compare different implementations with controlled exposure
5. **Operational Control**: Manage resource-intensive features during peak loads
6. **Subscription Management**: Tie feature access to user subscription levels
7. **Environment Control**: Different behavior in development, testing, and production

## Implementation Considerations

1. **Performance**: Efficient feature flag checking, especially for high-traffic paths
2. **Default Behavior**: Clear fallback behavior when flag evaluation fails
3. **Monitoring**: Track feature flag usage and impact on application behavior
4. **Flag Cleanup**: Process for removing unused feature flags over time
5. **Flag Discovery**: Tools for developers to discover available feature flags
6. **Testing**: Ability to test code paths for both enabled and disabled states

## Advanced Techniques

### Gradual Rollout Strategy

Implementing a gradual rollout based on user IDs:

```rust
pub struct GradualRolloutCondition {
    feature_name: String,
    rollout_percentage: u8,
}

impl FeatureCondition for GradualRolloutCondition {
    fn evaluate(&self, context: &FeatureContext) -> bool {
        if let Some(user) = &context.user {
            // Create a deterministic hash based on user ID and feature name
            let seed = format!("{}:{}", user.id, self.feature_name);
            let hash = calculate_hash(&seed);
            
            // Map hash to 0-100 range and compare with rollout percentage
            let user_value = hash % 100;
            return user_value < self.rollout_percentage as u64;
        }
        
        false
    }
}
```

### Feature Combinations

Handling complex conditions with multiple factors:

```rust
pub struct AndCondition {
    conditions: Vec<Box<dyn FeatureCondition>>,
}

impl FeatureCondition for AndCondition {
    fn evaluate(&self, context: &FeatureContext) -> bool {
        self.conditions.iter().all(|c| c.evaluate(context))
    }
}

pub struct OrCondition {
    conditions: Vec<Box<dyn FeatureCondition>>,
}

impl FeatureCondition for OrCondition {
    fn evaluate(&self, context: &FeatureContext) -> bool {
        self.conditions.iter().any(|c| c.evaluate(context))
    }
}
```

### Feature Dependencies

Handling features that depend on other features:

```rust
pub struct DependsOnFeatureCondition {
    dependency: String,
}

impl FeatureCondition for DependsOnFeatureCondition {
    fn evaluate(&self, context: &FeatureContext) -> bool {
        // Get the feature service from the context
        if let Some(feature_service) = context.get_service::<FeatureService>() {
            return feature_service.is_enabled(&self.dependency, context).unwrap_or(false);
        }
        
        false
    }
}
```

### Feature Metrics

Tracking feature flag usage:

```rust
pub struct MetricsWrappedFeatureService {
    inner: Arc<FeatureService>,
    metrics: Arc<dyn MetricsCollector>,
}

impl MetricsWrappedFeatureService {
    pub fn is_enabled(&self, name: &str, context: &FeatureContext) -> Result<bool, AppError> {
        let start = Instant::now();
        let result = self.inner.is_enabled(name, context);
        
        // Record metrics
        let elapsed = start.elapsed();
        self.metrics.record_timing("feature.check.duration", elapsed);
        
        if let Ok(value) = result {
            self.metrics.increment_counter(&format!("feature.{}.{}", name, if value { "enabled" } else { "disabled" }));
        } else {
            self.metrics.increment_counter(&format!("feature.{}.error", name));
        }
        
        result
    }
}
```

## Related Patterns

- **Strategy Pattern**: Feature flags often select between strategy implementations
- **Factory Pattern**: Creating different implementations based on feature flags
- **Decorator Pattern**: Adding optional behavior when features are enabled
- **Configuration Pattern**: Feature flags are a special form of configuration
- **Cache Provider Pattern**: Caching feature flag values for performance

## References

- [Configuration Example](../../examples/configuration-example.md)
- [API Resource Pattern](./api-resource-pattern.md) 