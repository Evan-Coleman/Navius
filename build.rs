// use std::fs;
// use utoipa_gen::{OpenApi, ToTokens};

// fn main() {
//     // ✅ Load the OpenAPI YAML file
//     let yaml = fs::read_to_string("/openapi/petstore-swagger.yaml")
//         .expect("Failed to read OpenAPI YAML file");

//     // ✅ Parse YAML into OpenAPI structure
//     let openapi: OpenApi = serde_yaml::from_str(&yaml).expect("Invalid OpenAPI YAML format");

//     // ✅ Generate Rust code from OpenAPI
//     let generated_code = openapi.to_token_stream().to_string();

//     // ✅ Write generated Rust code to `src/generated.rs`
//     fs::write("src/generated.rs", generated_code).expect("Failed to write generated Rust code");
// }
