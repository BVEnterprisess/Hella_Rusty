//! # Metrics Collection
//!
//! This module provides comprehensive metrics collection and monitoring capabilities
//! for Layer 1 (Discovery). It tracks performance, health, and operational metrics
//! for all discovery components.

use crate::types::*;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_gauge, register_histogram, Counter, Encoder, Gauge, Histogram,
    Registry, TextEncoder,
};
use std::collections::HashMap;
use tracing::{debug, error, info};

lazy_static! {
    /// Prometheus registry for Layer 1 metrics
    static ref REGISTRY: Registry = Registry::new();

    /// Counter for total systems discovered
    static ref SYSTEMS_DISCOVERED: Counter = register_counter!(
        "layer1_systems_discovered_total",
        "Total number of systems discovered by Layer 1"
    ).expect("Can't create systems_discovered metric");

    /// Counter for health checks performed
    static ref HEALTH_CHECKS_TOTAL: Counter = register_counter!(
        "layer1_health_checks_total",
        "Total number of health checks performed"
    ).expect("Can't create health_checks_total metric");

    /// Counter for data points collected
    static ref DATA_POINTS_COLLECTED: Counter = register_counter!(
        "layer1_data_points_collected_total",
        "Total number of data points collected"
    ).expect("Can't create data_points_collected metric");

    /// Counter for alerts generated
    static ref ALERTS_GENERATED: Counter = register_counter!(
        "layer1_alerts_generated_total",
        "Total number of alerts generated"
    ).expect("Can't create alerts_generated metric");

    /// Gauge for current active systems
    static ref ACTIVE_SYSTEMS: Gauge = register_gauge!(
        "layer1_active_systems",
        "Number of currently active systems"
    ).expect("Can't create active_systems metric");

    /// Gauge for current data sources
    static ref ACTIVE_DATA_SOURCES: Gauge = register_gauge!(
        "layer1_active_data_sources",
        "Number of currently active data sources"
    ).expect("Can't create active_data_sources metric");

    /// Histogram for scan duration
    static ref SCAN_DURATION_SECONDS: Histogram = register_histogram!(
        "layer1_scan_duration_seconds",
        "Duration of environmental scans in seconds"
    ).expect("Can't create scan_duration_seconds metric");

    /// Histogram for health check duration
    static ref HEALTH_CHECK_DURATION_SECONDS: Histogram = register_histogram!(
        "layer1_health_check_duration_seconds",
        "Duration of health checks in seconds"
    ).expect("Can't create health_check_duration_seconds metric");

    /// Histogram for data collection duration
    static ref COLLECTION_DURATION_SECONDS: Histogram = register_histogram!(
        "layer1_collection_duration_seconds",
        "Duration of data collection cycles in seconds"
    ).expect("Can't create collection_duration_seconds metric");

    /// Gauge for system health score
    static ref SYSTEM_HEALTH_SCORE: Gauge = register_gauge!(
        "layer1_system_health_score",
        "Overall system health score (0.0 to 1.0)"
    ).expect("Can't create system_health_score metric");

    /// Counter for scan errors
    static ref SCAN_ERRORS_TOTAL: Counter = register_counter!(
        "layer1_scan_errors_total",
        "Total number of scan errors"
    ).expect("Can't create scan_errors_total metric");

    /// Counter for collection errors
    static ref COLLECTION_ERRORS_TOTAL: Counter = register_counter!(
        "layer1_collection_errors_total",
        "Total number of collection errors"
    ).expect("Can't create collection_errors_total metric");
}

/// Metrics collector for Layer 1
pub struct MetricsCollector {
    registry: Registry,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Result<Self, DiscoveryError> {
        Ok(Self {
            registry: REGISTRY.clone(),
        })
    }

    /// Record a system discovery
    pub fn record_system_discovered(&self, system_type: &str) {
        SYSTEMS_DISCOVERED.inc();
        ACTIVE_SYSTEMS.inc();

        debug!("Recorded system discovery: {}", system_type);
    }

    /// Record a health check
    pub fn record_health_check(&self, duration_seconds: f64, status: &HealthStatus) {
        HEALTH_CHECKS_TOTAL.inc();
        HEALTH_CHECK_DURATION_SECONDS.observe(duration_seconds);

        match status {
            HealthStatus::Healthy => {
                debug!("Recorded healthy health check");
            }
            HealthStatus::Warning => {
                warn!("Recorded health check with warning");
            }
            HealthStatus::Critical => {
                error!("Recorded critical health check");
            }
            HealthStatus::Unknown => {
                debug!("Recorded health check with unknown status");
            }
        }
    }

