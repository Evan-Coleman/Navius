---
title: "Generated Code Standards"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

---
title: "Generated Code Management"
description: "# Add a new API client"
category: reference
tags:
  - api
  - documentation
last_updated: March 27, 2025
version: 1.0
---
# Generated Code Management

This document outlines the guidelines and processes for managing generated code in the Navius project.

## Overview

Generated code in the Navius project includes:
- API clients generated from OpenAPI specifications
- Generated models and data transfer objects
- Generated documentation

## Generated Code Location

All generated code is stored in the `target/generated` directory, which follows Rust's convention for build artifacts:

```
target/
  └── generated/
      ├── openapi/           # OpenAPI specifications
      ├── petstore_api/      # Generated API clients 
      └── ...                # Other generated code
```

## Generation Process

### API Client Generation

API clients are generated using the OpenAPI Generator. The process is automated via these scripts:

- `.devtools/scripts/add_api.sh` - Adds a new API client
- `.devtools/scripts/regenerate_api.sh` - Regenerates existing API clients

### Usage Example

```bash
# Add a new API client
.devtools/scripts/add_api.sh --name petstore --url https://petstore.swagger.io/v2/swagger.json

# Regenerate an existing API client
.devtools/scripts/regenerate_api.sh petstore
```

## Migration from Legacy Structure

Previously, generated code was stored in the `/generated` directory at the project root. To migrate existing code to the new location, run:

```bash
.devtools/scripts/migrate_generated.sh
```

This script will:
1. Copy all content from `/generated` to `/target/generated`
2. Update import references in source files
3. Leave the original directory untouched for backward compatibility

After verifying that everything works, you can delete the old directory:

```bash
rm -rf generated
```

## Configuration

API client generation is configured in `config/api_registry.json`. This file contains:

- API specifications and URLs
- Models to include or exclude
- Other generation options

## Best Practices

### When to Generate Code

Generate code when:
- Integrating with external APIs that provide OpenAPI specifications
- Working with complex data models that are tedious to manually implement
- The code follows a standardized pattern that is easy to templatize

### When Not to Generate Code

Avoid code generation when:
- The resulting code would be difficult to maintain
- The generated code would be heavily customized
- The component requires complex business logic

### Working with Generated Code

1. **Do not modify generated code directly** - Changes will be lost when regenerating
2. **Extend generated code** - Create wrapper classes or extension methods
3. **Regenerate regularly** - Keep generated code in sync with the source specification
4. **Commit specifications, not generated code** - Store specifications in version control, not the generated code

## Version Control

Generated code is intentionally excluded from version control:

- The `target/` directory is already in `.gitignore`
- CI/CD pipelines regenerate code as needed

## Import Guidelines

When importing generated code:

```rust
// Preferred approach - use the generated_apis module
use crate::generated_apis::petstore_api::models::Pet;

// Avoid direct imports from target/generated
// ❌ use crate::target::generated::petstore_api::models::Pet;
``` 

## Related Documents
- [API Standards](api-standards.md) - API design guidelines
- [Error Handling](error-handling.md) - Error handling patterns
