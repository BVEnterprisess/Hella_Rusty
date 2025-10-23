//! # Environmental Scanner
//!
//! The Environmental Scanner is responsible for discovering and cataloging system components,
//! network topology, and available resources in the environment. It provides comprehensive
//! environmental awareness to support intelligent planning and resource allocation.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Environmental scanner for system discovery
pub struct EnvironmentalScanner {
    config: ScannerConfig,
    systems: Arc<Mutex<HashMap<SystemId, DiscoveredSystem>>>,
    network_topology: Arc<Mutex<NetworkTopology>>,
    resource_inventory: Arc<Mutex<ResourceInventory>>,
    discovery_cache: Arc<Mutex<DiscoveryCache>>,
    probes: Vec<Box<dyn SystemProbe>>,
    is_running: Arc<Mutex<bool>>,
}

impl EnvironmentalScanner {
    /// Create a new environmental scanner
    pub async fn new(config: ScannerConfig) -> Result<Self, DiscoveryError> {
        let systems = Arc::new(Mutex::new(HashMap::new()));
        let network_topology = Arc::new(Mutex::new(NetworkTopology {
            segments: Vec::new(),
            connections: HashMap::new(),
            health_status: NetworkHealth {
                score: 1.0,
                latency_ms: None,
                packet_loss_percent: None,
                bandwidth_utilization: None,
            },
        }));
        let resource_inventory = Arc::new(Mutex::new(ResourceInventory {
            total_cpu_cores: 0,
            total_memory_mb: 0,
            total_disk_gb: 0,
            available: ResourceAvailability {
                cpu_cores: 0,
                memory_mb: 0,
                disk_gb: 0,
                network_mbps: 0,
            },
        }));
        let discovery_cache = Arc::new(Mutex::new(DiscoveryCache {
            systems: HashMap::new(),
            last_scan: Utc::now(),
            cache_ttl: Duration::from_secs(config.cache_ttl_seconds),
        }));

        let mut probes: Vec<Box<dyn SystemProbe>> = Vec::new();

        // Add default probes
        probes.push(Box::new(LocalSystemProbe::new()));
        probes.push(Box::new(NetworkProbe::new()));
        probes.push(Box::new(ContainerProbe::new()));
        probes.push(Box::new(ServiceProbe::new()));

        Ok(Self {
            config,
            systems,
            network_topology,
            resource_inventory,
            discovery_cache,
            probes,
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start the environmental scanner
    pub async fn start(&mut self) -> Result<(), DiscoveryError> {
        info!("Starting Environmental Scanner");
        *self.is_running.lock().await = true;

        // Perform initial scan
        self.perform_scan().await?;

        // Start periodic scanning
        let config = self.config.clone();
        let systems = self.systems.clone();
        let network_topology = self.network_topology.clone();
        let resource_inventory = self.resource_inventory.clone();
        let discovery_cache = self.discovery_cache.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let scan_interval = Duration::from_secs(config.scan_interval_seconds);
            let mut interval = tokio::time::interval(scan_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !*is_running.lock().await {
                            break;
                        }

                        if let Err(e) = Self::perform_periodic_scan(
                            &config,
                            &systems,
                            &network_topology,
                            &resource_inventory,
                            &discovery_cache,
                        ).await {
                            error!("Periodic scan failed: {}", e);
                        }
                    }
                }
            }
        });

