//! Input validation and sanitization utilities

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub struct InputValidator {
    max_text_length: usize,
    max_json_size: usize,
    allowed_domains: Vec<String>,
}

impl InputValidator {
    pub fn new() -> Self {
        Self {
            max_text_length: 10000,
            max_json_size: 1024 * 1024, // 1MB
            allowed_domains: vec![
                "github.com".to_string(),
                "huggingface.co".to_string(),
                "openai.com".to_string(),
            ],
        }
    }

    pub fn validate_text(&self, text: &str, field_name: &str) -> Result<(), ValidationError> {
        if text.is_empty() {
            return Err(ValidationError {
                field: field_name.to_string(),
                message: "Text cannot be empty".to_string(),
            });
        }

        if text.len() > self.max_text_length {
            return Err(ValidationError {
                field: field_name.to_string(),
                message: format!("Text exceeds maximum length of {}", self.max_text_length),
            });
        }

        // Check for potentially harmful content
        if self.contains_harmful_patterns(text) {
            return Err(ValidationError {
                field: field_name.to_string(),
                message: "Text contains potentially harmful content".to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_json(&self, json_str: &str, field_name: &str) -> Result<serde_json::Value, ValidationError> {
        if json_str.len() > self.max_json_size {
            return Err(ValidationError {
                field: field_name.to_string(),
                message: format!("JSON exceeds maximum size of {}", self.max_json_size),
            });
        }

        match serde_json::from_str(json_str) {
            Ok(value) => Ok(value),
            Err(e) => Err(ValidationError {
                field: field_name.to_string(),
                message: format!("Invalid JSON: {}", e),
            }),
        }
    }

    pub fn validate_url(&self, url: &str, field_name: &str) -> Result<(), ValidationError> {
        match url::Url::parse(url) {
            Ok(parsed) => {
                if let Some(domain) = parsed.domain() {
                    if !self.allowed_domains.contains(&domain.to_string()) {
                        return Err(ValidationError {
                            field: field_name.to_string(),
                            message: format!("Domain {} is not allowed", domain),
                        });
                    }
                }
                Ok(())
            }
            Err(_) => Err(ValidationError {
                field: field_name.to_string(),
                message: "Invalid URL format".to_string(),
            }),
        }
    }

    pub fn sanitize_filename(&self, filename: &str) -> String {
        // Remove or replace dangerous characters
        let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        let mut sanitized = filename.to_string();

        for &ch in &dangerous_chars {
            sanitized = sanitized.replace(ch, "_");
        }

        // Ensure filename is not too long
        if sanitized.len() > 255 {
            sanitized = sanitized[..255].to_string();
        }

        sanitized
    }

    fn contains_harmful_patterns(&self, text: &str) -> bool {
        let harmful_patterns = [
            r"(?i)rm\s+-rf\s+/",
            r"(?i)format\s+c:",
            r"(?i)shutdown\s+",
            r"<script[^>]*>.*?</script>",
            r"javascript:",
            r"vbscript:",
            r"onload\s*=",
            r"onerror\s*=",
        ];

        harmful_patterns.iter().any(|pattern| {
            if let Ok(regex) = Regex::new(pattern) {
                regex.is_match(text)
            } else {
                false
            }
        })
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_request_payload(payload: &serde_json::Value) -> Result<(), Vec<ValidationError>> {
    let validator = InputValidator::new();
    let mut errors = Vec::new();

    if let Some(text) = payload.get("prompt").and_then(|v| v.as_str()) {
        if let Err(e) = validator.validate_text(text, "prompt") {
            errors.push(e);
        }
    }

    if let Some(url) = payload.get("model_url").and_then(|v| v.as_str()) {
        if let Err(e) = validator.validate_url(url, "model_url") {
            errors.push(e);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_validation() {
        let validator = InputValidator::new();

        // Valid text
        assert!(validator.validate_text("Hello, world!", "test").is_ok());

        // Empty text
        assert!(validator.validate_text("", "test").is_err());

        // Text too long
        let long_text = "a".repeat(20000);
        assert!(validator.validate_text(&long_text, "test").is_err());
    }

    #[test]
    fn test_json_validation() {
        let validator = InputValidator::new();

        // Valid JSON
        assert!(validator.validate_json(r#"{"key": "value"}"#, "test").is_ok());

        // Invalid JSON
        assert!(validator.validate_json(r#"{"key": "value""#, "test").is_err());

        // JSON too large
        let large_json = format!("{{\"data\": \"{}\"}}", "x".repeat(2_000_000));
        assert!(validator.validate_json(&large_json, "test").is_err());
    }

    #[test]
    fn test_filename_sanitization() {
        let validator = InputValidator::new();

        assert_eq!(validator.sanitize_filename("normal_file.txt"), "normal_file.txt");
        assert_eq!(validator.sanitize_filename("bad/file:name.txt"), "bad_file_name.txt");
        assert_eq!(validator.sanitize_filename("../../../etc/passwd"), "______etc_passwd");
    }

    #[test]
    fn test_harmful_content_detection() {
        let validator = InputValidator::new();

        assert!(validator.contains_harmful_patterns("rm -rf /"));
        assert!(validator.contains_harmful_patterns("format c:"));
        assert!(validator.contains_harmful_patterns("<script>alert('xss')</script>"));
        assert!(!validator.contains_harmful_patterns("Hello, world!"));
    }
}