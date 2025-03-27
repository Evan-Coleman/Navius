#!/bin/bash

# Script to install Git hooks for test automation

# Define color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Installing Git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Create pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

echo "Running pre-commit checks..."

# Check if any Rust files were changed
RUST_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.rs$')

if [ -z "$RUST_FILES" ]; then
  echo "No Rust files changed, skipping tests."
  exit 0
fi

# Run rustfmt on changed files
echo "Running rustfmt on changed files..."
for file in $RUST_FILES; do
  if [ -f "$file" ]; then
    rustfmt --check "$file"
    if [ $? -ne 0 ]; then
      echo "ERROR: $file has formatting errors. Run 'cargo fmt' to fix."
      exit 1
    fi
  fi
done

# Run clippy on the project
echo "Running clippy..."
cargo clippy -- -D warnings
if [ $? -ne 0 ]; then
  echo "ERROR: Clippy found issues. Please fix them before committing."
  exit 1
fi

# Run tests for modules with changed files
echo "Running tests for changed modules..."
MODULES=$(echo "$RUST_FILES" | sed -E 's/src\/([a-z_]+)\/.*/\1/' | sort -u)

if [ -n "$MODULES" ]; then
  for module in $MODULES; do
    echo "Testing module: $module"
    cargo test $module
    if [ $? -ne 0 ]; then
      echo "ERROR: Tests failed for module $module. Please fix them before committing."
      exit 1
    fi
  done
else
  # If we couldn't determine specific modules, run all tests
  echo "Running all tests..."
  cargo test
  if [ $? -ne 0 ]; then
    echo "ERROR: Tests failed. Please fix them before committing."
    exit 1
  fi
fi

echo "All pre-commit checks passed!"
exit 0
EOF

# Create pre-push hook for more extensive testing
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash

echo "Running pre-push checks..."

# Run full test suite
echo "Running full test suite..."
cargo test
if [ $? -ne 0 ]; then
  echo "ERROR: Tests failed. Please fix them before pushing."
  exit 1
fi

# Run coverage check if tarpaulin is installed
if command -v cargo-tarpaulin &> /dev/null; then
  echo "Checking test coverage..."
  
  if [ -f ".devtools/scripts/check_coverage.sh" ]; then
    .devtools/scripts/check_coverage.sh
    if [ $? -ne 0 ]; then
      echo "WARNING: Coverage is below target. Consider adding more tests."
      # Don't fail the push for coverage issues, just warn
    fi
  else
    cargo tarpaulin --out Html
    echo "Coverage report generated at tarpaulin-report.html"
  fi
fi

echo "All pre-push checks passed!"
exit 0
EOF

# Make hooks executable
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push

echo -e "${GREEN}Git hooks installed successfully!${NC}"
echo "Pre-commit hook will run format checks and tests on changed files."
echo "Pre-push hook will run the full test suite and coverage checks."
echo ""
echo -e "${YELLOW}Note: To skip hooks temporarily, use ${NC}git commit --no-verify${YELLOW} or ${NC}git push --no-verify" 