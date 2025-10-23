//! # Safety Validator - Safety and Security Validation Engine
//!
//! The Safety Validator ensures that all operations maintain system safety and security.
//! It performs comprehensive safety checks including access control, resource limits,
//! data validation, and security policy enforcement.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Safety validator for comprehensive safety checking
pub struct SafetyValidator {
    safety_rules: Vec<SafetyRule>,
    access_control_list: HashMap<String, Vec<String>>,
    resource_limits: HashMap<String, f64>,
}

impl SafetyValidator {
    /// Create a new safety validator
    pub async fn new() -> Result<Self> {
        let validator = Self {
            safety_rules: Self::load_default_safety_rules().await?,
            access_control_list: Self::load_access_control_list().await?,
            resource_limits: Self::load_resource_limits().await?,
        };

        info!("Safety validator initialized with {} rules", validator.safety_rules.len());
        Ok(validator)
    }

    /// Validate safety of an operation
    pub async fn validate_safety(&self, request: &ValidationRequest) -> Result<SafetyStatus> {
        info!("Validating safety for operation: {}", request.id);

        let mut safety_checks = Vec::new();
        let mut issues = Vec::new();
        let mut total_score = 0.0;
        let mut check_count = 0;

        // Apply all safety rules
        for rule in &self.safety_rules {
            if rule.is_enabled && self.rule_applies_to_request(rule, request) {
                let check_result = self.execute_safety_rule(rule, request).await?;
                safety_checks.push(check_result.clone());
                total_score += check_result.score;
                check_count += 1;

                if !matches!(check_result.status, CheckStatus::Passed) {
                    issues.push(format!("{}: {}", rule.name, check_result.message));
                }
            }
        }

        // Add default checks if no rules applied
        if safety_checks.is_empty() {
            safety_checks.push(SafetyCheck {
                check_id: Uuid::new_v4(),
                check_type: SafetyCheckType::AccessControl,
                status: CheckStatus::Passed,
                score: 1.0,
                message: "Default safety check passed".to_string(),
                timestamp: Utc::now(),
            });
            total_score = 1.0;
            check_count = 1;
        }

        let safety_score = if check_count > 0 { total_score / check_count as f64 } else { 1.0 };
        let is_safe = safety_score >= 0.8; // 80% safety threshold

        Ok(SafetyStatus {
            is_safe,
            safety_score,
            issues,
            safety_checks,
            timestamp: Utc::now(),
        })
    }

    /// Validate system-wide safety
    pub async fn validate_system_safety(&self) -> Result<SafetyStatus> {
        debug!("Validating system safety");

        let mut safety_checks = Vec::new();
        let mut issues = Vec::new();
        let mut total_score = 0.0;
        let mut check_count = 0;

        // System-level safety checks
        let system_checks = vec![
            ("Resource Usage", self.check_resource_usage().await?),
            ("Access Control", self.check_access_control_integrity().await?),
            ("Network Security", self.check_network_security().await?),
            ("Data Protection", self.check_data_protection().await?),
            ("Process Integrity", self.check_process_integrity().await?),
        ];

        for (check_name, (status, score, message)) in system_checks {
            let check = SafetyCheck {
                check_id: Uuid::new_v4(),
                check_type: SafetyCheckType::AccessControl, // Default type
                status,
                score,
                message: format!("{}: {}", check_name, message),
                timestamp: Utc::now(),
            };

            safety_checks.push(check.clone());
            total_score += score;
            check_count += 1;

            if !matches!(status, CheckStatus::Passed) {
                issues.push(format!("{}: {}", check_name, message));
            }
        }

        let safety_score = if check_count > 0 { total_score / check_count as f64 } else { 1.0 };
        let is_safe = safety_score >= 0.9; // Higher threshold for system safety

        Ok(SafetyStatus {
            is_safe,
            safety_score,
            issues,
            safety_checks,
            timestamp: Utc::now(),
        })
    }

