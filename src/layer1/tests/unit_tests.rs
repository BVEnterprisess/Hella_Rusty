//! # Layer 1 Unit Tests
//!
//! Comprehensive unit tests for all Layer 1 (Discovery) components.

use layer1_discovery::*;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_service_creation() {
        let config = DiscoveryConfig::default();
        let service = DiscoveryService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_discovery_service_health_check() {
        let config = DiscoveryConfig::default();
        let service = DiscoveryService::new(config).await.unwrap();
        let health = service.health_check().await.unwrap();
        assert_eq!(health.service, "layer1-discovery");
        assert!(matches!(health.status, ServiceStatus::Healthy | ServiceStatus::Degraded));
    }

    #[tokio::test]
    async fn test_environmental_scanner_creation() {
        let config = ScannerConfig::default();
        let scanner = EnvironmentalScanner::new(config).await;
        assert!(scanner.is_ok());
    }

    #[tokio::test]
    async fn test_system_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = SystemMonitor::new(config).await;
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_data_collector_creation() {
        let config = CollectorConfig::default();
        let collector = DataCollector::new(config).await;
        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_integration_hub_creation() {
        let config = IntegrationConfig::default();
        let hub = IntegrationHub::new(config).await;
        assert!(hub.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.is_ok());
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.scanner.scan_interval_seconds, 300);
        assert_eq!(config.monitor.check_interval_seconds, 60);
        assert_eq!(config.collector.collection_interval_seconds, 30);
        assert_eq!(config.integration.layer_timeout_seconds, 10);
    }

    #[test]
    fn test_system_types() {
        assert_eq!(SystemType::Server, SystemType::Server);
        assert_eq!(SystemType::Container, SystemType::Container);
        assert_eq!(SystemType::Service, SystemType::Service);
        assert_eq!(SystemType::Database, SystemType::Database);
        assert_eq!(SystemType::NetworkDevice, SystemType::NetworkDevice);
    }

    #[test]
    fn test_system_status() {
        assert_eq!(SystemStatus::Online, SystemStatus::Online);
        assert_eq!(SystemStatus::Offline, SystemStatus::Offline);
        assert_eq!(SystemStatus::Degraded, SystemStatus::Degraded);
        assert_eq!(SystemStatus::Unknown, SystemStatus::Unknown);
        assert_eq!(SystemStatus::Maintenance, SystemStatus::Maintenance);
    }

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Warning, HealthStatus::Warning);
        assert_eq!(HealthStatus::Critical, HealthStatus::Critical);
        assert_eq!(HealthStatus::Unknown, HealthStatus::Unknown);
    }

    #[test]
    fn test_alert_severity() {
        assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
        assert_eq!(AlertSeverity::Warning, AlertSeverity::Warning);
        assert_eq!(AlertSeverity::Error, AlertSeverity::Error);
        assert_eq!(AlertSeverity::Critical, AlertSeverity::Critical);
    }

    #[test]
    fn test_data_source_types() {
        assert_eq!(DataSourceType::SystemMetrics, DataSourceType::SystemMetrics);
        assert_eq!(DataSourceType::ApplicationLogs, DataSourceType::ApplicationLogs);
        assert_eq!(DataSourceType::NetworkTraffic, DataSourceType::NetworkTraffic);
        assert_eq!(DataSourceType::ExternalAPI, DataSourceType::ExternalAPI);
        assert_eq!(DataSourceType::Database, DataSourceType::Database);
    }

    #[test]
    fn test_issue_severity() {
        assert_eq!(IssueSeverity::Low, IssueSeverity::Low);
        assert_eq!(IssueSeverity::Medium, IssueSeverity::Medium);
        assert_eq!(IssueSeverity::High, IssueSeverity::High);
        assert_eq!(IssueSeverity::Critical, IssueSeverity::Critical);
    }

    #[test]
    fn test_issue_categories() {
        assert_eq!(IssueCategory::Security, IssueCategory::Security);
        assert_eq!(IssueCategory::Performance, IssueCategory::Performance);
        assert_eq!(IssueCategory::Configuration, IssueCategory::Configuration);
        assert_eq!(IssueCategory::Network, IssueCategory::Network);
        assert_eq!(IssueCategory::Resource, IssueCategory::Resource);
    }

    #[test]
    fn test_service_status() {
        assert_eq!(ServiceStatus::Healthy, ServiceStatus::Healthy);
        assert_eq!(ServiceStatus::Degraded, ServiceStatus::Degraded);
        assert_eq!(ServiceStatus::Unhealthy, ServiceStatus::Unhealthy);
        assert_eq!(ServiceStatus::Starting, ServiceStatus::Starting);
        assert_eq!(ServiceStatus::Stopping, ServiceStatus::Stopping);
    }

    #[test]
    fn test_collection_error_types() {
        assert_eq!(CollectionErrorType::ConnectionError, CollectionErrorType::ConnectionError);
        assert_eq!(CollectionErrorType::AuthenticationError, CollectionErrorType::AuthenticationError);
        assert_eq!(CollectionErrorType::TimeoutError, CollectionErrorType::TimeoutError);
        assert_eq!(CollectionErrorType::DataFormatError, CollectionErrorType::DataFormatError);
        assert_eq!(CollectionErrorType::PermissionError, CollectionErrorType::PermissionError);
    }

    #[test]
    fn test_health_check_types() {
        assert_eq!(HealthCheckType::Connectivity, HealthCheckType::Connectivity);
        assert_eq!(HealthCheckType::Performance, HealthCheckType::Performance);
        assert_eq!(HealthCheckType::ResourceUsage, HealthCheckType::ResourceUsage);
        assert_eq!(HealthCheckType::ServiceAvailability, HealthCheckType::ServiceAvailability);
        assert_eq!(HealthCheckType::Security, HealthCheckType::Security);
    }

    #[test]
    fn test_probe_types() {
        let local_probe = LocalSystemProbe::new();
        assert!(matches!(local_probe.get_probe_type(), ProbeType::LocalSystem));

        let network_probe = NetworkProbe::new();
        assert!(matches!(network_probe.get_probe_type(), ProbeType::Network));

        let container_probe = ContainerProbe::new();
        assert!(matches!(container_probe.get_probe_type(), ProbeType::Container));

        let service_probe = ServiceProbe::new();
        assert!(matches!(service_probe.get_probe_type(), ProbeType::Service));
    }

    #[tokio::test]
    async fn test_local_system_probe_scan() {
        let probe = LocalSystemProbe::new();
        let result = probe.scan().await;
        assert!(result.is_ok());

        let system_info = result.unwrap();
        assert_eq!(system_info.system_type, SystemType::Server);
        assert!(!system_info.name.is_empty());
        assert_eq!(system_info.address, "127.0.0.1");
        assert!(!system_info.capabilities.is_empty());
        assert!(system_info.resources.cpu_cores.is_some());
        assert!(system_info.resources.memory_mb.is_some());
    }

    #[tokio::test]
    async fn test_system_metrics_source() {
        let source = SystemMetricsSource::new();
        let result = source.collect_data().await;
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.source_id, "system-metrics");
        assert_eq!(batch.source_type, DataSourceType::SystemMetrics);
        assert!(!batch.data_points.is_empty());
        assert!(batch.quality_score > 0.9);

        // Check for expected metrics
        let metric_names: Vec<&str> = batch.data_points.iter()
            .map(|dp| dp.metric_name.as_str())
            .collect();

        assert!(metric_names.contains(&"cpu_usage_percent"));
        assert!(metric_names.contains(&"memory_usage_percent"));
        assert!(metric_names.contains(&"disk_usage_percent"));
    }

    #[tokio::test]
    async fn test_application_logs_source() {
        let source = ApplicationLogsSource::new();
        let result = source.collect_data().await;
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.source_id, "application-logs");
        assert_eq!(batch.source_type, DataSourceType::ApplicationLogs);
        assert!(!batch.data_points.is_empty());
        assert!(batch.quality_score > 0.8);

        // Check for expected log metrics
        let metric_names: Vec<&str> = batch.data_points.iter()
            .map(|dp| dp.metric_name.as_str())
            .collect();

        assert!(metric_names.contains(&"log_entries_count"));
        assert!(metric_names.contains(&"error_rate"));
    }

    #[tokio::test]
    async fn test_network_traffic_source() {
        let source = NetworkTrafficSource::new();
        let result = source.collect_data().await;
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.source_id, "network-traffic");
        assert_eq!(batch.source_type, DataSourceType::NetworkTraffic);
        assert!(!batch.data_points.is_empty());
        assert!(batch.quality_score > 0.8);

        // Check for expected network metrics
        let metric_names: Vec<&str> = batch.data_points.iter()
            .map(|dp| dp.metric_name.as_str())
            .collect();

        assert!(metric_names.contains(&"bytes_received_per_sec"));
        assert!(metric_names.contains(&"bytes_transmitted_per_sec"));
        assert!(metric_names.contains(&"active_connections"));
    }

    #[tokio::test]
    async fn test_cpu_usage_check() {
        let check = CpuUsageCheck::new(80.0);
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::Performance);
        assert!(health.metrics.contains_key("cpu_usage"));
        assert!(health.duration_ms >= 0);
        assert!(health.timestamp <= Utc::now());
    }

    #[tokio::test]
    async fn test_memory_usage_check() {
        let check = MemoryUsageCheck::new(85.0);
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::ResourceUsage);
        assert!(health.metrics.contains_key("memory_usage"));
        assert!(health.duration_ms >= 0);
        assert!(health.timestamp <= Utc::now());
    }

    #[tokio::test]
    async fn test_disk_usage_check() {
        let check = DiskUsageCheck::new(90.0);
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::ResourceUsage);
        assert!(health.metrics.contains_key("disk_usage"));
        assert!(health.duration_ms >= 0);
        assert!(health.timestamp <= Utc::now());
    }

    #[tokio::test]
    async fn test_network_connectivity_check() {
        let check = NetworkConnectivityCheck::new();
        let result = check.check_health().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.check_type, HealthCheckType::Connectivity);
        assert!(health.metrics.contains_key("connectivity"));
        assert!(health.duration_ms >= 0);
        assert!(health.timestamp <= Utc::now());
    }

    #[test]
    fn test_metrics_collector_operations() {
        let collector = MetricsCollector::new().unwrap();

        // Test recording operations
        collector.record_system_discovered("server");
        collector.record_health_check(100.0, &HealthStatus::Healthy);
        collector.record_data_collection(1000, 0.5);
        collector.record_alert(&AlertSeverity::Warning);
        collector.record_scan(2.5, 5, 0);
        collector.record_collection_error(&CollectionErrorType::TimeoutError);
        collector.update_system_health_score(0.95);
        collector.update_active_data_sources(3.0);

        // Test metrics retrieval
        let summary = collector.get_metrics_summary().unwrap();
        assert!(summary.contains_key("systems_discovered_total"));
        assert!(summary.contains_key("health_checks_total"));
        assert!(summary.contains_key("data_points_collected_total"));
        assert!(summary.contains_key("alerts_generated_total"));
        assert!(summary.contains_key("active_systems"));
        assert!(summary.contains_key("system_health_score"));
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

    #[test]
    fn test_discovered_system_creation() {
        let system = DiscoveredSystem {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            system_type: SystemType::Server,
            address: "192.168.1.100".to_string(),
            port: Some(8080),
            status: SystemStatus::Online,
            capabilities: vec!["web-server".to_string(), "api".to_string()],
            resources: SystemResources {
                cpu_cores: Some(4),
                memory_mb: Some(8192),
                disk_gb: Some(500),
                network_mbps: Some(1000),
                gpu_info: None,
            },
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("os".to_string(), "Linux".to_string());
                metadata.insert("version".to_string(), "Ubuntu 20.04".to_string());
                metadata
            },
            discovered_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(system.id, "test-server");
        assert_eq!(system.name, "Test Server");
        assert_eq!(system.system_type, SystemType::Server);
        assert_eq!(system.status, SystemStatus::Online);
        assert!(system.capabilities.contains(&"web-server".to_string()));
        assert_eq!(system.resources.cpu_cores, Some(4));
        assert_eq!(system.resources.memory_mb, Some(8192));
    }

    #[test]
    fn test_data_point_creation() {
        let data_point = DataPoint {
            metric_name: "cpu_usage".to_string(),
            value: 75.5,
            unit: "percent".to_string(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("host".to_string(), "server1".to_string());
                tags.insert("region".to_string(), "us-west".to_string());
                tags
            },
            timestamp: Utc::now(),
        };

        assert_eq!(data_point.metric_name, "cpu_usage");
        assert_eq!(data_point.value, 75.5);
        assert_eq!(data_point.unit, "percent");
        assert_eq!(data_point.tags.get("host"), Some(&"server1".to_string()));
        assert_eq!(data_point.tags.get("region"), Some(&"us-west".to_string()));
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "alert-123".to_string(),
            severity: AlertSeverity::Critical,
            title: "High CPU Usage".to_string(),
            description: "CPU usage has exceeded 90% for 5 minutes".to_string(),
            system_id: Some("server1".to_string()),
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };

        assert_eq!(alert.id, "alert-123");
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.title, "High CPU Usage");
        assert_eq!(alert.system_id, Some("server1".to_string()));
        assert!(!alert.acknowledged);
    }

    #[test]
    fn test_scan_issue_creation() {
        let issue = ScanIssue {
            severity: IssueSeverity::High,
            title: "System Unreachable".to_string(),
            description: "System server1 is not responding to ping".to_string(),
            system_id: Some("server1".to_string()),
            category: IssueCategory::Network,
            suggestions: vec![
                "Check network connectivity".to_string(),
                "Verify system is powered on".to_string(),
                "Check firewall settings".to_string(),
            ],
        };

        assert_eq!(issue.severity, IssueSeverity::High);
        assert_eq!(issue.title, "System Unreachable");
        assert_eq!(issue.category, IssueCategory::Network);
        assert_eq!(issue.suggestions.len(), 3);
    }

    #[test]
    fn test_health_check_creation() {
        let health_check = HealthCheck {
            check_id: "cpu-check".to_string(),
            system_id: "server1".to_string(),
            check_type: HealthCheckType::Performance,
            status: HealthStatus::Warning,
            duration_ms: 150,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("cpu_usage".to_string(), 85.5);
                metrics
            },
            timestamp: Utc::now(),
        };

        assert_eq!(health_check.check_id, "cpu-check");
        assert_eq!(health_check.system_id, "server1");
        assert_eq!(health_check.check_type, HealthCheckType::Performance);
        assert_eq!(health_check.status, HealthStatus::Warning);
        assert_eq!(health_check.duration_ms, 150);
        assert_eq!(health_check.metrics.get("cpu_usage"), Some(&85.5));
    }

    #[test]
    fn test_data_batch_creation() {
        let data_points = vec![
            DataPoint {
                metric_name: "cpu_usage".to_string(),
                value: 75.5,
                unit: "percent".to_string(),
                tags: HashMap::new(),
                timestamp: Utc::now(),
            },
            DataPoint {
                metric_name: "memory_usage".to_string(),
                value: 60.2,
                unit: "percent".to_string(),
                tags: HashMap::new(),
                timestamp: Utc::now(),
            },
        ];

        let batch = DataBatch {
            source_id: "system-metrics".to_string(),
            timestamp: Utc::now(),
            data_points,
            quality_score: 0.95,
            metadata: HashMap::new(),
        };

        assert_eq!(batch.source_id, "system-metrics");
        assert_eq!(batch.data_points.len(), 2);
        assert_eq!(batch.quality_score, 0.95);
    }

    #[test]
    fn test_system_state_creation() {
        let environmental_state = EnvironmentalState {
            systems: HashMap::new(),
            network_topology: NetworkTopology {
                segments: Vec::new(),
                connections: HashMap::new(),
                health_status: NetworkHealth {
                    score: 1.0,
                    latency_ms: None,
                    packet_loss_percent: None,
                    bandwidth_utilization: None,
                },
            },
            resource_inventory: ResourceInventory {
                total_cpu_cores: 16,
                total_memory_mb: 32768,
                total_disk_gb: 2000,
                available: ResourceAvailability {
                    cpu_cores: 8,
                    memory_mb: 16384,
                    disk_gb: 1000,
                    network_mbps: 1000,
                },
            },
            last_scan: Utc::now(),
        };

        let monitoring_state = MonitoringState {
            health_checks: HashMap::new(),
            performance_metrics: PerformanceMetrics {
                cpu_usage_percent: 45.5,
                memory_usage_percent: 60.2,
                disk_usage_percent: 30.1,
                network_io: NetworkIOMetrics {
                    bytes_received_per_sec: 1024 * 100,
                    bytes_transmitted_per_sec: 1024 * 50,
                    active_connections: 25,
                    connection_errors: 0,
                },
                processes: Vec::new(),
            },
            alerts: Vec::new(),
            last_update: Utc::now(),
        };

        let collection_state = CollectionState {
            data_sources: HashMap::new(),
            recent_batches: Vec::new(),
            statistics: CollectionStatistics {
                total_data_points: 10000,
                data_points_per_second: 50.0,
                success_rate: 0.98,
                avg_latency_ms: 25.5,
                quality_score: 0.95,
            },
            last_collection: Utc::now(),
        };

        let system_state = SystemState {
            environmental: environmental_state,
            monitoring: monitoring_state,
            collection: collection_state,
            timestamp: Utc::now(),
        };

        assert_eq!(system_state.environmental.resource_inventory.total_cpu_cores, 16);
        assert_eq!(system_state.monitoring.performance_metrics.cpu_usage_percent, 45.5);
        assert_eq!(system_state.collection.statistics.total_data_points, 10000);
    }
}