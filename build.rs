use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/openapi/generate-api.sh");
    println!("cargo:rerun-if-changed=src/openapi/petstore-swagger.yaml");
    println!("cargo:rerun-if-changed=src/openapi/petstore-config.yaml");

    // Check if API generation should be skipped
    if env::var("SKIP_API_GEN").is_ok() {
        println!("cargo:warning=Skipping API model processing (SKIP_API_GEN is set)");
        return;
    }

    // Process the generated model files to add Utoipa annotations
    process_model_files().unwrap_or_else(|e| {
        println!("cargo:warning=Failed to process model files: {}", e);
    });
}

fn process_model_files() -> Result<(), Box<dyn std::error::Error>> {
    let models_dir = Path::new("src/petstore_api/models");

    // Ensure the directory exists
    if !models_dir.exists() {
        // Directory doesn't exist yet, silently skip processing
        return Ok(());
    }

    // Process all .rs files in the models directory
    for entry in fs::read_dir(models_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "rs")
            && path.file_name().map_or(false, |name| name != "mod.rs")
        {
            // Skip mod.rs as it's just re-exports
            add_utoipa_annotations(&path)?;
        }
    }

    Ok(())
}

fn add_utoipa_annotations(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file content
    let mut content = String::new();
    let mut file = fs::File::open(file_path)?;
    file.read_to_string(&mut content)?;

    println!("cargo:warning=Processing file: {}", file_path.display());

    // Check if we already processed this file
    if content.contains("#[derive(") && content.contains("ToSchema") {
        println!(
            "cargo:warning=File already has ToSchema: {}",
            file_path.display()
        );
        return Ok(());
    }

    // Add utoipa imports if not present
    if !content.contains("use utoipa::") {
        content = content.replace(
            "use serde::{Deserialize, Serialize};",
            "use serde::{Deserialize, Serialize};\nuse utoipa::ToSchema;",
        );
        println!(
            "cargo:warning=Added ToSchema import to {}",
            file_path.display()
        );
    }

    // Add ToSchema derive to struct definitions - handle various attribute patterns
    let patterns = [
        "#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]",
        "#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]",
        "#[derive(Debug, Serialize, Deserialize)]",
    ];

    let mut updated_content = content.clone();
    for pattern in patterns {
        if content.contains(pattern) {
            println!(
                "cargo:warning=Found pattern in {}: {}",
                file_path.display(),
                pattern
            );
            // Replace )] with , ToSchema)]
            let replacement = pattern.replace(")]", ", ToSchema)]");
            updated_content = updated_content.replace(pattern, &replacement);
        }
    }

    // Only update if we actually made changes
    if updated_content != content {
        let mut file = fs::File::create(file_path)?;
        file.write_all(updated_content.as_bytes())?;
        println!(
            "cargo:warning=Added Utoipa annotations to {}",
            file_path.display()
        );
    } else {
        println!("cargo:warning=No changes made to {}", file_path.display());
    }

    Ok(())
}
