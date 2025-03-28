---
title: Documentation Directory Structure Design
description: Proposed directory structure for the documentation overhaul
category: roadmaps
tags:
  - documentation
  - organization
  - structure
related:
  - ../12_document_overhaul.md
  - documentation-overhaul-guide.md
  - documentation-audit.md
last_updated: March 23, 2025
version: 1.0
---

# Documentation Directory Structure Design

## Proposed Directory Structure

Based on our documentation audit, we propose the following directory structure for the Navius documentation:

```
/docs
  ├── README.md                       # Main documentation index
  │
  ├── getting-started/                # Quick start documentation
  │   ├── README.md                   # Getting started index
  │   ├── installation.md             # Installation guide
  │   ├── development-setup.md        # Development environment setup
  │   └── first-steps.md              # First steps with the framework
  │
  ├── guides/                         # User and developer guides
  │   ├── README.md                   # Guides index
  │   ├── development/                # Development guides
  │   │   ├── README.md               # Development guides index
  │   │   ├── development-workflow.md # Development workflow
  │   │   └── testing.md              # Testing guide
  │   ├── features/                   # Feature-specific guides
  │   │   ├── README.md               # Features index
  │   │   ├── authentication.md       # Authentication guide
  │   │   └── api-integration.md      # API integration guide
  │   └── deployment/                 # Deployment guides
  │       ├── README.md               # Deployment guides index
  │       ├── aws-deployment.md       # AWS deployment guide
  │       └── postgresql-setup.md     # PostgreSQL setup guide
  │
  ├── reference/                      # Technical reference documentation
  │   ├── README.md                   # Reference index
  │   ├── api/                        # API reference
  │   │   ├── README.md               # API reference index
  │   │   ├── resources.md            # API resources reference
  │   │   └── patterns.md             # API patterns reference
  │   ├── architecture/               # Architecture reference
  │   │   ├── README.md               # Architecture index
  │   │   ├── project-structure.md    # Project structure reference
  │   │   └── diagrams/               # Architecture diagrams
  │   │       ├── app-core.md         # App-core interaction diagrams
  │   │       ├── app-modules.md      # App module diagrams
  │   │       └── core-modules.md     # Core module diagrams
  │   └── standards/                  # Code standards
  │       ├── README.md               # Standards index
  │       ├── naming-conventions.md   # Naming conventions
  │       ├── import-patterns.md      # Import patterns
  │       └── security-standards.md   # Security standards
  │
  ├── contributing/                   # Contribution guidelines
  │   ├── README.md                   # Contributing index
  │   ├── contributing-guide.md       # Main contribution guide
  │   ├── onboarding.md               # Onboarding guide
  │   ├── ide-setup.md                # IDE setup guide
  │   └── code-review.md              # Code review process
  │
  ├── roadmaps/                       # Development roadmaps
  │   ├── README.md                   # Roadmaps index
  │   ├── active/                     # Active roadmaps
  │   │   ├── README.md               # Active roadmaps index
  │   │   └── [numbered roadmaps].md  # Individual active roadmaps
  │   ├── completed/                  # Completed roadmaps
  │   │   ├── README.md               # Completed roadmaps index
  │   │   └── [completed roadmaps].md # Individual completed roadmaps
  │   └── implementation/             # Implementation guides
  │       ├── README.md               # Implementation guides index
  │       ├── template.md             # Roadmap implementation template
  │       └── [specific guides].md    # Specific implementation guides
  │
  └── resources/                      # Additional resources
      ├── README.md                   # Resources index
      ├── glossary.md                 # Glossary of terms
      ├── troubleshooting.md          # Troubleshooting guide
      └── faq.md                      # Frequently asked questions
```

## Naming Conventions

### Directory Naming
- Use kebab-case for all directory names
- Use descriptive, concise names
- Group related documents together
- Avoid abbreviations unless universally understood

### File Naming
- Use kebab-case for all filenames
- Include document type in name where appropriate (e.g., auth-guide.md, api-reference.md)
- Use README.md for index files in each directory
- Prefix roadmap files with numbers for ordering (e.g., 01-dependency-injection.md)

## Document Types and Templates

### Index Documents (README.md)
- Purpose: Provide navigation and overview for a section
- Required sections:
  - Introduction
  - Document list with brief descriptions
  - Most important/frequently accessed documents
  - Getting started information (where appropriate)

### Guide Documents
- Purpose: Walk users through processes or features
- Required sections:
  - Overview
  - Prerequisites
  - Step-by-step instructions
  - Examples
  - Troubleshooting
  - Related documents

### Reference Documents
- Purpose: Provide technical specifications
- Required sections:
  - Overview
  - Detailed specifications
  - Examples
  - Related documents

### Roadmap Documents
- Purpose: Outline feature development plans
- Required sections:
  - Overview
  - Current status
  - Target state
  - Implementation steps
  - Progress tracking
  - Success criteria

## Migration Strategy

The documentation migration will proceed in the following phases:

1. **Create new directory structure**
   - Create all directories as outlined above
   - Create basic README.md files for each directory

2. **Migrate core documents**
   - Move installation and setup guides to getting-started
   - Move contribution guidelines to contributing
   - Move project structure and architecture documents to reference/architecture

3. **Reorganize guides**
   - Sort existing guides into development, features, and deployment categories
   - Standardize guide formats
   - Update cross-references

4. **Restructure roadmaps**
   - Separate active and completed roadmaps
   - Move implementation guides to implementation directory
   - Ensure consistent roadmap format

5. **Update cross-references**
   - Update all links to reflect new structure
   - Add related documents sections
   - Create breadcrumb navigation

6. **Add missing documents**
   - Create any missing index files
   - Add glossary and FAQ
   - Develop troubleshooting guide

## Navigation System

To improve navigation throughout the documentation:

1. **Breadcrumb Navigation**
   - Add breadcrumb path at top of each document
   - Format: Main Category > Subcategory > Document

2. **Related Documents**
   - Add related documents section to each file
   - Include direct links to related content

3. **Document Tags**
   - Add tags to document metadata
   - Create tag index for discovery

4. **Main Documentation Index**
   - Comprehensive index at docs/README.md
   - Multiple navigation paths
   - Quick links to most-used documents 