use std::collections::HashMap;

use crate::generated_apis::petstore_api::models::{Category, Tag, Upet};
use crate::models::LoggableResponse;

/// Implementation of LoggableResponse for Upet (Pet) model
impl LoggableResponse for Upet {
    fn log_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        summary.insert("id".to_string(), self.id.to_string());
        summary.insert("name".to_string(), self.name.clone());

        // Truncate status to reasonable length if present
        if let Some(status) = &self.status {
            let truncated = if status.len() > 50 {
                format!("{}...", &status[..47])
            } else {
                status.clone()
            };
            summary.insert("status".to_string(), truncated);
        }

        // Add tag count
        summary.insert("tags_count".to_string(), self.tags.len().to_string());

        // Add category if present
        if let Some(category) = &self.category {
            summary.insert("category".to_string(), category.name.clone());
        }

        summary
    }
}
