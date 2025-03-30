# Navius Framework Development Tools

This directory contains various tools to support the development and maintenance of the Navius framework.

## API Inventory Tool

The API Inventory tool extracts and catalogs public APIs from Rust crates to assist with the API review process. It generates a structured report of all public items in the specified crates.

### Building

To build the API Inventory tool:

```bash
cd workspace_migration/tools
cargo build --bin api_inventory
```

### Usage

```bash
cargo run --bin api_inventory -- [OPTIONS] [CRATES...]
```

#### Options

- `--output=<path>`: Output file path (default: `api_inventory.md`)
- `--format=<format>`: Output format (md, json, csv) (default: md)
- `--workspace`: Process all workspace crates
- `--with-docs`: Include documentation text in the report
- `--help`: Show this help message

#### Examples

Process all crates in the workspace:

```bash
cargo run --bin api_inventory -- --workspace
```

Process specific crates:

```bash
cargo run --bin api_inventory -- ../examples/crates/navius-core ../examples/crates/navius-di
```

Generate JSON output:

```bash
cargo run --bin api_inventory -- --workspace --format=json --output=api_inventory.json
```

### Output

The tool generates a report in the specified format containing:

1. A summary of all crates and their public API items
2. A detailed inventory of all public structs, enums, traits, functions, etc.
3. Documentation status for each item
4. Source file locations for each item

### Integration with API Review Process

This tool is designed to support the [API Review process](../docs/api-review-guidelines.md) by:

1. Creating a complete inventory of public APIs as the first step of the review process
2. Identifying documentation gaps (items without documentation)
3. Providing a structure for systematically reviewing all public APIs

The markdown output is designed to be used as a tracking document during the API review process, with reviewers adding comments and status updates as they work through the inventory.

## Future Tools

Additional development tools planned for this directory include:

- API Change Analyzer: Compare API versions to identify breaking changes
- Documentation Completeness Checker: Verify documentation coverage and quality
- API Usage Explorer: Identify how APIs are used across the codebase

## Contributing

To contribute to these tools, please follow the standard Navius contribution process outlined in the root CONTRIBUTING.md file.

*Updated: March 29, 2025* 