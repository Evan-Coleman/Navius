//! Navius Framework Workspace
//!
//! This is a dummy library for the workspace root to satisfy Cargo.
//! See individual crates for the actual functionality.

/// Returns the framework name
pub fn framework_name() -> &'static str {
    "Navius Framework"
}

/// Returns the framework version
pub fn framework_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
