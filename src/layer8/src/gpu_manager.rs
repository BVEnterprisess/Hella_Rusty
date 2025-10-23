//! # GPU Manager
//!
//! Manages GPU resources, allocation, monitoring, and optimization
//! for the autonomous AI system.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// GPU resource manager
pub struct GpuManager {
    /// GPU allocation tracking
    gpu_allocations: Arc<RwLock<HashMap<String, GpuAllocation>>>,
    /// GPU status monitoring
    gpu_status: Arc<RwLock<HashMap<String, GpuDeviceStatus>>>,
    /// Configuration
    config: GpuConfig,
    /// Performance metrics
    metrics: Arc<RwLock<GpuMetrics>>,
}

impl GpuManager {
    /// Create a new GPU manager
    pub async fn new(config: ResourceConfig) -> Result<Self> {
        info!("Initializing GPU manager...");

        let manager = Self {
            gpu_allocations: Arc::new(RwLock::new(HashMap::new())),
            gpu_status: Arc::new(RwLock::new(HashMap::new())),
            config: GpuConfig::from_resource_config(config),
            metrics: Arc::new(RwLock::new(GpuMetrics::default())),
        };

        // Initialize GPU status
        manager.discover_gpus().await?;

        info!("✅ GPU manager initialized successfully");
        Ok(manager)
    }

    /// Start the GPU manager service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting GPU manager service...");

        // Start monitoring tasks
        self.start_monitoring().await?;

