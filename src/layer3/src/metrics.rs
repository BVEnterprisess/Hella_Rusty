use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Validation metrics collector
pub struct ValidationMetrics {
    // Operation metrics
    operations_validated: std::sync::atomic::AtomicU64,
    validation_successes: std::sync::atomic::AtomicU64,
    validation_failures: std::sync::atomic::AtomicU64,
    safety_violations: std::sync::atomic::AtomicU64,
    integrity_failures: std::sync::atomic::AtomicU64,
    compliance_violations: std::sync::atomic::AtomicU64,

    // Performance metrics
    validation_duration_seconds: std::sync::atomic::AtomicU64,
    average_validation_time_ms: std::sync::atomic::AtomicU64,

    // System health metrics
    system_health_score: std::sync::atomic::AtomicU64,
    active_safety_controls: std::sync::atomic::AtomicU64,
    risk_assessments_performed: std::sync::atomic::AtomicU64,

    // Cache metrics
    cache_hits: std::sync::atomic::AtomicU64,
    cache_misses: std::sync::atomic::AtomicU64,

    // Error metrics
    validation_errors: std::sync::atomic::AtomicU64,
    timeout_errors: std::sync::atomic::AtomicU64,
    configuration_errors: std::sync::atomic::AtomicU64,
}

impl ValidationMetrics {
    /// Create a new metrics collector
    pub async fn new() -> Result<Self> {
        Ok(Self {
            operations_validated: std::sync::atomic::AtomicU64::new(0),
            validation_successes: std::sync::atomic::AtomicU64::new(0),
            validation_failures: std::sync::atomic::AtomicU64::new(0),
            safety_violations: std::sync::atomic::AtomicU64::new(0),
            integrity_failures: std::sync::atomic::AtomicU64::new(0),
            compliance_violations: std::sync::atomic::AtomicU64::new(0),
            validation_duration_seconds: std::sync::atomic::AtomicU64::new(0),
            average_validation_time_ms: std::sync::atomic::AtomicU64::new(0),
            system_health_score: std::sync::atomic::AtomicU64::new(100),
            active_safety_controls: std::sync::atomic::AtomicU64::new(0),
            risk_assessments_performed: std::sync::atomic::AtomicU64::new(0),
            cache_hits: std::sync::atomic::AtomicU64::new(0),
            cache_misses: std::sync::atomic::AtomicU64::new(0),
            validation_errors: std::sync::atomic::AtomicU64::new(0),
            timeout_errors: std::sync::atomic::AtomicU64::new(0),
            configuration_errors: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// Get metrics snapshot
    pub async fn snapshot(&self) -> Result<ValidationMetricsSnapshot> {
        let operations_validated = self.operations_validated.load(std::sync::atomic::Ordering::Relaxed);
        let validation_successes = self.validation_successes.load(std::sync::atomic::Ordering::Relaxed);
        let validation_failures = self.validation_failures.load(std::sync::atomic::Ordering::Relaxed);
        let safety_violations = self.safety_violations.load(std::sync::atomic::Ordering::Relaxed);
        let integrity_failures = self.integrity_failures.load(std::sync::atomic::Ordering::Relaxed);
        let compliance_violations = self.compliance_violations.load(std::sync::atomic::Ordering::Relaxed);

        let total_validations = validation_successes + validation_failures;
        let success_rate = if total_validations > 0 {
            (validation_successes as f64 / total_validations as f64) * 100.0
        } else {
            100.0
        };

        let total_violations = safety_violations + integrity_failures + compliance_violations;
        let violation_rate = if operations_validated > 0 {
            (total_violations as f64 / operations_validated as f64) * 100.0
        } else {
            0.0
        };

        Ok(ValidationMetricsSnapshot {
            timestamp: Utc::now(),
            operations_validated,
            validation_successes,
            validation_failures,
            success_rate,
            safety_violations,
            integrity_failures,
            compliance_violations,
            violation_rate,
            average_validation_time_ms: self.average_validation_time_ms.load(std::sync::atomic::Ordering::Relaxed),
            system_health_score: self.system_health_score.load(std::sync::atomic::Ordering::Relaxed),
            active_safety_controls: self.active_safety_controls.load(std::sync::atomic::Ordering::Relaxed),
            risk_assessments_performed: self.risk_assessments_performed.load(std::sync::atomic::Ordering::Relaxed),
            cache_hit_rate: self.calculate_cache_hit_rate(),
            validation_errors: self.validation_errors.load(std::sync::atomic::Ordering::Relaxed),
            timeout_errors: self.timeout_errors.load(std::sync::atomic::Ordering::Relaxed),
            configuration_errors: self.configuration_errors.load(std::sync::atomic::Ordering::Relaxed),
        })
    }

    /// Record operation validation
    pub async fn record_validation(&self, result: &ValidationResult) {
        self.operations_validated.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if result.is_valid {
            self.validation_successes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.validation_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        if !result.safety_status.is_safe {
            self.safety_violations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        if !result.integrity_status.is_valid {
            self.integrity_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        if !result.compliance_status.is_compliant {
            self.compliance_violations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        // Update average validation time
        let current_avg = self.average_validation_time_ms.load(std::sync::atomic::Ordering::Relaxed);
        let total_validations = self.operations_validated.load(std::sync::atomic::Ordering::Relaxed);
        let new_avg = ((current_avg as u128 * (total_validations - 1) as u128 + result.validation_time_ms) / total_validations as u128) as u64;
        self.average_validation_time_ms.store(new_avg, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record system validation
    pub async fn record_system_validation(&self, health_score: f64) {
        self.system_health_score.store((health_score * 100.0) as u64, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record risk assessment
    pub async fn record_risk_assessment(&self) {
        self.risk_assessments_performed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record safety control activation
    pub async fn record_safety_control(&self, control_count: u64) {
        self.active_safety_controls.store(control_count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record cache access
    pub async fn record_cache_access(&self, hit: bool) {
        if hit {
            self.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Record validation error
    pub async fn record_error(&self, error_type: &str) {
        match error_type {
            "validation" => self.validation_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            "timeout" => self.timeout_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            "configuration" => self.configuration_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _ => warn!("Unknown error type: {}", error_type),
        }
    }

    /// Calculate cache hit rate
    fn calculate_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.cache_misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        let snapshot = self.snapshot().await?;

        let mut output = String::new();

        output.push_str(&format!("# HELP layer3_operations_validated_total Total operations validated\n"));
        output.push_str(&format!("# TYPE layer3_operations_validated_total counter\n"));
        output.push_str(&format!("layer3_operations_validated_total {}\n", snapshot.operations_validated));

        output.push_str(&format!("# HELP layer3_validation_success_rate_percent Validation success rate\n"));
        output.push_str(&format!("# TYPE layer3_validation_success_rate_percent gauge\n"));
        output.push_str(&format!("layer3_validation_success_rate_percent {}\n", snapshot.success_rate));

        output.push_str(&format!("# HELP layer3_safety_violations_total Total safety violations\n"));
        output.push_str(&format!("# TYPE layer3_safety_violations_total counter\n"));
        output.push_str(&format!("layer3_safety_violations_total {}\n", snapshot.safety_violations));

        output.push_str(&format!("# HELP layer3_system_health_score System health score\n"));
        output.push_str(&format!("# TYPE layer3_system_health_score gauge\n"));
        output.push_str(&format!("layer3_system_health_score {}\n", snapshot.system_health_score));

        output.push_str(&format!("# HELP layer3_average_validation_time_ms Average validation time\n"));
        output.push_str(&format!("# TYPE layer3_average_validation_time_ms gauge\n"));
        output.push_str(&format!("layer3_average_validation_time_ms {}\n", snapshot.average_validation_time_ms));

        Ok(output)
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationMetricsSnapshot {
    pub timestamp: chrono::DateTime<Utc>,
    pub operations_validated: u64,
    pub validation_successes: u64,
    pub validation_failures: u64,
    pub success_rate: f64,
    pub safety_violations: u64,
    pub integrity_failures: u64,
    pub compliance_violations: u64,
    pub violation_rate: f64,
    pub average_validation_time_ms: u64,
    pub system_health_score: u64,
    pub active_safety_controls: u64,
    pub risk_assessments_performed: u64,
    pub cache_hit_rate: f64,
    pub validation_errors: u64,
    pub timeout_errors: u64,
    pub configuration_errors: u64,
}

impl Default for ValidationMetricsSnapshot {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            operations_validated: 0,
            validation_successes: 0,
            validation_failures: 0,
            success_rate: 100.0,
            safety_violations: 0,
            integrity_failures: 0,
            compliance_violations: 0,
            violation_rate: 0.0,
            average_validation_time_ms: 0,
            system_health_score: 100,
            active_safety_controls: 0,
            risk_assessments_performed: 0,
            cache_hit_rate: 0.0,
            validation_errors: 0,
            timeout_errors: 0,
            configuration_errors: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = ValidationMetrics::new().await;
        assert!(metrics.is_ok());
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
}