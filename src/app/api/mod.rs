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

use crate::core::router::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;

// Add your API modules here:
// pub mod users;
// pub mod products;
// pub mod orders;

// Pet-related modules removed for stability

/// Example API implementations
pub mod examples;

/// API handler implementations
pub mod handlers;

/// Helper function to merge all API routes together
///
/// # Example
/// ```
/// use axum::Router;
/// use std::sync::Arc;
/// use navius::app::api::routes;
/// use navius::core::router::AppState;
///
/// let state = Arc::new(AppState::default());
/// let app: Router<Arc<AppState>> = Router::new().nest("/api", routes()).with_state(state);
/// ```
pub fn routes() -> Router<Arc<AppState>> {
    let mut router = Router::new();

    // Swagger Petstore example route
    #[cfg(feature = "examples")]
    {
        router = router.route(
            "/swagger-petstore/pets/{id}",
            get(crate::examples::api::swagger_petstore::fetch_pet_handler),
        );
    }

    // Pet-related routes removed for stability

    // Add your own routes below:
    // router = router.merge(users::routes());
    // router = router.merge(products::routes());
    // router = router.merge(orders::routes());

    router
}
