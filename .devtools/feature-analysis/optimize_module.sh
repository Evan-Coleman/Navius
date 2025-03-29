#!/bin/bash
#
# Feature Flag Optimization Helper Script
# This script helps optimize a module by adding feature gates
#

set -e

# Check if module name was provided
if [ $# -lt 1 ]; then
  echo "Usage: $0 <module_name> [feature_name]"
  echo "Example: $0 database"
  echo "Example: $0 auth auth"
  exit 1
fi

MODULE_NAME=$1
FEATURE_NAME=${2:-$MODULE_NAME}  # Use module name as feature name if not provided

echo "=== Feature Flag Optimization for $MODULE_NAME module ==="
echo "Target feature flag: $FEATURE_NAME"

# Set base directory
BASE_DIR="$(pwd)"
REPORT_DIR="$BASE_DIR/.devtools/feature-analysis/report/optimizations"
mkdir -p "$REPORT_DIR"

# Step 1: Find all Rust files in the module
echo ""
echo "Step 1: Finding Rust files in the module..."
MODULE_FILES=$(find "$BASE_DIR/src" -path "*/$MODULE_NAME/*.rs" -o -path "*/src/$MODULE_NAME.rs" | sort)
FILE_COUNT=$(echo "$MODULE_FILES" | wc -l | tr -d '[:space:]')

if [ "$FILE_COUNT" -eq 0 ]; then
  echo "No files found for module $MODULE_NAME. Check the module name and try again."
  exit 1
fi

echo "Found $FILE_COUNT files in $MODULE_NAME module:"
echo "$MODULE_FILES" | nl

# Step 2: Check for feature flag usage in these files
echo ""
echo "Step 2: Checking for existing feature flags..."
FEATURE_COUNT=0

for file in $MODULE_FILES; do
  FLAGS=$(grep -E "#\[cfg\(.*feature" "$file" || true)
  if [ -n "$FLAGS" ]; then
    echo "Feature flags found in $(basename $file):"
    echo "$FLAGS" | sed 's/^/  /'
    FEATURE_COUNT=$((FEATURE_COUNT + 1))
  fi
done

echo "Found feature flags in $FEATURE_COUNT out of $FILE_COUNT files."

# Step 3: Check for dependencies in Cargo.toml
echo ""
echo "Step 3: Checking for dependencies related to $MODULE_NAME..."
CARGO_TOML="$BASE_DIR/Cargo.toml"
DEPS=$(grep -E -A 1 "^(.*$MODULE_NAME.*| *\"$MODULE_NAME\")" "$CARGO_TOML" || true)

if [ -n "$DEPS" ]; then
  echo "Found dependencies that might be related to $MODULE_NAME:"
  echo "$DEPS" | sed 's/^/  /'
else
  echo "No dependencies found directly matching $MODULE_NAME."
  echo "You'll need to manually identify which dependencies should be made optional."
fi

# Step 4: Check feature definitions
echo ""
echo "Step 4: Checking feature definitions..."
FEATURE_DEF=$(grep -E -A 1 "$FEATURE_NAME *=" "$CARGO_TOML" || true)

if [ -n "$FEATURE_DEF" ]; then
  echo "Feature '$FEATURE_NAME' is defined in Cargo.toml:"
  echo "$FEATURE_DEF" | sed 's/^/  /'
else
  echo "Feature '$FEATURE_NAME' is not defined in Cargo.toml."
  echo "You'll need to add a feature definition."
fi

# Step 5: Generate optimization report
echo ""
echo "Step 5: Generating optimization plan..."

REPORT_FILE="$REPORT_DIR/${MODULE_NAME}_optimization.md"

cat > "$REPORT_FILE" << EOF
# $MODULE_NAME Module Optimization Plan

## Module Overview
- Files: $FILE_COUNT
- Current feature usage: $FEATURE_COUNT files have feature flags
- Target feature: \`$FEATURE_NAME\`

## Files to Update
$(echo "$MODULE_FILES" | awk '{print "- " $0}')

## Dependencies to Make Optional
*Identify these from Cargo.toml based on the output from Step 3*

## Required Code Changes

### 1. Update Cargo.toml
\`\`\`toml
# Add or modify feature definition
[features]
$FEATURE_NAME = [] # Add any dependent features or optional dependencies here

# Make dependencies optional
[dependencies]
# example = { version = "1.0", optional = true }
# ...

# Map optional dependencies to the feature
[dependencies.example]
optional = true
\`\`\`

### 2. Update Module Exports
\`\`\`rust
// In the module's mod.rs or lib.rs
#[cfg(feature = "$FEATURE_NAME")]
pub mod submodule;

// Re-export key types conditionally
#[cfg(feature = "$FEATURE_NAME")]
pub use submodule::{Type1, Type2};
\`\`\`

### 3. Conditional Service Registration
\`\`\`rust
// In application startup code
#[cfg(feature = "$FEATURE_NAME")]
app.service($MODULE_NAME::service());
\`\`\`

### 4. Add Feature Documentation
\`\`\`rust
/// $MODULE_NAME module
/// 
/// This module requires the \`$FEATURE_NAME\` feature to be enabled.
/// 
/// # Example
/// 
/// Enable the feature in Cargo.toml:
/// \`\`\`toml
/// [features]
/// $FEATURE_NAME = []
/// \`\`\`
/// 
/// Use the module:
/// \`\`\`rust
/// #[cfg(feature = "$FEATURE_NAME")]
/// use navius::core::$MODULE_NAME;
/// \`\`\`
pub mod $MODULE_NAME {
    // ...
}
\`\`\`

## Testing Plan
1. Build with feature enabled: \`cargo build --features $FEATURE_NAME\`
2. Build without feature: \`cargo build --no-default-features\`
3. Verify all tests pass: \`cargo test --features $FEATURE_NAME\`
4. Compare binary sizes to measure improvement
EOF

echo "Optimization plan generated at $REPORT_FILE"
echo ""
echo "Next steps:"
echo "1. Review the optimization plan"
echo "2. Make the recommended changes"
echo "3. Test with and without the feature enabled"
echo "4. Measure the impact on binary size" 