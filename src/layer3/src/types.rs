//! # Layer 3 Types - Core Data Structures for Validation System
//!
//! This module defines the core data structures used throughout the Layer 3 validation system.
//! These types represent validation requests, results, safety checks, integrity validation,
//! compliance checking, and risk assessment.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use validator::Validate;

/// Types of operations that require validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    DataProcessing,
    ModelTraining,
    AgentDeployment,
    ResourceAllocation,
    ConfigurationChange,
    SecurityUpdate,
    SystemMaintenance,
    DataAccess,
    NetworkCommunication,
    FileOperation,
}

/// Security levels for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Public,
    Standard,
    Confidential,
    Restricted,
    TopSecret,
}

/// Validation request from other layers
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidationRequest {
    pub id: Uuid,
    pub operation_type: OperationType,
    pub parameters: HashMap<String, String>,
    pub context: ValidationContext,
    pub data: ValidationData,
    pub timestamp: DateTime<Utc>,
}

/// Context information for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext {
    pub user_id: Option<String>,
    pub session_id: Option<Uuid>,
    pub source_layer: String,
    pub target_layer: String,
    pub security_level: SecurityLevel,
    pub compliance_requirements: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Data payload for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationData {
    None,
    Text(String),
    Binary(Vec<u8>),
    Json(serde_json::Value),
    Metrics(HashMap<String, f64>),
    Model(ModelData),
    Configuration(ConfigData),
}

/// Model data for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelData {
    pub model_id: Uuid,
    pub model_type: String,
    pub parameters: HashMap<String, f64>,
    pub size_bytes: u64,
    pub checksum: String,
    pub training_data_hash: String,
}

/// Configuration data for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub config_type: String,
    pub parameters: HashMap<String, String>,
    pub checksum: String,
    pub version: String,
}

/// Overall validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: Uuid,
    pub is_valid: bool,
    pub safety_status: SafetyStatus,
    pub integrity_status: IntegrityStatus,
    pub compliance_status: ComplianceStatus,
    pub risk_assessment: RiskAssessment,
    pub validation_time_ms: u128,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Safety validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyStatus {
    pub is_safe: bool,
    pub safety_score: f64,
    pub issues: Vec<String>,
    pub safety_checks: Vec<SafetyCheck>,
    pub timestamp: DateTime<Utc>,
}

/// Individual safety check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    pub check_id: Uuid,
    pub check_type: SafetyCheckType,
    pub status: CheckStatus,
    pub score: f64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyCheckType {
    AccessControl,
    DataValidation,
    ResourceLimits,
    NetworkSecurity,
    ModelSafety,
    ConfigurationSafety,
    PerformanceSafety,
    ComplianceSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

/// Integrity validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityStatus {
    pub is_valid: bool,
    pub integrity_score: f64,
    pub issues: Vec<String>,
    pub checks: Vec<IntegrityCheck>,
    pub timestamp: DateTime<Utc>,
}

/// Individual integrity check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheck {
    pub check_id: Uuid,
    pub check_type: IntegrityCheckType,
    pub status: CheckStatus,
    pub score: f64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityCheckType {
    DataIntegrity,
    ModelIntegrity,
    ConfigurationIntegrity,
    SystemIntegrity,
    NetworkIntegrity,
    ProcessIntegrity,
    MemoryIntegrity,
    FileIntegrity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub is_compliant: bool,
    pub compliance_score: f64,
    pub issues: Vec<String>,
    pub checks: Vec<ComplianceCheck>,
    pub timestamp: DateTime<Utc>,
}

/// Individual compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub check_id: Uuid,
    pub regulation: String,
    pub requirement: String,
    pub status: CheckStatus,
    pub score: f64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Risk assessment levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskAssessment {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

/// System validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemValidationReport {
    pub timestamp: DateTime<Utc>,
    pub overall_status: SystemStatus,
    pub safety_status: SafetyStatus,
    pub integrity_status: IntegrityStatus,
    pub compliance_status: ComplianceStatus,
    pub issues: Vec<SystemIssue>,
    pub recommendations: Vec<String>,
}

