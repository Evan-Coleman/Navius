#!/bin/bash
#
# code-example-extractor.sh
#
# Extract code examples from Markdown files.
#
# Usage:
#   ./code-example-extractor.sh --file <file_path> [--output <output_file>] [--quiet]
#

# Handle arguments
FILE=""
OUTPUT=""
QUIET=false

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --file)
            FILE="$2"
            shift
            shift
            ;;
        --output)
            OUTPUT="$2"
            shift
            shift
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        *)
            if [[ "$QUIET" == "false" ]]; then
                echo "Unknown option: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Check if file exists
if [[ ! -f "$FILE" ]]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "File not found: $FILE"
    fi
    exit 1
fi

# If no output file specified, use stdout
if [[ -z "$OUTPUT" ]]; then
    OUTPUT="/dev/stdout"
fi

# Extract code examples
if [[ "$QUIET" == "false" ]]; then
    echo "Extracting code examples from $FILE..."
fi

# Create output file with extracted code examples
{
    echo "# Code examples extracted from $FILE"
    echo "# $(date)"
    echo ""
    
    # Extract Rust code blocks
    grep -n -A 50 "\`\`\`rust" "$FILE" | sed -n '/```rust/,/```/p' > "$OUTPUT"
    
    if [[ "$QUIET" == "false" ]]; then
        echo "Code examples extracted to $OUTPUT"
    fi
} > "$OUTPUT"

exit 0 