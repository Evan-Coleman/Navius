/// Macro for feature-conditional code at runtime
#[macro_export]
macro_rules! when_feature_enabled {
    ($app_state:expr, $feature:expr, $body:block) => {
        if $app_state.runtime_features().is_enabled($feature) {
            $body
        }
    };
}

/// Macro for compile-time feature checks
#[macro_export]
macro_rules! has_feature {
    ($feature:expr) => {{
        #[cfg(feature = $feature)]
        {
            true
        }
        #[cfg(not(feature = $feature))]
        {
            false
        }
    }};
}

/// Execute code only when a feature is enabled at compile time
#[macro_export]
macro_rules! with_feature {
    ($feature:expr, $body:block) => {
        #[cfg(feature = $feature)]
        {
            $body
        }
    };
}

/// Fallback execution when a feature is not enabled
#[macro_export]
macro_rules! feature_or_else {
    ($app_state:expr, $feature:expr, $if_enabled:block, $else_block:block) => {
        if $app_state.runtime_features().is_enabled($feature) {
            $if_enabled
        } else {
            $else_block
        }
    };
}