    /// Execute a specific safety rule
    async fn execute_safety_rule(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<SafetyCheck> {
        let check_id = Uuid::new_v4();

        let (status, score, message) = match rule.rule_type {
            SafetyRuleType::AccessControl => {
                self.check_access_control(rule, request).await?
            }
            SafetyRuleType::ResourceLimits => {
                self.check_resource_limits(rule, request).await?
            }
            SafetyRuleType::DataValidation => {
                self.check_data_validation(rule, request).await?
            }
            SafetyRuleType::NetworkSecurity => {
                self.check_network_security_rule(rule, request).await?
            }
            SafetyRuleType::ModelSafety => {
                self.check_model_safety(rule, request).await?
            }
            SafetyRuleType::ConfigurationSafety => {
                self.check_configuration_safety(rule, request).await?
            }
        };

        Ok(SafetyCheck {
            check_id,
            check_type: SafetyCheckType::AccessControl, // Map rule type to check type
            status,
            score,
            message,
            timestamp: Utc::now(),
        })
    }

    /// Check access control for the request
    async fn check_access_control(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        if let Some(user_id) = &request.context.user_id {
            if let Some(allowed_operations) = self.access_control_list.get(user_id) {
                let operation_allowed = allowed_operations.iter()
                    .any(|op| self.operation_matches_rule(op, &request.operation_type));

                if operation_allowed {
                    Ok((CheckStatus::Passed, 1.0, "Access control check passed".to_string()))
                } else {
                    Ok((CheckStatus::Failed, 0.0, "Access denied for this operation".to_string()))
                }
            } else {
                Ok((CheckStatus::Warning, 0.5, "User not found in access control list".to_string()))
            }
        } else {
            Ok((CheckStatus::Warning, 0.7, "No user context provided".to_string()))
        }
    }

    /// Check resource limits for the request
    async fn check_resource_limits(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        let resource_usage = self.estimate_resource_usage(request).await?;

        if let Some(limit) = self.resource_limits.get(&rule.resource_type) {
            if resource_usage <= *limit {
                Ok((CheckStatus::Passed, 1.0, format!("Resource usage within limits: {:.2}", resource_usage)))
            } else {
                Ok((CheckStatus::Failed, 0.0, format!("Resource usage exceeds limit: {:.2} > {:.2}", resource_usage, limit)))
            }
        } else {
            Ok((CheckStatus::Warning, 0.8, "Resource limit not configured".to_string()))
        }
    }

    /// Check data validation for the request
    async fn check_data_validation(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        let validation_score = match &request.data {
            ValidationData::Text(text) => {
                self.validate_text_data(text).await?
            }
            ValidationData::Binary(data) => {
                self.validate_binary_data(data).await?
            }
            ValidationData::Json(json) => {
                self.validate_json_data(json).await?
            }
            ValidationData::Model(model) => {
                self.validate_model_data(model).await?
            }
            ValidationData::Configuration(config) => {
                self.validate_config_data(config).await?
            }
            ValidationData::Metrics(metrics) => {
                self.validate_metrics_data(metrics).await?
            }
            ValidationData::None => {
                0.5 // Neutral score for no data
            }
        };

        let status = if validation_score >= 0.9 {
            CheckStatus::Passed
        } else if validation_score >= 0.7 {
            CheckStatus::Warning
        } else {
            CheckStatus::Failed
        };

        Ok((status, validation_score, format!("Data validation score: {:.2}", validation_score)))
    }

    /// Check network security for the request
    async fn check_network_security_rule(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        match request.operation_type {
            OperationType::NetworkCommunication => {
                // Check if network operation is allowed
                let is_allowed = request.context.security_level != SecurityLevel::TopSecret ||
                    request.parameters.get("encrypted") == Some(&"true".to_string());

                if is_allowed {
                    Ok((CheckStatus::Passed, 1.0, "Network communication allowed".to_string()))
                } else {
                    Ok((CheckStatus::Failed, 0.0, "Unencrypted network communication not allowed".to_string()))
                }
            }
            _ => {
                Ok((CheckStatus::Passed, 1.0, "Network security check not applicable".to_string()))
            }
        }
    }

    /// Check model safety for the request
    async fn check_model_safety(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        if let ValidationData::Model(model_data) = &request.data {
            // Validate model parameters and training data
            let safety_score = if model_data.parameters.values().all(|&v| v.is_finite() && v.abs() < 1000.0) {
                0.95
            } else {
                0.3
            };

            let status = if safety_score >= 0.8 {
                CheckStatus::Passed
            } else if safety_score >= 0.6 {
                CheckStatus::Warning
            } else {
                CheckStatus::Failed
            };

            Ok((status, safety_score, format!("Model safety score: {:.2}", safety_score)))
        } else {
            Ok((CheckStatus::Passed, 1.0, "Model safety check not applicable".to_string()))
        }
    }

    /// Check configuration safety for the request
    async fn check_configuration_safety(&self, rule: &SafetyRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        if let ValidationData::Configuration(config_data) = &request.data {
            // Validate configuration parameters
            let safety_score = if config_data.parameters.values().all(|v| !v.is_empty() && v.len() < 10000) {
                0.9
            } else {
                0.4
            };

            let status = if safety_score >= 0.8 {
                CheckStatus::Passed
            } else {
                CheckStatus::Warning
            };

            Ok((status, safety_score, format!("Configuration safety score: {:.2}", safety_score)))
        } else {
            Ok((CheckStatus::Passed, 1.0, "Configuration safety check not applicable".to_string()))
        }
    }

    /// System-level safety checks
    async fn check_resource_usage(&self) -> Result<(CheckStatus, f64, String)> {
        // Mock resource usage check
        Ok((CheckStatus::Passed, 0.85, "Resource usage within normal limits".to_string()))
    }

    async fn check_access_control_integrity(&self) -> Result<(CheckStatus, f64, String)> {
        // Mock access control integrity check
        Ok((CheckStatus::Passed, 0.95, "Access control integrity verified".to_string()))
    }

    async fn check_network_security(&self) -> Result<(CheckStatus, f64, String)> {
        // Mock network security check
        Ok((CheckStatus::Passed, 0.9, "Network security protocols active".to_string()))
    }

    async fn check_data_protection(&self) -> Result<(CheckStatus, f64, String)> {
        // Mock data protection check
        Ok((CheckStatus::Passed, 0.92, "Data protection measures in place".to_string()))
    }

    async fn check_process_integrity(&self) -> Result<(CheckStatus, f64, String)> {
        // Mock process integrity check
        Ok((CheckStatus::Passed, 0.88, "Process integrity verified".to_string()))
    }

    /// Validate text data
    async fn validate_text_data(&self, text: &str) -> Result<f64> {
        let mut score = 1.0;

        // Check for potentially malicious content
        let suspicious_patterns = ["script", "eval", "exec", "system", "sql"];
        for pattern in &suspicious_patterns {
            if text.to_lowercase().contains(pattern) {
                score -= 0.2;
            }
        }

        // Check length
        if text.len() > 1000000 {
            score -= 0.3;
        }

        // Check for encoding issues
        if text.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            score -= 0.1;
        }

        Ok(score.max(0.0))
    }

