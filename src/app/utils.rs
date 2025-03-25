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

//! # Application Utilities
//!
//! This module contains user-defined utility functions and helpers that extend the core utilities.
//! Core utility functionality is provided by `crate::core::utils`.

use crate::core::{error::Result, utils};

// Re-export core utilities for convenience
pub use crate::core::utils::*;

// Add your custom utilities below
// Example:
// pub mod string_utils;
// pub mod validation_utils;
//
// Remember to:
// 1. Keep functions focused and single-purpose
// 2. Use proper error handling with AppError
// 3. Add comprehensive documentation
// 4. Include unit tests
// 5. Avoid duplicating core functionality
