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

//! User-extensible models
//!
//! This module provides a place for users to define their own models
//! that extend the core models. Follow the established patterns
//! and error handling guidelines when creating new models.

use crate::core::error::Result;

// Add your custom models below
// Example:
// #[derive(Debug, Serialize, Deserialize)]
// pub struct User {
//     pub id: String,
//     pub name: String,
//     pub email: String,
// }
//
// impl User {
//     pub fn validate(&self) -> Result<()> {
//         // Validation logic here
//         Ok(())
//     }
// }

/// Marker trait for user-defined models
///
/// Implement this trait for custom models to ensure they
/// follow the core architecture patterns.
pub trait UserModel {
    /// Validate the model data
    fn validate(&self) -> Result<()>;
}

// Example implementation:
// impl UserModel for User {
//     fn validate(&self) -> Result<()> {
//         if self.email.is_empty() {
//             return Err(AppError::validation("Email cannot be empty"));
//         }
//         Ok(())
//     }
// }
