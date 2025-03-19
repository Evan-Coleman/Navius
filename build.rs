use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/openapi/generate-api.sh");
    println!("cargo:rerun-if-changed=src/openapi/petstore-swagger.yaml");
    println!("cargo:rerun-if-changed=src/openapi/petstore-config.yaml");

    // No special processing needed now that we've removed utoipa
}
