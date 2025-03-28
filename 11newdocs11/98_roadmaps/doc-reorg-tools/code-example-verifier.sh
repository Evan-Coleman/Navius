#!/usr/bin/env bash

# code-example-verifier.sh
# Script to extract and verify Rust code examples from markdown files
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
OUTPUT_DIR="target/code-verification"
EXAMPLES_DIR="$OUTPUT_DIR/examples"
REPORT_DIR="$OUTPUT_DIR/reports"
DEFAULT_IMPORTS="
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::io::{self, Read, Write};
use navius::core::error::AppError;
use navius::app::Application;
use navius::core::config::Config;
"
WRAPPER_TEMPLATE='fn example_wrapper() {
    // This is a wrapper function to make code fragments compilable
    // Code fragments often do not have function wrappers in documentation
    #[allow(unused_variables, dead_code, unused_imports)]
    {
        %CODE%
    }
}

// Main function for standalone examples
#[cfg(test)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only added for standalone examples that need a main function
    Ok(())
}'

# Create output directories
mkdir -p "$EXAMPLES_DIR" "$REPORT_DIR"

# Function to extract code examples from a markdown file
extract_code_examples() {
    local doc="$1"
    local filename=$(basename "$doc" .md)
    local dirname=$(dirname "$doc")
    local category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    
    if [[ -z "$category" ]]; then
        category="misc"
    fi
    
    # Create output directory
    local out_dir="$EXAMPLES_DIR/${category}/${filename}"
    mkdir -p "$out_dir"
    
    echo "Extracting code examples from $doc..."
    
    # Detect Rust code blocks
    local rust_blocks=$(grep -n "^```rust" "$doc" | cut -d: -f1)
    local total_blocks=$(echo "$rust_blocks" | wc -l | tr -d ' ')
    
    if [[ $total_blocks -eq 0 || -z "$rust_blocks" ]]; then
        echo "No Rust code blocks found in $doc"
        return 0
    fi
    
    echo "Found $total_blocks Rust code blocks"
    
    # Create a file for tracking extracted examples
    local tracking_file="$out_dir/examples.csv"
    echo "Block,StartLine,EndLine,FileName,IssuesFound,Status" > "$tracking_file"
    
    # Extract each Rust code block
    local block_count=0
    local start_line=""
    local end_line=""
    local in_block=false
    local block_content=""
    
    while IFS= read -r line_num content; do
        if [[ "$content" == "```rust" && "$in_block" == false ]]; then
            in_block=true
            start_line="$line_num"
            block_content=""
        elif [[ "$content" == "```" && "$in_block" == true ]]; then
            in_block=false
            end_line="$line_num"
            ((block_count++))
            
            local example_file="$out_dir/example_${block_count}.rs"
            local issues=""
            local status="Unknown"
            
            # Write the extracted code to a file
            echo "// Example $block_count from $doc (lines $start_line-$end_line)" > "$example_file"
            echo "$DEFAULT_IMPORTS" >> "$example_file"
            
            # Check if the code is a complete function or needs wrapping
            if [[ "$block_content" == *"fn main"* || "$block_content" == *"fn "* ]]; then
                # Block has its own function definition, use as-is
                echo "$block_content" >> "$example_file"
            else
                # Wrap in a function to make it compilable
                echo "${WRAPPER_TEMPLATE/\%CODE\%/$block_content}" >> "$example_file"
            fi
            
            # Check for common issues
            if [[ "$block_content" != *"use "* && "$block_content" == *"::"* ]]; then
                issues="$issues;Missing imports"
            fi
            
            if [[ "$block_content" != *"fn "* && "$block_content" != *"struct "* && "$block_content" != *"enum "* && "$block_content" != *"impl "* ]]; then
                issues="$issues;Code fragment without declarations"
            fi
            
            if [[ "$block_content" == *".unwrap()"* || "$block_content" == *".expect("* ]]; then
                issues="$issues;Uses unwrap or expect"
            fi
            
            if [[ "$block_content" == *"Err("* && "$block_content" != *"return Err("* && "$block_content" != *"-> Result"* ]]; then
                issues="$issues;Error handling issues"
            fi
            
            # Try to compile the example
            local compile_result
            if rustc --edition=2021 -o /dev/null "$example_file" 2>/dev/null; then
                status="Compiles"
            else
                status="Fails"
                issues="$issues;Compilation failed"
            fi
            
            # Record the example in tracking file
            echo "$block_count,$start_line,$end_line,$example_file,\"$issues\",$status" >> "$tracking_file"
        fi
        
        if [[ "$in_block" == true && "$content" != "```rust" ]]; then
            block_content="${block_content}${content}
"
        fi
    done < <(nl -ba "$doc" | sed 's/^ \+//')
    
    echo "Extracted $block_count code examples from $doc"
    return 0
}

