#!/bin/bash

# code-example-tagger.sh - Add language tags to code blocks in markdown files

# Display usage information
usage() {
    echo "Usage: $0 [--file <file_path>] [--dry-run] [--quiet]"
    echo "   or: $0 <file_or_directory>"
    echo
    echo "Arguments:"
    echo "  --file <file_path>    Path to a markdown file to process"
    echo "  <file_or_directory>   Path to a markdown file or directory (alternative syntax)"
    echo "  --dry-run             Show what would be done without making changes"
    echo "  --quiet               Suppress informational output"
    echo
    echo "Examples:"
    echo "  $0 --file docs/getting-started.md     # Process a single file"
    echo "  $0 docs/examples/                      # Process all markdown files in a directory"
    exit 1
}

# Initialize parameters
FILE=""
DRY_RUN=false
QUIET=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --file)
            FILE="$2"
            shift
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            # If no --file parameter provided, treat first argument as the file/directory
            if [[ -z "$FILE" ]]; then
                FILE="$1"
            else
                if [[ "$QUIET" == "false" ]]; then
                    echo "Unknown option: $1"
                    usage
                fi
            fi
            shift
            ;;
    esac
done

# Check for required arguments
if [ -z "$FILE" ]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "Error: No file or directory specified"
    fi
    usage
fi

# Initialize counters
total_files=0
files_modified=0
total_blocks_tagged=0

# Process a single file
process_file() {
    local file=$1
    local temp_file=$(mktemp)
    local blocks_tagged=0
    local in_code_block=0
    local code_content=""
    
    if [[ "$QUIET" == "false" ]]; then
        echo "Processing: $file"
    fi
    
    # If dry run, just report what would be done
    if [[ "$DRY_RUN" == "true" ]]; then
        # Count the untagged code blocks
        blocks_tagged=$(grep -c '^```$' "$file" || true)
        
        if [[ $blocks_tagged -gt 0 ]]; then
            if [[ "$QUIET" == "false" ]]; then
                echo "  Would tag $blocks_tagged code blocks in $file"
            fi
            total_blocks_tagged=$((total_blocks_tagged + blocks_tagged))
            files_modified=$((files_modified + 1))
        else
            if [[ "$QUIET" == "false" ]]; then
                echo "  No untagged code blocks found in $file"
            fi
        fi
        return
    fi
    
    while IFS= read -r line; do
        # Code block start without language tag
        if [ $in_code_block -eq 0 ] && [ "$line" = "```" ]; then
            in_code_block=1
            code_content=""
            echo "$line" >> "$temp_file"
            continue
        fi
        
        # Code block start with language tag
        if [ $in_code_block -eq 0 ] && [[ "$line" = ```* ]] && [ "$line" != "```" ]; then
            in_code_block=2  # Already has language tag
            echo "$line" >> "$temp_file"
            continue
        fi
        
        # Code block end
        if [ $in_code_block -gt 0 ] && [ "$line" = "```" ]; then
            if [ $in_code_block -eq 1 ]; then
                # Add rust as default language tag
                sed -i '' -e '$d' "$temp_file"  # Remove the last line
                echo "```rust" >> "$temp_file"   # Add with language tag
                echo "```" >> "$temp_file"
                blocks_tagged=$((blocks_tagged + 1))
            else
                # This was a block with language already
                echo "$line" >> "$temp_file"
            fi
            in_code_block=0
            continue
        fi
        
        # Collect content in code block for future improvements
        if [ $in_code_block -eq 1 ]; then
            code_content="$code_content
$line"
        fi
        
        # Copy all other lines
        echo "$line" >> "$temp_file"
    done < "$file"
    
    # Update file if changes were made
    if [ $blocks_tagged -gt 0 ]; then
        mv "$temp_file" "$file"
        files_modified=$((files_modified + 1))
        total_blocks_tagged=$((total_blocks_tagged + blocks_tagged))
        if [[ "$QUIET" == "false" ]]; then
            echo "  Tagged $blocks_tagged code blocks in $file"
        fi
    else
        rm "$temp_file"
        if [[ "$QUIET" == "false" ]]; then
            echo "  No untagged code blocks found in $file"
        fi
    fi
}

# Process a file or directory
if [[ -d "$FILE" ]]; then
    # Process all markdown files in the directory
    if [[ "$QUIET" == "false" ]]; then
        echo "Finding markdown files in: $FILE"
    fi
    
    total_files=$(find "$FILE" -type f -name "*.md" | wc -l)
    total_files=$(echo "$total_files" | tr -d ' ')
    
    if [[ "$QUIET" == "false" ]]; then
        echo "Found $total_files markdown files"
    fi
    
    find "$FILE" -type f -name "*.md" | while read -r file; do
        process_file "$file"
    done
elif [[ -f "$FILE" ]]; then
    # Check if it's a markdown file
    if [[ "$FILE" == *.md ]]; then
        total_files=1
        process_file "$FILE"
    else
        if [[ "$QUIET" == "false" ]]; then
            echo "Error: '$FILE' is not a markdown file"
        fi
        exit 1
    fi
else
    if [[ "$QUIET" == "false" ]]; then
        echo "Error: '$FILE' is not a valid file or directory"
    fi
    exit 1
fi

# Print summary
if [[ "$QUIET" == "false" ]]; then
    echo
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Code block tagging simulation complete!"
    else
        echo "Code block tagging complete!"
    fi
    echo "-----------------"
    echo "Files processed:       $total_files"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Files that would be modified: $files_modified"
        echo "Code blocks that would be tagged: $total_blocks_tagged"
    else
        echo "Files modified:        $files_modified"
        echo "Code blocks tagged:    $total_blocks_tagged"
    fi
fi

# Return success if any blocks were tagged/would be tagged
if [[ $total_blocks_tagged -gt 0 ]]; then
    exit 0
else
    # Special exit code to indicate "No untagged code blocks found"
    # This helps batch-fix.sh detect if changes were made
    exit 2
fi
