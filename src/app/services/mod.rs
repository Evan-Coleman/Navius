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

//! User-facing services that can be extended or customized by application developers.
//!
//! This module contains all the service implementations that are intended
//! to be customized or extended by users of the framework.

// Pet repositories imports removed for stability
use std::any::Any;
use std::sync::Arc;

// Re-export these types for convenience
// Pet DTOs removed for stability
pub use error::ServiceError;
// Pet service removed for stability

/// Default implementation of ServiceRegistry that can be used by applications
pub struct DefaultServiceRegistry {
    // service fields go here
    // Pet service removed for stability
}

impl DefaultServiceRegistry {
    /// Create a new DefaultServiceRegistry with all required services
    pub fn new() -> Self {
        Self {
            // Initialize your services here
            // Pet service removed for stability
        }
    }
}

// Add missing trait implementation from the "server-info" rule
impl crate::core::services::ServiceRegistryTrait for DefaultServiceRegistry {
    // Pet service removed for stability
}

impl Default for DefaultServiceRegistry {
    fn default() -> Self {
        panic!("DefaultServiceRegistry requires dependencies to be initialized")
    }
}
