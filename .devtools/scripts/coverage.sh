#!/bin/bash
# Navius Coverage Script
# This script runs tarpaulin to generate code coverage reports

set -e

# Use target/tarpaulin for all coverage output
COVERAGE_DIR="target/tarpaulin"
COVERAGE_FILE="$COVERAGE_DIR/navius-coverage.json"
HTML_REPORT="$COVERAGE_DIR/tarpaulin-report.html"

# Create coverage directory if it doesn't exist
mkdir -p "$COVERAGE_DIR"

# Function to display help message
show_help() {
  echo "Navius Coverage Script"
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  -h, --help             Show this help message"
  echo "  -m, --module MODULE    Run coverage for specific module (e.g., core::utils::api_resource)"
  echo "  -f, --full             Run full coverage analysis for the entire codebase"
  echo "  -r, --report           Generate HTML report from existing JSON data"
  echo "  -b, --baseline         Save current coverage as baseline"
  echo "  -c, --compare          Compare current coverage with baseline"
  echo "  --html                 Generate HTML report alongside JSON (optional)"
  echo ""
  echo "Examples:"
  echo "  $0 --full              Run full coverage analysis (JSON only)"
  echo "  $0 --full --html       Run full coverage analysis with HTML report"
  echo "  $0 -m core::utils      Run coverage for core::utils module"
  echo "  $0 -r                  Generate HTML report from existing JSON data"
  echo "  $0 -b                  Save current coverage as baseline"
  echo "  $0 -c                  Compare current coverage with baseline"
}

# Function to run tarpaulin for a specific module
run_module_coverage() {
  echo "Running coverage analysis for module: $1"
  cargo tarpaulin --packages navius --lib --out Json --output-file "$COVERAGE_FILE" --line -- "$1"
  
  if [ "$GENERATE_HTML" = true ]; then
    echo "Generating HTML report..."
    cargo tarpaulin --packages navius --lib --out Html --output-dir "$COVERAGE_DIR" --line -- "$1"
    echo "HTML report generated at $HTML_REPORT"
  fi
  
  echo "Coverage analysis complete. Results saved to $COVERAGE_FILE"
}

# Function to run full coverage analysis
run_full_coverage() {
  echo "Running full coverage analysis..."
  cargo tarpaulin --packages navius --lib --out Json --output-file "$COVERAGE_FILE" --line
  
  if [ "$GENERATE_HTML" = true ]; then
    echo "Generating HTML report..."
    cargo tarpaulin --packages navius --lib --out Html --output-dir "$COVERAGE_DIR" --line
    echo "HTML report generated at $HTML_REPORT"
  fi
  
  echo "Coverage analysis complete. Results saved to $COVERAGE_FILE"
}

# Function to generate HTML report from existing JSON data
generate_report() {
  if [ -f "$COVERAGE_FILE" ]; then
    echo "Generating HTML report from existing data..."
    cargo tarpaulin --packages navius --lib --out Html --output-dir "$COVERAGE_DIR" --line
    echo "HTML report generated at $HTML_REPORT"
  else
    echo "Error: $COVERAGE_FILE not found. Run coverage analysis first."
    exit 1
  fi
}

# Function to save current coverage as baseline
save_baseline() {
  if [ -f "$COVERAGE_FILE" ]; then
    cp "$COVERAGE_FILE" "$COVERAGE_DIR/baseline-coverage.json"
    echo "Baseline coverage saved to $COVERAGE_DIR/baseline-coverage.json"
  else
    echo "Error: $COVERAGE_FILE not found. Run coverage analysis first."
    exit 1
  fi
}

# Function to compare current coverage with baseline
compare_coverage() {
  if [ ! -f "$COVERAGE_FILE" ]; then
    echo "Error: $COVERAGE_FILE not found. Run coverage analysis first."
    exit 1
  fi
  
  if [ ! -f "$COVERAGE_DIR/baseline-coverage.json" ]; then
    echo "Error: Baseline coverage file not found. Run --baseline first."
    exit 1
  fi
  
  echo "Comparing coverage with baseline..."
  # Extract coverage percentages
  current=$(grep -o '"line_rate":[0-9.]*' "$COVERAGE_FILE" | cut -d ':' -f2)
  baseline=$(grep -o '"line_rate":[0-9.]*' "$COVERAGE_DIR/baseline-coverage.json" | cut -d ':' -f2)
  
  if [ -z "$current" ] || [ -z "$baseline" ]; then
    echo "Error: Could not extract coverage rates from JSON files."
    exit 1
  fi
  
  echo "Baseline coverage: $baseline"
  echo "Current coverage: $current"
  
  # Calculate difference
  diff=$(awk "BEGIN {print $current - $baseline}")
  
  if (( $(echo "$diff > 0" | bc -l) )); then
    echo "Coverage increased by $diff"
  elif (( $(echo "$diff < 0" | bc -l) )); then
    echo "Coverage decreased by $(echo "$diff * -1" | bc -l)"
  else
    echo "Coverage unchanged"
  fi
}

# Initialize HTML generation flag
GENERATE_HTML=false

# Parse command line arguments
if [ $# -eq 0 ]; then
  show_help
  exit 0
fi

while [ $# -gt 0 ]; do
  case "$1" in
    -h|--help)
      show_help
      exit 0
      ;;
    -m|--module)
      MODULE="$2"
      shift 2
      ;;
    -f|--full)
      RUN_FULL=true
      shift
      ;;
    -r|--report)
      GENERATE_REPORT=true
      shift
      ;;
    -b|--baseline)
      SAVE_BASELINE=true
      shift
      ;;
    -c|--compare)
      COMPARE=true
      shift
      ;;
    --html)
      GENERATE_HTML=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

# Execute requested operations
if [ "$RUN_FULL" = true ]; then
  run_full_coverage
fi

if [ -n "$MODULE" ]; then
  run_module_coverage "$MODULE"
fi

if [ "$GENERATE_REPORT" = true ]; then
  generate_report
fi

if [ "$SAVE_BASELINE" = true ]; then
  save_baseline
fi

if [ "$COMPARE" = true ]; then
  compare_coverage
fi 