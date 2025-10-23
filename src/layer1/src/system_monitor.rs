//! # System Monitor
//!
//! The System Monitor provides continuous health checking, performance monitoring, and alerting
//! capabilities for discovered systems. It tracks system performance metrics, detects anomalies,
//! and generates alerts for system issues.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// System monitor for health checking and performance monitoring
pub struct SystemMonitor {
    config: MonitorConfig,
    health_checks: Arc<Mutex<Vec<Box<dyn HealthCheck>>>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    alerts: Arc<Mutex<Vec<Alert>>>,
    is_running: Arc<Mutex<bool>>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub async fn new(config: MonitorConfig) -> Result<Self, DiscoveryError> {
        let health_checks = Arc::new(Mutex::new(Vec::new()));
        let performance_metrics = Arc::new(Mutex::new(PerformanceMetrics {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            disk_usage_percent: 0.0,
            network_io: NetworkIOMetrics {
                bytes_received_per_sec: 0,
                bytes_transmitted_per_sec: 0,
                active_connections: 0,
                connection_errors: 0,
            },
            processes: Vec::new(),
        }));
        let alerts = Arc::new(Mutex::new(Vec::new()));

        let mut monitor = Self {
            config,
            health_checks,
            performance_metrics,
            alerts,
            is_running: Arc::new(Mutex::new(false)),
        };

        // Initialize default health checks
        monitor.initialize_default_checks().await?;

        Ok(monitor)
    }

