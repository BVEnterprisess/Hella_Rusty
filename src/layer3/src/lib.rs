//! # Layer 3 (Validation) - System Integrity and Safety Validation
//!
//! Layer 3 is responsible for system integrity validation, safety checking, and compliance
//! enforcement across the Project Chimera autonomous AI system. It ensures that all
//! operations are safe, compliant, and maintain system integrity.
//!
//! ## Core Responsibilities
//!
//! - **Safety Validation**: Ensure operations don't compromise system safety
//! - **Integrity Checking**: Validate system state and data integrity
//! - **Compliance Validation**: Ensure regulatory and policy compliance
//! - **Risk Mitigation**: Implement safety controls and risk mitigation
//! - **Security Validation**: Validate security controls and access permissions
//! - **Performance Validation**: Ensure performance requirements are met
//!
//! ## Architecture
//!
//! The validation system consists of several key components:
//!
//! - **ValidationService**: Main validation orchestration and coordination
//! - **SafetyValidator**: Safety and security validation engine
//! - **IntegrityChecker**: System integrity and data validation
//! - **ComplianceValidator**: Regulatory and policy compliance checking
//! - **RiskMitigator**: Risk mitigation and safety control implementation
//! - **ValidationMetrics**: Performance monitoring and observability

pub mod validation_service;
pub mod safety_validator;
pub mod integrity_checker;
pub mod compliance_validator;
pub mod risk_mitigator;
pub mod types;
pub mod metrics;

pub use validation_service::ValidationService;
pub use safety_validator::SafetyValidator;
pub use integrity_checker::IntegrityChecker;
pub use compliance_validator::ComplianceValidator;
pub use risk_mitigator::RiskMitigator;
pub use types::*;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Main validation service that orchestrates all validation activities
pub struct ValidationEngine {
    validation_service: Arc<ValidationService>,
    safety_validator: Arc<SafetyValidator>,
    integrity_checker: Arc<IntegrityChecker>,
    compliance_validator: Arc<ComplianceValidator>,
    risk_mitigator: Arc<RiskMitigator>,
    metrics: Arc<metrics::ValidationMetrics>,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub async fn new() -> Result<Self> {
        let validation_service = Arc::new(ValidationService::new().await?);
        let safety_validator = Arc::new(SafetyValidator::new().await?);
        let integrity_checker = Arc::new(IntegrityChecker::new().await?);
        let compliance_validator = Arc::new(ComplianceValidator::new().await?);
        let risk_mitigator = Arc::new(RiskMitigator::new().await?);
        let metrics = Arc::new(metrics::ValidationMetrics::new().await?);

        Ok(Self {
            validation_service,
            safety_validator,
            integrity_checker,
            compliance_validator,
            risk_mitigator,
            metrics,
        })
    }

    /// Validate a complete operation including safety, integrity, and compliance
    pub async fn validate_operation(&self, operation: ValidationRequest) -> Result<ValidationResult> {
        info!("Validating operation: {}", operation.id);

        // Record metrics
        self.metrics.operations_received.inc();
        let start_time = std::time::Instant::now();

        // Safety validation first (fail-fast)
        let safety_result = self.safety_validator.validate_safety(&operation).await?;
        if !safety_result.is_safe {
            self.metrics.safety_violations.inc();
            return Ok(ValidationResult {
                id: operation.id,
                is_valid: false,
                safety_status: safety_result,
                integrity_status: IntegrityStatus::NotValidated,
                compliance_status: ComplianceStatus::NotValidated,
                risk_assessment: RiskAssessment::Unknown,
                validation_time_ms: start_time.elapsed().as_millis(),
                recommendations: vec!["Operation blocked due to safety violation".to_string()],
                timestamp: Utc::now(),
            });
        }

        // Integrity validation
        let integrity_status = self.integrity_checker.validate_integrity(&operation).await?;

        // Compliance validation
        let compliance_status = self.compliance_validator.validate_compliance(&operation).await?;

        // Risk assessment
        let risk_assessment = self.risk_mitigator.assess_risks(&operation).await?;

        // Overall validation result
        let is_valid = safety_result.is_safe && integrity_status.is_valid && compliance_status.is_compliant;

        if !is_valid {
            self.metrics.validation_failures.inc();
        } else {
            self.metrics.validation_successes.inc();
        }

        // Record completion metrics
        let duration = start_time.elapsed();
        self.metrics.validation_duration_seconds.observe(duration.as_secs_f64());

        let result = ValidationResult {
            id: operation.id,
            is_valid,
            safety_status: safety_result,
            integrity_status,
            compliance_status,
            risk_assessment,
            validation_time_ms: duration.as_millis(),
            recommendations: self.generate_recommendations(&safety_result, &integrity_status, &compliance_status, &risk_assessment),
            timestamp: Utc::now(),
        };

        info!("Validation completed: {} - {}", operation.id, if is_valid { "PASSED" } else { "FAILED" });
        Ok(result)
    }

