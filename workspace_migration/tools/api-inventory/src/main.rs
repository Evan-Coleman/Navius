use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// API Inventory Tool for Navius Framework
///
/// Creates an inventory of public API elements across the Navius crates
/// to support the API Review process.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the workspace root
    #[clap(long, default_value = ".")]
    workspace_root: String,

    /// Output directory for generated reports
    #[clap(long, default_value = "./api-review")]
    output_dir: String,

    /// Output format (markdown, json, or both)
    #[clap(long, default_value = "markdown")]
    format: String,

    /// Include private items in the inventory (marked as internal)
    #[clap(long)]
    include_private: bool,

    /// Comma-separated list of crates to exclude
    #[clap(long)]
    exclude_crates: Option<String>,

    /// Only analyze specified comma-separated list of crates
    #[clap(long)]
    focus_crates: Option<String>,

    /// Minimum acceptable documentation coverage percentage
    #[clap(long, default_value = "80")]
    doc_threshold: u8,
}

/// Represents a crate in the workspace
#[derive(Debug, Serialize, Deserialize)]
struct CrateInfo {
    name: String,
    version: String,
    path: PathBuf,
    public_items: Vec<ApiItem>,
    doc_coverage: f32,
    status: CrateStatus,
    dependencies: Vec<String>,
}

/// Status of a crate based on documentation coverage
#[derive(Debug, Serialize, Deserialize)]
enum CrateStatus {
    Good,
    Warning,
    Critical,
}

/// Types of API items
#[derive(Debug, Serialize, Deserialize)]
enum ApiItemType {
    Struct,
    Enum,
    Trait,
    Function,
    Macro,
    Constant,
    TypeAlias,
}

/// Documentation status for an API item
#[derive(Debug, Serialize, Deserialize)]
enum DocStatus {
    Complete,
    Partial,
    Missing,
}

/// Represents a single API item
#[derive(Debug, Serialize, Deserialize)]
struct ApiItem {
    name: String,
    item_type: ApiItemType,
    file_path: PathBuf,
    line_number: usize,
    doc_status: DocStatus,
    visibility: String,
    methods: Option<Vec<ApiItem>>,
    variants: Option<Vec<ApiItem>>,
    is_critical: bool,
}

/// Complete API inventory
#[derive(Debug, Serialize, Deserialize)]
struct ApiInventory {
    framework_version: String,
    tool_version: String,
    generated_date: String,
    crates: Vec<CrateInfo>,
    total_items: usize,
    doc_coverage: f32,
    type_breakdown: HashMap<String, usize>,
    doc_status_breakdown: HashMap<String, usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Navius API Inventory Tool");
    println!("-------------------------");
    println!("Analyzing workspace at: {}", args.workspace_root);

    // Create output directory if it doesn't exist
    let output_dir = Path::new(&args.output_dir);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Get the list of crates to analyze
    let crates = find_crates(&args)?;
    println!("Found {} crates to analyze", crates.len());

    // Analyze each crate
    let mut api_inventory = create_empty_inventory();
    for crate_path in crates {
        println!("Analyzing crate at: {}", crate_path.display());
        let crate_info = analyze_crate(&crate_path, &args)?;
        api_inventory.crates.push(crate_info);
    }

    // Calculate summary statistics
    calculate_inventory_stats(&mut api_inventory);

    // Generate reports
    match args.format.as_str() {
        "markdown" => generate_markdown_reports(&api_inventory, output_dir)?,
        "json" => generate_json_report(&api_inventory, output_dir)?,
        "both" => {
            generate_markdown_reports(&api_inventory, output_dir)?;
            generate_json_report(&api_inventory, output_dir)?;
        }
        _ => return Err("Invalid format. Use 'markdown', 'json', or 'both'".into()),
    }

    println!("Inventory generation complete!");
    println!("Reports generated in: {}", output_dir.display());
    println!("Total public items: {}", api_inventory.total_items);
    println!("Documentation coverage: {:.1}%", api_inventory.doc_coverage);

    Ok(())
}

