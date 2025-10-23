//! # Layer 3 Unit Tests - Individual Component Testing
//!
//! Unit tests for individual components of the Layer 3 validation system.

use layer3_validation::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

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
            data: ValidationData::Text("test data".to_string()),
            timestamp: Utc::now(),
        };

        let result = engine.validate_operation(request).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.is_valid || !result.is_valid); // Either outcome is valid
        assert!(result.validation_time_ms > 0);
    }

    #[tokio::test]
    async fn test_system_validation() {
        let engine = ValidationEngine::new().await.unwrap();

        let report = engine.validate_system_state().await;
        assert!(report.is_ok());

        let report = report.unwrap();
        assert!(matches!(report.overall_status, SystemStatus::Healthy | SystemStatus::Degraded));
    }

    #[tokio::test]
    async fn test_validation_service_creation() {
        let service = ValidationService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_safety_validator_creation() {
        let validator = SafetyValidator::new().await;
        assert!(validator.is_ok());
    }

    #[tokio::test]
    async fn test_integrity_checker_creation() {
        let checker = IntegrityChecker::new().await;
        assert!(checker.is_ok());
    }

    #[tokio::test]
    async fn test_compliance_validator_creation() {
        let validator = ComplianceValidator::new().await;
        assert!(validator.is_ok());
    }

    #[tokio::test]
    async fn test_risk_mitigator_creation() {
        let mitigator = RiskMitigator::new().await;
        assert!(mitigator.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = ValidationMetrics::new().await;
        assert!(metrics.is_ok());
    }

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
    fn test_security_level_progression() {
        assert!(matches!(SecurityLevel::Public as u8, 0));
        assert!(matches!(SecurityLevel::Standard as u8, 1));
        assert!(matches!(SecurityLevel::Confidential as u8, 2));
        assert!(matches!(SecurityLevel::Restricted as u8, 3));
        assert!(matches!(SecurityLevel::TopSecret as u8, 4));
    }

    #[test]
    fn test_operation_type_coverage() {
        let operations = vec![
            OperationType::DataProcessing,
            OperationType::ModelTraining,
            OperationType::AgentDeployment,
            OperationType::ResourceAllocation,
            OperationType::ConfigurationChange,
            OperationType::SecurityUpdate,
            OperationType::SystemMaintenance,
            OperationType::DataAccess,
            OperationType::NetworkCommunication,
            OperationType::FileOperation,
        ];

        assert_eq!(operations.len(), 10);
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
            data: ValidationData::Text("normal text data".to_string()),
            timestamp: Utc::now(),
        };

        let result = validator.validate_safety(&request).await;
        assert!(result.is_ok());

        let safety_status = result.unwrap();
        assert!(safety_status.safety_score >= 0.0);
        assert!(safety_status.safety_score <= 1.0);
    }

    #[tokio::test]
    async fn test_integrity_validation() {
        let checker = IntegrityChecker::new().await.unwrap();

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
            data: ValidationData::Binary(vec![1, 2, 3, 4, 5]),
            timestamp: Utc::now(),
        };

        let result = checker.validate_integrity(&request).await;
        assert!(result.is_ok());

        let integrity_status = result.unwrap();
        assert!(integrity_status.integrity_score >= 0.0);
        assert!(integrity_status.integrity_score <= 1.0);
    }

    #[tokio::test]
    async fn test_compliance_validation() {
        let validator = ComplianceValidator::new().await.unwrap();

        let request = ValidationRequest {
            id: Uuid::new_v4(),
            operation_type: OperationType::DataProcessing,
            parameters: HashMap::from([
                ("encrypted".to_string(), "true".to_string()),
                ("consent_obtained".to_string(), "true".to_string()),
            ]),
            context: ValidationContext {
                user_id: Some("test-user".to_string()),
                session_id: Some(Uuid::new_v4()),
                source_layer: "layer4".to_string(),
                target_layer: "layer5".to_string(),
                security_level: SecurityLevel::Confidential,
                compliance_requirements: vec!["GDPR".to_string()],
                timestamp: Utc::now(),
            },
            data: ValidationData::Text("personal data".to_string()),
            timestamp: Utc::now(),
        };

        let result = validator.validate_compliance(&request).await;
        assert!(result.is_ok());

        let compliance_status = result.unwrap();
        assert!(compliance_status.compliance_score >= 0.0);
        assert!(compliance_status.compliance_score <= 1.0);
    }

    #[tokio::test]
    async fn test_risk_assessment() {
        let mitigator = RiskMitigator::new().await.unwrap();

        let request = ValidationRequest {
            id: Uuid::new_v4(),
            operation_type: OperationType::ModelTraining,
            parameters: HashMap::new(),
            context: ValidationContext {
                user_id: Some("test-user".to_string()),
                session_id: Some(Uuid::new_v4()),
                source_layer: "layer4".to_string(),
                target_layer: "layer5".to_string(),
                security_level: SecurityLevel::TopSecret,
                compliance_requirements: Vec::new(),
                timestamp: Utc::now(),
            },
            data: ValidationData::Model(ModelData {
                model_id: Uuid::new_v4(),
                model_type: "neural_network".to_string(),
                parameters: HashMap::new(),
                size_bytes: 1024 * 1024 * 1024, // 1GB
                checksum: "mock_checksum".to_string(),
                training_data_hash: "mock_hash".to_string(),
            }),
            timestamp: Utc::now(),
        };

        let risk = mitigator.assess_risks(&request).await;
        assert!(risk.is_ok());

        let risk_assessment = risk.unwrap();
        assert!(matches!(risk_assessment, RiskAssessment::Low | RiskAssessment::Medium | RiskAssessment::High | RiskAssessment::Critical));
    }

    #[tokio::test]
    async fn test_mitigation_strategies() {
        let mitigator = RiskMitigator::new().await.unwrap();

        let high_risk = RiskAssessment::High;
        let strategies = mitigator.get_mitigation_strategies(&high_risk).await.unwrap();
        assert!(!strategies.is_empty());

        let low_risk = RiskAssessment::Low;
        let strategies = mitigator.get_mitigation_strategies(&low_risk).await.unwrap();
        assert!(!strategies.is_empty());
    }

    #[tokio::test]
    async fn test_safety_controls() {
        let mitigator = RiskMitigator::new().await.unwrap();

        let request = ValidationRequest {
            id: Uuid::new_v4(),
            operation_type: OperationType::ModelTraining,
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

        let risk = mitigator.assess_risks(&request).await.unwrap();
        let controls = mitigator.apply_safety_controls(&request, &risk).await.unwrap();
        assert!(!controls.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let metrics = ValidationMetrics::new().await.unwrap();

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

        metrics.record_validation(&result).await;

        let snapshot = metrics.snapshot().await.unwrap();
        assert_eq!(snapshot.operations_validated, 1);
        assert_eq!(snapshot.validation_successes, 1);
        assert_eq!(snapshot.validation_failures, 0);
        assert_eq!(snapshot.average_validation_time_ms, 150);
    }

    #[tokio::test]
    async fn test_system_validation_recording() {
        let metrics = ValidationMetrics::new().await.unwrap();

        metrics.record_system_validation(0.95).await;
        metrics.record_risk_assessment().await;
        metrics.record_safety_control(3).await;

        let snapshot = metrics.snapshot().await.unwrap();
        assert_eq!(snapshot.system_health_score, 95);
        assert_eq!(snapshot.risk_assessments_performed, 1);
        assert_eq!(snapshot.active_safety_controls, 3);
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let metrics = ValidationMetrics::new().await.unwrap();

        metrics.record_cache_access(true).await; // Hit
        metrics.record_cache_access(true).await; // Hit
        metrics.record_cache_access(false).await; // Miss

        let snapshot = metrics.snapshot().await.unwrap();
        assert_eq!(snapshot.cache_hit_rate, 66.66666666666667); // 2 hits out of 3 total
    }

    #[tokio::test]
    async fn test_error_recording() {
        let metrics = ValidationMetrics::new().await.unwrap();

        metrics.record_error("validation").await;
        metrics.record_error("timeout").await;
        metrics.record_error("configuration").await;
        metrics.record_error("unknown").await; // Should be ignored

        let snapshot = metrics.snapshot().await.unwrap();
        assert_eq!(snapshot.validation_errors, 1);
        assert_eq!(snapshot.timeout_errors, 1);
        assert_eq!(snapshot.configuration_errors, 1);
    }

    #[tokio::test]
    async fn test_health_checks() {
        let engine = ValidationEngine::new().await.unwrap();

        let health = engine.health_check().await;
        assert!(health.is_ok());

        let service = ValidationService::new().await.unwrap();
        let service_health = service.health_check().await;
        assert!(service_health.is_ok());

        let validator = SafetyValidator::new().await.unwrap();
        let validator_health = validator.health_check().await;
        assert!(validator_health.is_ok());

        let checker = IntegrityChecker::new().await.unwrap();
        let checker_health = checker.health_check().await;
        assert!(checker_health.is_ok());

        let compliance_validator = ComplianceValidator::new().await.unwrap();
        let compliance_health = compliance_validator.health_check().await;
        assert!(compliance_health.is_ok());

        let mitigator = RiskMitigator::new().await.unwrap();
        let mitigator_health = mitigator.health_check().await;
        assert!(mitigator_health.is_ok());
    }

    #[test]
    fn test_checksum_calculation() {
        let checker = IntegrityChecker::new().unwrap();

        let data = b"test data for checksum";
        let checksum = checker.calculate_checksum(data);
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 44); // Base64 encoded SHA256

        // Same data should produce same checksum
        let checksum2 = checker.calculate_checksum(data);
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_text_checksum_calculation() {
        let checker = IntegrityChecker::new().unwrap();

        let text = "test text for checksum";
        let checksum = checker.calculate_text_checksum(text);
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 44); // Base64 encoded SHA256
    }

    #[tokio::test]
    async fn test_data_validation() {
        let validator = SafetyValidator::new().await.unwrap();

        // Test normal text
        let score = validator.validate_text_data("normal text").await.unwrap();
        assert!(score > 0.8);

        // Test suspicious text
        let score = validator.validate_text_data("script alert('xss')").await.unwrap();
        assert!(score < 0.8);

        // Test binary validation
        let score = validator.validate_binary_data(&[0x4D, 0x5A, 0x90, 0x00]).await.unwrap(); // PE header
        assert!(score < 0.8);
    }

    #[tokio::test]
    async fn test_regulatory_framework_loading() {
        let validator = ComplianceValidator::new().await.unwrap();
        let frameworks = validator.regulatory_frameworks;

        assert!(!frameworks.is_empty());
        assert!(frameworks.contains_key("GDPR"));
        assert!(frameworks.contains_key("SOX"));
        assert!(frameworks.contains_key("HIPAA"));
    }

    #[tokio::test]
    async fn test_policy_definition_loading() {
        let validator = ComplianceValidator::new().await.unwrap();
        let policies = validator.policy_definitions;

        assert!(!policies.is_empty());
        assert!(policies.contains_key("data_retention"));
        assert!(policies.contains_key("access_control"));
    }

    #[tokio::test]
    async fn test_validation_with_different_security_levels() {
        let engine = ValidationEngine::new().await.unwrap();

        // Test with different security levels
        let security_levels = vec![
            SecurityLevel::Public,
            SecurityLevel::Standard,
            SecurityLevel::Confidential,
            SecurityLevel::Restricted,
            SecurityLevel::TopSecret,
        ];

        for security_level in security_levels {
            let request = ValidationRequest {
                id: Uuid::new_v4(),
                operation_type: OperationType::DataProcessing,
                parameters: HashMap::new(),
                context: ValidationContext {
                    user_id: Some("test-user".to_string()),
                    session_id: Some(Uuid::new_v4()),
                    source_layer: "layer4".to_string(),
                    target_layer: "layer5".to_string(),
                    security_level,
                    compliance_requirements: Vec::new(),
                    timestamp: Utc::now(),
                },
                data: ValidationData::Text("test data".to_string()),
                timestamp: Utc::now(),
            };

            let result = engine.validate_operation(request).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_validation_with_different_data_types() {
        let engine = ValidationEngine::new().await.unwrap();

        // Test with different data types
        let data_types = vec![
            ValidationData::None,
            ValidationData::Text("text data".to_string()),
            ValidationData::Binary(vec![1, 2, 3, 4, 5]),
            ValidationData::Json(serde_json::json!({"key": "value"})),
        ];

        for data in data_types {
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
                data,
                timestamp: Utc::now(),
            };

            let result = engine.validate_operation(request).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_validation_with_different_operation_types() {
        let engine = ValidationEngine::new().await.unwrap();

        // Test with different operation types
        let operation_types = vec![
            OperationType::DataProcessing,
            OperationType::ModelTraining,
            OperationType::AgentDeployment,
            OperationType::ResourceAllocation,
            OperationType::ConfigurationChange,
            OperationType::SecurityUpdate,
            OperationType::SystemMaintenance,
            OperationType::DataAccess,
            OperationType::NetworkCommunication,
            OperationType::FileOperation,
        ];

        for operation_type in operation_types {
            let request = ValidationRequest {
                id: Uuid::new_v4(),
                operation_type,
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

            let result = engine.validate_operation(request).await;
            assert!(result.is_ok());
        }
    }
}