    /// Start the system monitor
    pub async fn start(&mut self) -> Result<(), DiscoveryError> {
        info!("Starting System Monitor");
        *self.is_running.lock().await = true;

        // Start health checking loop
        let config = self.config.clone();
        let health_checks = self.health_checks.clone();
        let performance_metrics = self.performance_metrics.clone();
        let alerts = self.alerts.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let check_interval = Duration::from_secs(config.check_interval_seconds);
            let mut interval = interval(check_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !*is_running.lock().await {
                            break;
                        }

                        if let Err(e) = Self::perform_health_checks(
                            &config,
                            &health_checks,
                            &performance_metrics,
                            &alerts,
                        ).await {
                            error!("Health check cycle failed: {}", e);
                        }
                    }
                }
            }
        });

        info!("System Monitor started successfully");
        Ok(())
    }

    /// Stop the system monitor
    pub async fn stop(&mut self) -> Result<(), DiscoveryError> {
        info!("Stopping System Monitor");
        *self.is_running.lock().await = false;
        info!("System Monitor stopped successfully");
        Ok(())
    }

    /// Get current monitoring state
    pub async fn get_state(&self) -> Result<MonitoringState, DiscoveryError> {
        let health_checks = self.health_checks.lock().await;
        let performance_metrics = self.performance_metrics.lock().await;
        let alerts = self.alerts.lock().await;

        let mut checks_map = HashMap::new();
        for check in health_checks.iter() {
            let result = check.check_health().await?;
            checks_map.insert(check.get_check_id(), result);
        }

        Ok(MonitoringState {
            health_checks: checks_map,
            performance_metrics: performance_metrics.clone(),
            alerts: alerts.clone(),
            last_update: Utc::now(),
        })
    }

    /// Perform comprehensive health check
    pub async fn full_check(&mut self) -> Result<MonitoringResult, DiscoveryError> {
        info!("Performing full system health check");

        let mut checks_performed = 0;
        let mut healthy_checks = 0;
        let mut warning_checks = 0;
        let mut critical_checks = 0;
        let mut total_duration = 0.0;
        let mut issues = Vec::new();

        let health_checks = self.health_checks.lock().await;
        for check in health_checks.iter() {
            let start_time = std::time::Instant::now();
            match check.check_health().await {
                Ok(health_result) => {
                    checks_performed += 1;
                    let duration = start_time.elapsed().as_secs_f64() * 1000.0;
                    total_duration += duration;

                    match health_result.status {
                        HealthStatus::Healthy => healthy_checks += 1,
                        HealthStatus::Warning => warning_checks += 1,
                        HealthStatus::Critical => critical_checks += 1,
                        HealthStatus::Unknown => {}
                    }

                    if health_result.status != HealthStatus::Healthy {
                        issues.push(MonitoringIssue {
                            check_id: health_result.check_id,
                            system_id: "unknown".to_string(), // Would be populated from actual check
                            description: format!("Health check failed: {:?}", health_result.status),
                            severity: match health_result.status {
                                HealthStatus::Warning => IssueSeverity::Medium,
                                HealthStatus::Critical => IssueSeverity::High,
                                _ => IssueSeverity::Low,
                            },
                            threshold: None,
                            actual_value: None,
                        });
                    }
                }
                Err(e) => {
                    error!("Health check {} failed: {}", check.get_check_name(), e);
                    issues.push(MonitoringIssue {
                        check_id: check.get_check_id(),
                        system_id: "unknown".to_string(),
                        description: format!("Health check error: {}", e),
                        severity: IssueSeverity::High,
                        threshold: None,
                        actual_value: None,
                    });
                }
            }
        }

        let avg_check_duration = if checks_performed > 0 {
            total_duration / checks_performed as f64
        } else {
            0.0
        };

        Ok(MonitoringResult {
            checks_performed,
            healthy_checks,
            warning_checks,
            critical_checks,
            avg_check_duration_ms: avg_check_duration,
            issues,
        })
    }

    /// Get monitor health status
    pub async fn health_check(&self) -> Result<ComponentHealth, DiscoveryError> {
        let is_running = *self.is_running.lock().await;
        let health_checks_count = self.health_checks.lock().await.len();
        let alerts_count = self.alerts.lock().await.len();

        let status = if is_running && alerts_count == 0 {
            ServiceStatus::Healthy
        } else if is_running {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "system-monitor".to_string(),
            status,
            check_duration_ms: 0,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("health_checks_count".to_string(), health_checks_count as f64);
                metrics.insert("active_alerts".to_string(), alerts_count as f64);
                metrics.insert("check_interval_seconds".to_string(), self.config.check_interval_seconds as f64);
                metrics
            },
        })
    }

    /// Add a new health check
    pub async fn add_health_check(&self, check: Box<dyn HealthCheck>) -> Result<(), DiscoveryError> {
        self.health_checks.lock().await.push(check);
        Ok(())
    }

    /// Remove a health check by ID
    pub async fn remove_health_check(&self, check_id: &CheckId) -> Result<(), DiscoveryError> {
        let mut checks = self.health_checks.lock().await;
        checks.retain(|check| check.get_check_id() != check_id);
        Ok(())
    }

    /// Generate alert for system issue
    pub async fn generate_alert(
        &self,
        severity: AlertSeverity,
        title: String,
        description: String,
        system_id: Option<SystemId>,
    ) -> Result<(), DiscoveryError> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            title,
            description,
            system_id,
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };

        self.alerts.lock().await.push(alert);
        info!("Generated alert: {} - {}", alert.title, alert.description);
        Ok(())
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        acknowledged_by: String,
    ) -> Result<(), DiscoveryError> {
        let mut alerts = self.alerts.lock().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by);
            alert.acknowledged_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Initialize default health checks
    async fn initialize_default_checks(&mut self) -> Result<(), DiscoveryError> {
        // CPU usage check
        self.add_health_check(Box::new(CpuUsageCheck::new(
            self.config.cpu_alert_threshold,
        ))).await?;

        // Memory usage check
        self.add_health_check(Box::new(MemoryUsageCheck::new(
            self.config.memory_alert_threshold,
        ))).await?;

        // Disk usage check
        self.add_health_check(Box::new(DiskUsageCheck::new(
            self.config.disk_alert_threshold,
        ))).await?;

        // Network connectivity check
        self.add_health_check(Box::new(NetworkConnectivityCheck::new())).await?;

        info!("Initialized {} default health checks", 4);
        Ok(())
    }

    /// Perform health checks cycle
    async fn perform_health_checks(
        config: &MonitorConfig,
        health_checks: &Arc<Mutex<Vec<Box<dyn HealthCheck>>>>,
        performance_metrics: &Arc<Mutex<PerformanceMetrics>>,
        alerts: &Arc<Mutex<Vec<Alert>>>,
    ) -> Result<(), DiscoveryError> {
        debug!("Starting health check cycle");

        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        let mut total_disk = 0.0;
        let mut checks_performed = 0;

        let checks = health_checks.lock().await;
        for check in checks.iter() {
            match check.check_health().await {
                Ok(health_result) => {
                    checks_performed += 1;

                    // Update performance metrics based on check results
                    match check.get_check_type() {
                        HealthCheckType::Connectivity => {
                            // Update network metrics
                        }
                        HealthCheckType::Performance => {
                            if let Some(cpu) = health_result.metrics.get("cpu_usage") {
                                total_cpu += cpu;
                            }
                            if let Some(memory) = health_result.metrics.get("memory_usage") {
                                total_memory += memory;
                            }
                            if let Some(disk) = health_result.metrics.get("disk_usage") {
                                total_disk += disk;
                            }
                        }
                        HealthCheckType::ResourceUsage => {
                            // Update resource metrics
                        }
                        HealthCheckType::ServiceAvailability => {
                            // Update service availability metrics
                        }
                        HealthCheckType::Security => {
                            // Update security metrics
                        }
                        HealthCheckType::Custom(_) => {
                            // Handle custom metrics
                        }
                    }

                    // Generate alerts for critical issues
                    if health_result.status == HealthStatus::Critical {
                        let _ = Self::generate_alert_from_health_result(
                            alerts,
                            &health_result,
                            AlertSeverity::Critical,
                        ).await;
                    } else if health_result.status == HealthStatus::Warning {
                        let _ = Self::generate_alert_from_health_result(
                            alerts,
                            &health_result,
                            AlertSeverity::Warning,
                        ).await;
                    }
                }
                Err(e) => {
                    error!("Health check {} failed: {}", check.get_check_name(), e);
                }
            }
        }

        // Update performance metrics
        if checks_performed > 0 {
            let mut metrics = performance_metrics.lock().await;
            metrics.cpu_usage_percent = total_cpu / checks_performed as f64;
            metrics.memory_usage_percent = total_memory / checks_performed as f64;
            metrics.disk_usage_percent = total_disk / checks_performed as f64;
        }

        debug!("Health check cycle completed");
        Ok(())
    }

    /// Generate alert from health check result
    async fn generate_alert_from_health_result(
        alerts: &Arc<Mutex<Vec<Alert>>>,
        health_result: &HealthCheck,
        severity: AlertSeverity,
    ) -> Result<(), DiscoveryError> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            title: format!("Health check failed: {}", health_result.check_id),
            description: format!(
                "Health check {} failed with status {:?}",
                health_result.check_id, health_result.status
            ),
            system_id: Some("system".to_string()), // Would be populated from actual system
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };

        alerts.lock().await.push(alert);
        Ok(())
    }
}

