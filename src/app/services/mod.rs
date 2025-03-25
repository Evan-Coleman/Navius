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

use crate::app::database::repositories::pet_repository::PetRepository;
use std::any::Any;
use std::sync::Arc;

// Make these modules public so they can be imported by other modules
pub mod dto;
pub mod error;
pub mod pet_service;

// Re-export these types for convenience
pub use dto::{CreatePetDto, UpdatePetDto};
pub use error::ServiceError;
pub use pet_service::{IPetService, PetService};

/// Default implementation of ServiceRegistry that can be used by applications
pub struct DefaultServiceRegistry {
    // service fields go here
    pet_service: Arc<dyn Any + Send + Sync>,
}

impl DefaultServiceRegistry {
    /// Create a new DefaultServiceRegistry with all required services
    pub fn new(pet_repository: Arc<dyn PetRepository>) -> Self {
        Self {
            // Initialize your services here
            pet_service: Arc::new(PetService::new(pet_repository)) as Arc<dyn Any + Send + Sync>,
        }
    }
}

// Add missing trait implementation from the "server-info" rule
impl crate::core::services::ServiceRegistryTrait for DefaultServiceRegistry {
    // Implement access methods
    fn pet_service(&self) -> &dyn Any {
        self.pet_service.as_ref()
    }
}

impl Default for DefaultServiceRegistry {
    fn default() -> Self {
        panic!("DefaultServiceRegistry requires dependencies to be initialized")
    }
}
