use std::collections::HashMap;

/// Trait for responses that can generate log summaries
pub trait LoggableResponse {
    /// Returns a summary of the response for logging purposes with all fields
    fn log_summary(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Returns a filtered summary with only the specified fields
    fn filtered_summary(&self, fields: &[String]) -> HashMap<String, String> {
        let full_summary = self.log_summary();

        if fields.is_empty() {
            return full_summary;
        }

        full_summary
            .into_iter()
            .filter(|(key, _)| fields.contains(key))
            .collect()
    }

    /// Returns a preview string for the response
    fn log_preview(&self) -> String {
        self.preview_with_fields(&[])
    }

    /// Returns a preview string with only specified fields
    fn preview_with_fields(&self, fields: &[String]) -> String {
        let summary = if fields.is_empty() {
            self.log_summary()
        } else {
            self.filtered_summary(fields)
        };

        if summary.is_empty() {
            return "No preview available".to_string();
        }

        summary
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