/// Trait for health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check_health(&self) -> Result<HealthCheck, HealthError>;

    /// Get the check identifier
    fn get_check_id(&self) -> CheckId;

    /// Get the check name
    fn get_check_name(&self) -> &str;

    /// Get the check type
    fn get_check_type(&self) -> HealthCheckType;

    /// Get the check timeout
    fn get_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// CPU usage health check
struct CpuUsageCheck {
    threshold: f64,
    check_id: CheckId,
}

impl CpuUsageCheck {
    fn new(threshold: f64) -> Self {
        Self {
            threshold,
            check_id: "cpu-usage".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for CpuUsageCheck {
    async fn check_health(&self) -> Result<HealthCheck, HealthError> {
        // Get CPU usage from system
        let sys = sysinfo::System::new_all();
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        let status = if cpu_usage > self.threshold {
            HealthStatus::Critical
        } else if cpu_usage > self.threshold * 0.8 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), cpu_usage as f64);

        Ok(HealthCheck {
            check_id: self.check_id.clone(),
            system_id: "local".to_string(),
            check_type: HealthCheckType::Performance,
            status,
            duration_ms: 0, // Would be calculated
            error_message: None,
            metrics,
            timestamp: Utc::now(),
        })
    }

    fn get_check_id(&self) -> CheckId {
        self.check_id.clone()
    }

    fn get_check_name(&self) -> &str {
        "CPU Usage Check"
    }

    fn get_check_type(&self) -> HealthCheckType {
        HealthCheckType::Performance
    }
}

/// Memory usage health check
struct MemoryUsageCheck {
    threshold: f64,
    check_id: CheckId,
}

impl MemoryUsageCheck {
    fn new(threshold: f64) -> Self {
        Self {
            threshold,
            check_id: "memory-usage".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for MemoryUsageCheck {
    async fn check_health(&self) -> Result<HealthCheck, HealthError> {
        let sys = sysinfo::System::new_all();
        let total_memory = sys.total_memory() as f64;
        let used_memory = sys.used_memory() as f64;
        let memory_usage = (used_memory / total_memory) * 100.0;

        let status = if memory_usage > self.threshold {
            HealthStatus::Critical
        } else if memory_usage > self.threshold * 0.8 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        let mut metrics = HashMap::new();
        metrics.insert("memory_usage".to_string(), memory_usage);

        Ok(HealthCheck {
            check_id: self.check_id.clone(),
            system_id: "local".to_string(),
            check_type: HealthCheckType::ResourceUsage,
            status,
            duration_ms: 0,
            error_message: None,
            metrics,
            timestamp: Utc::now(),
        })
    }

    fn get_check_id(&self) -> CheckId {
        self.check_id.clone()
    }

    fn get_check_name(&self) -> &str {
        "Memory Usage Check"
    }

    fn get_check_type(&self) -> HealthCheckType {
        HealthCheckType::ResourceUsage
    }
}

/// Disk usage health check
struct DiskUsageCheck {
    threshold: f64,
    check_id: CheckId,
}

impl DiskUsageCheck {
    fn new(threshold: f64) -> Self {
        Self {
            threshold,
            check_id: "disk-usage".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for DiskUsageCheck {
    async fn check_health(&self) -> Result<HealthCheck, HealthError> {
        // This would check actual disk usage
        // For now, return a placeholder
        let disk_usage = 50.0; // Placeholder value

        let status = if disk_usage > self.threshold {
            HealthStatus::Critical
        } else if disk_usage > self.threshold * 0.8 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        let mut metrics = HashMap::new();
        metrics.insert("disk_usage".to_string(), disk_usage);

        Ok(HealthCheck {
            check_id: self.check_id.clone(),
            system_id: "local".to_string(),
            check_type: HealthCheckType::ResourceUsage,
            status,
            duration_ms: 0,
            error_message: None,
            metrics,
            timestamp: Utc::now(),
        })
    }

    fn get_check_id(&self) -> CheckId {
        self.check_id.clone()
    }

    fn get_check_name(&self) -> &str {
        "Disk Usage Check"
    }

    fn get_check_type(&self) -> HealthCheckType {
        HealthCheckType::ResourceUsage
    }
}

/// Network connectivity health check
struct NetworkConnectivityCheck {
    check_id: CheckId,
}

impl NetworkConnectivityCheck {
    fn new() -> Self {
        Self {
            check_id: "network-connectivity".to_string(),
        }
    }
}

#[async_trait]
impl HealthCheck for NetworkConnectivityCheck {
    async fn check_health(&self) -> Result<HealthCheck, HealthError> {
        // Check basic network connectivity
        // This would ping a reliable external service or check local network interfaces
        let status = HealthStatus::Healthy; // Placeholder

        let mut metrics = HashMap::new();
        metrics.insert("connectivity".to_string(), 1.0);

        Ok(HealthCheck {
            check_id: self.check_id.clone(),
            system_id: "local".to_string(),
            check_type: HealthCheckType::Connectivity,
            status,
            duration_ms: 0,
            error_message: None,
            metrics,
            timestamp: Utc::now(),
        })
    }

    fn get_check_id(&self) -> CheckId {
        self.check_id.clone()
    }

    fn get_check_name(&self) -> &str {
        "Network Connectivity Check"
    }

    fn get_check_type(&self) -> HealthCheckType {
        HealthCheckType::Connectivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = SystemMonitor::new(config).await;
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_cpu_usage_check() {
        let check = CpuUsageCheck::new(80.0);
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::Performance);
        assert!(health.metrics.contains_key("cpu_usage"));
    }

    #[tokio::test]
    async fn test_memory_usage_check() {
        let check = MemoryUsageCheck::new(85.0);
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::ResourceUsage);
        assert!(health.metrics.contains_key("memory_usage"));
    }

    #[tokio::test]
    async fn test_network_connectivity_check() {
        let check = NetworkConnectivityCheck::new();
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::Connectivity);
        assert!(health.metrics.contains_key("connectivity"));
    }
}