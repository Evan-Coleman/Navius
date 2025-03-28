#!/usr/bin/env bash

# run-validation.sh
# Script to run all the validation tools for the documentation reorganization
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
OUTPUT_DIR="target/validation-report"
TOOLS_DIR="11newdocs11/98_roadmaps/doc-reorg-tools"
REPORT_INDEX="$OUTPUT_DIR/index.html"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "===== Phase 2 Documentation Validation Runner ====="
echo "This tool runs all the validation tools and creates a comprehensive report"
echo ""

# Function to run a validation tool and track its execution
run_tool() {
    local tool_name="$1"
    local tool_path="$2"
    local args="$3"
    local start_time=$(date +%s)
    
    echo "Running $tool_name..."
    
    if $tool_path $args; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo "✓ $tool_name completed successfully in $duration seconds"
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo "✗ $tool_name failed after $duration seconds"
        return 1
    fi
}

# Run all validation tools
echo "Starting validation at $(date)"
echo "========================================"

# Extract code examples
run_tool "Code Example Extractor" "$TOOLS_DIR/code-example-extractor.sh" ""

# Verify code examples
run_tool "Code Example Verifier" "$TOOLS_DIR/code-example-verifier.sh" ""

# Fix code examples with common issues
run_tool "Code Example Fixer" "$TOOLS_DIR/code-example-fixer.sh" "--all"

# Analyze internal links
run_tool "Link Analyzer" "$TOOLS_DIR/link-analyzer.sh" ""

# Validate documents
run_tool "Document Validator" "$TOOLS_DIR/document-validator.sh" ""

echo "========================================"
echo "All validation tools completed at $(date)"

# Create a consolidated report
echo "Creating consolidated report..."

# Extract summary data
CODE_EXAMPLES_TOTAL=$(find target/code-verification -name "*.rs" | wc -l | tr -d ' ')
CODE_EXAMPLES_PASSING=$(grep ",Compiles$" target/code-verification/examples/*/examples.csv 2>/dev/null | wc -l | tr -d ' ')
CODE_EXAMPLES_FAILING=$(grep ",Fails$" target/code-verification/examples/*/examples.csv 2>/dev/null | wc -l | tr -d ' ')
CODE_EXAMPLES_FIXED=$(grep ",\"Compiles\"$" target/code-verification/fixed/*/*/fixes.csv 2>/dev/null | wc -l | tr -d ' ')

LINKS_TOTAL=$(cat target/link-analysis/link_tracking.csv 2>/dev/null | tail -n +2 | awk -F, '{sum+=$3} END {print sum}')
LINKS_VALID=$(cat target/link-analysis/link_tracking.csv 2>/dev/null | tail -n +2 | awk -F, '{sum+=$5} END {print sum}')
LINKS_BROKEN=$(cat target/link-analysis/link_tracking.csv 2>/dev/null | tail -n +2 | awk -F, '{sum+=$4} END {print sum}')

