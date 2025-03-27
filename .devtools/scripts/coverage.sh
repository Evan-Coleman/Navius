#!/bin/bash
# Navius Coverage Script
# This script runs tarpaulin to generate code coverage reports
# 
# Usage: 
#   ./.devtools/scripts/coverage.sh --full         # Run coverage on the full codebase
#   ./.devtools/scripts/coverage.sh -m path::to::module  # Run coverage on a specific module
#   ./.devtools/scripts/coverage.sh -b             # Save current results as baseline
#   ./.devtools/scripts/coverage.sh -c             # Compare with baseline
#   ./.devtools/scripts/coverage.sh --help         # Show help message

set -e

# Use target/tarpaulin for all coverage output
COVERAGE_DIR="target/tarpaulin"
COVERAGE_FILE="$COVERAGE_DIR/navius-coverage.json"
HTML_REPORT="$COVERAGE_DIR/tarpaulin-report.html"
BASELINE_FILE="$COVERAGE_DIR/baseline-coverage.json"
XML_REPORT="$COVERAGE_DIR/cobertura.xml"

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
  
  # Build tarpaulin args list
  local tarpaulin_args="--packages navius --lib --bins --tests --line"
  
  # Always output JSON to our standard location
  tarpaulin_args="$tarpaulin_args --out Json --output-file $COVERAGE_FILE"
  
  # Add XML output for more detailed reporting
  tarpaulin_args="$tarpaulin_args --out Xml --output-path $XML_REPORT"
  
  # Add HTML if requested
  if [ "$GENERATE_HTML" = true ]; then
    tarpaulin_args="$tarpaulin_args --out Html --output-dir $COVERAGE_DIR"
  fi
  
  # Run tarpaulin with the module
  cargo tarpaulin $tarpaulin_args -- "$1"
  
  echo "Coverage analysis complete. Results saved to $COVERAGE_FILE"
  
  if [ "$GENERATE_HTML" = true ]; then
    echo "HTML report generated at $HTML_REPORT"
  fi
}

# Function to run full coverage analysis
run_full_coverage() {
  echo "Running full coverage analysis..."
  
  # Build tarpaulin args list
  local tarpaulin_args="--packages navius --lib --bins --tests --line"
  
  # Always output JSON to our standard location
  tarpaulin_args="$tarpaulin_args --out Json --output-file $COVERAGE_FILE"
  
  # Add XML output for more detailed reporting
  tarpaulin_args="$tarpaulin_args --out Xml --output-path $XML_REPORT"
  
  # Add HTML if requested
  if [ "$GENERATE_HTML" = true ]; then
    tarpaulin_args="$tarpaulin_args --out Html --output-dir $COVERAGE_DIR"
  fi
  
  # Run tarpaulin for the full codebase
  cargo tarpaulin $tarpaulin_args
  
  echo "Coverage analysis complete. Results saved to $COVERAGE_FILE"
  
  if [ "$GENERATE_HTML" = true ]; then
    echo "HTML report generated at $HTML_REPORT"
  fi
}

# Function to generate HTML report from existing JSON data
generate_report() {
  if [ -f "$COVERAGE_FILE" ]; then
    echo "Generating HTML report from existing data..."
    
    # Try using tarpaulin's HTML report first
    if cargo tarpaulin --packages navius --lib --out Html --output-dir "$COVERAGE_DIR" --line; then
      echo "HTML report generated at $HTML_REPORT"
    else
      # Fall back to using pycobertura if tarpaulin HTML generation fails
      if [ -f "$XML_REPORT" ] && command -v python3 >/dev/null 2>&1; then
        python3 -m pycobertura show --format html "$XML_REPORT" > "$COVERAGE_DIR/coverage.html"
        echo "HTML report generated at $COVERAGE_DIR/coverage.html"
      else
        echo "Warning: Could not generate HTML report. XML report not found or Python not available."
      fi
    fi
  else
    echo "Error: $COVERAGE_FILE not found. Run coverage analysis first."
    exit 1
  fi
}