    /// Validate binary data
    async fn validate_binary_data(&self, data: &[u8]) -> Result<f64> {
        let mut score = 1.0;

        // Check size
        if data.len() > 100 * 1024 * 1024 {
            score -= 0.3; // 100MB limit
        }

        // Check for suspicious file signatures
        if data.len() >= 4 {
            let signature = &data[0..4];
            let suspicious_signatures = [
                [0x4D, 0x5A, 0x90, 0x00], // PE executable
                [0x7F, 0x45, 0x4C, 0x46], // ELF executable
                [0x89, 0x50, 0x4E, 0x47], // PNG (might be malicious)
            ];

            for sig in &suspicious_signatures {
                if signature == sig {
                    score -= 0.4;
                    break;
                }
            }
        }

        Ok(score.max(0.0))
    }

    /// Validate JSON data
    async fn validate_json_data(&self, json: &serde_json::Value) -> Result<f64> {
        let mut score = 1.0;

        // Check for deeply nested structures (potential DoS)
        let depth = self.calculate_json_depth(json);
        if depth > 20 {
            score -= 0.3;
        }

        // Check for extremely large values
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key.len() > 1000 || value.to_string().len() > 10000 {
                    score -= 0.2;
                }
            }
        }

        Ok(score.max(0.0))
    }

    /// Validate model data
    async fn validate_model_data(&self, model: &ModelData) -> Result<f64> {
        let mut score = 1.0;

        // Check model size
        if model.size_bytes > 10 * 1024 * 1024 * 1024 {
            score -= 0.3; // 10GB limit
        }

        // Check parameter bounds
        for (name, value) in &model.parameters {
            if !value.is_finite() || value.abs() > 10000.0 {
                score -= 0.2;
            }
        }

        // Check checksum validity
        if model.checksum.is_empty() || model.checksum.len() != 64 {
            score -= 0.3;
        }

        Ok(score.max(0.0))
    }

    /// Validate configuration data
    async fn validate_config_data(&self, config: &ConfigData) -> Result<f64> {
        let mut score = 1.0;

        // Check for dangerous configuration options
        let dangerous_options = ["debug", "unsafe", "disable_security", "allow_root"];
        for (key, value) in &config.parameters {
            for dangerous in &dangerous_options {
                if key.to_lowercase().contains(dangerous) && value.to_lowercase() == "true" {
                    score -= 0.4;
                }
            }
        }

        // Check checksum
        if config.checksum.is_empty() {
            score -= 0.2;
        }

        Ok(score.max(0.0))
    }

    /// Validate metrics data
    async fn validate_metrics_data(&self, metrics: &HashMap<String, f64>) -> Result<f64> {
        let mut score = 1.0;

        // Check for invalid metric values
        for (name, value) in metrics {
            if !value.is_finite() || value.is_nan() {
                score -= 0.3;
            }

            // Check for suspicious metric names
            if name.contains("..") || name.contains("/") || name.contains("\\") {
                score -= 0.2;
            }
        }

        Ok(score.max(0.0))
    }

    /// Calculate JSON depth
    fn calculate_json_depth(&self, value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Object(obj) => {
                1 + obj.values().map(|v| self.calculate_json_depth(v)).max().unwrap_or(0)
            }
            serde_json::Value::Array(arr) => {
                1 + arr.iter().map(|v| self.calculate_json_depth(v)).max().unwrap_or(0)
            }
            _ => 1,
        }
    }

    /// Estimate resource usage for a request
    async fn estimate_resource_usage(&self, request: &ValidationRequest) -> Result<f64> {
        let base_usage = match request.operation_type {
            OperationType::DataProcessing => 1.0,
            OperationType::ModelTraining => 10.0,
            OperationType::AgentDeployment => 2.0,
            OperationType::ResourceAllocation => 0.5,
            OperationType::ConfigurationChange => 0.1,
            OperationType::SecurityUpdate => 0.2,
            OperationType::SystemMaintenance => 3.0,
            OperationType::DataAccess => 0.3,
            OperationType::NetworkCommunication => 0.5,
            OperationType::FileOperation => 0.2,
        };

        // Adjust based on data size
        let data_multiplier = match &request.data {
            ValidationData::Binary(data) => data.len() as f64 / 1024.0 / 1024.0, // MB
            ValidationData::Text(text) => text.len() as f64 / 1024.0 / 1024.0, // MB
            ValidationData::Model(model) => model.size_bytes as f64 / 1024.0 / 1024.0 / 1024.0, // GB
            _ => 1.0,
        };

        Ok(base_usage * data_multiplier.max(1.0))
    }

    /// Check if a rule applies to a request
    fn rule_applies_to_request(&self, rule: &SafetyRule, request: &ValidationRequest) -> bool {
        match rule.scope {
            RuleScope::All => true,
            RuleScope::OperationType(op_type) => request.operation_type == op_type,
            RuleScope::SecurityLevel(sec_level) => request.context.security_level == sec_level,
            RuleScope::User(user_pattern) => {
                if let Some(user_id) = &request.context.user_id {
                    user_id.contains(&user_pattern)
                } else {
                    false
                }
            }
            RuleScope::Custom(condition) => {
                self.evaluate_custom_condition(&condition, request)
            }
        }
    }

    /// Evaluate custom condition
    fn evaluate_custom_condition(&self, condition: &str, request: &ValidationRequest) -> bool {
        // Simple condition evaluation (could be enhanced with more sophisticated logic)
        if condition == "high_security" {
            matches!(request.context.security_level, SecurityLevel::Restricted | SecurityLevel::TopSecret)
        } else if condition == "data_operation" {
            matches!(request.operation_type, OperationType::DataProcessing | OperationType::DataAccess)
        } else {
            false
        }
    }

    /// Check if operation matches rule pattern
    fn operation_matches_rule(&self, rule_op: &str, request_op: &OperationType) -> bool {
        match rule_op {
            "data_processing" => matches!(request_op, OperationType::DataProcessing),
            "model_training" => matches!(request_op, OperationType::ModelTraining),
            "agent_deployment" => matches!(request_op, OperationType::AgentDeployment),
            "resource_allocation" => matches!(request_op, OperationType::ResourceAllocation),
            "configuration_change" => matches!(request_op, OperationType::ConfigurationChange),
            "security_update" => matches!(request_op, OperationType::SecurityUpdate),
            "system_maintenance" => matches!(request_op, OperationType::SystemMaintenance),
            "data_access" => matches!(request_op, OperationType::DataAccess),
            "network_communication" => matches!(request_op, OperationType::NetworkCommunication),
            "file_operation" => matches!(request_op, OperationType::FileOperation),
            "all" => true,
            _ => false,
        }
    }

    /// Load default safety rules
    async fn load_default_safety_rules() -> Result<Vec<SafetyRule>> {
        let mut rules = Vec::new();

        rules.push(SafetyRule {
            id: Uuid::new_v4(),
            name: "Access Control Check".to_string(),
            description: "Validate user access permissions".to_string(),
            rule_type: SafetyRuleType::AccessControl,
            scope: RuleScope::All,
            threshold: 1.0,
            is_enabled: true,
            parameters: HashMap::new(),
        });

        rules.push(SafetyRule {
            id: Uuid::new_v4(),
            name: "Resource Limits Check".to_string(),
            description: "Ensure resource usage within limits".to_string(),
            rule_type: SafetyRuleType::ResourceLimits,
            scope: RuleScope::All,
            threshold: 0.8,
            is_enabled: true,
            parameters: HashMap::from([
                ("resource_type".to_string(), "CPU".to_string()),
                ("max_usage".to_string(), "80".to_string()),
            ]),
        });

        rules.push(SafetyRule {
            id: Uuid::new_v4(),
            name: "Data Validation Check".to_string(),
            description: "Validate data integrity and safety".to_string(),
            rule_type: SafetyRuleType::DataValidation,
            scope: RuleScope::All,
            threshold: 0.9,
            is_enabled: true,
            parameters: HashMap::new(),
        });

        Ok(rules)
    }

    /// Load access control list
    async fn load_access_control_list() -> Result<HashMap<String, Vec<String>>> {
        let mut acl = HashMap::new();

        // Default access control entries
        acl.insert("admin".to_string(), vec!["all".to_string()]);
        acl.insert("user".to_string(), vec!["data_processing".to_string(), "data_access".to_string()]);
        acl.insert("guest".to_string(), vec!["data_access".to_string()]);

        Ok(acl)
    }

    /// Load resource limits
    async fn load_resource_limits() -> Result<HashMap<String, f64>> {
        let mut limits = HashMap::new();

        limits.insert("CPU".to_string(), 80.0); // 80% max
        limits.insert("Memory".to_string(), 90.0); // 90% max
        limits.insert("GPU".to_string(), 95.0); // 95% max
        limits.insert("Network".to_string(), 70.0); // 70% max

        Ok(limits)
    }

    /// Health check for the safety validator
    pub async fn health_check(&self) -> Result<()> {
        if self.safety_rules.is_empty() {
            warn!("No safety rules configured");
        }

        if self.access_control_list.is_empty() {
            warn!("No access control list configured");
        }

        if self.resource_limits.is_empty() {
            warn!("No resource limits configured");
        }

        Ok(())
    }
}

