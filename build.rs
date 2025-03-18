use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/openapi/generate-api.sh");
    println!("cargo:rerun-if-changed=src/openapi/petstore-swagger.yaml");
    println!("cargo:rerun-if-changed=src/openapi/petstore-config.yaml");

    // We're no longer using utoipa annotations, so we don't need to process model files
    println!("cargo:warning=Skipping API model processing - no longer using utoipa annotations");
}
