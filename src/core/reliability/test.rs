#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::{
        AppConfig, CircuitBreakerConfig, ConcurrencyConfig, RateLimitConfig, ReliabilityConfig,
        RetryConfig, ServerConfig, TimeoutConfig,
    };
    use crate::core::reliability::{
        CircuitBreakerLayer, ConcurrencyLimitLayer, RateLimitLayer, RetryLayer, apply_reliability,
        build_circuit_breaker_layer, build_concurrency_layer, build_rate_limit_layer,
        build_retry_layer, build_timeout_layer,
    };
    use crate::core::router::AppState;
    use crate::core::utils::api_resource::ApiResourceRegistry;
    use axum::Router;
    use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
    use proptest::prelude::*;
    use reqwest::Client;
    use std::sync::Arc;
    use std::time::Duration;
    use std::time::SystemTime;

    #[test]
    fn test_build_retry_layer() {
        // Test with enabled config
        let config = RetryConfig {
            enabled: true,
            max_attempts: 3,
            base_delay_ms: 10,
            max_delay_ms: 100,
            use_exponential_backoff: true,
            retry_status_codes: vec![503, 502],
        };

        let retry_layer = build_retry_layer(&config);
        assert!(retry_layer.is_some());

        // Test with disabled config
        let disabled_config = RetryConfig {
            enabled: false,
            ..config
        };

        let retry_layer = build_retry_layer(&disabled_config);
        assert!(retry_layer.is_none());
    }

    #[test]
    fn test_build_circuit_breaker_layer() {
        // Test with enabled config
        let config = CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 5,
            reset_timeout_ms: 30000,
            success_threshold: 2,
            window_seconds: 60,
            failure_percentage: 50,
            use_consecutive_failures: true,
            failure_status_codes: vec![500, 503],
        };

        let cb_layer = build_circuit_breaker_layer(&config);
        assert!(cb_layer.is_some());

        // Test with disabled config
        let disabled_config = CircuitBreakerConfig {
            enabled: false,
            ..config
        };

        let cb_layer = build_circuit_breaker_layer(&disabled_config);
        assert!(cb_layer.is_none());
    }

    #[test]
    fn test_build_timeout_layer() {
        // Test with enabled config
        let config = TimeoutConfig {
            enabled: true,
            timeout_seconds: 30,
        };

        let timeout_layer = build_timeout_layer(&config);
        assert!(timeout_layer.is_some());

        // Test with disabled config
        let disabled_config = TimeoutConfig {
            enabled: false,
            ..config
        };

        let timeout_layer = build_timeout_layer(&disabled_config);
        assert!(timeout_layer.is_none());
    }

    #[test]
    fn test_build_rate_limit_layer() {
        // Test with enabled config
        let config = RateLimitConfig {
            enabled: true,
            requests_per_window: 100,
            window_seconds: 60,
            per_client: true,
        };

        let rate_limit_layer = build_rate_limit_layer(&config);
        assert!(rate_limit_layer.is_some());

        // Test with disabled config
        let disabled_config = RateLimitConfig {
            enabled: false,
            ..config
        };

        let rate_limit_layer = build_rate_limit_layer(&disabled_config);
        assert!(rate_limit_layer.is_none());
    }

    #[test]
    fn test_build_concurrency_layer() {
        // Test with enabled config
        let config = ConcurrencyConfig {
            enabled: true,
            max_concurrent_requests: 50,
        };

        let concurrency_layer = build_concurrency_layer(&config);
        assert!(concurrency_layer.is_some());

        // Test with disabled config
        let disabled_config = ConcurrencyConfig {
            enabled: false,
            ..config
        };

        let concurrency_layer = build_concurrency_layer(&disabled_config);
        assert!(concurrency_layer.is_none());
    }

    proptest! {
        #[test]
        fn test_retry_config_random_values(
            max_attempts in 1u32..10u32,
            base_delay_ms in 5u64..1000u64,
            max_delay_ms in 100u64..5000u64,
            use_exponential_backoff in proptest::bool::ANY,
            retry_codes_count in 1usize..5usize
        ) {
            // Generate random status codes between 400-599 for retry
            let retry_status_codes: Vec<u16> = (0..retry_codes_count)
                .map(|_| {
                    let random_offset = (rand::random::<f32>() * 200.0) as u16;
                    400 + random_offset.min(199)
                })
                .collect();

            let config = RetryConfig {
                enabled: true,
                max_attempts,
                base_delay_ms,
                max_delay_ms,
                use_exponential_backoff,
                retry_status_codes,
            };

            let retry_layer = build_retry_layer(&config);
            prop_assert!(retry_layer.is_some());
        }

        #[test]
        fn test_circuit_breaker_config_random_values(
            failure_threshold in 1u32..20u32,
            reset_timeout_ms in 100u64..60000u64,
            success_threshold in 1u32..10u32,
            window_seconds in 1u64..300u64,
            failure_percentage in 10u8..100u8,
            use_consecutive_failures in proptest::bool::ANY,
            failure_codes_count in 1usize..5usize
        ) {
            // Generate random status codes between 400-599 for failure detection
            let failure_status_codes: Vec<u16> = (0..failure_codes_count)
                .map(|_| {
                    let random_offset = (rand::random::<f32>() * 200.0) as u16;
                    400 + random_offset.min(199)
                })
                .collect();

            let config = CircuitBreakerConfig {
                enabled: true,
                failure_threshold,
                reset_timeout_ms,
                success_threshold,
                window_seconds,
                failure_percentage,
                use_consecutive_failures,
                failure_status_codes,
            };

            let cb_layer = build_circuit_breaker_layer(&config);
            prop_assert!(cb_layer.is_some());
        }
    }

    #[test]
    fn test_apply_reliability() {
        // Create metrics handle
        let metrics_recorder = PrometheusBuilder::new().build_recorder();
        let metrics_handle = metrics_recorder.handle();

        // Create minimal AppState for testing
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            start_time: SystemTime::now(),
            cache_registry: Some(Arc::new(CacheRegistry::default())),
            client: Some(Client::new()),
            db_pool: None,
            token_client: Some(Arc::new(MockTokenClient::new())),
            metrics_handle: Some(metrics_handle),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        });

        // Create a router
        let router = Router::new().with_state(state);

        // Create a reliability config with only timeout enabled
        let reliability_config = ReliabilityConfig {
            timeout: TimeoutConfig {
                enabled: true,
                timeout_seconds: 30,
            },
            retry: RetryConfig {
                enabled: false,
                ..Default::default()
            },
            circuit_breaker: CircuitBreakerConfig {
                enabled: false,
                ..Default::default()
            },
            rate_limit: RateLimitConfig {
                enabled: false,
                ..Default::default()
            },
            concurrency: ConcurrencyConfig {
                enabled: false,
                ..Default::default()
            },
        };

        // Apply reliability to the router
        let _router_with_reliability = apply_reliability(router, &reliability_config);

        // We're mainly testing that this doesn't panic
        assert!(true);
    }
}