/// Find all crates in the workspace
fn find_crates(args: &Args) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let workspace_root = Path::new(&args.workspace_root);
    let mut crates = Vec::new();

    // Process focus_crates if specified
    if let Some(focus) = &args.focus_crates {
        let focused: Vec<&str> = focus.split(',').map(|s| s.trim()).collect();
        for crate_name in focused {
            // Look for this crate in common locations
            let potential_paths = [
                workspace_root.join(crate_name),
                workspace_root.join("crates").join(crate_name),
                workspace_root
                    .join("workspace_migration")
                    .join("examples")
                    .join("crates")
                    .join(crate_name),
            ];

            for path in potential_paths {
                if path.join("Cargo.toml").exists() {
                    crates.push(path);
                    break;
                }
            }
        }
        return Ok(crates);
    }

    // Otherwise, find all crates
    let excluded: Vec<String> = if let Some(exclude) = &args.exclude_crates {
        exclude.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };

    // Look for crates in known subdirectories
    let search_paths = [
        workspace_root
            .join("workspace_migration")
            .join("examples")
            .join("crates"),
        workspace_root.join("crates"),
    ];

    for search_path in &search_paths {
        if search_path.exists() {
            for entry in fs::read_dir(search_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.join("Cargo.toml").exists() {
                    let crate_name = path.file_name().unwrap().to_string_lossy().to_string();
                    if !excluded.contains(&crate_name) {
                        crates.push(path);
                    }
                }
            }
        }
    }

    println!("Found crates: {:?}", crates);
    Ok(crates)
}

/// Analyze a single crate
fn analyze_crate(crate_path: &Path, args: &Args) -> Result<CrateInfo, Box<dyn std::error::Error>> {
    // Extract crate name and version from Cargo.toml
    let cargo_toml_path = crate_path.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)?;
    let (name, version) = extract_crate_info(&cargo_toml_content)?;

    // Find all source files
    let src_path = crate_path.join("src");
    let mut source_files = Vec::new();

    for entry in WalkDir::new(&src_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "rs")
        })
    {
        source_files.push(entry.path().to_path_buf());
    }

    println!(
        "Found {} source files in crate {}",
        source_files.len(),
        name
    );

    // Extract API items
    let mut public_items = Vec::new();
    for file_path in &source_files {
        let file_content = fs::read_to_string(file_path)?;
        let file_items = extract_api_items(file_path, &file_content, args.include_private)?;
        public_items.extend(file_items);
    }

    // Calculate documentation coverage
    let doc_coverage = calculate_doc_coverage(&public_items);

    // Determine crate status based on coverage
    let status = if doc_coverage >= args.doc_threshold as f32 {
        CrateStatus::Good
    } else if doc_coverage >= args.doc_threshold as f32 * 0.8 {
        CrateStatus::Warning
    } else {
        CrateStatus::Critical
    };

    // Extract dependencies
    let dependencies = extract_dependencies(&cargo_toml_content)?;

    Ok(CrateInfo {
        name,
        version,
        path: crate_path.to_path_buf(),
        public_items,
        doc_coverage,
        status,
        dependencies,
    })
}

/// Extract crate name and version from Cargo.toml
fn extract_crate_info(cargo_toml: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Simple parsing for demonstration purposes
    // In a real implementation, use toml crate to parse properly
    let mut name = String::new();
    let mut version = String::new();

    for line in cargo_toml.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            name = line
                .split('=')
                .nth(1)
                .map_or("", |s| s.trim())
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("version") {
            version = line
                .split('=')
                .nth(1)
                .map_or("", |s| s.trim())
                .trim_matches('"')
                .to_string();
        }
    }

    if name.is_empty() || version.is_empty() {
        return Err("Could not extract crate name or version".into());
    }

    Ok((name, version))
}

/// Extract dependencies from Cargo.toml
fn extract_dependencies(cargo_toml: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut dependencies = Vec::new();
    let mut in_dependencies_section = false;

    for line in cargo_toml.lines() {
        let line = line.trim();

        if line == "[dependencies]" {
            in_dependencies_section = true;
            continue;
        } else if line.starts_with('[') && line != "[dependencies]" {
            in_dependencies_section = false;
        }

        if in_dependencies_section && !line.is_empty() && !line.starts_with('#') {
            let dep = line.split('=').next().map_or("", |s| s.trim()).to_string();
            if !dep.is_empty() {
                dependencies.push(dep);
            }
        }
    }

    Ok(dependencies)
}

