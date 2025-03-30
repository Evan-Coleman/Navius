// API Inventory Tool
//
// This script extracts public API elements from Rust crates to assist with the API review process.
// It generates a structured report of all public items in the specified crates.
//
// Usage:
// cargo run --bin api_inventory -- [OPTIONS] [CRATES...]
//
// Options:
//   --output=<path>     Output file path (default: api_inventory.md)
//   --format=<format>   Output format (md, json, csv) (default: md)
//   --workspace         Process all workspace crates
//   --with-docs         Include documentation text in the report
//   --help              Show this help message

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct ApiItem {
    crate_name: String,
    item_type: String,
    name: String,
    path: String,
    has_docs: bool,
    docs: String,
    source_file: String,
    line_number: u32,
    is_deprecated: bool,
}

struct ApiInventory {
    items: Vec<ApiItem>,
    crates: Vec<String>,
}

impl ApiInventory {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            crates: Vec::new(),
        }
    }

    fn add_item(&mut self, item: ApiItem) {
        if !self.crates.contains(&item.crate_name) {
            self.crates.push(item.crate_name.clone());
        }
        self.items.push(item);
    }

    fn write_markdown(&self, output_path: &Path) -> io::Result<()> {
        let mut file = File::create(output_path)?;

        writeln!(&mut file, "# Navius API Inventory\n")?;
        writeln!(
            &mut file,
            "**Generated:** {}\n",
            chrono::Local::now().format("%B %d, %Y")
        )?;

        writeln!(&mut file, "## Overview\n")?;
        writeln!(
            &mut file,
            "This document contains an inventory of public API elements across {} crates in the Navius framework.\n",
            self.crates.len()
        )?;

        writeln!(
            &mut file,
            "| Crate | Structs | Enums | Traits | Functions | Total Items |"
        )?;
        writeln!(
            &mut file,
            "|-------|---------|-------|--------|-----------|-------------|"
        )?;

        // Create a summary table by crate
        let mut crate_summary = HashMap::new();
        for item in &self.items {
            let entry = crate_summary
                .entry(item.crate_name.clone())
                .or_insert_with(|| HashMap::new());
            let count = entry.entry(item.item_type.clone()).or_insert(0);
            *count += 1;
        }

        for crate_name in &self.crates {
            let summary = crate_summary.get(crate_name).unwrap_or(&HashMap::new());
            let structs = summary.get("struct").unwrap_or(&0);
            let enums = summary.get("enum").unwrap_or(&0);
            let traits = summary.get("trait").unwrap_or(&0);
            let functions = summary.get("fn").unwrap_or(&0);
            let total = self
                .items
                .iter()
                .filter(|i| &i.crate_name == crate_name)
                .count();

            writeln!(
                &mut file,
                "| {} | {} | {} | {} | {} | {} |",
                crate_name, structs, enums, traits, functions, total
            )?;
        }

        writeln!(&mut file, "\n## Detailed Inventory\n")?;

        // Sort crates alphabetically
        let mut sorted_crates = self.crates.clone();
        sorted_crates.sort();

        for crate_name in sorted_crates {
            writeln!(&mut file, "### {}\n", crate_name)?;

            // Group by item type
            let mut item_types = Vec::new();
            for item in &self.items {
                if item.crate_name == crate_name && !item_types.contains(&item.item_type) {
                    item_types.push(item.item_type.clone());
                }
            }

            item_types.sort();

            for item_type in item_types {
                writeln!(&mut file, "#### {}s\n", item_type)?;

                writeln!(&mut file, "| Name | Path | Documentation | Source |")?;
                writeln!(&mut file, "|------|------|---------------|--------|")?;

                let mut items_of_type: Vec<_> = self
                    .items
                    .iter()
                    .filter(|i| i.crate_name == crate_name && i.item_type == item_type)
                    .collect();

                // Sort by name
                items_of_type.sort_by(|a, b| a.name.cmp(&b.name));

                for item in items_of_type {
                    let docs_status = if item.has_docs { "✅" } else { "❌" };
                    let name_display = if item.is_deprecated {
                        format!("~~{}~~ (deprecated)", item.name)
                    } else {
                        item.name.clone()
                    };

                    writeln!(
                        &mut file,
                        "| {} | `{}` | {} | {}:{} |",
                        name_display, item.path, docs_status, item.source_file, item.line_number
                    )?;
                }

                writeln!(&mut file, "")?;
            }
        }

        Ok(())
    }
}

fn process_crate(crate_path: &Path, inventory: &mut ApiInventory) -> io::Result<()> {
    println!("Processing crate at {}", crate_path.display());

    // Get the crate name from Cargo.toml
    let cargo_path = crate_path.join("Cargo.toml");
    let mut cargo_file = File::open(cargo_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    let crate_name = cargo_contents
        .lines()
        .find(|line| line.trim().starts_with("name"))
        .and_then(|line| line.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"'))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Could not find crate name"))?;

    // Run cargo doc to generate documentation
    println!("Generating documentation for {}", crate_name);
    let output = Command::new("cargo")
        .args(&["doc", "--no-deps", "--lib"])
        .current_dir(crate_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to generate documentation",
        ));
    }

    // Scan source files to extract public items
    let src_dir = crate_path.join("src");
    scan_directory(&src_dir, crate_name, inventory)?;

    Ok(())
}

