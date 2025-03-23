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

//! User-facing API endpoints that can be extended or customized by application developers.
//!
//! This module contains all the API endpoint implementations that are intended
//! to be customized or extended by users of the framework.

// Add your API modules here:
// pub mod users;
// pub mod products;
// pub mod orders;

/// Example API implementations
pub mod examples;

/// Helper function to merge all API routes together
///
/// # Example
/// ```
/// use axum::Router;
/// use navius::app::api::routes;
///
/// let app = Router::new().nest("/api", routes());
/// ```
pub fn routes() -> axum::Router {
    use axum::Router;
    use axum::routing::get;

    // Create a router with our example endpoints
    Router::new().route("/pets/:id", get(examples::pet::fetch_pet_handler))

    // Add your own routes below:
    // .merge(users::routes())
    // .merge(products::routes())
    // .merge(orders::routes())
}
