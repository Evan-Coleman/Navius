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

//! # Custom Cache Providers
//!
//! This module allows you to define custom cache providers that extend the core caching system.

// Re-export core cache types
pub use crate::core::cache::CacheStats;

// Example: Add your custom cache providers here
// pub mod two_tier_cache;
// pub mod distributed_cache;
// pub mod encrypted_cache;

// Cache providers
pub mod two_tier_cache;

// Export public types
pub use two_tier_cache::TwoTierCache;
