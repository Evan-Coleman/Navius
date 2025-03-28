---
title: Documentation Validation Tools
description: Suite of tools for validating documentation quality during the reorganization project
category: Documentation
last_updated: March 28, 2025
---

# Documentation Validation Tools

## Overview

This directory contains a suite of tools developed to support the documentation reorganization project. These tools help validate document structure, check for broken links, extract and verify code examples, and generate comprehensive reports.

## Available Tools

| Tool | Description |
|------|-------------|
| [`simple-validate.sh`](#simple-validatesh) | Validates a single document's frontmatter, structure, code examples, and links |
| [`simple-batch-validate.sh`](#simple-batch-validatesh) | Runs validation on multiple documents and generates a consolidated report |
| [`generate-summary.sh`](#generate-summarysh) | Creates an executive summary of validation results with actionable recommendations |
| [`fix-frontmatter.sh`](#fix-frontmattersh) | Checks for missing frontmatter and adds it when needed |
| [`fix-duplicate-sections.sh`](#fix-duplicate-sectionssh) | Identifies and removes duplicate sections from markdown files |
| [`fix-links.sh`](#fix-linkssh) | Detects and fixes broken links to markdown files |
| [`run-link-analysis.sh`](#run-link-analysissh) | Generates a comprehensive analysis of internal links with fix priorities |
| [`missing-sections-report.sh`](#missing-sections-reportsh) | Generates a detailed report of files missing required sections |
| [`code-example-tagger.sh`](#code-example-taggersh) | Identifies and adds language tags to untagged code blocks |
| [`batch-fix.sh`](#batch-fixsh) | Runs all fix tools on markdown files in a specified directory or a single file to address common issues in bulk |
| [`run-daily-fixes.sh`](#run-daily-fixessh) | Executes daily link fixes according to the action plan |
| [`analyze-fix-logs.sh`](#analyze-fix-logssh) | Generates reports on link fix progress from log files |
| [`setup-environment.sh`](#setup-environmentsh) | Prepares the environment for documentation tools |
| [`run-tests.sh`](#run-testssh) | Tests the functionality of documentation tools |

## Supporting Documents

| Document | Description |
|----------|-------------|
| [`section-templates.md`](#section-templatesmd) | Templates for manually adding standard sections to documents |
| [`validation-tracking-template.md`](#validation-tracking-templatemd) | Template for tracking validation progress across the documentation |
| [`phase3-preparation-plan.md`](#phase3-preparation-planmd) | Guidelines for transitioning from Phase 2 to Phase 3 |

## Tool Usage

### simple-validate.sh

A standalone validator that checks a single document for common issues.

```bash
./simple-validate.sh <file_path>
```

This tool validates:
- Frontmatter (presence and required fields)
- Document structure (main heading, overview, related documents sections)
- Code examples (counts total and Rust-specific blocks)
- Internal links (validates they point to existing files)

### simple-batch-validate.sh

Runs validation on multiple documents in a directory and generates a consolidated report.

```bash
./simple-batch-validate.sh <directory> [output_file] [--verbose]
```

- `<directory>`: Directory containing markdown files to validate
- `[output_file]`: Optional path for the output report (default: `reports/validation-report.md`)
- `--verbose`: Show detailed progress information during processing

This tool:
- Processes all markdown files in the specified directory
- Checks for frontmatter issues (missing titles, descriptions, dates)
- Validates required sections based on document type and location
- Identifies broken links to other documents
- Detects duplicate sections
- Generates a comprehensive markdown report with:
  - Summary statistics of issues found
  - Detailed breakdown by file and issue type
  - Specific recommendations based on findings
  - Prioritized next steps

Reports are stored in the `reports/` directory with filenames based on the directory being validated.

### generate-summary.sh

Creates an executive summary of validation results with actionable recommendations.

```bash
./generate-summary.sh [report_directory]
```

- `[report_directory]`: Optional path to directory containing validation results (default: `./reports`)

This tool analyzes validation results to:
- Identify documents with the most code examples
- Find the most referenced documents
- List documents requiring attention (frontmatter, structure, or link issues)
- Generate specific recommendations for improving documentation quality

### fix-frontmatter.sh

Checks for missing frontmatter in markdown files and adds a basic template if missing.

```bash
./fix-frontmatter.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet]
```

- `--file <file_path>`: Path to the markdown file to check and fix
- `--output <output_file>`: Optional path for the output file (default: overwrites original file)
- `--dry-run`: Run in simulation mode without making actual changes
- `--quiet`: Suppress informational output

This tool:
- Checks if a document has frontmatter (YAML section at the beginning)
- If missing, extracts title from first heading or filename
- Adds a standardized frontmatter template with the current date

### fix-duplicate-sections.sh

Identifies and removes duplicate sections from markdown files, particularly focusing on sections that repeat at the end of documents.

```bash
./fix-duplicate-sections.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet] [--verbose]
```

- `--file <file_path>`: Path to the markdown file to check and fix
- `--output <output_file>`: Optional path for the output file (default: overwrites original file)
- `--dry-run`: Run in simulation mode without making actual changes
- `--quiet`: Suppress informational output
- `--verbose`: Show detailed information about sections found and analysis process

This tool:
- Analyzes markdown files for duplicate section headers (e.g., duplicate "## Overview" sections)
- Uses multiple detection strategies to find duplicate patterns:
  - Identifies template indicators commonly found in duplicated content
  - Detects section headers that appear more than once
  - Handles the specific case of duplicated standard sections after line ~170
- Only removes content from the second occurrence of a section to the end of the file
- Can work as a standalone tool or as part of the batch-fix.sh process
- Takes a conservative approach to avoid removing unique content

Common use case: During document migration, some files ended up with duplicated sections added to the end. This tool safely removes those duplicates while preserving the original content.

### fix-links.sh

Detects broken links in markdown files and fixes them by finding alternative targets.

```bash
./fix-links.sh --dir <directory> [--dry-run] [--verbose]
```

- `--dir <directory>`: Directory containing markdown files to process
- `--dry-run`: Run in simulation mode without making actual changes
- `--verbose`: Show detailed information about broken links and fixes

This tool:
- Scans markdown files for links to other markdown files
- Checks if linked files exist
- Uses intelligent path mapping to fix common link patterns 
- Handles both absolute and relative links
- Fixes issues with missing file extensions, case sensitivity, and path resolution
- Can be run in dry-run mode to safely preview changes
- Provides detailed reports of broken links and fixes

Common use cases:
- Fix links after document migration or restructuring
- Update links to reflect new file organization
- Fix links that were correctly structured but referenced files with the wrong case

### run-link-analysis.sh

Generates a comprehensive analysis of internal links across the documentation to identify and prioritize link fixes.

```bash
./run-link-analysis.sh [--dir DIRECTORY] [--output OUTPUT_FILE] [--verbose]
```

- `--dir <directory>`: Directory to analyze (default: 11newdocs11)
- `--output <file>`: Path for the output report (default: reports/link-analysis-report-YYYYMMDD.md)
- `--verbose`: Show detailed information during analysis
- `--base-dir <base_dir>`: Base directory of the repository (default: /Users/goblin/dev/git/navius)

This tool:
- Scans all markdown files in a directory structure
- Identifies broken internal links
- Calculates success rates for each directory
- Prioritizes fixes based on document importance and broken link count
- Generates a detailed report with actionable information
- Creates a prioritized list of documents that need link fixes

The report includes:
- Overall statistics on link status across the documentation
- Breakdown by directory with success rates
- List of high-priority documents needing fixes
- Common link issues and resolutions
- Day-by-day action plan for implementing fixes

### missing-sections-report.sh

Generates a comprehensive report of files missing required sections across a directory of markdown files.

```bash
./missing-sections-report.sh <directory> [--output <report_file>] [--verbose]
```

- `<directory>`: Directory containing markdown files to analyze
- `--output <report_file>`: Optional path for the output report file (default: script_directory/missing-sections-report.md)
- `--verbose`: Show detailed progress output during scanning

This tool:
- Scans all markdown files in a directory (recursively)
- Identifies which required sections are missing from each file
- Categorizes files based on their location in the documentation hierarchy
- Provides different section requirements for different document types
- Generates a detailed markdown report with:
  - Executive summary statistics
  - Category breakdown of non-compliant files
  - Detailed list of each file and its missing sections
  - List of fully compliant files that can serve as examples
  - Recommendations for improving compliance
- Handles alternative section names and special cases
- Reports are stored in the script's directory by default (11newdocs11/98_roadmaps/doc-reorg-tools/)

Unlike the deprecated add-sections.sh, this tool does not modify files but instead provides a clear report of what needs to be fixed manually for better quality control.

### add-sections.sh (Deprecated)

> **DEPRECATED**: This tool is no longer recommended for use. It has been replaced by [`missing-sections-report.sh`](#missing-sections-reportsh) which provides reporting functionality without modifying files.

This script was previously used to automatically add missing sections to markdown files. Due to the complexity and potential issues with automatic section insertion, we've moved to a reporting-based approach instead.

```bash
./add-sections.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet] [--verbose]
```

### code-example-tagger.sh

Identifies code blocks without language tags and attempts to add appropriate tags.

```bash
./code-example-tagger.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet]
```

- `--file <file_path>`: Path to the markdown file to check and fix
- `--output <output_file>`: Optional path for the output file (default: overwrites original file)
- `--dry-run`: Run in simulation mode without making actual changes
- `--quiet`: Suppress informational output

This tool:
- Scans for code blocks without language specifiers
- Analyzes code content to determine the likely language (Rust, Bash, YAML, JSON, etc.)
- Adds appropriate language tags to improve syntax highlighting and validation

### batch-fix.sh

Runs all fix tools on markdown files in a specified directory or a single file to address common issues in bulk.

```bash
./batch-fix.sh <directory|file> [--dry-run] [--quiet]
```

- `<directory|file>`: Directory containing markdown files to fix or a single markdown file
- `--dry-run`: Run in simulation mode without making actual changes
- `--quiet`: Suppress informational output

This tool:
- Works with both individual files and directories
- Processes all markdown files in the specified directory when given a directory
- Runs fix-frontmatter.sh, add-sections.sh, and code-example-tagger.sh on each file
- Provides a summary of fixes applied or that would be applied (in dry-run mode)
- Maintains counters of files fixed by each tool

### run-daily-fixes.sh

Executes daily link fixes according to the action plan outlined in the documentation reorganization roadmap.

```bash
./run-daily-fixes.sh [--day DAY] [--dry-run] [--verbose]
```

- `--day <day>`: Override to run fixes for a specific day of the week (e.g., monday, tuesday)
- `--dry-run`: Run in simulation mode without making actual changes
- `--verbose`: Show detailed information during execution

This tool:
- Automatically determines the current day and runs the appropriate fixes
- Follows the week 1 action plan for systematic link fixing across documentation
- Generates validation reports after applying fixes
- Updates the link analysis report to track progress
- Can simulate fixes with the --dry-run option
- Allows manual selection of a day's tasks with the --day option

Weekly schedule:
- Friday (March 28): Generate baseline report and set up tools
- Saturday (March 29): Fix links in 01_getting_started
- Sunday (March 30): Fix links in API examples
- Monday (March 31): Fix links in database examples and API reference
- Tuesday (April 1): Fix links in guides/deployment
- Wednesday (April 2): Fix links in contributing and reference/security
- Thursday (April 3): Fix remaining links in lower priority directories

### analyze-fix-logs.sh

Generates reports on link fix progress from log files.

```bash
./analyze-fix-logs.sh [--logs-dir DIR] [--format FORMAT] [--verbose]
```

- `--logs-dir DIR`: Directory containing log files (default: logs/)
- `--format FORMAT`: Output format: md or csv (default: md)
- `--verbose`: Show detailed progress information

This tool:
- Analyzes log files from link fixing operations
- Tracks progress over time with success rates and fix counts
- Identifies common patterns in unfixable links
- Provides visualization of progress in tables and charts
- Generates reports in markdown or CSV format
- Automatically creates reports and logs directories if needed
- Provides actionable recommendations based on analysis

The reports include:
- Summary tables with success rates by directory
- Progress charts showing trends over time
- Effectiveness analysis with success rate calculations
- Lists of common unfixable link patterns for targeted improvements
- Recommended next steps based on the analysis

This tool is particularly useful for monitoring the effectiveness of the daily fix routine and identifying areas that need additional attention or custom fixes.

### setup-environment.sh

Prepares the environment for documentation tools by creating necessary directories and ensuring all prerequisites are met.

```bash
./setup-environment.sh [--verbose]
```

- `--verbose`: Show detailed progress information

This tool:
- Creates the standard directory structure for logs, reports, templates, data, and backups
- Makes all script files executable
- Creates template files for frontmatter and other common content
- Sets up .gitignore to properly handle generated files
- Verifies that all required system tools are available
- Creates a known-issues.md file to track common problems and solutions

Directory structure created:
- `logs/` - For all tool execution logs
- `logs/daily/` - Daily execution logs
- `reports/` - For generated reports
- `reports/daily/` - Daily reports
- `templates/` - Reusable templates for frontmatter, etc.
- `data/` - Data files and configuration
- `backups/` - Backup files organized by date

This tool should be run before using any other documentation tools to ensure the environment is properly set up.

### run-tests.sh

Tests the functionality of the documentation tools to ensure they are working correctly.

```bash
./run-tests.sh [--verbose] [--test TEST_NAME]
```

- `--verbose`: Show detailed test output including command output
- `--test TEST_NAME`: Run only the specified test instead of all tests

This tool:
- Creates a test environment with sample files for validation
- Runs tests for each tool to verify functionality
- Provides feedback on test success/failure with visual pass/fail indicators
- Creates test data with various issues to validate fixing tools:
  - Valid documents that should pass validation
  - Documents with broken links to test link fixing
  - Documents without frontmatter to test frontmatter fixing
  - Documents with duplicate sections to test section fixing
- Cleans up test files after completion (unless run with --verbose)

Available tests:
- `setup-environment`: Tests the environment setup tool
- `fix-links-valid`: Tests link fixing on valid documents
- `fix-links-broken`: Tests link fixing on documents with broken links
- `simple-batch-validate-valid`: Tests validation on valid documents
- `simple-batch-validate-issues`: Tests validation on documents with issues
- `run-daily-fixes-dry-run`: Tests daily fix script in dry-run mode
- `analyze-fix-logs`: Tests log analysis functionality

This tool is particularly useful during development or after updates to ensure that changes haven't broken existing functionality.

## Typical Workflow

1. Set up the environment:
   ```bash
   ./setup-environment.sh --verbose
   ```

2. Run validation on a document:
   ```bash
   ./simple-validate.sh 11newdocs11/98_roadmaps/30_documentation-reorganization-roadmap.md
   ```

3. Validate all documents in a directory:
   ```bash
   ./simple-batch-validate.sh 11newdocs11/98_roadmaps
   ```

4. Generate an executive summary:
   ```bash
   ./generate-summary.sh
   ```

5. Fix frontmatter issues:
   ```bash
   ./fix-frontmatter.sh --file 11newdocs11/01_getting_started/installation.md
   ```

6. Analyze internal links across the documentation:
   ```bash
   ./run-link-analysis.sh --verbose
   ```

7. Fix broken links in a file (dry run first):
   ```bash
   ./fix-links.sh --file 11newdocs11/05_reference/api/client.md --dry-run --verbose
   ```

8. Generate a report of files missing required sections:
   ```bash
   ./missing-sections-report.sh 11newdocs11/02_examples
   ```

9. Add language tags to code blocks:
   ```bash
   ./code-example-tagger.sh --file 11newdocs11/05_reference/api/client.md
   ```

10. Apply frontmatter and code block fixes to a directory:
    ```bash
    ./batch-fix.sh 11newdocs11/01_getting_started
    ```

11. Fix duplicate sections in a file:
    ```bash
    ./fix-duplicate-sections.sh --file 11newdocs11/05_reference/architecture/diagrams/app-core-interactions.md --verbose
    ```

12. Analyze link fix progress:
    ```bash
    ./analyze-fix-logs.sh --verbose
    ```

13. Run tests to ensure tools are working correctly:
    ```bash
    ./run-tests.sh --verbose
    ```

## Supporting Document Details

### section-templates.md

Provides standardized templates for manually adding missing sections to documentation files. Contains templates for:

- Getting Started Document sections (Overview, Prerequisites, Installation, Configuration, etc.)
- Examples Document sections (Overview, Prerequisites, Setup, Step-by-Step Guide, etc.)
- Reference Document sections (Overview, API, Examples, Best Practices, etc.)

Use these templates to manually add missing sections identified by the missing-sections-report.sh tool.

### validation-tracking-template.md

Template for tracking validation progress across the documentation reorganization project. Contains:

- Summary tables of validation progress by category 
- Tools applied with success rates
- Detailed directory tracking with file status
- Two-week action plan with daily tasks
- Critical issues tracking

### phase3-preparation-plan.md

Guidelines and preparation steps for transitioning from Phase 2 to Phase 3 of the project, including:

- Phase 2 completion requirements
- Gap analysis preparation
- Content creation preparation
- Implementation tools
- Transition timeline

## Output Files

- Individual validation results are stored in the `reports` directory with the naming pattern `filename_validation.txt`
- The batch validation report is stored as `validation-report.md` (or custom name if specified)
- The executive summary is stored as `validation-summary.md`
- The missing sections report is stored as `missing-sections-report.md`

## Related Documents

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Documentation Reorganization Instructions](../30_documentation-reorganization-instructions.md)
- [Documentation Standards](/11newdocs11/05_reference/standards/documentation-standards.md)