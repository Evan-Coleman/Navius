---
title: Documentation Standards Guide
description: Style guide and standards for Navius documentation
category: roadmaps
tags:
  - documentation
  - standards
  - style-guide
  - markdown
related:
  - ../12_document_overhaul.md
  - documentation-overhaul-guide.md
  - directory-structure-design.md
last_updated: March 23, 2025
version: 1.0
---

# Documentation Standards Guide

## Markdown Style Guide

### Document Structure

#### Metadata Header
Every document must include a YAML metadata header:

```yaml
---
title: Document Title
description: Brief description of the document
category: guides | reference | roadmaps | contributing
tags:
  - tag1
  - tag2
related:
  - path/to/related/doc1.md
  - path/to/related/doc2.md
last_updated: YYYY-MM-DD
version: 1.0
---
```

#### Heading Structure
- Use a single `#` for the document title
- Start with `##` for main sections
- Use increasing heading levels for subsections
- Don't skip heading levels (e.g., don't go from `##` to `####`)
- Keep headings concise and descriptive

### Text Formatting

#### Paragraphs
- Use a single blank line between paragraphs
- Keep paragraphs focused on a single topic
- Aim for 3-5 sentences per paragraph maximum

#### Emphasis
- Use **bold** (`**text**`) for emphasis or UI elements
- Use *italic* (`*text*`) for introduced terms or parameters
- Use `code` (`` `code` ``) for code snippets, commands, or filenames
- Avoid using ALL CAPS for emphasis

#### Lists
- Use unordered lists (`-`) for items without specific order
- Use ordered lists (`1.`) for sequential steps or prioritized items
- Maintain consistent indentation for nested lists
- Include a blank line before and after lists

```markdown
- Item 1
- Item 2
  - Nested item 1
  - Nested item 2
- Item 3

1. First step
2. Second step
   1. Substep 1
   2. Substep 2
3. Third step
```

### Code Elements

#### Inline Code
Use backticks for inline code:
```markdown
The `Config` struct contains the application configuration.
```

#### Code Blocks
Use triple backticks with a language specifier:

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

#### Command Line Examples
For command line examples, use `bash` or `shell` as the language:

````markdown
```bash
cargo run --release
```
````

### Links and References

#### Internal Links
Use relative links to reference other documents:
```markdown
See the [Installation Guide](../guides/installation.md) for more information.
```

#### External Links
Use complete URLs for external links:
```markdown
Visit the [Rust website](https://www.rust-lang.org/) for more information.
```

#### Images
Include images with alt text:
```markdown
![Architecture Diagram](../images/architecture.png)
```

### Tables

Use tables for structured data:
```markdown
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

### Notes and Callouts

Use blockquotes with prefixes for notes and warnings:
```markdown
> **Note:** This is important information.

> **Warning:** This is a critical warning.
```

## Document Type Standards

### Index Documents (README.md)

Required sections:
1. **Introduction** - Brief overview of the section
2. **Document List** - List of documents with descriptions
3. **Key Documents** - Highlighted important documents
4. **Getting Started** - Quick start information (if applicable)

Example:
```markdown
# Guide Documentation

This directory contains guides for using and developing with the Navius framework.

## Document List

- [Development Workflow](development-workflow.md) - Guide to the development process
- [Testing Guide](testing.md) - How to write and run tests
- [Authentication](authentication.md) - Setting up and using authentication

## Key Documents

If you're new to development, start with:
- [Development Workflow](development-workflow.md)
- [Project Structure](../reference/project-structure.md)

## Getting Started

For new developers, we recommend following these guides in order:
1. [Installation Guide](../getting-started/installation.md)
2. [Development Setup](../getting-started/development-setup.md)
3. [Development Workflow](development-workflow.md)
```

### Guide Documents

Required sections:
1. **Overview** - What the guide covers
2. **Prerequisites** - Required knowledge or setup
3. **Step-by-step Instructions** - Detailed procedural steps
4. **Examples** - Practical examples of concepts
5. **Troubleshooting** - Common issues and solutions
6. **Related Documents** - Links to related information

Example:
```markdown
# Authentication Guide

## Overview
This guide explains how to implement authentication in your Navius application.

