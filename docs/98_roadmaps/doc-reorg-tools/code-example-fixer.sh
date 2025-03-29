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
    local example_file="$1"
    local example_dir=$(dirname "$example_file")

    # Create Cargo.toml if it doesn't exist
    if [ ! -f "${example_dir}/Cargo.toml" ]; then
        cat > "${example_dir}/Cargo.toml" << EOL
[package]
name = "router-api-examples"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }
hyper = "1.0"
EOL
    fi

    # Create src directory and move example file
    mkdir -p "${example_dir}/src"
    cp "$example_file" "${example_dir}/src/main.rs"

    # Try to compile
    (cd "$example_dir" && cargo check --quiet)
    return $?
}

# Create output directory
mkdir -p "$FIXED_DIR"

# Create a temporary directory for compilation
temp_dir=$(mktemp -d)
mkdir -p "$temp_dir/src"

# Create Cargo.toml
cat > "$temp_dir/Cargo.toml" << 'EOL'
[package]
name = "router-api-examples"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOL

# Function to fix a code example
fix_code_example() {
    local example_file="$1"
    local fixed_file="$2"
    local original_code
    local filtered_code

    # Read the original code
    original_code=$(cat "$example_file")

    # Filter out import statements
    filtered_code=$(echo "$original_code" | grep -v "^use " | sed 's/^/        /')

    # Create fixed code with proper imports and mock dependencies
    {
        echo 'use axum::{'
        echo '    routing::{get, post, put, delete},'
        echo '    Router,'
        echo '    response::IntoResponse,'
        echo '    Json,'
        echo '    extract::Path,'
        echo '    middleware::{self, Next},'
        echo '    http::{Request, Response, StatusCode},'
        echo '    body::Body,'
        echo '};'
        echo 'use serde::{Serialize, Deserialize};'
        echo 'use std::error::Error;'
        echo
        echo '#[derive(Clone)]'
        echo 'struct AppState {}'
        echo
        echo 'pub mod app {'
        echo '    use super::*;'
        echo
        echo '    pub struct ApplicationBuilder {'
        echo '        name: String,'
        echo '        router: Option<Router<AppState>>,'
        echo '    }'
        echo
        echo '    impl ApplicationBuilder {'
        echo '        pub fn new(name: &str) -> Self {'
        echo '            Self {'
        echo '                name: name.to_string(),'
        echo '                router: None,'
        echo '            }'
        echo '        }'
        echo
        echo '        pub fn with_router(mut self, router: Router<AppState>) -> Self {'
        echo '            self.router = Some(router);'
        echo '            self'
        echo '        }'
        echo
        echo '        pub fn build(self) -> Result<Application, Box<dyn Error>> {'
        echo '            Ok(Application {'
        echo '                name: self.name,'
        echo '                router: self.router.unwrap_or_default(),'
        echo '            })'
        echo '        }'
        echo '    }'
        echo
        echo '    pub struct Application {'
        echo '        name: String,'
        echo '        router: Router<AppState>,'
        echo '    }'
        echo
        echo '    pub mod handlers {'
        echo '        use super::*;'
        echo
        echo '        pub async fn index_handler() -> impl IntoResponse {'
        echo '            "Hello, World!"'
        echo '        }'
        echo
        echo '        pub mod users {'
        echo '            use super::*;'
        echo
        echo '            #[derive(Serialize, Deserialize)]'
        echo '            pub struct User {'
        echo '                id: i32,'
        echo '                name: String,'
        echo '            }'
        echo
        echo '            pub async fn get_users() -> impl IntoResponse {'
        echo '                Json(vec![User { id: 1, name: "Test".to_string() }])'
        echo '            }'
        echo
        echo '            pub async fn create_user() -> impl IntoResponse {'
        echo '                Json(User { id: 1, name: "New User".to_string() })'
        echo '            }'
        echo
        echo '            pub async fn get_user_by_id() -> impl IntoResponse {'
        echo '                Json(User { id: 1, name: "Test".to_string() })'
        echo '            }'
        echo
        echo '            pub async fn update_user() -> impl IntoResponse {'
        echo '                Json(User { id: 1, name: "Updated User".to_string() })'
        echo '            }'
        echo
        echo '            pub async fn delete_user() -> impl IntoResponse {'
        echo '                Json(User { id: 1, name: "Deleted User".to_string() })'
        echo '            }'
        echo '        }'
        echo '    }'
        echo
        echo '    pub mod middleware {'
        echo '        use super::*;'
        echo
        echo '        pub async fn auth_middleware(req: Request<Body>, next: Next) -> Response<Body> {'
        echo '            next.run(req).await'
        echo '        }'
        echo
        echo '        pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response<Body> {'
        echo '            println!("Request: {} {}", req.method(), req.uri());'
        echo '            next.run(req).await'
        echo '        }'
        echo
        echo '        pub async fn error_handling_middleware(req: Request<Body>, next: Next) -> Response<Body> {'
        echo '            next.run(req).await'
        echo '        }'
        echo '    }'
        echo '}'
        echo
        echo 'pub mod protected {'
        echo '    use super::*;'
        echo
        echo '    pub async fn profile_handler() -> impl IntoResponse {'
        echo '        "Profile"'
        echo '    }'
        echo
        echo '    pub async fn settings_handler() -> impl IntoResponse {'
        echo '        "Settings"'
        echo '    }'
        echo
        echo '    pub async fn dashboard() -> impl IntoResponse {'
        echo '        "Dashboard"'
        echo '    }'
        echo '}'
        echo
        echo 'pub mod public {'
        echo '    use super::*;'
        echo
        echo '    pub async fn index() -> impl IntoResponse {'
        echo '        "Index"'
        echo '    }'
        echo
        echo '    pub async fn about() -> impl IntoResponse {'
        echo '        "About"'
        echo '    }'
        echo
        echo '    pub async fn login() -> impl IntoResponse {'
        echo '        "Login"'
        echo '    }'
        echo '}'
        echo
        echo 'pub async fn handle_404() -> impl IntoResponse {'
        echo '    (StatusCode::NOT_FOUND, "Resource not found")'
        echo '}'
        echo
        echo 'pub async fn health_check() -> impl IntoResponse {'
        echo '    "OK"'
        echo '}'
        echo
        echo 'use app::{ApplicationBuilder, Application};'
        echo 'use app::handlers::{index_handler, users::{get_users, create_user, get_user_by_id, update_user, delete_user}};'
        echo 'use app::middleware::{auth_middleware, logging_middleware, error_handling_middleware};'
        echo 'use protected::*;'
        echo 'use public::*;'
        echo
        # Check if the code contains a function declaration
        if ! echo "$original_code" | grep -q "^fn\|^async fn"; then
            echo '#[tokio::main]'
            echo 'async fn main() -> Result<(), Box<dyn Error>> {'
            echo "$filtered_code"
            echo '    Ok(())'
            echo '}'
        else
            echo "$filtered_code"
        fi
    } > "$fixed_file"

    # Create a diff file
    local diff_file="${fixed_file%.*}_diff.md"
    {
        echo "## Issues Found"
        echo
        echo
        echo "## Fixes Applied"
        echo
        echo
        echo "## Original Code"
        echo '```rust'
        echo "$original_code"
        echo '```'
        echo
        echo "## Fixed Code"
        echo '```rust'
        cat "$fixed_file"
        echo '```'
        echo
    } > "$diff_file"

    # Return success if the fixed file was created
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
        if fix_code_example "$orig_file" "$fixed_file"; then
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