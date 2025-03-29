#!/bin/bash
#
# Feature Flag Optimization Analysis Tool
# This script analyzes dependencies against features and provides optimization recommendations
#

set -e

# Set base directory
BASE_DIR="$(pwd)"
# Create output directory
REPORT_DIR="$BASE_DIR/.devtools/feature-analysis/report"
mkdir -p "$REPORT_DIR"

echo "=== Running Cargo Dependency Analysis ==="
echo "Analyzing dependency to feature relationships..."

# Run our Python analyzer
cd "$BASE_DIR/.devtools/feature-analysis"
python3 analyze_deps.py
mv dependency_analysis.md "$REPORT_DIR/dependency_matrix.md"

# Get current binary sizes
echo ""
echo "=== Measuring Binary Sizes ==="

# Make sure we have a clean build
cd "$BASE_DIR"
cargo clean

# Build with all features
echo "Building with all features..."
cargo build --release
echo "Binary size (all features):"
ls -lh target/release/navius | awk '{print $5}'
cp target/release/navius "$REPORT_DIR/navius-all-features"

# Build with minimal features
echo ""
echo "Building with minimal features (logging + prometheus)..."
cargo build --release --no-default-features --features "logging,prometheus,metrics"
echo "Binary size (minimal features):"
ls -lh target/release/navius | awk '{print $5}'
cp target/release/navius "$REPORT_DIR/navius-minimal"

# Output summary
echo ""
echo "=== Analysis Complete ==="
echo "Report generated at $REPORT_DIR/dependency_matrix.md"
echo "Binary size comparison:"
echo "- Full build: $(ls -lh $REPORT_DIR/navius-all-features | awk '{print $5}')"
echo "- Minimal build: $(ls -lh $REPORT_DIR/navius-minimal | awk '{print $5}')"
echo "Potential savings: $(( $(stat -f %z $REPORT_DIR/navius-all-features) - $(stat -f %z $REPORT_DIR/navius-minimal) )) bytes"

echo ""
echo "Next steps:"
echo "1. Review the dependency matrix to identify optimization opportunities"
echo "2. Update the Feature Flag Optimization Roadmap based on the findings"
echo "3. Begin implementing optimizations according to the roadmap" 