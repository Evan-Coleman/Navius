---
title: "batch-fix.sh Documentation"
description: "Automatically apply essential fixes to documentation files"
category: "Documentation Tools"
tags: ["documentation", "automation", "quality"]
last_updated: "2025-03-28"
version: "2.1"
---

# batch-fix.sh

## Overview

The `batch-fix.sh` script automates essential fixes to markdown documentation files. It focuses on:

1. Fixing missing frontmatter
2. Detecting and removing duplicate sections
3. Finding and fixing broken links to other markdown files

The script avoids automatically adding missing sections, as this is now a manual process using the templates in `section-templates.md`.

## Prerequisites

- Bash shell
- Access to the documentation repository

## Usage

```bash
./batch-fix.sh [--dir DIRECTORY] [--file FILE] [--verbose] [--dry-run]
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `--dir DIRECTORY` | Directory containing markdown files to process |
| `--file FILE` | Single markdown file to process |
| `--verbose` | Enable detailed output during processing |
| `--dry-run` | Run in simulation mode without making actual changes |

Either `--dir` or `--file` must be specified.

## How It Works

The script performs the following tasks in sequence:

1. **Frontmatter Validation**: Checks if frontmatter exists and adds it if missing
2. **Duplicate Section Detection**: Identifies and removes duplicate sections
3. **Link Validation**: Finds broken links and attempts to fix them

Each step is executed sequentially to ensure proper document structure before making further changes.

## Examples

### Process a single file with verbose output

```bash
./batch-fix.sh --file path/to/document.md --verbose
```

### Process all files in a directory with dry run

```bash
./batch-fix.sh --dir path/to/directory --verbose --dry-run
```

### Process files in multiple directories

```bash
for dir in 01_getting_started 02_examples 03_contributing; do
  ./batch-fix.sh --dir 11newdocs11/$dir --verbose
done
```

## Related Tools

- `fix-frontmatter.sh` - For fixing just frontmatter issues
- `fix-duplicate-sections.sh` - For removing duplicate sections only
- `fix-links.sh` - For fixing just broken links
- `missing-sections-report.sh` - To generate reports of files with missing sections
- `section-templates.md` - Templates for manually adding missing sections

## Best Practices

1. Run with `--dry-run` first to see what changes would be made
2. Use with `--verbose` flag to see detailed output
3. After running this script, use `missing-sections-report.sh` to identify files needing manual section additions
4. When adding missing sections, use the templates in `section-templates.md`
5. Consider running after file reorganization to fix broken links

## Known Limitations

- Link fixing is based on filename matching, which may not always find the correct replacement
- The script doesn't add missing sections - this requires manual intervention
- Link fixing only applies to markdown file links, not other types of links (HTML, external URLs)
- Files must have a `.md` extension to be processed 