#!/bin/bash
# Navius Coverage Script
# This script runs tarpaulin to generate code coverage reports

set -e

COVERAGE_FILE="navius-coverage.json"
HTML_REPORT="coverage/tarpaulin-report.html"
COVERAGE_DIR="coverage"

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
  echo ""
  echo "Examples:"
  echo "  $0 --full              Run full coverage analysis"
  echo "  $0 -m core::utils      Run coverage for core::utils module"
  echo "  $0 -r                  Generate HTML report from existing JSON data"
  echo "  $0 -b                  Save current coverage as baseline"
  echo "  $0 -c                  Compare current coverage with baseline"
}

# Function to run tarpaulin for a specific module
run_module_coverage() {
  echo "Running coverage analysis for module: $1"
  cargo tarpaulin --packages navius --lib --out Json --output-file "$COVERAGE_FILE" --line -- "$1"
  
  # Also save to target directory for integration with other tools
  cargo tarpaulin --packages navius --lib --out Json --output-file "target/@navius-coverage.json" --line -- "$1"
  
  # Generate HTML report
  cargo tarpaulin --packages navius --lib --out Html --output-dir "$COVERAGE_DIR" --line -- "$1"
  
  echo "Coverage analysis complete. Results saved to $COVERAGE_FILE"
  echo "Additional JSON file saved to target/@navius-coverage.json"
  echo "HTML report generated at $HTML_REPORT"
}

# Function to run full coverage analysis
run_full_coverage() {
  echo "Running full coverage analysis..."
  cargo tarpaulin --packages navius --lib --out Json --output-file "$COVERAGE_FILE" --line
  
  # Also save to target directory for integration with other tools
  cargo tarpaulin --packages navius --lib --out Json --output-file "target/@navius-coverage.json" --line
  
  # Generate HTML report
  cargo tarpaulin --packages navius --lib --out Html --output-dir "$COVERAGE_DIR" --line
  
  echo "Full coverage analysis complete. Results saved to $COVERAGE_FILE"
  echo "Additional JSON file saved to target/@navius-coverage.json"
  echo "HTML report generated at $HTML_REPORT"
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
      run_module_coverage "$2"
      shift 2
      ;;
    -f|--full)
      run_full_coverage
      shift
      ;;
    -r|--report)
      generate_report
      shift
      ;;
    -b|--baseline)
      save_baseline
      shift
      ;;
    -c|--compare)
      compare_coverage
      shift
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done 