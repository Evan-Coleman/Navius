---
title: "fix-links.sh Documentation"
description: "Tool for detecting and fixing broken links in markdown documentation"
category: "Documentation Tools"
tags: ["documentation", "automation", "quality", "links"]
last_updated: "2025-03-28"
version: "1.0"
---

# fix-links.sh

## Overview

The `fix-links.sh` script scans markdown files for links to other markdown files and checks if the linked files exist. It can identify broken links and automatically fix them by finding suitable replacements based on filename matches.

## Prerequisites

- Bash shell
- Access to the documentation repository
- `find` and `grep` utilities

## Usage

```bash
./fix-links.sh [--dir DIRECTORY] [--file FILE] [--dry-run] [--verbose] [--base-dir BASE_DIR] [--docs-root DOCS_ROOT]
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `--dir DIRECTORY` | Directory containing markdown files to process |
| `--file FILE` | Single markdown file to process |
| `--dry-run` | Run in simulation mode without making actual changes |
| `--verbose` | Enable detailed output during processing |
| `--base-dir BASE_DIR` | Base directory of the repository (default: /Users/goblin/dev/git/navius) |
| `--docs-root DOCS_ROOT` | Root directory of documentation (default: 11newdocs11) |

Either `--dir` or `--file` must be specified.

## How It Works

1. **Link Extraction**: The script extracts all markdown links that point to `.md` files.
2. **Link Validation**: Each link is checked to determine if the target file exists.
3. **Alternative Finding**: For broken links, the script searches for files with the same name elsewhere in the documentation.
4. **Link Correction**: When a suitable alternative is found, the script replaces the broken link with the path to the alternative file.

## Examples

### Check a single file for broken links (dry run)

```bash
./fix-links.sh --file path/to/document.md --dry-run --verbose
```

### Fix broken links in a directory

```bash
./fix-links.sh --dir path/to/directory --verbose
```

### Fix links with custom base directory

```bash
./fix-links.sh --dir path/to/directory --base-dir /custom/path --docs-root docs
```

## Output

When running with the `--verbose` flag, the script provides detailed information about:
- Files being processed
- Broken links found
- Potential matches discovered
- Links that were fixed (or would be fixed in dry-run mode)

At the end of directory processing, a summary is displayed showing:
- Total number of files processed
- Number of files with fixed links
- Total number of links fixed

## Best Practices

1. Always run with `--dry-run` first to see what changes would be made
2. Use `--verbose` to get detailed information about link problems
3. Run as part of your documentation quality process after content changes
4. Consider running after moving files or restructuring directories

## Limitations

- The script only fixes links to `.md` files
- Link correction is based on filename matching, which may not always find the correct replacement
- Links with anchors (#sections) are detected but the anchors aren't validated
- HTML links are not processed

## Related Tools

- `fix-frontmatter.sh` - For fixing frontmatter issues
- `batch-fix.sh` - For running multiple fix tools together
- `missing-sections-report.sh` - For identifying missing document sections 