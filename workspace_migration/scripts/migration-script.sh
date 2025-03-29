#!/bin/bash
# Workspace Migration Script
# This script helps automate the process of migrating to a workspace structure

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print a header
print_header() {
    echo -e "\n${BLUE}==== $1 ====${NC}\n"
}

# Function to print success message
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error message
error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# Function to print warning message
warning() {
    echo -e "${YELLOW}! $1${NC}"
}

# Function to create a crate directory
create_crate() {
    crate_name=$1
    crate_description=$2
    
    print_header "Creating crate: $crate_name"
    
    # Create directory structure
    mkdir -p "crates/$crate_name/src"
    success "Created directory structure"
    
    # Create Cargo.toml
    cat > "crates/$crate_name/Cargo.toml" <<EOL
[package]
name = "$crate_name"
version = "0.1.0"
edition = "2024"
description = "$crate_description"
license = "Apache-2.0"
repository = "https://github.com/Evan-Coleman/Navius"
readme = "README.md"

# Dependencies section uses workspace inheritance
[dependencies]
# Core dependencies from workspace
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }

# Add other dependencies as needed

# Development dependencies
[dev-dependencies]
mockito = { workspace = true }
proptest = { workspace = true }
test-context = { workspace = true }

# The crate has minimal feature flags
[features]
default = []
EOL
    success "Created Cargo.toml"
    
    # Create lib.rs
    cat > "crates/$crate_name/src/lib.rs" <<EOL
//! $crate_description

// Modules
pub mod error;

// Re-exports
pub use error::{Error, Result};

/// Initialize the module
pub fn init() -> Result<()> {
    tracing::info!("Initializing $crate_name");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let result = init();
        assert!(result.is_ok());
    }
}
EOL
    success "Created lib.rs"
    
    # Create error.rs
    cat > "crates/$crate_name/src/error.rs" <<EOL
//! Error types for $crate_name

use std::fmt;
use thiserror::Error;

/// A specialized Result type for $crate_name operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for $crate_name.
#[derive(Error, Debug)]
pub enum Error {
    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),

    /// A validation error occurred.
    #[error("Validation error: {0}")]
    Validation(String),

    /// An error that originated in the core crate.
    #[error("Core error: {0}")]
    Core(#[from] navius_core::error::Error),
}

impl Error {
    /// Create a new internal error.
    pub fn internal<T: fmt::Display>(msg: T) -> Self {
        Self::Internal(msg.to_string())
    }

    /// Create a new validation error.
    pub fn validation<T: fmt::Display>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let err = Error::internal("Something went wrong");
        assert!(matches!(err, Error::Internal(_)));
    }
}
EOL
    success "Created error.rs"
    
    # Create README.md
    cat > "crates/$crate_name/README.md" <<EOL
# $crate_name

$crate_description

## Usage

\`\`\`rust
use $crate_name::init;

fn main() {
    init().expect("Failed to initialize $crate_name");
}
\`\`\`
EOL
    success "Created README.md"
    
    echo -e "\n${GREEN}Successfully created $crate_name crate!${NC}"
}

# Function to update workspace Cargo.toml
update_workspace_toml() {
    print_header "Updating workspace Cargo.toml"
    
    # Check if already converted to workspace
    if grep -q "\[workspace\]" Cargo.toml; then
        warning "Cargo.toml already contains workspace configuration"
        return
    fi
    
    # Backup original Cargo.toml
    cp Cargo.toml Cargo.toml.bak
    success "Created backup of Cargo.toml at Cargo.toml.bak"
    
    # Create new workspace Cargo.toml
    cat > Cargo.toml.new <<EOL
# Navius workspace Cargo.toml
# Converted from feature flags to workspace approach

[workspace]
members = [
    "crates/navius-core",
    # Add new crates here as they are created
]

resolver = "2" # Use the modern resolver for proper feature resolution

# Shared dependencies section
[workspace.dependencies]
# Copy dependencies from the original Cargo.toml and adapt as needed
EOL

    # Extract dependencies from original Cargo.toml
    sed -n '/\[dependencies\]/,/\[dev-dependencies\]/p' Cargo.toml | grep -v "\[dependencies\]\|\[dev-dependencies\]" >> Cargo.toml.new
    
    # Add dev dependencies
    echo -e "\n# Development dependencies available to all workspace members\n[workspace.dev-dependencies]" >> Cargo.toml.new
    sed -n '/\[dev-dependencies\]/,/\[build-dependencies\]/p' Cargo.toml | grep -v "\[dev-dependencies\]\|\[build-dependencies\]" >> Cargo.toml.new
    
    # Add profiles
    cat >> Cargo.toml.new <<EOL

# Profiles shared across the workspace
[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
debug = false
lto = "thin"
codegen-units = 1
strip = true

[profile.test]
opt-level = 0
debug = true

# This package section is for the root crate
[package]
EOL

    # Extract package info
    sed -n '/\[package\]/,/\[features\]/p' Cargo.toml | grep -v "\[package\]\|\[features\]" >> Cargo.toml.new
    
    # Add main crate dependencies
    cat >> Cargo.toml.new <<EOL

# Root package dependencies - minimal, mostly just for workspace management
[dependencies]
# Core crate
navius-core = { path = "crates/navius-core", version = "0.1.0" }

# Add other crates as they are created
EOL

    # Move new file into place
    mv Cargo.toml.new Cargo.toml
    success "Updated Cargo.toml with workspace configuration"
}

# Function to analyze code for module extraction
analyze_module() {
    module_name=$1
    
    print_header "Analyzing code for module: $module_name"
    
    # Find all files that mention the module
    echo "Files that mention $module_name:"
    grep -r "$module_name" --include="*.rs" src/ || true
    
    # Count occurrences of feature flags
    echo -e "\nFeature flag usage:"
    grep -r "feature\s*=\s*\"$module_name\"" --include="*.rs" src/ | wc -l
    
    echo -e "\nCurrent implementation files:"
    find src/ -name "*$module_name*.rs" -o -path "*/$module_name/*"
}

# Main script execution
print_header "Navius Workspace Migration Script"

# Check command line arguments
if [ $# -eq 0 ]; then
    echo "Usage:"
    echo "  $0 init                       # Initialize workspace structure"
    echo "  $0 create <crate-name> <desc> # Create a new crate"
    echo "  $0 analyze <module-name>      # Analyze code for module extraction"
    exit 1
fi

command=$1

case $command in
    init)
        # Initialize workspace structure
        mkdir -p crates
        update_workspace_toml
        create_crate "navius-core" "Core functionality for the Navius framework"
        ;;
    create)
        if [ $# -lt 3 ]; then
            error "Missing crate name or description"
        fi
        create_crate "$2" "$3"
        ;;
    analyze)
        if [ $# -lt 2 ]; then
            error "Missing module name to analyze"
        fi
        analyze_module "$2"
        ;;
    *)
        error "Unknown command: $command"
        ;;
esac

success "Operation completed successfully" 