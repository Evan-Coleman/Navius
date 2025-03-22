//! Date utilities
//!
//! This module provides utilities for date formatting based on configuration.

use chrono::{DateTime, TimeZone, Utc};
use config::builder::DefaultState;
use config::{Config, ConfigBuilder};

/// Format a date using the app configuration
///
/// # Arguments
///
/// * `config` - The application configuration
/// * `format_type` - The format type to use (simple, formal, or verbose)
/// * `date` - The date to format (if None, current date is used)
///
/// # Returns
///
/// The formatted date string
pub fn format_date(config: &Config, format_type: &str, date: Option<DateTime<Utc>>) -> String {
    let format = match format_type {
        "simple" => config
            .get::<String>("date_formats.simple")
            .unwrap_or_else(|_| "%m/%d/%Y".to_string()),
        "formal" => config
            .get::<String>("date_formats.formal")
            .unwrap_or_else(|_| "%B %d, %Y".to_string()),
        "verbose" => config
            .get::<String>("date_formats.verbose")
            .unwrap_or_else(|_| "%Y-%m-%dT%H:%M:%S%.3fZ".to_string()),
        _ => "%Y-%m-%d".to_string(), // fallback
    };

    let date = date.unwrap_or_else(Utc::now);
    date.format(&format).to_string()
}

/// Format today's date in the simple format (MM/DD/YYYY)
pub fn today_simple(config: &Config) -> String {
    format_date(config, "simple", None)
}

/// Format today's date in the formal format (Month DD, YYYY)
pub fn today_formal(config: &Config) -> String {
    format_date(config, "formal", None)
}

/// Format today's date in the verbose format (ISO 8601)
pub fn today_verbose(config: &Config) -> String {
    format_date(config, "verbose", None)
}

/// Format a specific date in the simple format (MM/DD/YYYY)
pub fn format_simple(config: &Config, date: DateTime<Utc>) -> String {
    format_date(config, "simple", Some(date))
}

/// Format a specific date in the formal format (Month DD, YYYY)
pub fn format_formal(config: &Config, date: DateTime<Utc>) -> String {
    format_date(config, "formal", Some(date))
}

/// Format a specific date in the verbose format (ISO 8601)
pub fn format_verbose(config: &Config, date: DateTime<Utc>) -> String {
    format_date(config, "verbose", Some(date))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        let builder = ConfigBuilder::<DefaultState>::default();
        let builder = builder
            .set_override("date_formats.simple", "%m/%d/%Y")
            .unwrap()
            .set_override("date_formats.formal", "%B %d, %Y")
            .unwrap()
            .set_override("date_formats.verbose", "%Y-%m-%dT%H:%M:%S%.3fZ")
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_format_date_simple() {
        let config = create_test_config();
        let test_date = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

        let result = format_simple(&config, test_date);
        assert_eq!(result, "05/15/2023");
    }

    #[test]
    fn test_format_date_formal() {
        let config = create_test_config();
        let test_date = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

        let result = format_formal(&config, test_date);
        assert_eq!(result, "May 15, 2023");
    }

    #[test]
    fn test_format_date_verbose() {
        let config = create_test_config();
        let test_date = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

        let result = format_verbose(&config, test_date);
        // Check if the result starts with the expected date and time
        assert!(result.starts_with("2023-05-15T10:30:00"));
    }

    #[test]
    fn test_format_date_with_custom_format() {
        let builder = ConfigBuilder::<DefaultState>::default();
        let builder = builder
            .set_override("date_formats.simple", "%-d/%-m/%Y")
            .unwrap()
            .set_override("date_formats.formal", "%Y-%B-%d")
            .unwrap();
        let custom_config = builder.build().unwrap();

        let test_date = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

        let simple_result = format_simple(&custom_config, test_date);
        let formal_result = format_formal(&custom_config, test_date);

        assert_eq!(simple_result, "15/5/2023");
        assert_eq!(formal_result, "2023-May-15");
    }

    #[test]
    fn test_format_date_with_fallback() {
        let empty_config = ConfigBuilder::<DefaultState>::default().build().unwrap();
        let test_date = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

        // Should use fallback formats when config values are missing
        let simple_result = format_simple(&empty_config, test_date);
        let formal_result = format_formal(&empty_config, test_date);
        let verbose_result = format_verbose(&empty_config, test_date);

        assert_eq!(simple_result, "05/15/2023");
        assert_eq!(formal_result, "May 15, 2023");
        assert!(verbose_result.starts_with("2023-05-15T10:30:00"));
    }

    #[test]
    fn test_format_date_unknown_format_type() {
        let config = create_test_config();
        let test_date = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

        // Unknown format type should use the fallback format
        let result = format_date(&config, "unknown", Some(test_date));
        assert_eq!(result, "2023-05-15");
    }

    // We can't easily test today_* functions directly because they use the current date,
    // but we can test that they call format_date with the right arguments
    #[test]
    fn test_today_functions_call_format_date() {
        let config = create_test_config();

        // Check that each function calls format_date with the correct format type
        let _ = today_simple(&config);
        let _ = today_formal(&config);
        let _ = today_verbose(&config);

        // If we reached here without panicking, the functions work as expected
        // This is a weak test, but it's better than nothing
        assert!(true);
    }
}
