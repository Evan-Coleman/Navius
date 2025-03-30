# API Inventory Tool

**Version:** 0.1.0  
**Updated:** March 29, 2025

## Overview

The API Inventory Tool is a critical component of the Navius API Review process. It automatically extracts and catalogs all public API elements from the Navius crate ecosystem, generating a structured inventory that serves as the foundation for the API Review phase.

## Features

- **Comprehensive API Discovery**: Automatically identifies all public API items across the Navius workspace
- **Documentation Status Tracking**: Flags items with missing or incomplete documentation
- **Interface Consistency Analysis**: Highlights potentially inconsistent naming or parameter patterns
- **Dependency Mapping**: Shows relationships between crates and their public interfaces
- **Markdown Report Generation**: Creates human-readable reports for review
- **JSON Export**: Enables further processing or visualization of API data

## Installation

The API Inventory Tool is available within the Navius workspace:

```bash
cd workspace_migration/tools/api-inventory
cargo build --release
```

## Usage

### Basic Inventory Generation

```bash
cd workspace_migration/tools/api-inventory
cargo run -- --workspace-root ../../ --output-dir ../../docs/api-review
```

### Command Line Options

- `--workspace-root <PATH>`: Path to the root of the Navius workspace
- `--output-dir <PATH>`: Directory where reports will be generated
- `--format <FORMAT>`: Output format (markdown, json, or both) [default: markdown]
- `--include-private`: Include private items in the inventory (marked as internal)
- `--exclude-crates <CRATES>`: Comma-separated list of crates to exclude
- `--focus-crates <CRATES>`: Only analyze specified comma-separated list of crates
- `--doc-threshold <PERCENT>`: Minimum acceptable documentation coverage percentage [default: 80]

## Output Structure

The tool generates the following files:

- `api-inventory-summary.md`: Overview of all crates and their API status
- `{crate-name}-api.md`: Detailed inventory for each crate
- `documentation-gaps.md`: List of all items with missing documentation
- `api-inventory.json`: Complete data export in JSON format (if JSON format selected)

### Example Report Content

#### API Inventory Summary

```markdown
# Navius API Inventory Summary

Generated: 2025-03-29

## Overview

- Total Crates: 12
- Total Public Items: 342
- Documentation Coverage: 76%
- Public Types: 128
- Public Functions: 189
- Public Traits: 25

## Crate Summary

| Crate | Public Items | Doc Coverage | Status |
|-------|--------------|--------------|--------|
| navius-core | 87 | 92% | ✅ |
| navius-http | 64 | 83% | ✅ |
| navius-db | 51 | 78% | ⚠️ |
| navius-auth | 42 | 66% | ❌ |
| ... | ... | ... | ... |
```

#### Crate-Specific Report

```markdown
# navius-http API Inventory

## Public Structs

- **HttpClient** - `src/client.rs:24`
  - Documentation: Complete
  - Methods: 8
  - Visibility: Public
  
- **RequestBuilder** - `src/request.rs:42`
  - Documentation: Incomplete
  - Methods: 12
  - Visibility: Public
  
## Public Traits

- **HttpHandler** - `src/handler.rs:18`
  - Documentation: Complete
  - Methods: 3
  - Visibility: Public
  
## Public Functions

- **create_default_client()** - `src/client.rs:156`
  - Documentation: Missing
  - Visibility: Public
  
...
```

## Integration with API Review Process

The API Inventory Tool is designed to integrate with the broader API Review process:

1. **Inventory Phase** (April 1-7, 2025): Run the tool to generate the initial API inventory
2. **Design Evaluation Phase** (April 8-21, 2025): Review the inventory reports to evaluate API design
3. **Reporting**: Track progress of documentation and API improvements over time

## Future Tools

Additional tools planned for this directory include:

1. **API Change Analyzer**: Compare API versions to identify breaking changes
2. **Documentation Completeness Checker**: In-depth analysis of documentation quality
3. **API Usage Explorer**: Analyze how public APIs are used within the codebase

## Contributing

To contribute to the API Inventory Tool, please follow the standard Navius contribution process as outlined in the root `CONTRIBUTING.md` file.

---

*Updated: March 29, 2025* 