    /// Validate system state and integrity
    pub async fn validate_system_state(&self) -> Result<SystemValidationReport> {
        debug!("Validating system state");

        self.metrics.system_validations.inc();

        let safety_status = self.safety_validator.validate_system_safety().await?;
        let integrity_status = self.integrity_checker.validate_system_integrity().await?;
        let compliance_status = self.compliance_validator.validate_system_compliance().await?;

        let overall_status = if safety_status.is_safe && integrity_status.is_valid && compliance_status.is_compliant {
            SystemStatus::Healthy
        } else {
            SystemStatus::Degraded
        };

        Ok(SystemValidationReport {
            timestamp: Utc::now(),
            overall_status,
            safety_status,
            integrity_status,
            compliance_status,
            issues: self.collect_system_issues(&safety_status, &integrity_status, &compliance_status).await?,
            recommendations: self.generate_system_recommendations(&safety_status, &integrity_status, &compliance_status),
        })
    }

    /// Get validation metrics
    pub async fn get_metrics(&self) -> Result<metrics::ValidationMetricsSnapshot> {
        self.metrics.snapshot().await
    }

    /// Health check for the validation engine
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut issues = Vec::new();

        // Check all components
        if let Err(e) = self.validation_service.health_check().await {
            issues.push(format!("Validation Service: {}", e));
        }

        if let Err(e) = self.safety_validator.health_check().await {
            issues.push(format!("Safety Validator: {}", e));
        }

        if let Err(e) = self.integrity_checker.health_check().await {
            issues.push(format!("Integrity Checker: {}", e));
        }

        if let Err(e) = self.compliance_validator.health_check().await {
            issues.push(format!("Compliance Validator: {}", e));
        }

        if let Err(e) = self.risk_mitigator.health_check().await {
            issues.push(format!("Risk Mitigator: {}", e));
        }

        if issues.is_empty() {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Degraded { issues })
        }
    }

    /// Generate recommendations based on validation results
    fn generate_recommendations(
        &self,
        safety: &SafetyStatus,
        integrity: &IntegrityStatus,
        compliance: &ComplianceStatus,
        risk: &RiskAssessment,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !safety.is_safe {
            recommendations.push("Address safety violations before proceeding".to_string());
        }

        if !integrity.is_valid {
            recommendations.push("Fix data integrity issues".to_string());
        }

        if !compliance.is_compliant {
            recommendations.push("Resolve compliance violations".to_string());
        }

        match risk {
            RiskAssessment::High => {
                recommendations.push("High risk operation - consider additional safety measures".to_string());
            }
            RiskAssessment::Medium => {
                recommendations.push("Medium risk operation - monitor closely".to_string());
            }
            RiskAssessment::Low => {
                recommendations.push("Low risk operation - proceed with standard monitoring".to_string());
            }
            RiskAssessment::Unknown => {
                recommendations.push("Risk assessment incomplete - gather more data".to_string());
            }
        }

        recommendations
    }

    /// Generate system-level recommendations
    fn generate_system_recommendations(
        &self,
        safety: &SafetyStatus,
        integrity: &IntegrityStatus,
        compliance: &ComplianceStatus,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !safety.is_safe {
            recommendations.push("System safety compromised - immediate attention required".to_string());
        }

        if !integrity.is_valid {
            recommendations.push("System integrity issues detected - run diagnostics".to_string());
        }

        if !compliance.is_compliant {
            recommendations.push("Compliance violations found - review and remediate".to_string());
        }

        if safety.is_safe && integrity.is_valid && compliance.is_compliant {
            recommendations.push("System validation passed - all checks healthy".to_string());
        }

        recommendations
    }

    /// Collect system issues from all validation components
    async fn collect_system_issues(
        &self,
        safety: &SafetyStatus,
        integrity: &IntegrityStatus,
        compliance: &ComplianceStatus,
    ) -> Result<Vec<SystemIssue>> {
        let mut issues = Vec::new();

        // Safety issues
        if !safety.is_safe {
            issues.push(SystemIssue {
                id: Uuid::new_v4(),
                component: "Safety".to_string(),
                severity: IssueSeverity::Critical,
                description: safety.issues.join("; "),
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Integrity issues
        if !integrity.is_valid {
            issues.push(SystemIssue {
                id: Uuid::new_v4(),
                component: "Integrity".to_string(),
                severity: IssueSeverity::High,
                description: integrity.issues.join("; "),
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Compliance issues
        if !compliance.is_compliant {
            issues.push(SystemIssue {
                id: Uuid::new_v4(),
                component: "Compliance".to_string(),
                severity: IssueSeverity::Medium,
                description: compliance.issues.join("; "),
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        Ok(issues)
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create ValidationEngine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_engine_creation() {
        let engine = ValidationEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_operation_validation() {
        let engine = ValidationEngine::new().await.unwrap();

        let operation = ValidationRequest {
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

        let result = engine.validate_operation(operation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_validation() {
        let engine = ValidationEngine::new().await.unwrap();

        let report = engine.validate_system_state().await;
        assert!(report.is_ok());
    }
}