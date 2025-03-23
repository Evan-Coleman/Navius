# Application Utilities

This directory contains user-defined utility functions and helpers for the Navius application. Use this module to define custom utilities that aren't already provided by the core utils module.

## Usage

To use utilities in your application code:

```rust
use crate::app::utils;
use crate::core::error::Result;

fn example() -> Result<String> {
    // Using a core utility function
    let formatted_date = utils::date::format_iso8601(chrono::Utc::now());
    
    // Using your custom utility function
    let validated_email = utils::validation_utils::validate_email("user@example.com")?;
    
    Ok(format!("User {} registered on {}", validated_email, formatted_date))
}
```

## Creating Custom Utilities

### Example: String Utilities

```rust
// src/app/utils/string_utils.rs
use crate::core::error::{AppError, Result};

/// Truncates a string to the specified maximum length,
/// adding an ellipsis (...) if truncation occurs.
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Make sure we don't cut in the middle of a UTF-8 character
        let mut truncated = String::with_capacity(max_len + 3);
        truncated.push_str(&s[..max_len.saturating_sub(3)]);
        truncated.push_str("...");
        truncated
    }
}

/// Strips all HTML tags from a string.
pub fn strip_html_tags(s: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(s, "").to_string()
}
```

### Example: Validation Utilities

```rust
// src/app/utils/validation_utils.rs
use crate::core::error::{AppError, Result};

/// Validates an email address format.
pub fn validate_email(email: &str) -> Result<String> {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    
    if re.is_match(email) {
        Ok(email.to_string())
    } else {
        Err(AppError::validation_error("Invalid email format"))
    }
}

/// Validates that a password meets complexity requirements.
pub fn validate_password(password: &str) -> Result<String> {
    let min_length = 8;
    let requires_uppercase = true;
    let requires_lowercase = true;
    let requires_digit = true;
    let requires_special = true;
    
    if password.len() < min_length {
        return Err(AppError::validation_error(
            format!("Password must be at least {} characters long", min_length)
        ));
    }
    
    if requires_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::validation_error("Password must contain at least one uppercase letter"));
    }
    
    // Add more validation as needed
    
    Ok(password.to_string())
}
```

## Best Practices

1. Keep utility functions focused on a single task
2. Use proper error handling with `AppError` and `Result` types
3. Document all functions with examples and edge cases
4. Add unit tests for all utility functions
5. Group related utilities in their own modules
6. Don't duplicate functionality that's already in the core utils
7. Follow Rust naming conventions: `snake_case` for functions and modules

## Core Utilities

The core utilities are provided by `crate::core::utils` and include:

- Date and time formatting functions
- API client helpers
- OpenAPI utilities
- Logging utilities

Do not modify the core utilities directly. Instead, use this directory to extend and provide additional utility functions for your specific application needs. 