//! Utility functions and helpers for the Chimera platform
//!
//! Common utilities for configuration, validation, and general operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod config;
pub mod validation;
pub mod metrics;

pub use config::*;
pub use validation::*;
pub use metrics::*;

pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub async fn load_json_file<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(
    path: P,
) -> Result<T, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(path).await?;
    let data: T = serde_json::from_str(&content)?;
    Ok(data)
}

pub async fn save_json_file<T: Serialize, P: AsRef<Path>>(
    data: &T,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(data)?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

pub fn merge_json(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    match (a, b) {
        (serde_json::Value::Object(mut a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                a.insert(k, v);
            }
            serde_json::Value::Object(a)
        }
        (a, _) => a,
    }
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn calculate_percentage(current: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        (current / total) * 100.0
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();

        assert_ne!(id1, id2);
        assert!(id1.len() > 0);
        assert!(id2.len() > 0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(5)), "5s");
        assert_eq!(format_duration(std::time::Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(std::time::Duration::from_secs(3665)), "1h 1m 5s");
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(50.0, 100.0), 50.0);
        assert_eq!(calculate_percentage(0.0, 100.0), 0.0);
        assert_eq!(calculate_percentage(100.0, 0.0), 0.0);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 5), "hi");
    }
}