/// Extract API items from a source file
fn extract_api_items(
    file_path: &Path,
    content: &str,
    include_private: bool,
) -> Result<Vec<ApiItem>, Box<dyn std::error::Error>> {
    // In a real implementation, this would use syntect, syn, or similar to parse Rust code
    // For this example, we'll use a simplistic approach

    let mut items = Vec::new();
    let mut line_number: usize = 0;

    for line in content.lines() {
        line_number += 1;
        let line = line.trim();

        // Very simplistic detection of public items
        if line.starts_with("pub ") {
            if line.contains(" struct ")
                || line.contains(" enum ")
                || line.contains(" trait ")
                || line.contains(" fn ")
            {
                let item_type = if line.contains(" struct ") {
                    ApiItemType::Struct
                } else if line.contains(" enum ") {
                    ApiItemType::Enum
                } else if line.contains(" trait ") {
                    ApiItemType::Trait
                } else if line.contains(" fn ") {
                    ApiItemType::Function
                } else {
                    continue;
                };

                // Extract name (very simplistic)
                let name = line
                    .split_whitespace()
                    .nth(2)
                    .unwrap_or("unknown")
                    .to_string();

                // Check for documentation
                let has_doc = content
                    .lines()
                    .skip(line_number.saturating_sub(5))
                    .take(5)
                    .any(|l| l.trim().starts_with("///") || l.trim().starts_with("/**"));

                let doc_status = if has_doc {
                    // Simple heuristic: if there are at least 3 doc lines, it's complete
                    let doc_lines = content
                        .lines()
                        .skip(line_number.saturating_sub(10))
                        .take(10)
                        .filter(|l| l.trim().starts_with("///") || l.trim().starts_with("/**"))
                        .count();

                    if doc_lines >= 3 {
                        DocStatus::Complete
                    } else {
                        DocStatus::Partial
                    }
                } else {
                    DocStatus::Missing
                };

                items.push(ApiItem {
                    name,
                    item_type,
                    file_path: file_path.to_path_buf(),
                    line_number,
                    doc_status,
                    visibility: "public".to_string(),
                    methods: None,
                    variants: None,
                    is_critical: false, // Would need more analysis to determine
                });
            }
        }
    }

    Ok(items)
}

/// Calculate documentation coverage for a list of API items
fn calculate_doc_coverage(items: &[ApiItem]) -> f32 {
    if items.is_empty() {
        return 100.0;
    }

    let total = items.len();
    let documented = items
        .iter()
        .filter(|item| matches!(item.doc_status, DocStatus::Complete | DocStatus::Partial))
        .count();

    (documented as f32 / total as f32) * 100.0
}

/// Create an empty API inventory
fn create_empty_inventory() -> ApiInventory {
    ApiInventory {
        framework_version: "0.4.0-dev".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        generated_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        crates: Vec::new(),
        total_items: 0,
        doc_coverage: 0.0,
        type_breakdown: HashMap::new(),
        doc_status_breakdown: HashMap::new(),
    }
}