    /// Record data collection
    pub fn record_data_collection(&self, data_points: u64, duration_seconds: f64) {
        DATA_POINTS_COLLECTED.inc_by(data_points as f64);
        COLLECTION_DURATION_SECONDS.observe(duration_seconds);

        debug!("Recorded data collection: {} points in {:.3}s", data_points, duration_seconds);
    }

    /// Record an alert
    pub fn record_alert(&self, severity: &AlertSeverity) {
        ALERTS_GENERATED.inc();

        match severity {
            AlertSeverity::Info => {
                debug!("Recorded info alert");
            }
            AlertSeverity::Warning => {
                warn!("Recorded warning alert");
            }
            AlertSeverity::Error => {
                error!("Recorded error alert");
            }
            AlertSeverity::Critical => {
                error!("Recorded critical alert");
            }
        }
    }

    /// Record a scan operation
    pub fn record_scan(&self, duration_seconds: f64, systems_found: u32, errors: u32) {
        SCAN_DURATION_SECONDS.observe(duration_seconds);
        ACTIVE_SYSTEMS.set(systems_found as f64);

        if errors > 0 {
            SCAN_ERRORS_TOTAL.inc_by(errors as f64);
        }

        debug!(
            "Recorded scan: {:.3}s duration, {} systems, {} errors",
            duration_seconds, systems_found, errors
        );
    }

    /// Record a collection error
    pub fn record_collection_error(&self, error_type: &CollectionErrorType) {
        COLLECTION_ERRORS_TOTAL.inc();

        match error_type {
            CollectionErrorType::ConnectionError => {
                warn!("Recorded connection error");
            }
            CollectionErrorType::AuthenticationError => {
                error!("Recorded authentication error");
            }
            CollectionErrorType::TimeoutError => {
                warn!("Recorded timeout error");
            }
            CollectionErrorType::DataFormatError => {
                warn!("Recorded data format error");
            }
            CollectionErrorType::PermissionError => {
                error!("Recorded permission error");
            }
            CollectionErrorType::Custom(_) => {
                warn!("Recorded custom error");
            }
        }
    }

    /// Update system health score
    pub fn update_system_health_score(&self, score: f64) {
        SYSTEM_HEALTH_SCORE.set(score);
        debug!("Updated system health score: {:.3}", score);
    }

    /// Update active data sources count
    pub fn update_active_data_sources(&self, count: f64) {
        ACTIVE_DATA_SOURCES.set(count);
        debug!("Updated active data sources: {}", count);
    }

    /// Get current metrics as Prometheus format
    pub fn get_metrics(&self) -> Result<String, DiscoveryError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let result = encoder.encode_to_string(&metric_families)?;

        Ok(result)
    }

    /// Get metrics summary as a hashmap
    pub fn get_metrics_summary(&self) -> Result<HashMap<String, f64>, DiscoveryError> {
        let mut summary = HashMap::new();

        // Get counter values
        summary.insert("systems_discovered_total".to_string(), SYSTEMS_DISCOVERED.get());
        summary.insert("health_checks_total".to_string(), HEALTH_CHECKS_TOTAL.get());
        summary.insert("data_points_collected_total".to_string(), DATA_POINTS_COLLECTED.get());
        summary.insert("alerts_generated_total".to_string(), ALERTS_GENERATED.get());
        summary.insert("scan_errors_total".to_string(), SCAN_ERRORS_TOTAL.get());
        summary.insert("collection_errors_total".to_string(), COLLECTION_ERRORS_TOTAL.get());

        // Get gauge values
        summary.insert("active_systems".to_string(), ACTIVE_SYSTEMS.get());
        summary.insert("active_data_sources".to_string(), ACTIVE_DATA_SOURCES.get());
        summary.insert("system_health_score".to_string(), SYSTEM_HEALTH_SCORE.get());

        Ok(summary)
    }

    /// Reset all metrics (for testing)
    pub fn reset_metrics(&self) {
        SYSTEMS_DISCOVERED.reset();
        HEALTH_CHECKS_TOTAL.reset();
        DATA_POINTS_COLLECTED.reset();
        ALERTS_GENERATED.reset();
        SCAN_ERRORS_TOTAL.reset();
        COLLECTION_ERRORS_TOTAL.reset();
        ACTIVE_SYSTEMS.set(0.0);
        ACTIVE_DATA_SOURCES.set(0.0);
        SYSTEM_HEALTH_SCORE.set(1.0);

        debug!("Reset all metrics");
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics collector")
    }
}

/// Performance timer for measuring operation duration
pub struct PerformanceTimer {
    start_time: std::time::Instant,
    operation_name: String,
}