        info!("Environmental Scanner started successfully");
        Ok(())
    }

    /// Stop the environmental scanner
    pub async fn stop(&mut self) -> Result<(), DiscoveryError> {
        info!("Stopping Environmental Scanner");
        *self.is_running.lock().await = false;
        info!("Environmental Scanner stopped successfully");
        Ok(())
    }

    /// Get current scanner state
    pub async fn get_state(&self) -> Result<EnvironmentalState, DiscoveryError> {
        let systems = self.systems.lock().await.clone();
        let network_topology = self.network_topology.lock().await.clone();
        let resource_inventory = self.resource_inventory.lock().await.clone();

        Ok(EnvironmentalState {
            systems,
            network_topology,
            resource_inventory,
            last_scan: Utc::now(),
        })
    }

    /// Perform a full environmental scan
    pub async fn full_scan(&mut self) -> Result<ScanResult, DiscoveryError> {
        info!("Performing full environmental scan");
        self.perform_scan().await?;

        let systems = self.systems.lock().await;
        let issues = self.analyze_scan_issues(&systems).await;

        Ok(ScanResult {
            systems_discovered: systems.len() as u32,
            data_collected: 0, // Will be calculated based on actual data size
            issues_found: issues,
            timestamp: Utc::now(),
        })
    }

    /// Get scanner health status
    pub async fn health_check(&self) -> Result<ComponentHealth, DiscoveryError> {
        let is_running = *self.is_running.lock().await;
        let systems_count = self.systems.lock().await.len();
        let cache_size = self.discovery_cache.lock().await.systems.len();

        let status = if is_running && systems_count > 0 {
            ServiceStatus::Healthy
        } else if is_running {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "environmental-scanner".to_string(),
            status,
            check_duration_ms: 0, // Will be set by actual health check timing
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("systems_discovered".to_string(), systems_count as f64);
                metrics.insert("cache_size".to_string(), cache_size as f64);
                metrics.insert("probes_count".to_string(), self.probes.len() as f64);
                metrics
            },
        })
    }

    /// Perform the actual scanning operation
    async fn perform_scan(&mut self) -> Result<(), DiscoveryError> {
        debug!("Starting environmental scan with {} probes", self.probes.len());

        let mut new_systems = HashMap::new();
        let mut total_resources = ResourceAvailability {
            cpu_cores: 0,
            memory_mb: 0,
            disk_gb: 0,
            network_mbps: 0,
        };

        // Run all probes concurrently
        let scan_timeout = Duration::from_secs(self.config.max_scan_duration_seconds);
        let probe_futures = self.probes.iter().map(|probe| async {
            timeout(scan_timeout, probe.scan()).await
        });

        for (probe_index, result) in probe_futures.enumerate() {
            match result {
                Ok(Ok(system_info)) => {
                    debug!("Probe {} discovered system: {}", probe_index, system_info.name);

                    let system_id = format!("{}-{}", system_info.system_type, system_info.name);
                    let discovered_system = DiscoveredSystem {
                        id: system_id.clone(),
                        name: system_info.name,
                        system_type: system_info.system_type,
                        address: system_info.address,
                        port: system_info.port,
                        status: SystemStatus::Online,
                        capabilities: system_info.capabilities,
                        resources: system_info.resources,
                        metadata: system_info.metadata,
                        discovered_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    new_systems.insert(system_id.clone(), discovered_system);

                    // Update resource totals
                    if let Some(cpu) = system_info.resources.cpu_cores {
                        total_resources.cpu_cores += cpu;
                    }
                    if let Some(memory) = system_info.resources.memory_mb {
                        total_resources.memory_mb += memory;
                    }
                    if let Some(disk) = system_info.resources.disk_gb {
                        total_resources.disk_gb += disk;
                    }
                    if let Some(network) = system_info.resources.network_mbps {
                        total_resources.network_mbps += network;
                    }
                }
                Ok(Err(e)) => {
                    warn!("Probe {} failed: {}", probe_index, e);
                }
                Err(_) => {
                    error!("Probe {} timed out", probe_index);
                }
            }
        }

        // Update systems
        *self.systems.lock().await = new_systems;

        // Update resource inventory
        let mut resource_inventory = self.resource_inventory.lock().await;
        resource_inventory.total_cpu_cores = total_resources.cpu_cores;
        resource_inventory.total_memory_mb = total_resources.memory_mb;
        resource_inventory.total_disk_gb = total_resources.disk_gb;
        resource_inventory.available = total_resources;

        // Update discovery cache
        let mut cache = self.discovery_cache.lock().await;
        cache.systems = self.systems.lock().await.clone();
        cache.last_scan = Utc::now();

        debug!("Environmental scan completed");
        Ok(())
    }

    /// Perform periodic scan (lighter version of full scan)
    async fn perform_periodic_scan(
        config: &ScannerConfig,
        systems: &Arc<Mutex<HashMap<SystemId, DiscoveredSystem>>>,
        network_topology: &Arc<Mutex<NetworkTopology>>,
        resource_inventory: &Arc<Mutex<ResourceInventory>>,
        discovery_cache: &Arc<Mutex<DiscoveryCache>>,
    ) -> Result<(), DiscoveryError> {
        debug!("Performing periodic environmental scan");

        // Quick health checks on existing systems
        let mut systems_map = systems.lock().await;
        for system in systems_map.values_mut() {
            // Update system status based on connectivity
            system.updated_at = Utc::now();
            // In a real implementation, this would ping the system or check its health endpoint
        }

        // Update cache
        let mut cache = discovery_cache.lock().await;
        cache.last_scan = Utc::now();

        debug!("Periodic environmental scan completed");
        Ok(())
    }

    /// Analyze scan results for issues
    async fn analyze_scan_issues(
        &self,
        systems: &HashMap<SystemId, DiscoveredSystem>,
    ) -> Vec<ScanIssue> {
        let mut issues = Vec::new();

        // Check for offline systems
        for system in systems.values() {
            if system.status == SystemStatus::Offline {
                issues.push(ScanIssue {
                    severity: IssueSeverity::High,
                    title: format!("System {} is offline", system.name),
                    description: format!("System {} at {} is not responding", system.name, system.address),
                    system_id: Some(system.id.clone()),
                    category: IssueCategory::Network,
                    suggestions: vec![
                        "Check network connectivity".to_string(),
                        "Verify system is powered on".to_string(),
                        "Check firewall settings".to_string(),
                    ],
                });
            }
        }

        // Check for resource constraints
        let resource_inventory = self.resource_inventory.lock().await;
        if resource_inventory.available.cpu_cores == 0 {
            issues.push(ScanIssue {
                severity: IssueSeverity::Medium,
                title: "No CPU resources available".to_string(),
                description: "No CPU cores are available across all discovered systems".to_string(),
                system_id: None,
                category: IssueCategory::Resource,
                suggestions: vec![
                    "Add more compute nodes".to_string(),
                    "Optimize existing resource allocation".to_string(),
                ],
            });
        }

        issues
    }
}

