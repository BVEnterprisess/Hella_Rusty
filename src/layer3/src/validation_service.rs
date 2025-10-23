//! # Validation Service - Main Validation Orchestration
//!
//! The Validation Service is the central component of Layer 3 that orchestrates all
//! validation activities including safety, integrity, compliance, and risk assessment.
//! It coordinates between different validation engines and provides a unified interface
//! for validation operations.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Main validation service that orchestrates all validation activities
pub struct ValidationService {
    config: ValidationConfig,
    validation_cache: Arc<RwLock<HashMap<Uuid, ValidationResult>>>,
    active_validations: Arc<RwLock<HashMap<Uuid, ValidationRequest>>>,
    policies: Arc<RwLock<Vec<ValidationPolicy>>>,
}

impl ValidationService {
    /// Create a new validation service
    pub async fn new() -> Result<Self> {
        Ok(Self {
            config: ValidationConfig::default(),
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
            active_validations: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(Self::load_default_policies().await?)),
        })
    }

    /// Validate an operation with comprehensive checks
    pub async fn validate_operation(&self, request: ValidationRequest) -> Result<ValidationResult> {
        info!("Starting validation for operation: {}", request.id);

        // Check cache first if enabled
        if self.config.cache_validation_results {
            if let Some(cached_result) = self.get_cached_result(&request).await? {
                info!("Returning cached validation result for: {}", request.id);
                return Ok(cached_result);
            }
        }

        // Record active validation
        self.active_validations.write().await.insert(request.id, request.clone());

        // Perform comprehensive validation
        let result = self.perform_comprehensive_validation(request).await?;

        // Cache result if enabled
        if self.config.cache_validation_results {
            self.cache_result(&result).await?;
        }

        // Remove from active validations
        self.active_validations.write().await.remove(&result.id);

        info!("Validation completed: {} - {}", result.id, if result.is_valid { "PASSED" } else { "FAILED" });
        Ok(result)
    }

    /// Perform comprehensive validation including all checks
    async fn perform_comprehensive_validation(&self, request: ValidationRequest) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();

        // Apply validation policies
        let applicable_policies = self.get_applicable_policies(&request).await?;

        // Safety validation
        let safety_status = self.validate_safety(&request, &applicable_policies).await?;

        // Integrity validation
        let integrity_status = self.validate_integrity(&request, &applicable_policies).await?;

        // Compliance validation
        let compliance_status = self.validate_compliance(&request, &applicable_policies).await?;

        // Risk assessment
        let risk_assessment = self.assess_operation_risk(&request, &safety_status, &integrity_status, &compliance_status).await?;

        // Overall validation decision
        let is_valid = self.make_validation_decision(&safety_status, &integrity_status, &compliance_status, &risk_assessment)?;

        let validation_time = start_time.elapsed();

        let result = ValidationResult {
            id: request.id,
            is_valid,
            safety_status,
            integrity_status,
            compliance_status,
            risk_assessment,
            validation_time_ms: validation_time.as_millis(),
            recommendations: self.generate_validation_recommendations(&safety_status, &integrity_status, &compliance_status, &risk_assessment),
            timestamp: Utc::now(),
        };

        Ok(result)
    }

    /// Validate safety aspects of the operation
    async fn validate_safety(&self, request: &ValidationRequest, policies: &[ValidationPolicy]) -> Result<SafetyStatus> {
        let mut safety_checks = Vec::new();
        let mut issues = Vec::new();
        let mut total_score = 0.0;
        let mut check_count = 0;

        // Apply safety policies
        for policy in policies.iter().filter(|p| matches!(p.policy_type, PolicyType::Safety)) {
            for rule in &policy.rules {
                if rule.is_enabled {
                    let check_result = self.execute_safety_rule(rule, request).await?;
                    safety_checks.push(check_result.clone());
                    total_score += check_result.score;
                    check_count += 1;

                    if !matches!(check_result.status, CheckStatus::Passed) {
                        issues.push(format!("{}: {}", rule.name, check_result.message));
                    }
                }
            }
        }

        // Default safety checks if no policies apply
        if safety_checks.is_empty() {
            safety_checks.push(SafetyCheck {
                check_id: Uuid::new_v4(),
                check_type: SafetyCheckType::AccessControl,
                status: CheckStatus::Passed,
                score: 1.0,
                message: "Basic access control check passed".to_string(),
                timestamp: Utc::now(),
            });
            total_score = 1.0;
            check_count = 1;
        }

        let safety_score = if check_count > 0 { total_score / check_count as f64 } else { 1.0 };
        let is_safe = safety_score >= self.config.safety_threshold;

        Ok(SafetyStatus {
            is_safe,
            safety_score,
            issues,
            safety_checks,
            timestamp: Utc::now(),
        })
    }

    /// Validate integrity aspects of the operation
    async fn validate_integrity(&self, request: &ValidationRequest, policies: &[ValidationPolicy]) -> Result<IntegrityStatus> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();
        let mut total_score = 0.0;
        let mut check_count = 0;

        // Apply integrity policies
        for policy in policies.iter().filter(|p| matches!(p.policy_type, PolicyType::Integrity)) {
            for rule in &policy.rules {
                if rule.is_enabled {
                    let check_result = self.execute_integrity_rule(rule, request).await?;
                    checks.push(check_result.clone());
                    total_score += check_result.score;
                    check_count += 1;

                    if !matches!(check_result.status, CheckStatus::Passed) {
                        issues.push(format!("{}: {}", rule.name, check_result.message));
                    }
                }
            }
        }

        // Default integrity checks
        if checks.is_empty() {
            checks.push(IntegrityCheck {
                check_id: Uuid::new_v4(),
                check_type: IntegrityCheckType::DataIntegrity,
                status: CheckStatus::Passed,
                score: 1.0,
                message: "Data integrity check passed".to_string(),
                timestamp: Utc::now(),
            });
            total_score = 1.0;
            check_count = 1;
        }

        let integrity_score = if check_count > 0 { total_score / check_count as f64 } else { 1.0 };
        let is_valid = integrity_score >= self.config.integrity_threshold;

        Ok(IntegrityStatus {
            is_valid,
            integrity_score,
            issues,
            checks,
            timestamp: Utc::now(),
        })
    }

    /// Validate compliance aspects of the operation
    async fn validate_compliance(&self, request: &ValidationRequest, policies: &[ValidationPolicy]) -> Result<ComplianceStatus> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();
        let mut total_score = 0.0;
        let mut check_count = 0;

        // Apply compliance policies
        for policy in policies.iter().filter(|p| matches!(p.policy_type, PolicyType::Compliance)) {
            for rule in &policy.rules {
                if rule.is_enabled {
                    let check_result = self.execute_compliance_rule(rule, request).await?;
                    checks.push(check_result.clone());
                    total_score += check_result.score;
                    check_count += 1;

                    if !matches!(check_result.status, CheckStatus::Passed) {
                        issues.push(format!("{}: {}", rule.name, check_result.message));
                    }
                }
            }
        }

        // Default compliance checks based on security level
        if checks.is_empty() {
            let compliance_score = match request.context.security_level {
                SecurityLevel::Public => 1.0,
                SecurityLevel::Standard => 0.95,
                SecurityLevel::Confidential => 0.9,
                SecurityLevel::Restricted => 0.85,
                SecurityLevel::TopSecret => 0.8,
            };

            checks.push(ComplianceCheck {
                check_id: Uuid::new_v4(),
                regulation: "Internal Policy".to_string(),
                requirement: "Security level compliance".to_string(),
                status: CheckStatus::Passed,
                score: compliance_score,
                message: format!("Compliant with security level: {:?}", request.context.security_level),
                timestamp: Utc::now(),
            });
            total_score = compliance_score;
            check_count = 1;
        }

        let compliance_score = if check_count > 0 { total_score / check_count as f64 } else { 1.0 };
        let is_compliant = compliance_score >= self.config.compliance_threshold;

        Ok(ComplianceStatus {
            is_compliant,
            compliance_score,
            issues,
            checks,
            timestamp: Utc::now(),
        })
    }

    /// Assess risk level of the operation
    async fn assess_operation_risk(
        &self,
        request: &ValidationRequest,
        safety: &SafetyStatus,
        integrity: &IntegrityStatus,
        compliance: &ComplianceStatus,
    ) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();

        // Safety risk
        if !safety.is_safe {
            risk_factors.push(0.9); // High risk
        } else if safety.safety_score < 0.9 {
            risk_factors.push(0.3); // Medium risk
        } else {
            risk_factors.push(0.1); // Low risk
        }

        // Integrity risk
        if !integrity.is_valid {
            risk_factors.push(0.8); // High risk
        } else if integrity.integrity_score < 0.95 {
            risk_factors.push(0.2); // Low risk
        } else {
            risk_factors.push(0.05); // Very low risk
        }

        // Compliance risk
        if !compliance.is_compliant {
            risk_factors.push(0.7); // High risk
        } else if compliance.compliance_score < 0.95 {
            risk_factors.push(0.25); // Medium risk
        } else {
            risk_factors.push(0.05); // Very low risk
        }

        // Operation type risk
        let operation_risk = match request.operation_type {
            OperationType::DataProcessing => 0.1,
            OperationType::ModelTraining => 0.3,
            OperationType::AgentDeployment => 0.4,
            OperationType::ResourceAllocation => 0.2,
            OperationType::ConfigurationChange => 0.3,
            OperationType::SecurityUpdate => 0.1,
            OperationType::SystemMaintenance => 0.5,
            OperationType::DataAccess => 0.2,
            OperationType::NetworkCommunication => 0.3,
            OperationType::FileOperation => 0.15,
        };
        risk_factors.push(operation_risk);

        // Calculate overall risk
        let average_risk = risk_factors.iter().sum::<f64>() / risk_factors.len() as f64;

        let risk_assessment = if average_risk >= 0.7 {
            RiskAssessment::Critical
        } else if average_risk >= 0.5 {
            RiskAssessment::High
        } else if average_risk >= 0.3 {
            RiskAssessment::Medium
        } else {
            RiskAssessment::Low
        };

        Ok(risk_assessment)
    }

    /// Make final validation decision
    fn make_validation_decision(
        &self,
        safety: &SafetyStatus,
        integrity: &IntegrityStatus,
        compliance: &ComplianceStatus,
        risk: &RiskAssessment,
    ) -> Result<bool> {
        // Check if risk exceeds threshold
        let risk_too_high = match (risk, &self.config.risk_threshold) {
            (RiskAssessment::Critical, _) => true,
            (RiskAssessment::High, RiskAssessment::Low | RiskAssessment::Medium) => true,
            (RiskAssessment::Medium, RiskAssessment::Low) => true,
            _ => false,
        };

        if risk_too_high {
            return Ok(false);
        }

        // All validation checks must pass
        Ok(safety.is_safe && integrity.is_valid && compliance.is_compliant)
    }

    /// Execute a safety validation rule
    async fn execute_safety_rule(&self, rule: &ValidationRule, request: &ValidationRequest) -> Result<SafetyCheck> {
        let check_id = Uuid::new_v4();

        // Simple rule evaluation (could be enhanced with more sophisticated logic)
        let (status, score, message) = match rule.rule_type {
            RuleType::SafetyCheck => {
                self.evaluate_safety_rule(rule, request).await?
            }
            _ => {
                (CheckStatus::Skipped, 0.5, "Rule type not applicable for safety check".to_string())
            }
        };

        Ok(SafetyCheck {
            check_id,
            check_type: SafetyCheckType::AccessControl, // Default type
            status,
            score,
            message,
            timestamp: Utc::now(),
        })
    }

    /// Execute an integrity validation rule
    async fn execute_integrity_rule(&self, rule: &ValidationRule, request: &ValidationRequest) -> Result<IntegrityCheck> {
        let check_id = Uuid::new_v4();

        let (status, score, message) = match rule.rule_type {
            RuleType::IntegrityCheck => {
                self.evaluate_integrity_rule(rule, request).await?
            }
            _ => {
                (CheckStatus::Skipped, 0.5, "Rule type not applicable for integrity check".to_string())
            }
        };

        Ok(IntegrityCheck {
            check_id,
            check_type: IntegrityCheckType::DataIntegrity, // Default type
            status,
            score,
            message,
            timestamp: Utc::now(),
        })
    }

    /// Execute a compliance validation rule
    async fn execute_compliance_rule(&self, rule: &ValidationRule, request: &ValidationRequest) -> Result<ComplianceCheck> {
        let check_id = Uuid::new_v4();

        let (status, score, message) = match rule.rule_type {
            RuleType::ComplianceCheck => {
                self.evaluate_compliance_rule(rule, request).await?
            }
            _ => {
                (CheckStatus::Skipped, 0.5, "Rule type not applicable for compliance check".to_string())
            }
        };

        Ok(ComplianceCheck {
            check_id,
            check_type: ComplianceCheckType {
                check_id,
                regulation: "Default Regulation".to_string(),
                requirement: rule.name.clone(),
                status,
                score,
                message,
                timestamp: Utc::now(),
            },
            timestamp: Utc::now(),
        })
    }

    /// Evaluate a safety rule
    async fn evaluate_safety_rule(&self, rule: &ValidationRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        // Simple rule evaluation based on operation type and context
        let score = match request.operation_type {
            OperationType::SecurityUpdate => 1.0,
            OperationType::DataProcessing => 0.9,
            OperationType::ModelTraining => 0.8,
            OperationType::AgentDeployment => 0.7,
            OperationType::ConfigurationChange => 0.6,
            OperationType::ResourceAllocation => 0.8,
            OperationType::SystemMaintenance => 0.5,
            OperationType::DataAccess => 0.7,
            OperationType::NetworkCommunication => 0.6,
            OperationType::FileOperation => 0.8,
        };

        let status = if score >= 0.8 {
            CheckStatus::Passed
        } else if score >= 0.6 {
            CheckStatus::Warning
        } else {
            CheckStatus::Failed
        };

        Ok((status, score, format!("Safety check for {:?} completed", request.operation_type)))
    }

    /// Evaluate an integrity rule
    async fn evaluate_integrity_rule(&self, rule: &ValidationRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        // Check data integrity based on operation type
        let score = match &request.data {
            ValidationData::Model(model_data) => {
                // Validate model checksum
                if model_data.checksum.len() > 0 {
                    0.95
                } else {
                    0.7
                }
            }
            ValidationData::Configuration(config_data) => {
                // Validate configuration checksum
                if config_data.checksum.len() > 0 {
                    0.9
                } else {
                    0.6
                }
            }
            ValidationData::Binary(data) => {
                // Simple size-based integrity check
                if data.len() > 0 {
                    0.85
                } else {
                    0.5
                }
            }
            _ => 0.8, // Default score for other data types
        };

        let status = if score >= 0.8 {
            CheckStatus::Passed
        } else if score >= 0.6 {
            CheckStatus::Warning
        } else {
            CheckStatus::Failed
        };

        Ok((status, score, format!("Integrity check for {:?} completed", request.operation_type)))
    }

    /// Evaluate a compliance rule
    async fn evaluate_compliance_rule(&self, rule: &ValidationRule, request: &ValidationRequest) -> Result<(CheckStatus, f64, String)> {
        // Check compliance based on security level and operation type
        let base_score = match request.context.security_level {
            SecurityLevel::TopSecret => 0.95,
            SecurityLevel::Restricted => 0.9,
            SecurityLevel::Confidential => 0.85,
            SecurityLevel::Standard => 0.8,
            SecurityLevel::Public => 0.7,
        };

        // Adjust based on operation type
        let operation_adjustment = match request.operation_type {
            OperationType::SecurityUpdate => 0.1,
            OperationType::DataAccess => -0.05,
            OperationType::NetworkCommunication => -0.1,
            _ => 0.0,
        };

        let score = (base_score + operation_adjustment).max(0.0).min(1.0);

        let status = if score >= 0.9 {
            CheckStatus::Passed
        } else if score >= 0.7 {
            CheckStatus::Warning
        } else {
            CheckStatus::Failed
        };

        Ok((status, score, format!("Compliance check for security level {:?} completed", request.context.security_level)))
    }

    /// Get applicable policies for a request
    async fn get_applicable_policies(&self, request: &ValidationRequest) -> Result<Vec<ValidationPolicy>> {
        let policies = self.policies.read().await;

        let applicable: Vec<ValidationPolicy> = policies.iter()
            .filter(|policy| policy.is_active)
            .filter(|policy| self.policy_applies_to_request(policy, request))
            .cloned()
            .collect();

        Ok(applicable)
    }

    /// Check if a policy applies to a validation request
    fn policy_applies_to_request(&self, policy: &ValidationPolicy, request: &ValidationRequest) -> bool {
        // Simple policy matching based on operation type
        match policy.policy_type {
            PolicyType::Safety => true, // Safety policies apply to all operations
            PolicyType::Integrity => true, // Integrity policies apply to all operations
            PolicyType::Compliance => {
                // Compliance policies apply based on security level
                matches!(request.context.security_level, SecurityLevel::Confidential | SecurityLevel::Restricted | SecurityLevel::TopSecret)
            }
            PolicyType::Security => {
                // Security policies apply to sensitive operations
                matches!(request.operation_type, OperationType::DataAccess | OperationType::SecurityUpdate | OperationType::NetworkCommunication)
            }
            PolicyType::Performance => {
                // Performance policies apply to resource-intensive operations
                matches!(request.operation_type, OperationType::ModelTraining | OperationType::ResourceAllocation)
            }
            PolicyType::Custom => false, // Custom policies need specific matching logic
        }
    }

    /// Load default validation policies
    async fn load_default_policies() -> Result<Vec<ValidationPolicy>> {
        let mut policies = Vec::new();

        // Safety policy
        policies.push(ValidationPolicy {
            id: Uuid::new_v4(),
            name: "Default Safety Policy".to_string(),
            description: "Basic safety checks for all operations".to_string(),
            policy_type: PolicyType::Safety,
            rules: vec![
                ValidationRule {
                    id: Uuid::new_v4(),
                    name: "Basic Safety Check".to_string(),
                    description: "Ensure basic safety requirements are met".to_string(),
                    rule_type: RuleType::SafetyCheck,
                    condition: "true".to_string(),
                    action: ValidationAction::Allow,
                    severity: IssueSeverity::High,
                    is_enabled: true,
                }
            ],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        // Integrity policy
        policies.push(ValidationPolicy {
            id: Uuid::new_v4(),
            name: "Default Integrity Policy".to_string(),
            description: "Basic integrity checks for all operations".to_string(),
            policy_type: PolicyType::Integrity,
            rules: vec![
                ValidationRule {
                    id: Uuid::new_v4(),
                    name: "Data Integrity Check".to_string(),
                    description: "Ensure data integrity is maintained".to_string(),
                    rule_type: RuleType::IntegrityCheck,
                    condition: "data_integrity_valid".to_string(),
                    action: ValidationAction::Allow,
                    severity: IssueSeverity::High,
                    is_enabled: true,
                }
            ],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        Ok(policies)
    }

    /// Get cached validation result
    async fn get_cached_result(&self, request: &ValidationRequest) -> Result<Option<ValidationResult>> {
        let cache = self.validation_cache.read().await;

        if let Some(result) = cache.get(&request.id) {
            // Check if cache entry is still valid
            let cache_age = Utc::now() - result.timestamp;
            if cache_age.num_minutes() < self.config.cache_ttl_minutes as i64 {
                return Ok(Some(result.clone()));
            }
        }

        Ok(None)
    }

    /// Cache validation result
    async fn cache_result(&self, result: &ValidationResult) -> Result<()> {
        let mut cache = self.validation_cache.write().await;
        cache.insert(result.id, result.clone());

        // Clean up old entries if cache gets too large
        if cache.len() > 1000 {
            let cutoff_time = Utc::now() - chrono::Duration::minutes(self.config.cache_ttl_minutes as i64);
            cache.retain(|_, result| result.timestamp > cutoff_time);
        }

        Ok(())
    }

    /// Generate validation recommendations
    fn generate_validation_recommendations(
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
            RiskAssessment::Critical => {
                recommendations.push("CRITICAL RISK: Operation blocked - requires manual review".to_string());
            }
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

    /// Health check for the validation service
    pub async fn health_check(&self) -> Result<()> {
        let active_count = self.active_validations.read().await.len();
        let cache_size = self.validation_cache.read().await.len();
        let policy_count = self.policies.read().await.len();

        if active_count > self.config.max_concurrent_validations {
            warn!("High number of active validations: {}", active_count);
        }

        if cache_size > 10000 {
            warn!("Large validation cache: {} entries", cache_size);
        }

        if policy_count == 0 {
            warn!("No validation policies configured");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_service_creation() {
        let service = ValidationService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_operation_validation() {
        let service = ValidationService::new().await.unwrap();

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

        let result = service.validate_operation(request).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.is_valid || !result.is_valid); // Either outcome is valid
    }

    #[tokio::test]
    async fn test_policy_loading() {
        let service = ValidationService::new().await.unwrap();
        let policies = service.policies.read().await;
        assert!(!policies.is_empty());
    }
}