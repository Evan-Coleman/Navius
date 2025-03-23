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

//! # Application Metrics
//!
//! This module contains application-specific metrics functions that can be
//! customized by users. Core metrics functionality is provided by `crate::core::metrics`.

// Re-export core metrics functionality
pub use crate::core::metrics::*;

// Example of defining application-specific metrics:
//
// pub fn record_user_login() {
//     metrics::counter!("user_logins_total", 1);
// }
//
// pub fn record_api_request_duration(endpoint: &str, duration_ms: f64) {
//     metrics::histogram!("api_request_duration_ms", duration_ms, "endpoint" => endpoint.to_string());
// }
//
// pub fn record_cache_hit_rate(rate: f64) {
//     metrics::gauge!("cache_hit_rate", rate);
// }