/// Discovery cache for performance optimization
struct DiscoveryCache {
    systems: HashMap<SystemId, DiscoveredSystem>,
    last_scan: DateTime<Utc>,
    cache_ttl: Duration,
}

impl DiscoveryCache {
    /// Check if cache is still valid
    fn is_valid(&self) -> bool {
        Utc::now() - self.last_scan < chrono::Duration::from_std(self.cache_ttl).unwrap_or_default()
    }

    /// Get cached system if available and valid
    fn get_system(&self, system_id: &SystemId) -> Option<&DiscoveredSystem> {
        if self.is_valid() {
            self.systems.get(system_id)
        } else {
            None
        }
    }
}

/// Trait for system probes
#[async_trait]
pub trait SystemProbe: Send + Sync {
    /// Perform the scan operation
    async fn scan(&self) -> Result<SystemInfo, ScanError>;

    /// Get the probe type
    fn get_probe_type(&self) -> ProbeType;

    /// Check if probe is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get probe configuration
    fn get_config(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Types of system probes
#[derive(Debug, Clone)]
pub enum ProbeType {
    LocalSystem,
    Network,
    Container,
    Service,
    Custom(String),
}

/// System information returned by probes
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub name: String,
    pub system_type: SystemType,
    pub address: String,
    pub port: Option<u16>,
    pub capabilities: Vec<String>,
    pub resources: SystemResources,
    pub metadata: HashMap<String, String>,
}

/// Local system probe for discovering the local machine
struct LocalSystemProbe {
    config: HashMap<String, String>,
}

impl LocalSystemProbe {
    fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl SystemProbe for LocalSystemProbe {
    async fn scan(&self) -> Result<SystemInfo, ScanError> {
        // Get local system information
        let sys = sysinfo::System::new_all();

        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();

        let resources = SystemResources {
            cpu_cores: Some(sys.cpus().len() as u32),
            memory_mb: Some(total_memory / 1024 / 1024),
            disk_gb: Some(0), // Would need to calculate actual disk space
            network_mbps: Some(1000), // Default assumption
            gpu_info: None,
        };

        Ok(SystemInfo {
            name: sys.host_name().unwrap_or_else(|| "localhost".to_string()),
            system_type: SystemType::Server,
            address: "127.0.0.1".to_string(),
            port: None,
            capabilities: vec![
                "local-compute".to_string(),
                "system-monitoring".to_string(),
            ],
            resources,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("os".to_string(), sys.os_version().unwrap_or_default());
                metadata.insert("kernel".to_string(), sys.kernel_version().unwrap_or_default());
                metadata.insert("uptime".to_string(), sys.uptime().to_string());
                metadata
            },
        })
    }

