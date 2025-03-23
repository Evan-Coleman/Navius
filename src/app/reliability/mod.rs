// Copyright (c) 2025 Navius Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Reliability Features
//!
//! This module contains functionality related to reliability, retries, circuit breaking,
//! and other resilience patterns.

use std::time::Duration;

// Example: Define custom retry policies for specific application use cases
// pub mod custom_retries;

// Example: Define custom rate limiters for specific endpoints
// pub mod api_rate_limits;

// Example: Define custom circuit breakers for external services
// pub mod service_circuit_breakers;

/// Create a retry policy suitable for database operations
///
/// This is an example function that demonstrates how to create a retry policy.
pub fn create_db_retry_policy() -> Duration {
    // This is a placeholder that returns a simple timeout duration
    // In a real application, you would implement a proper retry policy
    Duration::from_millis(100)
}
