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

//! # Application Caching
//!
//! This module contains user-facing caching functionality that can be customized.
//! Core caching capabilities are provided by `crate::core::cache`.

use crate::core::error::Result;

// Re-export core cache functionality
pub use crate::core::cache::*;

// Cache providers
pub mod providers;

/// Example: Create a cache factory for a specific use case
///
/// Note: In a real application, this would create an actual cache implementation
/// based on the core cache API
pub fn create_user_cache() -> Result<String> {
    // This is just a placeholder until we have access to the actual cache API
    // In a real implementation, this would create and return a cache instance

    Ok("Example cache".to_string())
}
