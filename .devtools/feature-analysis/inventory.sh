#!/bin/bash
#
# Feature Flag Inventory Tool
# This script analyzes the codebase to identify feature flag usage
#

set -e

# Set base directory
BASE_DIR="$(pwd)"
# Create output directory
REPORT_DIR="$BASE_DIR/.devtools/feature-analysis/report"
mkdir -p "$REPORT_DIR"

echo "=== Running Feature Flag Inventory Analysis ==="

# Run our Python analyzer
cd "$BASE_DIR"
python3 .devtools/feature-analysis/inventory_features.py

echo ""
echo "=== Analysis Complete ==="
echo "Feature inventory report generated at $REPORT_DIR/feature_inventory.md"

echo ""
echo "Next steps:"
echo "1. Review the feature inventory to identify patterns of feature usage"
echo "2. Identify modules that should be feature-gated but aren't"
echo "3. Continue with Phase 2 of the Feature Flag Optimization Roadmap" 