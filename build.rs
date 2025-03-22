use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Watch for changes to API definition files and scripts
    println!("cargo:rerun-if-changed=config/swagger");
    println!("cargo:rerun-if-changed=config/api_registry.json");
    println!("cargo:rerun-if-changed=.devtools/scripts/add_api.sh");
    println!("cargo:rerun-if-changed=.devtools/scripts/regenerate_api.sh");

    // Check if we need to generate API clients
    if should_generate_apis() {
        println!("cargo:warning=API clients need to be generated. Running regenerate_api.sh...");
        generate_apis();
    }
}

/// Checks if any API needs to be generated
fn should_generate_apis() -> bool {
    let api_registry_path = Path::new("config/api_registry.json");

    // If registry doesn't exist, no generation needed
    if !api_registry_path.exists() {
        return false;
    }

    // Read the API registry
    let registry_content = match fs::read_to_string(api_registry_path) {
        Ok(content) => content,
        Err(_) => return false,
    };

    // Parse the API registry
    let registry: serde_json::Value = match serde_json::from_str(&registry_content) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };

    // Check each API in the registry
    if let Some(apis) = registry["apis"].as_array() {
        for api in apis {
            let api_name = match api["name"].as_str() {
                Some(name) => name,
                None => continue,
            };

            // Check if generated directory exists for this API
            let api_dir = format!("target/generated/{}_api", api_name);
            if !Path::new(&api_dir).exists() {
                return true;
            }
        }
    }

    false
}

/// Run API generation script
fn generate_apis() {
    let output = Command::new("./.devtools/scripts/regenerate_api.sh")
        .output()
        .expect("Failed to execute regenerate_api.sh");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=API generation failed: {}", stderr);
    } else {
        println!("cargo:warning=API generation completed successfully");
    }
}