# Function to generate a report for a document
generate_report() {
    local doc="$1"
    local filename=$(basename "$doc" .md)
    local dirname=$(dirname "$doc")
    local category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    
    if [[ -z "$category" ]]; then
        category="misc"
    fi
    
    # Create report directory
    local report_dir="$REPORT_DIR/${category}"
    mkdir -p "$report_dir"
    
    local tracking_file="$EXAMPLES_DIR/${category}/${filename}/examples.csv"
    
    # Skip if no examples were found
    if [[ ! -f "$tracking_file" ]]; then
        echo "No code examples to report for $doc"
        return 0
    fi
    
    # Read the tracking file
    local total_examples=$(tail -n +2 "$tracking_file" | wc -l | tr -d ' ')
    local compiling=$(grep ",Compiles$" "$tracking_file" | wc -l | tr -d ' ')
    local failing=$(grep ",Fails$" "$tracking_file" | wc -l | tr -d ' ')
    
    # Count issue types
    local missing_imports=$(grep "Missing imports" "$tracking_file" | wc -l | tr -d ' ')
    local fragments=$(grep "Code fragment without declarations" "$tracking_file" | wc -l | tr -d ' ')
    local unwrap_usage=$(grep "Uses unwrap or expect" "$tracking_file" | wc -l | tr -d ' ')
    local error_handling=$(grep "Error handling issues" "$tracking_file" | wc -l | tr -d ' ')
    
    # Calculate completion percentage
    local completion_pct=0
    if [[ $total_examples -gt 0 ]]; then
        completion_pct=$((compiling * 100 / total_examples))
    fi
    
    # Generate HTML report
    local report_file="$report_dir/${filename}_report.html"
    
    echo "<!DOCTYPE html>
<html>
<head>
    <title>Code Example Verification: $filename</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #2c3e50; }
        h2 { color: #3498db; }
        .summary { background-color: #f8f9fa; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .stats { display: flex; flex-wrap: wrap; gap: 10px; }
        .stat-box { background-color: #e9ecef; padding: 10px; border-radius: 5px; flex: 1; min-width: 200px; }
        .good { color: #27ae60; }
        .bad { color: #e74c3c; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
        tr:hover { background-color: #f5f5f5; }
        .issues { color: #e74c3c; }
    </style>
</head>
<body>
    <h1>Code Example Verification Report</h1>
    <p>Document: $doc</p>
    <p>Generated: $(date)</p>
    
    <div class='summary'>
        <h2>Summary</h2>
        <div class='stats'>
            <div class='stat-box'>
                <h3>Total Examples</h3>
                <p>$total_examples</p>
            </div>
            <div class='stat-box'>
                <h3>Compiling</h3>
                <p class='good'>$compiling</p>
            </div>
            <div class='stat-box'>
                <h3>Failing</h3>
                <p class='bad'>$failing</p>
            </div>
            <div class='stat-box'>
                <h3>Completion</h3>
                <p>${completion_pct}%</p>
            </div>
        </div>
    </div>
    
    <h2>Issue Distribution</h2>
    <div class='stats'>
        <div class='stat-box'>
            <h3>Missing Imports</h3>
            <p>$missing_imports</p>
        </div>
        <div class='stat-box'>
            <h3>Code Fragments</h3>
            <p>$fragments</p>
        </div>
        <div class='stat-box'>
            <h3>Unwrap Usage</h3>
            <p>$unwrap_usage</p>
        </div>
        <div class='stat-box'>
            <h3>Error Handling</h3>
            <p>$error_handling</p>
        </div>
    </div>
    
    <h2>Examples Detail</h2>
    <table>
        <tr>
            <th>Block</th>
            <th>Lines</th>
            <th>Status</th>
            <th>Issues</th>
        </tr>" > "$report_file"
    
    # Add each example to the table
    tail -n +2 "$tracking_file" | while IFS=, read -r block start end file issues status; do
        issues="${issues//\"}"
        echo "        <tr>
            <td>$block</td>
            <td>$start-$end</td>
            <td>$status</td>
            <td class='issues'>$issues</td>
        </tr>" >> "$report_file"
    done
    
    echo "    </table>
    
    <h2>Next Steps</h2>
    <ol>
        <li>Fix examples with missing imports</li>
        <li>Address error handling issues</li>
        <li>Review code fragments that need more context</li>
        <li>Re-run verification after fixes</li>
    </ol>
</body>
</html>" >> "$report_file"
    
    echo "Generated report for $doc at $report_file"
    return 0
}

# Function to generate a summary report across all documents
generate_summary_report() {
    local summary_file="$REPORT_DIR/summary_report.html"
    local stats_file="$OUTPUT_DIR/stats.csv"
    
    echo "Document,Category,TotalExamples,Compiling,Failing,CompletionPct" > "$stats_file"
    
    # Find all tracking files
    find "$EXAMPLES_DIR" -name "examples.csv" | while read -r tracking_file; do
        local dir=$(dirname "$tracking_file")
        local category=$(basename "$(dirname "$dir")")
        local filename=$(basename "$dir")
        local doc_path="$DOCS_DIR/$category/$filename.md"
        
        # Read the tracking file
        local total_examples=$(tail -n +2 "$tracking_file" | wc -l | tr -d ' ')
        local compiling=$(grep ",Compiles$" "$tracking_file" | wc -l | tr -d ' ')
        local failing=$(grep ",Fails$" "$tracking_file" | wc -l | tr -d ' ')
        
        # Calculate completion percentage
        local completion_pct=0
        if [[ $total_examples -gt 0 ]]; then
            completion_pct=$((compiling * 100 / total_examples))
        fi
        
        echo "$doc_path,$category,$total_examples,$compiling,$failing,$completion_pct" >> "$stats_file"
    done
    
    # Calculate overall statistics
    local total_docs=$(tail -n +2 "$stats_file" | wc -l | tr -d ' ')
    local total_examples=$(tail -n +2 "$stats_file" | awk -F, '{sum+=$3} END {print sum}')
    local total_compiling=$(tail -n +2 "$stats_file" | awk -F, '{sum+=$4} END {print sum}')
    local total_failing=$(tail -n +2 "$stats_file" | awk -F, '{sum+=$5} END {print sum}')
    
    # Calculate overall completion percentage
    local overall_pct=0
    if [[ $total_examples -gt 0 ]]; then
        overall_pct=$((total_compiling * 100 / total_examples))
    fi
    
    # Calculate category statistics
    local categories=$(tail -n +2 "$stats_file" | cut -d, -f2 | sort | uniq)
    
    # Generate HTML summary report
    echo "<!DOCTYPE html>
<html>
<head>
    <title>Code Example Verification Summary</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #2c3e50; }
        h2 { color: #3498db; }
        .summary { background-color: #f8f9fa; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .stats { display: flex; flex-wrap: wrap; gap: 10px; }
        .stat-box { background-color: #e9ecef; padding: 10px; border-radius: 5px; flex: 1; min-width: 200px; }
        .good { color: #27ae60; }
        .bad { color: #e74c3c; }
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
    <h1>Code Example Verification Summary</h1>
    <p>Generated: $(date)</p>
    
    <div class='summary'>
        <h2>Overall Statistics</h2>
        <div class='stats'>
            <div class='stat-box'>
                <h3>Documents</h3>
                <p>$total_docs</p>
            </div>
            <div class='stat-box'>
                <h3>Total Examples</h3>
                <p>$total_examples</p>
            </div>
            <div class='stat-box'>
                <h3>Compiling</h3>
                <p class='good'>$total_compiling</p>
            </div>
            <div class='stat-box'>
                <h3>Failing</h3>
                <p class='bad'>$total_failing</p>
            </div>
        </div>
        <h3>Overall Completion: ${overall_pct}%</h3>
        <div class='progress-bar'>
            <div class='progress-fill' style='width: ${overall_pct}%'></div>
        </div>
    </div>
    
    <h2>By Category</h2>
    <table>
        <tr>
            <th>Category</th>
            <th>Documents</th>
            <th>Examples</th>
            <th>Compiling</th>
            <th>Failing</th>
            <th>Completion</th>
        </tr>" > "$summary_file"
    
    # Add each category to the table
    for category in $categories; do
        local cat_docs=$(grep ",$category," "$stats_file" | wc -l | tr -d ' ')
        local cat_examples=$(grep ",$category," "$stats_file" | awk -F, '{sum+=$3} END {print sum}')
        local cat_compiling=$(grep ",$category," "$stats_file" | awk -F, '{sum+=$4} END {print sum}')
        local cat_failing=$(grep ",$category," "$stats_file" | awk -F, '{sum+=$5} END {print sum}')
        
        local cat_pct=0
        if [[ $cat_examples -gt 0 ]]; then
            cat_pct=$((cat_compiling * 100 / cat_examples))
        fi
        
        echo "        <tr>
            <td>$category</td>
            <td>$cat_docs</td>
            <td>$cat_examples</td>
            <td>$cat_compiling</td>
            <td>$cat_failing</td>
            <td>
                <div class='progress-bar'>
                    <div class='progress-fill' style='width: ${cat_pct}%'></div>
                </div>
                ${cat_pct}%
            </td>
        </tr>" >> "$summary_file"
    done
    
    echo "    </table>
    
    <h2>Document Details</h2>
    <table>
        <tr>
            <th>Document</th>
            <th>Category</th>
            <th>Examples</th>
            <th>Compiling</th>
            <th>Failing</th>
            <th>Completion</th>
        </tr>" >> "$summary_file"
    
    # Add each document to the table
    tail -n +2 "$stats_file" | sort -t, -k2,2 -k1,1 | while IFS=, read -r doc category total compiling failing pct; do
        echo "        <tr>
            <td>$doc</td>
            <td>$category</td>
            <td>$total</td>
            <td>$compiling</td>
            <td>$failing</td>
            <td>
                <div class='progress-bar'>
                    <div class='progress-fill' style='width: ${pct}%'></div>
                </div>
                ${pct}%
            </td>
        </tr>" >> "$summary_file"
    done
    
    echo "    </table>
    
    <h2>Next Steps</h2>
    <ol>
        <li>Focus on documents with the lowest completion percentages</li>
        <li>Prioritize fixing examples in Getting Started and API Reference categories</li>
        <li>Apply common fixes across similar examples</li>
        <li>Re-run verification after each batch of fixes</li>
    </ol>
</body>
</html>" >> "$summary_file"
    
    echo "Generated summary report at $summary_file"
    return 0
}

# Main execution
echo "===== Code Example Verifier Tool ====="
echo "This tool extracts and verifies Rust code examples from markdown files"
echo "Part of the Phase 2 Completion Plan implementation"
echo ""

# Check if specific documents were provided
if [[ $# -gt 0 ]]; then
    for doc in "$@"; do
        if [[ -f "$doc" ]]; then
            extract_code_examples "$doc"
            generate_report "$doc"
        else
            echo "Warning: Document $doc not found, skipping"
        fi
    done
else
    # Process all markdown files
    echo "Searching for markdown files with Rust code examples..."
    
    # Find all markdown files and check for Rust code blocks
    find "$DOCS_DIR" -name "*.md" -exec grep -l "^```rust" {} \; | while read -r doc; do
        extract_code_examples "$doc"
        generate_report "$doc"
    done
fi

# Generate summary report
generate_summary_report

echo ""
echo "Code example verification complete!"
echo "Results are stored in $OUTPUT_DIR"
echo "Summary report generated at $REPORT_DIR/summary_report.html"
echo ""
echo "Next steps:"
echo "1. Review the individual reports for each document"
echo "2. Focus on fixing examples with the most common issues first"
echo "3. Re-run the verifier after making changes"
echo "4. Track progress using the summary report" 