        info!("✅ GPU manager service started successfully");
        Ok(())
    }

    /// Stop the GPU manager service
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping GPU manager service...");

        // Clean up allocations
        let mut allocations = self.gpu_allocations.write().await;
        for (gpu_id, allocation) in allocations.iter_mut() {
            if matches!(allocation.status, GpuAllocationStatus::Active) {
                warn!("Force terminating GPU allocation: {}", gpu_id);
                allocation.status = GpuAllocationStatus::Released;
            }
        }

        info!("✅ GPU manager service stopped successfully");
        Ok(())
    }

    /// Allocate GPU resources
    pub async fn allocate_gpu(&self, request: GpuAllocationRequest) -> ResourceResult<GpuAllocation> {
        debug!("Processing GPU allocation request: {:?}", request);

        // Validate request
        self.validate_gpu_request(&request).await?;

        // Find available GPU
        let available_gpu = self.find_available_gpu(&request.requirements).await?;

        // Create allocation
        let allocation = GpuAllocation {
            allocation_id: Uuid::new_v4(),
            gpu_id: available_gpu.id.clone(),
            requirements: request.requirements,
            allocated_at: Utc::now(),
            status: GpuAllocationStatus::Active,
            performance_metrics: GpuPerformanceMetrics::default(),
        };

        // Update tracking
        self.gpu_allocations.write().await.insert(available_gpu.id, allocation.clone());

        // Update GPU status
        self.update_gpu_status(&available_gpu.id, GpuDeviceStatus::Allocated).await?;

        info!("✅ Successfully allocated GPU {} for request", available_gpu.id);
        Ok(allocation)
    }

    /// Release GPU allocation
    pub async fn release_gpu(&self, allocation_id: Uuid) -> ResourceResult<()> {
        debug!("Releasing GPU allocation: {}", allocation_id);

        let mut allocations = self.gpu_allocations.write().await;
        if let Some(allocation) = allocations.values().find(|a| a.allocation_id == allocation_id) {
            // Update GPU status
            self.update_gpu_status(&allocation.gpu_id, GpuDeviceStatus::Available).await?;

            // Remove allocation
            allocations.retain(|_, a| a.allocation_id != allocation_id);

            info!("✅ Successfully released GPU allocation: {}", allocation_id);
            Ok(())
        } else {
            Err(ResourceError::GpuError {
                message: format!("GPU allocation not found: {}", allocation_id),
            })
        }
    }

    /// Get GPU status
    pub async fn get_status(&self) -> Result<GpuStatus> {
        let status_map = self.gpu_status.read().await;
        let allocations = self.gpu_allocations.read().await;

        let total_gpus = status_map.len();
        let available_gpus = status_map.values().filter(|s| matches!(s, GpuDeviceStatus::Available)).count();
        let allocated_gpus = allocations.len();

        let utilization: Vec<f64> = status_map.values()
            .map(|s| s.utilization_percentage)
            .collect();

        let memory_usage: Vec<u64> = status_map.values()
            .map(|s| s.memory_used_gb)
            .collect();

        let temperatures: Vec<f64> = status_map.values()
            .map(|s| s.temperature_celsius)
            .collect();

        Ok(GpuStatus {
            available_gpus: available_gpus as u32,
            total_gpus: total_gpus as u32,
            utilization,
            memory_usage_gb: memory_usage,
            temperatures,
            last_update: Utc::now(),
        })
    }

    /// Get GPU allocation by ID
    pub async fn get_allocation(&self, allocation_id: Uuid) -> Option<GpuAllocation> {
        self.gpu_allocations.read().await
            .values()
            .find(|a| a.allocation_id == allocation_id)
            .cloned()
    }

    /// Get all active GPU allocations
    pub async fn get_active_allocations(&self) -> Vec<GpuAllocation> {
        self.gpu_allocations.read().await
            .values()
            .filter(|a| matches!(a.status, GpuAllocationStatus::Active))
            .cloned()
            .collect()
    }

    /// Health check implementation
    pub async fn health_check(&self) -> Result<()> {
        // Check if we can access GPU status
        let _status = self.gpu_status.read().await;

        // Check if we can access allocations
        let _allocations = self.gpu_allocations.read().await;

        // Verify at least one GPU is available
        let status = self.get_status().await?;
        if status.total_gpus == 0 {
            return Err(anyhow::anyhow!("No GPUs available"));
        }

        Ok(())
    }

    /// Readiness check
    pub async fn is_ready(&self) -> bool {
        self.health_check().await.is_ok()
    }

    // Private helper methods

    async fn discover_gpus(&self) -> Result<()> {
        // In a real implementation, this would use nvidia-ml-py or similar
        // For now, we'll simulate GPU discovery
        let mut status_map = self.gpu_status.write().await;

        for i in 0..4 { // Simulate 4 GPUs
            let gpu_id = format!("gpu-{}", i);
            status_map.insert(gpu_id, GpuDeviceStatus {
                id: format!("gpu-{}", i),
                status: GpuDeviceStatus::Available,
                utilization_percentage: 0.0,
                memory_used_gb: 0,
                memory_total_gb: 24,
                temperature_celsius: 45.0 + (i as f64 * 5.0), // Simulate different temperatures
                power_usage_watts: 75 + (i * 25),
            });
        }

        info!("✅ Discovered {} GPUs", status_map.len());
        Ok(())
    }

    async fn find_available_gpu(&self, requirements: &GpuRequirements) -> ResourceResult<GpuDevice> {
        let status_map = self.gpu_status.read().await;

        for (gpu_id, status) in status_map.iter() {
            if matches!(status.status, GpuDeviceStatus::Available) &&
               status.memory_total_gb - status.memory_used_gb >= requirements.memory_gb {

                return Ok(GpuDevice {
                    id: gpu_id.clone(),
                    memory_gb: status.memory_total_gb,
                    compute_capability: "8.0".to_string(),
                    max_power_watts: 350,
                });
            }
        }

        Err(ResourceError::InsufficientResources {
            requested: ResourceRequirements {
                gpu_count: 1,
                gpu_memory_gb: requirements.memory_gb,
                cpu_cores: 0,
                ram_gb: 0,
                storage_gb: 0,
                special_requirements: Vec::new(),
            },
            available: self.get_available_resources().await?,
        })
    }

    async fn validate_gpu_request(&self, request: &GpuAllocationRequest) -> ResourceResult<()> {
        if request.requirements.memory_gb > self.config.max_memory_per_gpu_gb {
            return Err(ResourceError::GpuError {
                message: format!(
                    "Memory requirement {}GB exceeds maximum {}GB",
                    request.requirements.memory_gb,
                    self.config.max_memory_per_gpu_gb
                ),
            });
        }

        if request.duration_minutes > self.config.max_allocation_time_minutes {
            return Err(ResourceError::GpuError {
                message: format!(
                    "Duration {} minutes exceeds maximum {} minutes",
                    request.duration_minutes,
                    self.config.max_allocation_time_minutes
                ),
            });
        }

        Ok(())
    }

    async fn update_gpu_status(&self, gpu_id: &str, status: GpuDeviceStatus) -> Result<()> {
        let mut status_map = self.gpu_status.write().await;
        if let Some(gpu_status) = status_map.get_mut(gpu_id) {
            *gpu_status = status;
        }
        Ok(())
    }

    async fn get_available_resources(&self) -> ResourceResult<ResourceRequirements> {
        let status_map = self.gpu_status.read().await;

        let available_gpus = status_map.values()
            .filter(|s| matches!(s.status, GpuDeviceStatus::Available))
            .count() as u32;

        let available_memory = status_map.values()
            .filter(|s| matches!(s.status, GpuDeviceStatus::Available))
            .map(|s| s.memory_total_gb - s.memory_used_gb)
            .sum();

        Ok(ResourceRequirements {
            gpu_count: available_gpus,
            gpu_memory_gb: available_memory,
            cpu_cores: 0, // GPU manager doesn't manage CPU
            ram_gb: 0,    // GPU manager doesn't manage RAM
            storage_gb: 0, // GPU manager doesn't manage storage
            special_requirements: Vec::new(),
        })
    }

    async fn start_monitoring(&self) -> Result<()> {
        // In a real implementation, this would start background monitoring tasks
        // For now, we'll just log that monitoring started
        info!("📊 GPU monitoring started");
        Ok(())
    }
}