DOCS_TOTAL=$(cat target/doc-validation/validation_tracking.csv 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
DOCS_PASSING=$(grep ",Pass," target/doc-validation/validation_tracking.csv 2>/dev/null | wc -l | tr -d ' ')
DOCS_FAILING=$(grep ",Fail," target/doc-validation/validation_tracking.csv 2>/dev/null | wc -l | tr -d ' ')

# Calculate percentages
CODE_EXAMPLES_PCT=0
if [[ $CODE_EXAMPLES_TOTAL -gt 0 ]]; then
    CODE_EXAMPLES_PCT=$((CODE_EXAMPLES_PASSING * 100 / CODE_EXAMPLES_TOTAL))
fi

LINKS_PCT=0
if [[ $LINKS_TOTAL -gt 0 ]]; then
    LINKS_PCT=$((LINKS_VALID * 100 / LINKS_TOTAL))
fi

DOCS_PCT=0
if [[ $DOCS_TOTAL -gt 0 ]]; then
    DOCS_PCT=$((DOCS_PASSING * 100 / DOCS_TOTAL))
fi

OVERALL_PCT=$(( (CODE_EXAMPLES_PCT + LINKS_PCT + DOCS_PCT) / 3 ))

# Create index.html
cat > "$REPORT_INDEX" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Documentation Validation Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #2c3e50; }
        h2 { color: #3498db; }
        .summary { background-color: #f8f9fa; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .stats { display: flex; flex-wrap: wrap; gap: 10px; }
        .stat-box { background-color: #e9ecef; padding: 10px; border-radius: 5px; flex: 1; min-width: 200px; }
        .good { color: #27ae60; }
        .bad { color: #e74c3c; }
        .warning { color: #f39c12; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
        tr:hover { background-color: #f5f5f5; }
        .progress-bar { 
            height: 20px; 
            background-color: #ecf0f1; 
            border-radius: 10px; 
            overflow: hidden; 
        }
        .progress-fill { 
            height: 100%; 
            background-color: #2ecc71; 
        }
    </style>
</head>
<body>
    <h1>Documentation Validation Report</h1>
    <p>Generated: $(date)</p>
    
    <div class='summary'>
        <h2>Overall Phase 2 Status: ${OVERALL_PCT}% Complete</h2>
        <div class='progress-bar'>
            <div class='progress-fill' style='width: ${OVERALL_PCT}%'></div>
        </div>
    </div>
    
    <h2>Validation Results</h2>
    <div class='stats'>
        <div class='stat-box'>
            <h3>Code Examples</h3>
            <p>Total: $CODE_EXAMPLES_TOTAL</p>
            <p>Passing: <span class='good'>$CODE_EXAMPLES_PASSING</span></p>
            <p>Failing: <span class='bad'>$CODE_EXAMPLES_FAILING</span></p>
            <p>Fixed: <span class='good'>$CODE_EXAMPLES_FIXED</span></p>
            <p>Success Rate: ${CODE_EXAMPLES_PCT}%</p>
            <div class='progress-bar'>
                <div class='progress-fill' style='width: ${CODE_EXAMPLES_PCT}%'></div>
            </div>
            <p><a href="code-verification/reports/summary_report.html">View Details</a> | <a href="code-verification/fixed/summary.md">View Fixes</a></p>
        </div>
        
        <div class='stat-box'>
            <h3>Internal Links</h3>
            <p>Total: $LINKS_TOTAL</p>
            <p>Valid: <span class='good'>$LINKS_VALID</span></p>
            <p>Broken: <span class='bad'>$LINKS_BROKEN</span></p>
            <p>Success Rate: ${LINKS_PCT}%</p>
            <div class='progress-bar'>
                <div class='progress-fill' style='width: ${LINKS_PCT}%'></div>
            </div>
            <p><a href="link-analysis/link_tracking.csv">View Details</a></p>
        </div>
        
        <div class='stat-box'>
            <h3>Document Validation</h3>
            <p>Total: $DOCS_TOTAL</p>
            <p>Passing: <span class='good'>$DOCS_PASSING</span></p>
            <p>Failing: <span class='bad'>$DOCS_FAILING</span></p>
            <p>Success Rate: ${DOCS_PCT}%</p>
            <div class='progress-bar'>
                <div class='progress-fill' style='width: ${DOCS_PCT}%'></div>
            </div>
            <p><a href="doc-validation/validation_tracking.csv">View Details</a></p>
        </div>
    </div>
    
    <h2>Next Steps</h2>
    <ol>
        <li>Fix failing code examples, prioritizing those in high-traffic documents</li>
        <li>Fix broken internal links to ensure proper navigation</li>
        <li>Address document validation issues in Tier 1 documents first</li>
        <li>Re-run this validation tool regularly to track progress</li>
    </ol>
    
    <h2>Detailed Reports</h2>
    <ul>
        <li><a href="code-verification/reports/summary_report.html">Code Example Verification Report</a></li>
        <li><a href="code-verification/fixed/summary.md">Code Example Fixes Report</a></li>
        <li><a href="link-analysis/critical_paths.csv">Critical Path Links Report</a></li>
        <li><a href="doc-validation/validation_tracking.csv">Document Validation Report</a></li>
    </ul>
</body>
</html>
EOF

echo "Consolidated report created at $REPORT_INDEX"
echo ""
echo "Next steps:"
echo "1. Review the consolidated report and individual tool reports"
echo "2. Address issues in priority order (Tier 1 documents first)"
echo "3. Fix code examples, links, and document issues"
echo "4. Re-run this tool to track progress"
echo ""
echo "Open $REPORT_INDEX in a browser to view the consolidated report" 