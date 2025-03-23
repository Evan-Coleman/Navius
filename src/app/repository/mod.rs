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

//! # Application Repositories
//!
//! This module contains user-defined repositories that extend the core data access layer.
//! Define custom repositories for your application-specific models here.

use crate::core::{database::PgPool, error::Result, repository};

// Re-export core repositories for convenience
pub use crate::core::repository::*;

// Add your custom repositories below
// Example:
// pub mod user_repository;
// pub mod profile_repository;
//
// Remember to:
// 1. Use the provided PgPool for database access
// 2. Implement proper error handling using AppError
// 3. Follow the repository pattern conventions
// 4. Add unit tests for your repositories
// 5. Use transactions where appropriate