/// GPU configuration
#[derive(Debug, Clone)]
struct GpuConfig {
    /// Maximum memory per GPU in GB
    max_memory_per_gpu_gb: u64,
    /// Maximum allocation time in minutes
    max_allocation_time_minutes: u64,
    /// Monitoring interval in seconds
    monitoring_interval_seconds: u64,
    /// Temperature thresholds
    temperature_thresholds: TemperatureThresholds,
}

impl GpuConfig {
    fn from_resource_config(config: ResourceConfig) -> Self {
        Self {
            max_memory_per_gpu_gb: config.gpu_limits.max_memory_per_gpu_gb,
            max_allocation_time_minutes: config.gpu_limits.max_allocation_time_minutes,
            monitoring_interval_seconds: config.monitoring.metrics_interval_seconds,
            temperature_thresholds: TemperatureThresholds::default(),
        }
    }
}

/// Temperature thresholds for GPU monitoring
#[derive(Debug, Clone)]
struct TemperatureThresholds {
    /// Warning temperature in Celsius
    warning_celsius: f64,
    /// Critical temperature in Celsius
    critical_celsius: f64,
}

impl Default for TemperatureThresholds {
    fn default() -> Self {
        Self {
            warning_celsius: 80.0,
            critical_celsius: 90.0,
        }
    }
}

/// GPU allocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuAllocationRequest {
    /// Request ID
    pub request_id: Uuid,
    /// GPU requirements
    pub requirements: GpuRequirements,
    /// Priority level
    pub priority: Priority,
    /// Requested duration in minutes
    pub duration_minutes: u64,
    /// Requesting layer
    pub requesting_layer: String,
}

impl GpuAllocationRequest {
    /// Create a new GPU allocation request
    pub fn new(requirements: GpuRequirements, priority: Priority, requesting_layer: String) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            requirements,
            priority,
            duration_minutes: 60,
            requesting_layer,
        }
    }
}

/// GPU requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirements {
    /// GPU memory required in GB
    pub memory_gb: u64,
    /// Compute capability required (e.g., "8.0")
    pub compute_capability: Option<String>,
    /// Special features required
    pub features: Vec<String>,
}

impl Default for GpuRequirements {
    fn default() -> Self {
        Self {
            memory_gb: 8,
            compute_capability: Some("7.0".to_string()),
            features: Vec::new(),
        }
    }
}