# Function to save current coverage as baseline
save_baseline() {
  if [ -f "$COVERAGE_FILE" ]; then
    cp "$COVERAGE_FILE" "$BASELINE_FILE"
    echo "Baseline coverage saved to $BASELINE_FILE"
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
  
  if [ ! -f "$BASELINE_FILE" ]; then
    echo "Error: Baseline coverage file not found. Run --baseline first."
    exit 1
  fi
  
  echo "Comparing coverage with baseline..."
  
  # Extract coverage percentages using jq
  if command -v jq >/dev/null 2>&1; then
    current=$(jq -r '.percentage' "$COVERAGE_FILE")
    baseline=$(jq -r '.percentage' "$BASELINE_FILE")
  else
    # Fallback to grep if jq is not available
    current=$(grep -o '"line_rate":[0-9.]*' "$COVERAGE_FILE" | cut -d ':' -f2)
    baseline=$(grep -o '"line_rate":[0-9.]*' "$BASELINE_FILE" | cut -d ':' -f2)
  fi
  
  if [ -z "$current" ] || [ -z "$baseline" ]; then
    echo "Error: Could not extract coverage rates from JSON files."
    exit 1
  fi
  
  echo "Baseline coverage: $baseline%"
  echo "Current coverage:  $current%"
  
  # Calculate difference
  diff=$(echo "$current - $baseline" | bc)
  
  if (( $(echo "$diff > 0" | bc -l) )); then
    echo "Coverage change:   +$diff%"
  else
    echo "Coverage change:   $diff%"
  fi
  
  # Also compare module-specific coverage if a module was specified
  if [ ! -z "$MODULE" ] && command -v jq >/dev/null 2>&1; then
    echo ""
    MODULE_ESC=$(echo $MODULE | sed 's/::/\\\\::/g')
    
    CURRENT_MOD_COV=$(jq -r ".files[] | select(.path | contains(\"$MODULE_ESC\")) | .coverage" "$COVERAGE_FILE" | jq -s 'add/length')
    BASELINE_MOD_COV=$(jq -r ".files[] | select(.path | contains(\"$MODULE_ESC\")) | .coverage" "$BASELINE_FILE" | jq -s 'add/length')
    
    if [ ! -z "$CURRENT_MOD_COV" ] && [ ! -z "$BASELINE_MOD_COV" ]; then
      MOD_DIFF=$(echo "$CURRENT_MOD_COV - $BASELINE_MOD_COV" | bc)
      
      echo "Module: $MODULE"
      echo "Baseline coverage: $BASELINE_MOD_COV%"
      echo "Current coverage:  $CURRENT_MOD_COV%"
      
      if (( $(echo "$MOD_DIFF >= 0" | bc -l) )); then
        echo "Coverage change:   +$MOD_DIFF%"
      else
        echo "Coverage change:   $MOD_DIFF%"
      fi
    fi
  fi
}

# Initialize flags
GENERATE_HTML=false
MODULE=""

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

# Execute requested operations in the right order
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
  # If we just ran coverage analysis, we can save it as baseline
  # Otherwise, check if the coverage file exists
  if [ ! -f "$COVERAGE_FILE" ] && [ "$RUN_FULL" != true ] && [ -z "$MODULE" ]; then
    echo "No coverage results available. Running full coverage analysis first..."
    run_full_coverage
  fi
  save_baseline
fi

if [ "$COMPARE" = true ]; then
  # If we don't have current coverage data but need to compare, run analysis first
  if [ ! -f "$COVERAGE_FILE" ] && [ "$RUN_FULL" != true ] && [ -z "$MODULE" ]; then
    echo "No coverage results available. Running full coverage analysis first..."
    run_full_coverage
  fi
  compare_coverage
fi

echo "Coverage operation(s) complete. Results are in the $COVERAGE_DIR directory."
echo "For HTML reports, open $HTML_REPORT in your browser." 