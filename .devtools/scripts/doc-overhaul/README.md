# Documentation Utility Scripts

This directory contains scripts for validating and improving documentation in the Navius project.

## Recent Updates

**May 30, 2024**
- Created shell-agnostic utility library (`shell_utils.sh`) for cross-shell compatibility
- Fixed `generate_report.sh` for cross-shell compatibility
- Fixed `fix_frontmatter.sh` for cross-shell compatibility
- Fixed `fix_links.sh` for cross-shell compatibility
- Fixed `add_sections.sh` for cross-shell compatibility
- Fixed `improve_docs.sh` for cross-shell compatibility
- Fixed `comprehensive_test.sh` for cross-shell compatibility and enhanced visualization

## Shell Utilities Library

The `shell_utils.sh` library provides common functions used across documentation scripts:

- **File Operations**: `ensure_dir()`, `has_frontmatter()`, `extract_frontmatter()`, `get_frontmatter_field()`, `get_frontmatter_list()`
- **String Manipulation**: `trim()`, `to_lowercase()`, `get_today_date()`
- **Logging**: `log_info()`, `log_success()`, `log_warning()`, `log_error()`, `log_header()`
- **Error Handling**: `set_strict_mode()`, `check_command()`

## Scripts

### generate_report.sh

Generates a report on the quality of documentation.

**Usage:**
```bash
./generate_report.sh [--dir DIR] [--file FILE] [--vis] [--skip-linting]
```

**Options:**
- `--dir DIR`: Process all markdown files in the specified directory
- `--file FILE`: Process a single file
- `--vis`: Generate a visualization of documentation quality
- `--skip-linting`: Skip the markdownlint validation step

### fix_frontmatter.sh

Validates and fixes frontmatter in markdown files.

**Usage:**
```bash
./fix_frontmatter.sh [--dir DIR] [--file FILE] [--fix] [--dry-run] [--verbose] [--help]
```

**Options:**
- `--dir DIR`: Process all markdown files in the specified directory
- `--file FILE`: Process a single file
- `--fix`: Automatically apply fixes
- `--dry-run`: Show what would be fixed without applying changes
- `--verbose`: Show detailed information about the process

### fix_links.sh

Validates and fixes links in markdown files.

**Usage:**
```bash
./fix_links.sh [--dir DIR] [--file FILE] [--fix] [--check-only] [--recursive] [--help]
```

**Options:**
- `--dir DIR`: Process all markdown files in the specified directory
- `--file FILE`: Process a single file
- `--fix`: Automatically fix links
- `--check-only`: Only check for broken links without fixing
- `--recursive`: Process directories recursively

### add_sections.sh

Adds missing standard sections to markdown files based on document type.

**Usage:**
```bash
./add_sections.sh [--dir DIR] [--file FILE] [--apply] [--check-only] [--report] [--help]
```

**Options:**
- `--dir DIR`: Process all markdown files in the specified directory
- `--file FILE`: Process a single file
- `--apply`: Automatically add missing sections
- `--check-only`: Only check for missing sections without adding
- `--report`: Generate a report of missing sections

### improve_docs.sh

Interactive assistant for improving documentation, guiding you through a series of checks and improvements.

**Usage:**
```bash
./improve_docs.sh [--dir DIR] [--file FILE] [--interactive] [--auto] [--help]
```

**Options:**
- `--dir DIR`: Process all markdown files in the specified directory
- `--file FILE`: Process a single file
- `--interactive`: Run in interactive mode (default)
- `--auto`: Apply automatic fixes where possible

### comprehensive_test.sh

Performs advanced documentation validation and analysis, including document relationships, content quality, and readability metrics.

**Usage:**
```bash
./comprehensive_test.sh [--file FILE] [--dir DIRECTORY] [--csv] [--rules FILE] [--help]
```

**Options:**
- `--file FILE`: Analyze a single file instead of all documentation
- `--dir DIRECTORY`: Focus analysis on a specific directory
- `--csv`: Output results in CSV format for spreadsheet import
- `--rules FILE`: Use custom validation rules from specified file

The script generates a comprehensive report with:
- Document quality distribution
- Readability metrics
- Code block validation
- Document relationship visualization
- Improvement recommendations for documents with issues

## Workflow

1. Start with `generate_report.sh` to get an overview of documentation quality
2. Use `fix_frontmatter.sh` to fix metadata issues
3. Use `fix_links.sh` to fix broken links
4. Use `add_sections.sh` to add missing standard sections
5. Use `improve_docs.sh` for guided improvements
6. Use `comprehensive_test.sh` for detailed analysis and visualization of document relationships

## Using the Shell Utilities in Your Scripts

To use the shell utilities in your own scripts:

```bash
#!/bin/sh
# Your script

SCRIPT_DIR="$(dirname "$0")"
. "$SCRIPT_DIR/shell_utils.sh"

# Enable strict mode (exit on error, undefined variables, and pipe failures)
set_strict_mode

# Now you can use utility functions
log_info "Starting process"
ensure_dir "target/reports"

# Check if a file has frontmatter
if has_frontmatter "file.md"; then
    frontmatter=$(extract_frontmatter "file.md")
    title=$(get_frontmatter_field "$frontmatter" "title")
    log_success "Found title: $title"
fi
```

## Standards

The scripts enforce these standards:
- Frontmatter must include: title, description, category, tags
- All documents should have at least: introduction, usage/example, related documents sections
- Links should be valid within the documentation
- Code blocks should specify the language

## Important Notes

- Always verify changes made by these scripts
- For large-scale operations, use the `--dry-run` option first
- The scripts are designed for incremental improvement - focus on one file at a time for best results

## Documentation Overhaul Tools

This directory contains scripts for methodically improving documentation quality in a controlled, file-by-file approach. Instead of making bulk changes that could introduce errors, these tools focus on making targeted improvements to one file at a time.

## Available Scripts

### Main Script

- **improve_docs.sh**: The main user interface that guides you through the documentation improvement process. It will validate documentation, help you select files to work on, and apply necessary fixes one file at a time.

```bash
./scripts/doc-overhaul/improve_docs.sh
```

### Individual Tools

- **detailed_validation.sh**: Analyzes all documentation files without making changes, producing a detailed report of issues.

```bash
./scripts/doc-overhaul/detailed_validation.sh
```

- **fix_frontmatter.sh**: Adds or fixes frontmatter on a specific file.

```bash
./scripts/doc-overhaul/fix_frontmatter.sh path/to/file.md
```

- **add_sections.sh**: Adds only the Related Documents section to a document based on its type.

```bash
./scripts/doc-overhaul/add_sections.sh path/to/file.md
```

- **fix_links.sh**: Identifies and helps fix broken links in a single document.

```bash
./scripts/doc-overhaul/fix_links.sh path/to/file.md
```

## Workflow

The recommended workflow is:

1. Run `./scripts/doc-overhaul/improve_docs.sh` to start the guided process
2. Follow the interactive prompts to improve one document at a time
3. Review and approve changes before committing
4. Repeat until all documentation issues are fixed

## Link Path Standards

These scripts enforce the use of absolute paths from the project root for all internal documentation links, rather than relative paths. This has several key benefits:

1. **Resilience to Moves**: When files are relocated, absolute paths don't break
2. **Easier Maintenance**: No need to calculate relative paths with `../` which can be error-prone
3. **Better Readability**: Absolute paths make it immediately clear where the target document is located
4. **Consistent Pattern**: All links follow the same pattern

### Examples

```markdown
<!-- ✅ GOOD: Absolute paths from project root -->
[Installation Guide](/docs/guides/installation.md)
[Project Structure](/docs/architecture/project-structure.md)

<!-- ❌ BAD: Relative paths with ../ -->
[Installation Guide](../guides/installation.md)
[Project Structure](../architecture/project-structure.md)

<!-- ❌ BAD: Absolute paths without /docs/ prefix -->
[Installation Guide](/guides/installation.md)
[Project Structure](/architecture/project-structure.md)
```

### Important Note

All absolute paths should begin with `/docs/` to ensure they correctly resolve both in the repository and when the documentation is deployed. Paths that start with just `/` without the `docs/` prefix will be automatically corrected by the scripts.

## Philosophy

This approach follows these key principles:

1. **Control**: Make changes to one file at a time to maintain control of the process
2. **Verification**: Verify each change before committing
3. **Incremental Progress**: Break down a large overhaul into manageable steps
4. **Interactive**: Get human input on decisions that require judgment
5. **Reversible**: Every change can be reviewed and reversed if needed

## Documentation Standards

These tools enforce the following standards:

- All documents have proper frontmatter (title, description, category, tags, etc.)
- All documents include a Related Documents section for easy navigation
- All internal links use absolute paths from the project root (starting with /docs/)
- All links resolve correctly to existing files

## Benefits vs. Batch Processing

While batch processing scripts might seem faster, this file-by-file approach:

1. Reduces the risk of introducing errors
2. Allows for human judgment on complex decisions
3. Ensures higher quality results
4. Makes it easier to track progress
5. Creates smaller, more focused git commits

## Next Steps After Completion

Once the documentation overhaul is complete:

1. Update the roadmap status to reflect completion
2. Implement a documentation validation step in CI to maintain quality
3. Create documentation contribution guidelines based on the established standards
4. Set up automated documentation testing 