/// Calculate summary statistics for the API inventory
fn calculate_inventory_stats(inventory: &mut ApiInventory) {
    let mut total_items = 0;
    let mut total_documented = 0;
    let mut type_breakdown: HashMap<String, usize> = HashMap::new();
    let mut doc_status_breakdown: HashMap<String, usize> = HashMap::new();

    // Initialize counters
    doc_status_breakdown.insert("Complete".to_string(), 0);
    doc_status_breakdown.insert("Partial".to_string(), 0);
    doc_status_breakdown.insert("Missing".to_string(), 0);

    type_breakdown.insert("Structs".to_string(), 0);
    type_breakdown.insert("Enums".to_string(), 0);
    type_breakdown.insert("Traits".to_string(), 0);
    type_breakdown.insert("Functions".to_string(), 0);
    type_breakdown.insert("Other".to_string(), 0);

    for crate_info in &inventory.crates {
        total_items += crate_info.public_items.len();

        for item in &crate_info.public_items {
            // Count by documentation status
            match item.doc_status {
                DocStatus::Complete => {
                    *doc_status_breakdown
                        .entry("Complete".to_string())
                        .or_insert(0) += 1;
                    total_documented += 1;
                }
                DocStatus::Partial => {
                    *doc_status_breakdown
                        .entry("Partial".to_string())
                        .or_insert(0) += 1;
                    total_documented += 1;
                }
                DocStatus::Missing => {
                    *doc_status_breakdown
                        .entry("Missing".to_string())
                        .or_insert(0) += 1;
                }
            }

            // Count by type
            match item.item_type {
                ApiItemType::Struct => {
                    *type_breakdown.entry("Structs".to_string()).or_insert(0) += 1;
                }
                ApiItemType::Enum => {
                    *type_breakdown.entry("Enums".to_string()).or_insert(0) += 1;
                }
                ApiItemType::Trait => {
                    *type_breakdown.entry("Traits".to_string()).or_insert(0) += 1;
                }
                ApiItemType::Function => {
                    *type_breakdown.entry("Functions".to_string()).or_insert(0) += 1;
                }
                _ => {
                    *type_breakdown.entry("Other".to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    // Calculate overall documentation coverage
    let doc_coverage = if total_items > 0 {
        (total_documented as f32 / total_items as f32) * 100.0
    } else {
        100.0
    };

    inventory.total_items = total_items;
    inventory.doc_coverage = doc_coverage;
    inventory.type_breakdown = type_breakdown;
    inventory.doc_status_breakdown = doc_status_breakdown;
}

/// Generate Markdown reports
fn generate_markdown_reports(
    inventory: &ApiInventory,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate summary report
    let summary_path = output_dir.join("api-inventory-summary.md");
    let mut summary_file = File::create(summary_path)?;
    write_summary_markdown(inventory, &mut summary_file)?;

    // Generate per-crate reports
    for crate_info in &inventory.crates {
        let crate_path = output_dir.join(format!("{}-api.md", crate_info.name));
        let mut crate_file = File::create(crate_path)?;
        write_crate_markdown(crate_info, &mut crate_file)?;
    }

    // Generate documentation gaps report
    let gaps_path = output_dir.join("documentation-gaps.md");
    let mut gaps_file = File::create(gaps_path)?;
    write_gaps_markdown(inventory, &mut gaps_file)?;

    Ok(())
}

/// Write summary Markdown report
fn write_summary_markdown(
    inventory: &ApiInventory,
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "# Navius API Inventory Summary")?;
    writeln!(file)?;
    writeln!(file, "**Generated:** {}  ", inventory.generated_date)?;
    writeln!(
        file,
        "**Framework Version:** {}  ",
        inventory.framework_version
    )?;
    writeln!(file, "**Tool Version:** {}", inventory.tool_version)?;
    writeln!(file)?;
    writeln!(file, "## Overview")?;
    writeln!(file)?;
    writeln!(file, "- **Total Crates:** {}", inventory.crates.len())?;
    writeln!(file, "- **Total Public Items:** {}", inventory.total_items)?;
    writeln!(
        file,
        "- **Documentation Coverage:** {:.0}%",
        inventory.doc_coverage
    )?;

    // Write type breakdown
    let structs = inventory.type_breakdown.get("Structs").unwrap_or(&0);
    let enums = inventory.type_breakdown.get("Enums").unwrap_or(&0);
    let traits = inventory.type_breakdown.get("Traits").unwrap_or(&0);
    let functions = inventory.type_breakdown.get("Functions").unwrap_or(&0);

    writeln!(file, "- **Public Types:** {}", structs + enums)?;
    writeln!(file, "- **Public Functions:** {}", functions)?;
    writeln!(file, "- **Public Traits:** {}", traits)?;

    // Write documentation status
    writeln!(file)?;
    writeln!(file, "## Documentation Status")?;
    writeln!(file)?;
    writeln!(file, "| Status | Count | Percentage |")?;
    writeln!(file, "|--------|-------|------------|")?;

    let complete = inventory.doc_status_breakdown.get("Complete").unwrap_or(&0);
    let partial = inventory.doc_status_breakdown.get("Partial").unwrap_or(&0);
    let missing = inventory.doc_status_breakdown.get("Missing").unwrap_or(&0);

    let complete_pct = if inventory.total_items > 0 {
        (*complete as f32 / inventory.total_items as f32) * 100.0
    } else {
        0.0
    };

    let partial_pct = if inventory.total_items > 0 {
        (*partial as f32 / inventory.total_items as f32) * 100.0
    } else {
        0.0
    };

    let missing_pct = if inventory.total_items > 0 {
        (*missing as f32 / inventory.total_items as f32) * 100.0
    } else {
        0.0
    };

    writeln!(file, "| Complete | {} | {:.0}% |", complete, complete_pct)?;
    writeln!(file, "| Partial | {} | {:.0}% |", partial, partial_pct)?;
    writeln!(file, "| Missing | {} | {:.0}% |", missing, missing_pct)?;

    // Write crate summary
    writeln!(file)?;
    writeln!(file, "## Crate Summary")?;
    writeln!(file)?;
    writeln!(file, "| Crate | Public Items | Doc Coverage | Status |")?;
    writeln!(file, "|-------|--------------|--------------|--------|")?;

    for crate_info in &inventory.crates {
        let status_emoji = match crate_info.status {
            CrateStatus::Good => "✅",
            CrateStatus::Warning => "⚠️",
            CrateStatus::Critical => "❌",
        };

        writeln!(
            file,
            "| {} | {} | {:.0}% | {} |",
            crate_info.name,
            crate_info.public_items.len(),
            crate_info.doc_coverage,
            status_emoji
        )?;
    }

    // Add more sections like API breakdown, cross-crate dependencies, etc.

    writeln!(file)?;
    writeln!(file, "---")?;
    writeln!(file)?;
    writeln!(
        file,
        "*For detailed per-crate reports, see the individual crate files in this directory.*"
    )?;

    Ok(())
}

/// Write crate-specific Markdown report
fn write_crate_markdown(
    crate_info: &CrateInfo,
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "# {} API Inventory", crate_info.name)?;
    writeln!(file)?;
    writeln!(file, "**Version:** {}  ", crate_info.version)?;
    writeln!(
        file,
        "**Documentation Coverage:** {:.0}%  ",
        crate_info.doc_coverage
    )?;
    writeln!(
        file,
        "**Status:** {}",
        match crate_info.status {
            CrateStatus::Good => "✅ Good",
            CrateStatus::Warning => "⚠️ Warning",
            CrateStatus::Critical => "❌ Critical",
        }
    )?;
    writeln!(file)?;

    writeln!(file, "## Dependencies")?;
    writeln!(file)?;

    if crate_info.dependencies.is_empty() {
        writeln!(file, "No dependencies.")?;
    } else {
        writeln!(file, "- {}", crate_info.dependencies.join("\n- "))?;
    }
    writeln!(file)?;

    // Group items by type
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();
    let mut functions = Vec::new();
    let mut others = Vec::new();

    for item in &crate_info.public_items {
        match item.item_type {
            ApiItemType::Struct => structs.push(item),
            ApiItemType::Enum => enums.push(item),
            ApiItemType::Trait => traits.push(item),
            ApiItemType::Function => functions.push(item),
            _ => others.push(item),
        }
    }

    // Write each type section
    if !structs.is_empty() {
        writeln!(file, "## Public Structs")?;
        writeln!(file)?;
        write_items_table(file, &structs)?;
    }

    if !enums.is_empty() {
        writeln!(file, "## Public Enums")?;
        writeln!(file)?;
        write_items_table(file, &enums)?;
    }

    if !traits.is_empty() {
        writeln!(file, "## Public Traits")?;
        writeln!(file)?;
        write_items_table(file, &traits)?;
    }

    if !functions.is_empty() {
        writeln!(file, "## Public Functions")?;
        writeln!(file)?;
        write_items_table(file, &functions)?;
    }

    if !others.is_empty() {
        writeln!(file, "## Other Public Items")?;
        writeln!(file)?;
        write_items_table(file, &others)?;
    }

    Ok(())
}

/// Write items in a table format
fn write_items_table(
    file: &mut File,
    items: &[&ApiItem],
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "| Name | File | Line | Documentation |")?;
    writeln!(file, "|------|------|------|---------------|")?;

    for item in items {
        let doc_status = match item.doc_status {
            DocStatus::Complete => "✅ Complete",
            DocStatus::Partial => "⚠️ Partial",
            DocStatus::Missing => "❌ Missing",
        };

        let file_path = item
            .file_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        writeln!(
            file,
            "| {} | {} | {} | {} |",
            item.name, file_path, item.line_number, doc_status
        )?;
    }

    writeln!(file)?;
    Ok(())
}

/// Write documentation gaps report
fn write_gaps_markdown(
    inventory: &ApiInventory,
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(file, "# Documentation Gaps")?;
    writeln!(file)?;

    let mut missing_docs = Vec::new();
    let mut partial_docs = Vec::new();

    // Collect all items with missing or partial documentation
    for crate_info in &inventory.crates {
        for item in &crate_info.public_items {
            match item.doc_status {
                DocStatus::Missing => missing_docs.push((crate_info, item)),
                DocStatus::Partial => partial_docs.push((crate_info, item)),
                _ => {}
            }
        }
    }

    // Write items with missing documentation
    if !missing_docs.is_empty() {
        writeln!(
            file,
            "## Missing Documentation ({} items)",
            missing_docs.len()
        )?;
        writeln!(file)?;
        writeln!(file, "| Crate | Item | Type | File | Line |")?;
        writeln!(file, "|-------|------|------|------|------|")?;

        for (crate_info, item) in &missing_docs {
            let item_type = match item.item_type {
                ApiItemType::Struct => "Struct",
                ApiItemType::Enum => "Enum",
                ApiItemType::Trait => "Trait",
                ApiItemType::Function => "Function",
                ApiItemType::Macro => "Macro",
                ApiItemType::Constant => "Constant",
                ApiItemType::TypeAlias => "Type Alias",
            };

            let file_path = item
                .file_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            writeln!(
                file,
                "| {} | {} | {} | {} | {} |",
                crate_info.name, item.name, item_type, file_path, item.line_number
            )?;
        }
        writeln!(file)?;
    } else {
        writeln!(file, "## Missing Documentation")?;
        writeln!(file)?;
        writeln!(
            file,
            "No items with missing documentation were found. Great job! 🎉"
        )?;
        writeln!(file)?;
    }

    // Write items with partial documentation
    if !partial_docs.is_empty() {
        writeln!(
            file,
            "## Partial Documentation ({} items)",
            partial_docs.len()
        )?;
        writeln!(file)?;

        if partial_docs.len() > 100 {
            writeln!(
                file,
                "Showing the first 100 of {} items with partial documentation.",
                partial_docs.len()
            )?;
            partial_docs = partial_docs.into_iter().take(100).collect();
        }

        writeln!(file, "| Crate | Item | Type | File | Line |")?;
        writeln!(file, "|-------|------|------|------|------|")?;

        for (crate_info, item) in &partial_docs {
            let item_type = match item.item_type {
                ApiItemType::Struct => "Struct",
                ApiItemType::Enum => "Enum",
                ApiItemType::Trait => "Trait",
                ApiItemType::Function => "Function",
                ApiItemType::Macro => "Macro",
                ApiItemType::Constant => "Constant",
                ApiItemType::TypeAlias => "Type Alias",
            };

            let file_path = item
                .file_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            writeln!(
                file,
                "| {} | {} | {} | {} | {} |",
                crate_info.name, item.name, item_type, file_path, item.line_number
            )?;
        }
        writeln!(file)?;
    } else {
        writeln!(file, "## Partial Documentation")?;
        writeln!(file)?;
        writeln!(
            file,
            "No items with partial documentation were found. Great job! 🎉"
        )?;
        writeln!(file)?;
    }

    Ok(())
}

/// Generate JSON report
fn generate_json_report(
    inventory: &ApiInventory,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_path = output_dir.join("api-inventory.json");
    let json_string = serde_json::to_string_pretty(inventory)?;
    let mut json_file = File::create(json_path)?;
    writeln!(json_file, "{}", json_string)?;
    Ok(())
}

/// Extracts sections of code around API items for examples
fn extract_code_sample(file_path: &Path, line: usize) -> Option<String> {
    if !file_path.exists() {
        return None;
    }

    match fs::read_to_string(file_path) {
        Ok(contents) => {
            let lines: Vec<&str> = contents.lines().collect();
            let line_number: usize = line.saturating_sub(1); // 0-based indexing

            // Calculate boundaries to show context
            let start = line_number.saturating_sub(5);
            let end = std::cmp::min(line_number + 5, lines.len());

            // Generate the code sample with line numbers
            let mut sample = String::new();
            sample.push_str(&format!("```rust\n"));

            for (i, line) in lines[start..end].iter().enumerate() {
                let current_line = start + i + 1;
                let prefix = if current_line == line_number + 1 {
                    "➤ "
                } else {
                    "  "
                };
                sample.push_str(&format!("{}{}: {}\n", prefix, current_line, line));
            }

            sample.push_str("```");
            Some(sample)
        }
        Err(_) => None,
    }
}
