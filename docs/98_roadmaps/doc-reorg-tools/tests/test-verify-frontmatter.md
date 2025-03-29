# Test Markdown File

This is a test file to verify the frontmatter verification script. This file has intentionally incomplete frontmatter.

---
title: "Test Document"
description: "A test document for verifying the frontmatter script"
# Missing category field
tags: ["test", "frontmatter"]
# Missing last_updated field
---

## Introduction

This document is used to test the `verify-frontmatter.sh` script, which should identify:

1. Missing required fields (category, last_updated)
2. Validate the format of existing fields

## Test Cases

- **Missing Fields**: This document has deliberately omitted the 'category' and 'last_updated' fields
- **Format Validation**: The script should validate date formats and tag array formats
- **Autocorrection**: With the `--fix` option, the script should add the missing fields

## Expected Results

When running:

```bash
./verify-frontmatter.sh --file tests/test-verify-frontmatter.md
```

The script should report:
- Missing category field
- Missing last_updated field

When running with the fix option:
```bash
./verify-frontmatter.sh --file tests/test-verify-frontmatter.md --fix
```

The script should automatically add the missing fields with appropriate defaults. 