/// GPU device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    /// GPU ID
    pub id: String,
    /// Total memory in GB
    pub memory_gb: u64,
    /// Compute capability
    pub compute_capability: String,
    /// Maximum power consumption in watts
    pub max_power_watts: u32,
}

/// GPU allocation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuAllocation {
    /// Allocation ID
    pub allocation_id: Uuid,
    /// GPU ID
    pub gpu_id: String,
    /// Resource requirements
    pub requirements: GpuRequirements,
    /// Allocation timestamp
    pub allocated_at: DateTime<Utc>,
    /// Allocation status
    pub status: GpuAllocationStatus,
    /// Performance metrics
    pub performance_metrics: GpuPerformanceMetrics,
}

/// GPU allocation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpuAllocationStatus {
    /// GPU is actively allocated
    Active,
    /// GPU allocation is pending
    Pending,
    /// GPU allocation completed
    Completed,
    /// GPU allocation failed
    Failed,
    /// GPU allocation released
    Released,
}

impl Default for GpuAllocationStatus {
    fn default() -> Self {
        GpuAllocationStatus::Pending
    }
}

/// GPU device status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpuDeviceStatus {
    /// GPU is available for allocation
    Available,
    /// GPU is currently allocated
    Allocated,
    /// GPU is in maintenance mode
    Maintenance,
    /// GPU is offline or failed
    Offline,
}

impl Default for GpuDeviceStatus {
    fn default() -> Self {
        GpuDeviceStatus::Available
    }
}

/// GPU performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPerformanceMetrics {
    /// GPU utilization percentage (0.0 to 1.0)
    pub utilization_percentage: f64,
    /// Memory utilization percentage (0.0 to 1.0)
    pub memory_utilization_percentage: f64,
    /// Temperature in Celsius
    pub temperature_celsius: f64,
    /// Power consumption in watts
    pub power_usage_watts: u32,
    /// Memory bandwidth utilization
    pub memory_bandwidth_utilization: f64,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Default for GpuPerformanceMetrics {
    fn default() -> Self {
        Self {
            utilization_percentage: 0.0,
            memory_utilization_percentage: 0.0,
            temperature_celsius: 0.0,
            power_usage_watts: 0,
            memory_bandwidth_utilization: 0.0,
            last_update: Utc::now(),
        }
    }
}

/// GPU metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    /// Total GPU count
    pub total_gpus: u32,
    /// Available GPU count
    pub available_gpus: u32,
    /// Average utilization across all GPUs
    pub average_utilization: f64,
    /// Average temperature across all GPUs
    pub average_temperature: f64,
    /// Total power consumption
    pub total_power_watts: u32,
    /// Cost per hour for all GPUs
    pub cost_per_hour: f64,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Default for GpuMetrics {
    fn default() -> Self {
        Self {
            total_gpus: 0,
            available_gpus: 0,
            average_utilization: 0.0,
            average_temperature: 0.0,
            total_power_watts: 0,
            cost_per_hour: 0.0,
            last_update: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_manager_initialization() {
        let config = ResourceConfig::default();
        let manager = GpuManager::new(config).await;

        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_gpu_allocation() {
        let config = ResourceConfig::default();
        let manager = GpuManager::new(config).await.unwrap();

        let request = GpuAllocationRequest::new(
            GpuRequirements {
                memory_gb: 8,
                compute_capability: Some("7.0".to_string()),
                features: Vec::new(),
            },
            Priority::Normal,
            "layer7".to_string(),
        );

        let allocation = manager.allocate_gpu(request).await;
        assert!(allocation.is_ok());
    }

    #[tokio::test]
    async fn test_insufficient_gpu_memory() {
        let config = ResourceConfig::default();
        let manager = GpuManager::new(config).await.unwrap();

        let request = GpuAllocationRequest::new(
            GpuRequirements {
                memory_gb: 100, // More than available
                compute_capability: Some("7.0".to_string()),
                features: Vec::new(),
            },
            Priority::Normal,
            "layer7".to_string(),
        );

        let allocation = manager.allocate_gpu(request).await;
        assert!(matches!(allocation, Err(ResourceError::InsufficientResources { .. })));
    }
}