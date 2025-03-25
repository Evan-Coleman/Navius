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

//! # Application Authentication
//!
//! This module contains user-facing authentication functionality that can be customized.
//! Core authentication capabilities are provided by `crate::core::auth`.

// Re-export core auth functionality
pub use crate::core::auth::*;

// Example: Create a custom client module
pub mod client;