## Prerequisites
- Basic understanding of Rust and Axum
- Navius development environment set up
- Access to Microsoft Entra (formerly Azure AD)

## Step-by-step Instructions
1. **Configure Environment Variables**
   ```shell
   export ENTRA_CLIENT_ID=your-client-id
   export ENTRA_TENANT_ID=your-tenant-id
   ```

2. **Add Authentication Middleware**
   ```rust
   // Add authentication middleware code example
   ```

## Examples
Here's a complete example of a protected route:
```rust
// Full example code
```

## Troubleshooting
- **Token validation fails**: Check your tenant ID configuration
- **Auth middleware errors**: Ensure environment variables are set

## Related Documents
- [Security Standards](../reference/standards/security-standards.md)
- [API Integration Guide](api-integration.md)
```

### Reference Documents

Required sections:
1. **Overview** - Brief introduction to the reference
2. **Detailed Specifications** - In-depth technical details
3. **Examples** - Code examples or usage
4. **Related Documents** - Links to related information

Example:
```markdown
# API Resources Reference

## Overview
This document provides detailed reference for all API resources in the Navius framework.

## Detailed Specifications
### User Resource
- **Endpoint**: `/api/users`
- **Methods**: GET, POST, PUT, DELETE
- **Fields**:
  - `id`: UUID, primary identifier
  - `username`: String, unique username
  - `email`: String, user's email address

### Authentication Resource
- **Endpoint**: `/api/auth`
- **Methods**: POST
- **Fields**:
  - `username`: String, user's username
  - `password`: String, user's password

## Examples
### Fetching a User
```rust
// Example code for fetching a user
```

## Related Documents
- [API Patterns](patterns.md)
- [Authentication Guide](../guides/features/authentication.md)
```

### Roadmap Documents

Required sections:
1. **Overview** - Purpose and goals of the roadmap
2. **Current Status** - Current state of the feature/system
3. **Target State** - Desired end state
4. **Implementation Steps** - Step-by-step implementation plan
5. **Progress Tracking** - Tracking tasks and completion status
6. **Success Criteria** - Measurable completion criteria

Example:
```markdown
# API Versioning Roadmap

## Overview
This roadmap outlines the implementation of API versioning in the Navius framework to support backward compatibility and evolution of the API.

## Current Status
- No API versioning support
- Breaking changes require client updates
- No way to deprecate endpoints

## Target State
- Support for multiple API versions simultaneously
- Clear deprecation path for older versions
- Automated testing across versions
- Version-specific documentation

## Implementation Steps
1. **Design API Versioning Strategy**
   - [ ] Research versioning approaches
   - [ ] Document chosen strategy
   - [ ] Create sample implementation

2. **Implement Core Versioning Support**
   - [ ] Add version detection middleware
   - [ ] Implement router with version support
   - [ ] Create version selection logic

## Progress Tracking
- **Overall Progress**: 25% complete
- **Current Phase**: Design API Versioning Strategy
- **Next Milestone**: Core Versioning Support

## Success Criteria
- API requests can specify version
- Multiple versions of same endpoint can coexist
- Tests pass for all supported versions
- Documentation clearly indicates version support
```

## Document Validation Rules

1. **Required Metadata**
   - All documents must have the YAML metadata header
   - Title, description, category, and last_updated are required
   - Tags and related documents are recommended

2. **Required Sections**
   - Each document type must include the sections listed above
   - Section headings must match the specified format

3. **Link Validation**
   - All internal links must point to valid files
   - No broken links allowed
   - Relative paths preferred for internal links

4. **Formatting Validation**
   - Consistent heading structure
   - Proper code block formatting
   - Correct list formatting
   - Table formatting with proper alignment

5. **Content Validation**
   - No placeholder text in published documents
   - Examples must be valid and tested
   - Commands must be accurate
   - No duplicate content across documents

## Implementation Guidelines

When implementing this standard:

1. **Use consistent tooling**
   - Configure markdownlint with project standards
   - Use automated link checkers
   - Implement pre-commit hooks for validation

2. **Integrate with workflow**
   - Add documentation validation to CI pipeline
   - Include documentation review in PR process
   - Automate metadata validation

3. **Version appropriately**
   - Update version numbers on significant changes
   - Keep last_updated field current
   - Archive outdated versions