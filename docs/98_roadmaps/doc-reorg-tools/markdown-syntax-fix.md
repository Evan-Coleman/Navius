---
title: "Markdown Code Block Syntax Fix"
description: "Guidelines for fixing incorrect code block language markers in markdown documentation"
category: "Documentation Tools"
tags: ["documentation", "markdown", "syntax", "code-blocks"]
last_updated: "March 31, 2025"
version: "1.0"
---

# Markdown Code Block Syntax Fix

## Issue Description

We've identified a consistent pattern of incorrect markdown syntax in our documentation files. Many code blocks have language markers at both the beginning AND end of the code block, which is invalid markdown syntax:

```
INCORRECT SYNTAX:

```rust
// Rust code here
```rust  <-- This is incorrect
```

```
CORRECT SYNTAX:

```rust
// Rust code here
```  <-- Just triple backticks with no language marker
```

This issue can cause several problems:
- Broken syntax highlighting in documentation viewers
- Errors in documentation generation tools
- Inconsistent rendering across different platforms
- Potential parsing errors in markdown processors

## Affected Files

Initial analysis has identified this issue in several files, including:
- `authentication-example.md`
- `error-handling-example.md`
- `rest-api-example.md`
- Additional files likely also affected

## Fix Guidelines

### Manual Fix Process

For each markdown file:

1. Search for code blocks with language markers at both the beginning and end
2. Remove the language marker from the closing triple backticks
3. Ensure the language marker at the beginning is correct for the code type
4. Verify the fix by viewing the rendered markdown

### Automated Fix Script

We will develop a script to automate this process:

```bash
#!/bin/bash

# Script to fix incorrect markdown code block syntax

# Find files with potential issues (look for closing backticks with language markers)
find_affected_files() {
  grep -l "^\`\`\`[a-z]*$" $1 --include="*.md" -r
}

# Fix a single file
fix_file() {
  local file=$1
  echo "Fixing file: $file"
  
  # Create a backup
  cp "$file" "${file}.bak"
  
  # Replace closing code blocks with just backticks
  # This uses sed to find lines that only contain backticks followed by a language marker
  sed -i 's/^```[a-z][a-z]*$/```/g' "$file"
  
  echo "Fixed file: $file"
}

# Main function
main() {
  local directory=${1:-"11newdocs11"}
  
  echo "Searching for affected files in $directory..."
  local affected_files=$(find_affected_files "$directory")
  
  if [ -z "$affected_files" ]; then
    echo "No affected files found."
    exit 0
  fi
  
  echo "Found affected files:"
  echo "$affected_files"
  echo ""
  
  echo "Fixing files..."
  for file in $affected_files; do
    fix_file "$file"
  done
  
  echo "All files fixed."
}

# Run the script
main "$@"
```

## Documentation Update

We should also add this to our contributing guidelines to prevent the issue from recurring:

```markdown
### Code Block Syntax

When adding code examples to documentation, use the following syntax:

    ```language
    // Code here
    ```

The language identifier (e.g., `rust`, `bash`, `json`) should ONLY appear after the opening triple backticks. The closing triple backticks should have no language identifier.

✅ CORRECT:
```rust
let x = 5;
```

❌ INCORRECT:
```rust
let x = 5;
```rust
```

## Implementation Plan

1. Create fix script in `11newdocs11/98_roadmaps/doc-reorg-tools/`
2. Run script on all markdown files in `11newdocs11/` directory
3. Add syntax guideline to contributing documentation
4. Add check to validation script to catch this issue in future updates

## Common Language Markers Used

For reference, here are the common language markers we use in our documentation:

| Language | Marker |
|----------|--------|
| Rust | ```rust |
| Bash/Shell | ```bash or ```sh |
| JSON | ```json |
| YAML | ```yaml |
| SQL | ```sql |
| HTML | ```html |
| JavaScript | ```js |
| TypeScript | ```ts |
| CSS | ```css |
| Markdown | ```markdown |

## Related Documents

- [Week 1 Action Plan Tracker](week1-action-tracker.md)
- [Validation Status Dashboard](validation-status-dashboard.md)
- [Documentation Standards](../../05_reference/standards/documentation-standards.md) 