    fn get_probe_type(&self) -> ProbeType {
        ProbeType::LocalSystem
    }
}

/// Network probe for discovering network services
struct NetworkProbe {
    config: HashMap<String, String>,
}

impl NetworkProbe {
    fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl SystemProbe for NetworkProbe {
    async fn scan(&self) -> Result<SystemInfo, ScanError> {
        // Network scanning implementation would go here
        // This is a placeholder that would implement actual network discovery
        Err(ScanError::NetworkTimeout("Network probe not fully implemented".to_string()))
    }

    fn get_probe_type(&self) -> ProbeType {
        ProbeType::Network
    }
}

/// Container probe for discovering containers
struct ContainerProbe {
    config: HashMap<String, String>,
}

impl ContainerProbe {
    fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl SystemProbe for ContainerProbe {
    async fn scan(&self) -> Result<SystemInfo, ScanError> {
        // Container discovery implementation would go here
        // This is a placeholder that would implement actual container discovery
        Err(ScanError::NetworkTimeout("Container probe not fully implemented".to_string()))
    }

    fn get_probe_type(&self) -> ProbeType {
        ProbeType::Container
    }
}

/// Service probe for discovering running services
struct ServiceProbe {
    config: HashMap<String, String>,
}

impl ServiceProbe {
    fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl SystemProbe for ServiceProbe {
    async fn scan(&self) -> Result<SystemInfo, ScanError> {
        // Service discovery implementation would go here
        // This is a placeholder that would implement actual service discovery
        Err(ScanError::NetworkTimeout("Service probe not fully implemented".to_string()))
    }

    fn get_probe_type(&self) -> ProbeType {
        ProbeType::Service
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environmental_scanner_creation() {
        let config = ScannerConfig::default();
        let scanner = EnvironmentalScanner::new(config).await;
        assert!(scanner.is_ok());
    }

    #[tokio::test]
    async fn test_local_system_probe() {
        let probe = LocalSystemProbe::new();
        let result = probe.scan().await;
        assert!(result.is_ok());

        let system_info = result.unwrap();
        assert_eq!(system_info.system_type, SystemType::Server);
        assert!(!system_info.name.is_empty());
        assert_eq!(system_info.address, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_probe_types() {
        let local_probe = LocalSystemProbe::new();
        assert!(matches!(local_probe.get_probe_type(), ProbeType::LocalSystem));

        let network_probe = NetworkProbe::new();
        assert!(matches!(network_probe.get_probe_type(), ProbeType::Network));

        let container_probe = ContainerProbe::new();
        assert!(matches!(container_probe.get_probe_type(), ProbeType::Container));

        let service_probe = ServiceProbe::new();
        assert!(matches!(service_probe.get_probe_type(), ProbeType::Service));
    }
}