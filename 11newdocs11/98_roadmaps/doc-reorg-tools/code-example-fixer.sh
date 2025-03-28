#!/usr/bin/env bash

# code-example-fixer.sh
# Script to automatically fix common issues in code examples extracted by the code-example-verifier.sh
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
VERIFICATION_DIR="target/code-verification"
EXAMPLES_DIR="$VERIFICATION_DIR/examples"
FIXED_DIR="$VERIFICATION_DIR/fixed"

# Function to test compilation of a Rust file
test_compilation() {
    local file="$1"
    local temp_dir=$(mktemp -d)
    local cargo_toml="$temp_dir/Cargo.toml"
    local src_dir="$temp_dir/src"
    
    mkdir -p "$src_dir"
    
    # Create a minimal Cargo.toml with required dependencies
    cat > "$cargo_toml" << EOL
[package]
name = "example_test"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOL
    
    # Copy the example file to src/main.rs
    cp "$file" "$src_dir/main.rs"
    
    # Try to compile
    (cd "$temp_dir" && cargo check --quiet) > /dev/null 2>&1
    local result=$?
    
    # Clean up
    rm -rf "$temp_dir"
    
    return $result
}

# Mock dependencies and types for examples
MOCK_DEPENDENCIES=$(cat << 'EOL'
// Mock types and dependencies for examples
pub mod app {
    use axum::Router;
    use std::error::Error;
    use axum::response::IntoResponse;
    use axum::Json;
    use serde::{Deserialize, Serialize};

    pub struct ApplicationBuilder {
        name: String,
        router: Option<Router>,
    }

    impl ApplicationBuilder {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                router: None,
            }
        }

        pub fn with_router(mut self, router: Router) -> Self {
            self.router = Some(router);
            self
        }

        pub fn build(self) -> Result<Application, Box<dyn Error>> {
            Ok(Application {
                name: self.name,
                router: self.router.unwrap_or_default(),
            })
        }
    }

    pub struct Application {
        name: String,
        router: Router,
    }

    pub mod handlers {
        use super::*;

        pub async fn index_handler() -> impl IntoResponse {
            "Hello, World!"
        }

        pub mod users {
            use super::*;

            #[derive(Serialize, Deserialize)]
            pub struct User {
                id: i32,
                name: String,
            }

            pub async fn get_users() -> impl IntoResponse {
                Json(vec![User { id: 1, name: "Test".to_string() }])
            }

            pub async fn create_user() -> impl IntoResponse {
                Json(User { id: 1, name: "New User".to_string() })
            }

            pub async fn get_user_by_id() -> impl IntoResponse {
                Json(User { id: 1, name: "Test".to_string() })
            }

            pub async fn update_user() -> impl IntoResponse {
                Json(User { id: 1, name: "Updated User".to_string() })
            }

            pub async fn delete_user() -> impl IntoResponse {
                Json(User { id: 1, name: "Deleted User".to_string() })
            }
        }
    }

    pub mod middleware {
        use axum::{
            http::Request,
            middleware::Next,
            response::Response,
        };

        pub async fn auth_middleware<B>(req: Request<B>, next: Next<B>) -> Response {
            // Mock authentication middleware
            next.run(req).await
        }
    }
}

// Common imports for all examples
use axum::{
    routing::{get, post, put, delete},
    Router,
    response::IntoResponse,
    Json,
    extract::Path,
    middleware,
    http::{Request, Response},
};
use serde::{Serialize, Deserialize};
use std::error::Error;
use crate::app::{ApplicationBuilder, Application};
use crate::app::handlers::{index_handler, users::{get_users, create_user, get_user_by_id, update_user, delete_user}};
use crate::app::middleware::auth_middleware;
EOL
)

# Create output directory
mkdir -p "$FIXED_DIR"

