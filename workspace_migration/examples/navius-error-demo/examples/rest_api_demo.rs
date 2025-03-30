// Example REST API error handling to demonstrate the Navius error system
//
// This example shows how the Navius error system can be used to create
// consistent error responses in a REST API context.

use navius_error_demo::{Error, ErrorCode, Result, ResultExt, demo};
use std::env;
use std::io;
use std::path::Path;

// Simulate a REST API handler for user registration
fn register_user(username: &str, password: &str) -> Result<String> {
    // Validate input
    if username.is_empty() {
        return Err(Error::validation("Username cannot be empty"));
    }
    if password.is_empty() {
        return Err(Error::validation("Password cannot be empty"));
    }
    if username.len() < 3 {
        return Err(
            Error::validation("Username must be at least 3 characters long").with_details(
                serde_json::json!({
                    "field": "username",
                    "min_length": 3,
                    "actual_length": username.len()
                }),
            ),
        );
    }
    if password.len() < 8 {
        return Err(
            Error::validation("Password must be at least 8 characters long").with_details(
                serde_json::json!({
                    "field": "password",
                    "min_length": 8,
                    "actual_length": password.len()
                }),
            ),
        );
    }

    // Simulate a DB operation to check if username exists
    if username == "admin" {
        return Err(
            Error::conflict("Username already exists").with_details(serde_json::json!({
                "field": "username",
                "value": username
            })),
        );
    }

    // Simulate a successful registration
    let user_id = "user_123456";
    Ok(format!("User registered successfully with ID: {}", user_id))
}

// Simulate a REST API handler for retrieving a user profile
fn get_user_profile(user_id: &str) -> Result<String> {
    // Validate input
    if user_id.is_empty() {
        return Err(Error::validation("User ID cannot be empty"));
    }

    // Simulate a DB lookup
    if user_id != "user_123456" {
        return Err(Error::not_found(&format!(
            "User with ID {} not found",
            user_id
        )));
    }

    // Simulate an IO error while reading profile data
    let config_path = Path::new("/non/existent/config.json");
    let config_content = std::fs::read_to_string(config_path).not_found(&format!(
        "User profile configuration not found at {}",
        config_path.display()
    ))?;

    // This won't be reached due to the IO error
    Ok(format!("User profile: {}", config_content))
}

// Simulate a REST API handler for accessing protected resources
fn access_protected_resource(user_id: &str, resource_id: &str) -> Result<String> {
    // Validate input
    if user_id.is_empty() || resource_id.is_empty() {
        return Err(Error::validation("User ID and Resource ID are required"));
    }

    // Check authentication (simulated)
    if user_id != "user_123456" {
        return Err(Error::authentication("Invalid user credentials"));
    }

    // Check authorization (simulated)
    if resource_id.starts_with("admin_") {
        return Err(
            Error::authorization("Not authorized to access admin resources").with_details(
                serde_json::json!({
                    "resource_id": resource_id,
                    "required_role": "admin"
                }),
            ),
        );
    }

    // Simulate successful access
    Ok(format!("Resource {} accessed successfully", resource_id))
}

// Helper function to format errors as API responses
fn format_error_response(err: Error) -> String {
    format!(
        "HTTP Status: {}\n{}",
        err.code.status_code(),
        serde_json::to_string_pretty(&err.to_json()).unwrap()
    )
}

fn main() {
    // Example 1: Validation errors during user registration
    println!("Example 1: Validation Error during Registration");
    println!("-----------------------------------------------");
    match register_user("ab", "pass") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
    println!();

    // Example 2: Conflict error during user registration
    println!("Example 2: Conflict Error - Username already exists");
    println!("--------------------------------------------------");
    match register_user("admin", "password123") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
    println!();

    // Example 3: Not Found error
    println!("Example 3: Not Found Error - User doesn't exist");
    println!("-----------------------------------------------");
    match get_user_profile("non_existent_user") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
    println!();

    // Example 4: IO Error with context
    println!("Example 4: IO Error with Context - Config file not found");
    println!("-------------------------------------------------------");
    match get_user_profile("user_123456") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
    println!();

    // Example 5: Authentication Error
    println!("Example 5: Authentication Error - Invalid user");
    println!("--------------------------------------------");
    match access_protected_resource("invalid_user", "resource_1") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
    println!();

    // Example 6: Authorization Error
    println!("Example 6: Authorization Error - Access denied to admin resource");
    println!("------------------------------------------------------------");
    match access_protected_resource("user_123456", "admin_resource") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
    println!();

    // Example 7: Successful operation
    println!("Example 7: Success - Accessing regular resource");
    println!("--------------------------------------------");
    match access_protected_resource("user_123456", "resource_1") {
        Ok(result) => println!("Success: {}", result),
        Err(err) => println!("{}", format_error_response(err)),
    }
}
