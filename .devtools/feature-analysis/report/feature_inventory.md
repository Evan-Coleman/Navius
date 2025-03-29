# Feature Flag Inventory

## Summary

- Total Rust files analyzed: 165
- Files with feature flags: 6
- Features defined in Cargo.toml: 22
- Features used in codebase: 11

## Defined Features

### default

Dependencies:
- production
- tracing
- metrics
- logging
- database
- examples
- prometheus

Used in 1 files.

### production

No dependencies.

Used in 1 files.

### development

No dependencies.

Not used directly in code.

### tracing

Dependencies:
- tracing-subscriber
- tracing-appender

Not used directly in code.

### metrics

Dependencies:
- metrics-exporter-prometheus

Used in 1 files.

### logging

Dependencies:
- tracing

Not used directly in code.

### redis

No dependencies.

Not used directly in code.

### database

No dependencies.

Not used directly in code.

### examples

No dependencies.

Used in 1 files.

### test-utils

No dependencies.

Not used directly in code.

### experimental

No dependencies.

Not used directly in code.

### reliability

No dependencies.

Used in 1 files.

### caching

No dependencies.

Used in 1 files.

### auth

No dependencies.

Used in 1 files.

### advanced_metrics

No dependencies.

Used in 1 files.

### prometheus

Dependencies:
- metrics-exporter-prometheus

Not used directly in code.

### dynatrace

Dependencies:
- dep:opentelemetry-dynatrace

Not used directly in code.

### opentelemetry-jaeger

Dependencies:
- dep:opentelemetry-jaeger

Used in 3 files.

### otlp

Dependencies:
- dep:opentelemetry-otlp

Used in 3 files.

### postgres

Dependencies:
- database

Not used directly in code.

### sqlx-macros

No dependencies.

Not used directly in code.

### rusqlite

Dependencies:
- []  # Placeholder for future SQLite support

Not used directly in code.

## Feature Usage

### std

Status: ❌ Not defined in Cargo.toml

Used in 1 files:
- `target/release/build/crunchy-57f5b2c92120eeec/out/lib.rs`

### metrics

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/features/runtime.rs`

### caching

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/features/runtime.rs`

### reliability

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/features/runtime.rs`

### advanced_metrics

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/features/runtime.rs`

### auth

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/features/runtime.rs`

### opentelemetry-jaeger

Status: ✓ Defined in Cargo.toml

Used in 3 files:
- `src/core/observability/mod.rs`
- `src/core/observability/opentelemetry.rs`
- `src/core/observability/service.rs`

### otlp

Status: ✓ Defined in Cargo.toml

Used in 3 files:
- `src/core/observability/mod.rs`
- `src/core/observability/opentelemetry.rs`
- `src/core/observability/service.rs`

### default

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/handlers/core_actuator.rs`

### examples

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/handlers/core_actuator.rs`

### production

Status: ✓ Defined in Cargo.toml

Used in 1 files:
- `src/core/handlers/core_actuator.rs`

## Recommendations

### Unused Features

The following features are defined but not directly used in code (they might be used indirectly through dependencies):

- `development`
- `tracing`
- `logging`
- `redis`
- `database`
- `test-utils`
- `experimental`
- `prometheus`
- `dynatrace`
- `postgres`
- `sqlx-macros`
- `rusqlite`

### Undefined Features

The following features are used in code but not defined in Cargo.toml:

- `std`

