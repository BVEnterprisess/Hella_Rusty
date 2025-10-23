//! # Layer 1 (Discovery) - Environmental Awareness and Data Collection
//!
//! Layer 1 provides comprehensive environmental awareness and data collection capabilities
//! for Project Chimera. It serves as the foundation layer that discovers, monitors, and
//! collects data about the system environment, providing critical information to higher
//! layers for planning, validation, and execution.
//!
//! ## Core Components
//!
//! - **Environmental Scanner**: Discovers and catalogs system components and resources
//! - **System Monitor**: Continuously monitors system health and performance
//! - **Data Collector**: Collects data from multiple sources and formats
//! - **Integration Hub**: Manages communication with other layers
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Layer 1 - Discovery                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┤
//! │  │ Environmental│  │ System      │  │ Data       │  │ External│
//! │  │ Scanner     │  │ Monitor     │  │ Collector  │  │ API     │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┤
//! │  │ Data Storage│  │ Event       │  │ Integration│  │ Metrics │
//! │  │ & Caching   │  │ Processor   │  │ Hub        │  │ & Alert │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod types;
pub mod environmental_scanner;
pub mod system_monitor;
pub mod data_collector;
pub mod integration_hub;
pub mod metrics;

pub use types::*;
pub use environmental_scanner::*;
pub use system_monitor::*;
pub use data_collector::*;
pub use integration_hub::*;
pub use metrics::*;

/// Main discovery service that orchestrates all Layer 1 components
pub struct DiscoveryService {
    environmental_scanner: EnvironmentalScanner,
    system_monitor: SystemMonitor,
    data_collector: DataCollector,
    integration_hub: IntegrationHub,
    config: DiscoveryConfig,
}

impl DiscoveryService {
    /// Create a new discovery service with the given configuration
    pub async fn new(config: DiscoveryConfig) -> Result<Self, DiscoveryError> {
        let environmental_scanner = EnvironmentalScanner::new(config.scanner.clone()).await?;
        let system_monitor = SystemMonitor::new(config.monitor.clone()).await?;
        let data_collector = DataCollector::new(config.collector.clone()).await?;
        let integration_hub = IntegrationHub::new(config.integration.clone()).await?;

        Ok(Self {
            environmental_scanner,
            system_monitor,
            data_collector,
            integration_hub,
            config,
        })
    }

    /// Start the discovery service and all its components
    pub async fn start(&mut self) -> Result<(), DiscoveryError> {
        tracing::info!("Starting Layer 1 Discovery Service");

        // Start all components
        self.environmental_scanner.start().await?;
        self.system_monitor.start().await?;
        self.data_collector.start().await?;
        self.integration_hub.start().await?;

        tracing::info!("Layer 1 Discovery Service started successfully");
        Ok(())
    }

    /// Stop the discovery service and all its components
    pub async fn stop(&mut self) -> Result<(), DiscoveryError> {
        tracing::info!("Stopping Layer 1 Discovery Service");

        // Stop all components in reverse order
        self.integration_hub.stop().await?;
        self.data_collector.stop().await?;
        self.system_monitor.stop().await?;
        self.environmental_scanner.stop().await?;

        tracing::info!("Layer 1 Discovery Service stopped successfully");
        Ok(())
    }

    /// Get current system state from all components
    pub async fn get_system_state(&self) -> Result<SystemState, DiscoveryError> {
        let environmental_state = self.environmental_scanner.get_state().await?;
        let monitoring_state = self.system_monitor.get_state().await?;
        let collection_state = self.data_collector.get_state().await?;

        Ok(SystemState {
            environmental: environmental_state,
            monitoring: monitoring_state,
            collection: collection_state,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Trigger a full system scan across all components
    pub async fn trigger_full_scan(&mut self) -> Result<ScanResult, DiscoveryError> {
        tracing::info!("Triggering full system scan");

        let scan_result = self.environmental_scanner.full_scan().await?;
        let monitoring_result = self.system_monitor.full_check().await?;
        let collection_result = self.data_collector.collect_all().await?;

        // Send results to integration hub for distribution to other layers
        self.integration_hub
            .distribute_discovery_data(DiscoveryData::FullScanResult(FullScanResult {
                scan: scan_result,
                monitoring: monitoring_result,
                collection: collection_result,
                timestamp: chrono::Utc::now(),
            }))
            .await?;

        tracing::info!("Full system scan completed successfully");
        Ok(ScanResult {
            systems_discovered: 0, // Will be populated by actual scan
            data_collected: 0,
            issues_found: vec![],
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get service health status
    pub async fn health_check(&self) -> Result<ServiceHealth, DiscoveryError> {
        let scanner_health = self.environmental_scanner.health_check().await?;
        let monitor_health = self.system_monitor.health_check().await?;
        let collector_health = self.data_collector.health_check().await?;
        let hub_health = self.integration_hub.health_check().await?;

        let overall_status = if scanner_health.status == ServiceStatus::Healthy
            && monitor_health.status == ServiceStatus::Healthy
            && collector_health.status == ServiceStatus::Healthy
            && hub_health.status == ServiceStatus::Healthy
        {
            ServiceStatus::Healthy
        } else if scanner_health.status == ServiceStatus::Unhealthy
            || monitor_health.status == ServiceStatus::Unhealthy
            || collector_health.status == ServiceStatus::Unhealthy
            || hub_health.status == ServiceStatus::Unhealthy
        {
            ServiceStatus::Unhealthy
        } else {
            ServiceStatus::Degraded
        };

        Ok(ServiceHealth {
            service: "layer1-discovery".to_string(),
            status: overall_status,
            components: vec![
                scanner_health,
                monitor_health,
                collector_health,
                hub_health,
            ],
            timestamp: chrono::Utc::now(),
        })
    }
}

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
    async fn test_health_check() {
        let config = DiscoveryConfig::default();
        let service = DiscoveryService::new(config).await.unwrap();
        let health = service.health_check().await.unwrap();
        assert_eq!(health.service, "layer1-discovery");
    }
}