/// Safety rule definition
#[derive(Debug, Clone)]
struct SafetyRule {
    id: Uuid,
    name: String,
    description: String,
    rule_type: SafetyRuleType,
    scope: RuleScope,
    threshold: f64,
    is_enabled: bool,
    parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum SafetyRuleType {
    AccessControl,
    ResourceLimits,
    DataValidation,
    NetworkSecurity,
    ModelSafety,
    ConfigurationSafety,
}

#[derive(Debug, Clone)]
enum RuleScope {
    All,
    OperationType(OperationType),
    SecurityLevel(SecurityLevel),
    User(String),
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safety_validator_creation() {
        let validator = SafetyValidator::new().await;
        assert!(validator.is_ok());
    }

    #[tokio::test]
    async fn test_safety_validation() {
        let validator = SafetyValidator::new().await.unwrap();

        let request = ValidationRequest {
            id: Uuid::new_v4(),
            operation_type: OperationType::DataProcessing,
            parameters: HashMap::new(),
            context: ValidationContext {
                user_id: Some("user".to_string()),
                session_id: Some(Uuid::new_v4()),
                source_layer: "layer4".to_string(),
                target_layer: "layer5".to_string(),
                security_level: SecurityLevel::Standard,
                compliance_requirements: Vec::new(),
                timestamp: Utc::now(),
            },
            data: ValidationData::Text("test data".to_string()),
            timestamp: Utc::now(),
        };

        let result = validator.validate_safety(&request).await;
        assert!(result.is_ok());

        let safety_status = result.unwrap();
        assert!(safety_status.safety_score >= 0.0);
        assert!(safety_status.safety_score <= 1.0);
    }

    #[tokio::test]
    async fn test_system_safety_validation() {
        let validator = SafetyValidator::new().await.unwrap();

        let result = validator.validate_system_safety().await;
        assert!(result.is_ok());

        let safety_status = result.unwrap();
        assert!(safety_status.safety_score >= 0.0);
        assert!(safety_status.safety_score <= 1.0);
    }

    #[tokio::test]
    async fn test_data_validation() {
        let validator = SafetyValidator::new().await.unwrap();

        // Test text validation
        let score = validator.validate_text_data("normal text").await.unwrap();
        assert!(score > 0.8);

        // Test suspicious text
        let score = validator.validate_text_data("script alert('xss')").await.unwrap();
        assert!(score < 0.8);

        // Test binary validation
        let score = validator.validate_binary_data(&[0x4D, 0x5A, 0x90, 0x00]).await.unwrap(); // PE header
        assert!(score < 0.8);
    }
}