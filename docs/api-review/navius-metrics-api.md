# navius-metrics API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- tokio
- tracing
- serde
- serde_json
- thiserror
- async-trait
- navius-core
- metrics
- opentelemetry
- opentelemetry-semantic-conventions
- tracing-opentelemetry
- metrics-exporter-prometheus
- opentelemetry-dynatrace
- opentelemetry-jaeger
- opentelemetry-otlp

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| MetricsBuilder | lib.rs | 60 | ✅ Complete |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| MetricsError | lib.rs | 13 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| MetricsCollector: | lib.rs | 35 | ✅ Complete |
| MetricsExporter: | lib.rs | 51 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(service_name: | lib.rs | 68 | ⚠️ Partial |
| with_namespace(mut | lib.rs | 77 | ⚠️ Partial |
| with_label(mut | lib.rs | 83 | ⚠️ Partial |
| build(&self) | lib.rs | 89 | ⚠️ Partial |