/// Overall system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// System issue tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIssue {
    pub id: Uuid,
    pub component: String,
    pub severity: IssueSeverity,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub safety_threshold: f64,
    pub integrity_threshold: f64,
    pub compliance_threshold: f64,
    pub risk_threshold: RiskAssessment,
    pub enable_real_time_validation: bool,
    pub enable_continuous_monitoring: bool,
    pub validation_timeout_seconds: u64,
    pub max_concurrent_validations: usize,
    pub cache_validation_results: bool,
    pub cache_ttl_minutes: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            safety_threshold: 0.8,
            integrity_threshold: 0.9,
            compliance_threshold: 0.95,
            risk_threshold: RiskAssessment::Medium,
            enable_real_time_validation: true,
            enable_continuous_monitoring: true,
            validation_timeout_seconds: 30,
            max_concurrent_validations: 100,
            cache_validation_results: true,
            cache_ttl_minutes: 60,
        }
    }
}

/// Validation policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub policy_type: PolicyType,
    pub rules: Vec<ValidationRule>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    Safety,
    Integrity,
    Compliance,
    Security,
    Performance,
    Custom,
}

/// Individual validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub condition: String,
    pub action: ValidationAction,
    pub severity: IssueSeverity,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    SafetyCheck,
    IntegrityCheck,
    ComplianceCheck,
    SecurityCheck,
    PerformanceCheck,
    CustomCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationAction {
    Allow,
    Deny,
    Warn,
    Escalate,
    Log,
}

/// Validation metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    pub timestamp: DateTime<Utc>,
    pub operations_validated: u64,
    pub safety_violations: u64,
    pub integrity_failures: u64,
    pub compliance_violations: u64,
    pub average_validation_time_ms: f64,
    pub cache_hit_rate: f64,
    pub system_health_score: f64,
}

/// Health status for validation components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { issues: Vec<String> },
    Unhealthy { issues: Vec<String> },
}

/// Error types for the validation system
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Safety validation failed: {0}")]
    SafetyValidationFailed(String),

    #[error("Integrity validation failed: {0}")]
    IntegrityValidationFailed(String),

    #[error("Compliance validation failed: {0}")]
    ComplianceValidationFailed(String),

    #[error("Risk assessment failed: {0}")]
    RiskAssessmentFailed(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Integration error: {0}")]
    IntegrationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_request_creation() {
        let request = ValidationRequest {
            id: Uuid::new_v4(),
            operation_type: OperationType::DataProcessing,
            parameters: HashMap::new(),
            context: ValidationContext {
                user_id: Some("test-user".to_string()),
                session_id: Some(Uuid::new_v4()),
                source_layer: "layer4".to_string(),
                target_layer: "layer5".to_string(),
                security_level: SecurityLevel::Standard,
                compliance_requirements: Vec::new(),
                timestamp: Utc::now(),
            },
            data: ValidationData::None,
            timestamp: Utc::now(),
        };

        assert_eq!(request.operation_type, OperationType::DataProcessing);
        assert_eq!(request.context.security_level, SecurityLevel::Standard);
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult {
            id: Uuid::new_v4(),
            is_valid: true,
            safety_status: SafetyStatus {
                is_safe: true,
                safety_score: 0.95,
                issues: Vec::new(),
                safety_checks: Vec::new(),
                timestamp: Utc::now(),
            },
            integrity_status: IntegrityStatus {
                is_valid: true,
                integrity_score: 0.98,
                issues: Vec::new(),
                checks: Vec::new(),
                timestamp: Utc::now(),
            },
            compliance_status: ComplianceStatus {
                is_compliant: true,
                compliance_score: 1.0,
                issues: Vec::new(),
                checks: Vec::new(),
                timestamp: Utc::now(),
            },
            risk_assessment: RiskAssessment::Low,
            validation_time_ms: 150,
            recommendations: Vec::new(),
            timestamp: Utc::now(),
        };

        assert!(result.is_valid);
        assert_eq!(result.risk_assessment, RiskAssessment::Low);
        assert_eq!(result.validation_time_ms, 150);
    }

    #[test]
    fn test_system_status_progression() {
        assert!(matches!(SystemStatus::Healthy as u8, 0));
        assert!(matches!(SystemStatus::Degraded as u8, 1));
        assert!(matches!(SystemStatus::Unhealthy as u8, 2));
        assert!(matches!(SystemStatus::Critical as u8, 3));
    }

    #[test]
    fn test_security_level_progression() {
        assert!(matches!(SecurityLevel::Public as u8, 0));
        assert!(matches!(SecurityLevel::Standard as u8, 1));
        assert!(matches!(SecurityLevel::Confidential as u8, 2));
        assert!(matches!(SecurityLevel::Restricted as u8, 3));
        assert!(matches!(SecurityLevel::TopSecret as u8, 4));
    }
}