impl PerformanceTimer {
    /// Start a new performance timer
    pub fn start(operation_name: String) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            operation_name,
        }
    }

    /// Stop the timer and return the duration in seconds
    pub fn stop(self) -> f64 {
        let duration = self.start_time.elapsed();
        duration.as_secs_f64()
    }

    /// Stop the timer and record the duration in a histogram
    pub fn stop_and_record(self, histogram: &Histogram) -> f64 {
        let duration = self.stop();
        histogram.observe(duration);
        duration
    }
}

/// Scoped metrics recorder that automatically records duration
pub struct ScopedMetricsRecorder {
    timer: PerformanceTimer,
    histogram: Histogram,
}

impl ScopedMetricsRecorder {
    /// Create a new scoped metrics recorder
    pub fn new(operation_name: String, histogram: Histogram) -> Self {
        Self {
            timer: PerformanceTimer::start(operation_name),
            histogram,
        }
    }
}

impl Drop for ScopedMetricsRecorder {
    fn drop(&mut self) {
        let duration = self.timer.stop();
        self.histogram.observe(duration);
    }
}

/// Create a scoped metrics recorder for an operation
#[macro_export]
macro_rules! record_duration {
    ($histogram:expr, $operation:expr) => {
        let _recorder = ScopedMetricsRecorder::new($operation.to_string(), $histogram);
    };
}

/// Metrics utilities
pub struct MetricsUtils;

impl MetricsUtils {
    /// Calculate health score from various metrics
    pub fn calculate_health_score(
        healthy_checks: u32,
        total_checks: u32,
        avg_response_time_ms: f64,
        error_rate: f64,
    ) -> f64 {
        if total_checks == 0 {
            return 1.0;
        }

        let health_ratio = healthy_checks as f64 / total_checks as f64;
        let response_time_score = if avg_response_time_ms < 100.0 {
            1.0
        } else if avg_response_time_ms < 500.0 {
            0.8
        } else if avg_response_time_ms < 1000.0 {
            0.6
        } else {
            0.4
        };

        let error_score = 1.0 - error_rate;

        (health_ratio * 0.5 + response_time_score * 0.3 + error_score * 0.2).min(1.0).max(0.0)
    }

    /// Calculate data quality score
    pub fn calculate_data_quality_score(
        valid_points: u64,
        total_points: u64,
        avg_latency_ms: f64,
        completeness_ratio: f64,
    ) -> f64 {
        if total_points == 0 {
            return 1.0;
        }

        let validity_score = valid_points as f64 / total_points as f64;
        let latency_score = if avg_latency_ms < 50.0 {
            1.0
        } else if avg_latency_ms < 200.0 {
            0.8
        } else {
            0.5
        };

        (validity_score * 0.4 + latency_score * 0.3 + completeness_ratio * 0.3).min(1.0).max(0.0)
    }

    /// Format metrics for logging
    pub fn format_metrics_for_log(metrics: &HashMap<String, f64>) -> String {
        let mut parts = Vec::new();
        for (key, value) in metrics.iter() {
            parts.push(format!("{}={:.3}", key, value));
        }
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.is_ok());
    }

    #[test]
    fn test_health_score_calculation() {
        // Perfect health
        let score = MetricsUtils::calculate_health_score(100, 100, 50.0, 0.0);
        assert!((score - 1.0).abs() < 0.001);

        // Poor health
        let score = MetricsUtils::calculate_health_score(10, 100, 2000.0, 0.5);
        assert!(score < 0.5);

        // No checks
        let score = MetricsUtils::calculate_health_score(0, 0, 100.0, 0.1);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_data_quality_score_calculation() {
        // High quality data
        let score = MetricsUtils::calculate_data_quality_score(1000, 1000, 25.0, 1.0);
        assert!(score > 0.9);

        // Poor quality data
        let score = MetricsUtils::calculate_data_quality_score(100, 1000, 500.0, 0.3);
        assert!(score < 0.5);

        // No data
        let score = MetricsUtils::calculate_data_quality_score(0, 0, 100.0, 1.0);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::start("test_operation".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.stop();

        assert!(duration >= 0.01);
        assert!(duration < 1.0); // Should be much less than 1 second
    }

    #[test]
    fn test_metrics_formatting() {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 75.5);
        metrics.insert("memory_mb".to_string(), 1024.0);
        metrics.insert("active_connections".to_string(), 25.0);

        let formatted = MetricsUtils::format_metrics_for_log(&metrics);
        assert!(formatted.contains("cpu_usage=75.5"));
        assert!(formatted.contains("memory_mb=1024"));
        assert!(formatted.contains("active_connections=25"));
    }
}