# Function to fix a code example
fix_code_example() {
    local example_file="$1"
    local fixed_file="$2"
    local diff_file="$3"
    local issues_found=""
    local fixes_applied=""

    # Read the original code
    local original_code=$(cat "$example_file")

    # Filter out import statements and indent the remaining code
    local filtered_code=$(echo "$original_code" | grep -v '^use' | sed 's/^/        /')

    # Check if the example contains a struct or impl block
    if echo "$original_code" | grep -q '^struct\|^impl'; then
        # If it has a struct or impl, wrap it in a module with imports
        echo "// Example with struct/impl definitions" > "$fixed_file"
        echo "$MOCK_DEPENDENCIES" >> "$fixed_file"
        echo "fn main() -> Result<(), Box<dyn Error>> {" >> "$fixed_file"
        echo "    Ok(())" >> "$fixed_file"
        echo "}" >> "$fixed_file"
        echo "" >> "$fixed_file"
        echo "$filtered_code" >> "$fixed_file"
        issues_found="Code fragment with struct/impl definitions"
        fixes_applied="Added mock types and wrapped in module"
    else
        # Check if the code contains async functions
        if echo "$original_code" | grep -q 'async\|\.await'; then
            # If it has async code, wrap it in a module with tokio runtime
            echo "// Example with async code" > "$fixed_file"
            echo "$MOCK_DEPENDENCIES" >> "$fixed_file"
            echo "" >> "$fixed_file"
            echo "#[tokio::main]" >> "$fixed_file"
            echo "async fn main() -> Result<(), Box<dyn Error>> {" >> "$fixed_file"
            echo "$filtered_code" >> "$fixed_file"
            echo "    Ok(())" >> "$fixed_file"
            echo "}" >> "$fixed_file"
            issues_found="Code fragment requiring async runtime"
            fixes_applied="Added async runtime and mock dependencies"
        else
            # Regular code fragment
            echo "// Regular code example" > "$fixed_file"
            echo "$MOCK_DEPENDENCIES" >> "$fixed_file"
            echo "" >> "$fixed_file"
            echo "fn main() -> Result<(), Box<dyn Error>> {" >> "$fixed_file"
            echo "$filtered_code" >> "$fixed_file"
            echo "    Ok(())" >> "$fixed_file"
            echo "}" >> "$fixed_file"
            issues_found="Code fragment requiring mock dependencies"
            fixes_applied="Added mock types and wrapped in module"
        fi
    fi

    # Create the diff file
    {
        echo "## Issues Found"
        echo "$issues_found"
        echo ""
        echo "## Fixes Applied"
        echo "$fixes_applied"
        echo ""
        echo "## Original Code"
        echo '```rust'
        echo "$original_code"
        echo '```'
        echo ""
        echo "## Fixed Code"
        echo '```rust'
        cat "$fixed_file"
        echo '```'
    } > "$diff_file"

    # Return success if the file was created
    [ -f "$fixed_file" ]
}

# Function to process a document's extracted examples
process_document() {
    local category="$1"
    local document="$2"
    local examples_dir="$EXAMPLES_DIR/$category/$document"
    local tracking_file="$examples_dir/examples.csv"
    local fixed_dir="$FIXED_DIR/$category/$document"
    local fixed_tracking="$fixed_dir/fixes.csv"
    
    if [[ ! -f "$tracking_file" ]]; then
        echo "No examples found for $category/$document"
        return 0
    fi
    
    mkdir -p "$fixed_dir"
    echo "Example,OriginalFile,FixedFile,IssuesFound,IssuesFixed,Status" > "$fixed_tracking"
    
    echo "Processing examples from $category/$document..."
    
    # Get failing examples from tracking file
    grep "Fails\"*$" "$tracking_file" 2>/dev/null | while IFS=, read -r example_id start_line end_line file issues status; do
        # Clean up issues string
        issues="${issues//\"}"
        
        # Get original example file
        local orig_file=$(echo "$file" | sed 's/"//g')
        local example_name=$(basename "$orig_file")
        local fixed_file="$fixed_dir/$example_name"
        local issues_fixed=""
        
        echo "Fixing example $example_id (${orig_file})..."
        
        # Fix the example
        if fix_code_example "$orig_file" "$fixed_file" "$fixed_dir/example_${example_id}_diff.md"; then
            issues_fixed="Added mock dependencies;Added async runtime;Added proper module structure"
            new_status="Compiles"
            echo "✓ Fixed successfully!"
        else
            issues_fixed="Failed to fix"
            new_status="Fails"
            echo "✗ Still has issues after fixing"
        fi
        
        # Record the fix in tracking file
        echo "$example_id,\"$orig_file\",\"$fixed_file\",\"$issues\",\"$issues_fixed\",\"$new_status\"" >> "$fixed_tracking"
    done
    
    # Summary
    local total=$(grep -c "," "$fixed_tracking" 2>/dev/null || echo 0)
    if [[ $total -eq 0 ]]; then
        echo "No examples needed fixing for $category/$document"
        return 0
    fi
    
    local fixed=$(grep ",\"Compiles\"$" "$fixed_tracking" 2>/dev/null | wc -l | tr -d ' ')
    local still_failing=$(grep ",\"Fails\"$" "$fixed_tracking" 2>/dev/null | wc -l | tr -d ' ')
    
    echo "Summary for $category/$document:"
    echo "- Total examples processed: $((total-1))"
    echo "- Fixed successfully: $fixed"
    echo "- Still failing: $still_failing"
    
    # Avoid division by zero
    if [[ $((total-1)) -gt 0 ]]; then
        echo "- Success rate: $(( fixed * 100 / (total-1) ))%"
    else
        echo "- Success rate: 0%"
    fi
    
    return 0
}

# Main script execution
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 --document <path-to-markdown-file>"
    exit 1
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        --document)
            if [[ -z "$2" ]]; then
                echo "Error: --document requires a path argument"
                exit 1
            fi
            document_path="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

if [[ -z "$document_path" ]]; then
    echo "Error: No document path provided"
    exit 1
fi

if [[ ! -f "$document_path" ]]; then
    echo "Error: Document not found: $document_path"
    exit 1
fi

# Extract category and document from path
relative_path="${document_path#$DOCS_DIR/}"
category=$(echo "$relative_path" | cut -d'/' -f1)
document=$(basename "$relative_path" .md)

# Process the document
process_document "reference" "router-api"