fn scan_directory(dir: &Path, crate_name: &str, inventory: &mut ApiInventory) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_directory(&path, crate_name, inventory)?;
        } else if let Some(ext) = path.extension() {
            if ext == "rs" {
                scan_rust_file(&path, crate_name, inventory)?;
            }
        }
    }

    Ok(())
}

fn scan_rust_file(
    file_path: &Path,
    crate_name: &str,
    inventory: &mut ApiInventory,
) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let relative_path = file_path.strip_prefix("src").unwrap_or(file_path);
    let source_file = relative_path.to_string_lossy().to_string();

    // Very simple approach to finding public items
    // In a complete tool, you would use a proper Rust parser
    for (i, line) in contents.lines().enumerate() {
        let line_number = i as u32 + 1;
        let trimmed = line.trim();

        // Check for public items
        if trimmed.starts_with("pub ") {
            let mut parts = trimmed.split_whitespace();
            parts.next(); // Skip 'pub'

            // Check if this is 'pub(crate)' or similar visibility modifier
            let next_part = parts.next().unwrap_or("");
            if next_part.starts_with('(') {
                continue; // Skip items that aren't fully public
            }

            // Identify the item type (struct, enum, trait, fn, etc.)
            let item_type = if next_part == "struct"
                || next_part == "enum"
                || next_part == "trait"
                || next_part == "fn"
                || next_part == "type"
                || next_part == "mod"
                || next_part == "use"
            {
                next_part
            } else {
                continue; // Unknown or unsupported item type
            };

            // Extract the name
            let rest = parts.collect::<Vec<_>>().join(" ");
            let name = rest
                .split(&['{', ':', '<', '(', ';'][..])
                .next()
                .map(|s| s.trim())
                .unwrap_or("");

            if name.is_empty() {
                continue;
            }

            // Check for documentation
            let has_docs = contents
                .lines()
                .take(i)
                .rev()
                .take_while(|l| l.trim().starts_with("///") || l.trim().is_empty())
                .any(|l| l.trim().starts_with("///"));

            // Check for deprecation
            let is_deprecated = contents
                .lines()
                .take(i)
                .rev()
                .take_while(|l| !l.trim().is_empty() || l.trim().starts_with("#"))
                .any(|l| l.trim().starts_with("#[deprecated"));

            // Create a path
            let module_path = relative_path
                .to_str()
                .unwrap_or("")
                .replace(".rs", "")
                .replace("/", "::")
                .replace("\\", "::");

            let path = if module_path == "lib" || module_path == "main" {
                format!("{}::{}", crate_name, name)
            } else {
                format!("{}::{}::{}", crate_name, module_path, name)
            };

            // Extract docs if present
            let docs = if has_docs {
                contents
                    .lines()
                    .take(i)
                    .rev()
                    .take_while(|l| l.trim().starts_with("///") || l.trim().is_empty())
                    .filter(|l| l.trim().starts_with("///"))
                    .map(|l| l.trim().trim_start_matches("///").trim())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            };

            inventory.add_item(ApiItem {
                crate_name: crate_name.to_string(),
                item_type: item_type.to_string(),
                name: name.to_string(),
                path,
                has_docs,
                docs,
                source_file: source_file.clone(),
                line_number,
                is_deprecated,
            });
        }
    }

    Ok(())
}

fn find_crates(workspace_path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut crates = Vec::new();

    // Check for examples/crates directory
    let crates_dir = workspace_path.join("examples").join("crates");
    if crates_dir.exists() {
        for entry in fs::read_dir(crates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.join("Cargo.toml").exists() {
                crates.push(path);
            }
        }
    }

    crates.sort();
    Ok(crates)
}

fn main() -> io::Result<()> {
    // In a real implementation, you would properly parse the command line arguments
    // For this example, we'll just process all crates in the workspace

    println!("Navius API Inventory Tool");
    println!("------------------------");

    let workspace_path = PathBuf::from(".");
    let crates = find_crates(&workspace_path)?;

    println!("Found {} crates to process", crates.len());

    let mut inventory = ApiInventory::new();

    for crate_path in crates {
        process_crate(&crate_path, &mut inventory)?;
    }

    println!("Extracted {} public API items", inventory.items.len());

    // Create the output directory if it doesn't exist
    let output_dir = workspace_path.join("docs").join("api-review");
    fs::create_dir_all(&output_dir)?;

    // Write the inventory report
    let output_path = output_dir.join("api_inventory.md");
    inventory.write_markdown(&output_path)?;

    println!("Inventory written to {}", output_path.display());

    Ok(())
}
