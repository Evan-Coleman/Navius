---
title: "Link Test File"
description: "A test file with a broken link to verify the fix-links.sh script"
category: "test"
tags: ["test", "links", "validation"]
last_updated: "2025-03-28"
version: "1.0"
---

# Link Test File

This is a test file to verify that our `fix-links.sh` script correctly detects and fixes broken links.

## Working Links

These links should work correctly:

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [README](./README.md)
- [Missing Sections Report](./missing-sections-report.md)

## Broken Links

These links are broken and should be fixed by the script:

- [Old Documentation Standards](/05_reference/standards/documentation-standards.md)
- [Nonexistent File](./nonexistent-file.md)
- [Moved File](../../01_getting_started/old-file.md)

## Related Documents

- [Fix Links Documentation](./fix-links.md)
- [Batch Fix Documentation](./batch-fix.md) 
