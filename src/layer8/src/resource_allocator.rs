//! # Resource Allocator
//!
//! Core resource allocation engine for Layer 8. Manages GPU, CPU, memory,
//! and storage allocation across all layers of the autonomous AI system.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Resource allocation engine
pub struct ResourceAllocator {
    /// Current allocations
    allocations: Arc<RwLock<HashMap<Uuid, ResourceAllocation>>>,
    /// Available resources
    available_resources: Arc<RwLock<AvailableResources>>,
    /// Configuration
    config: ResourceConfig,
    /// Allocation policies
    policies: Arc<RwLock<AllocationPolicies>>,
}

impl ResourceAllocator {
    /// Create a new resource allocator
    pub async fn new(config: ResourceConfig) -> Result<Self> {
        info!("Initializing resource allocator...");

        let allocator = Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            available_resources: Arc::new(RwLock::new(AvailableResources::new().await?)),
            config,
            policies: Arc::new(RwLock::new(AllocationPolicies::default())),
        };

        info!("✅ Resource allocator initialized successfully");
        Ok(allocator)
    }

    /// Start the resource allocator service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting resource allocator service...");

        // Initialize available resources
        self.refresh_available_resources().await?;

        info!("✅ Resource allocator service started successfully");
        Ok(())
    }

    /// Stop the resource allocator service
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping resource allocator service...");

        // Clean up all allocations
        let mut allocations = self.allocations.write().await;
        for (allocation_id, allocation) in allocations.iter_mut() {
            if matches!(allocation.status, AllocationStatus::Active) {
                warn!("Force terminating active allocation: {}", allocation_id);
                allocation.status = AllocationStatus::Cancelled;
            }
        }

        info!("✅ Resource allocator service stopped successfully");
        Ok(())
    }

    /// Allocate resources for a request
    pub async fn allocate_resources(&self, request: ResourceRequest) -> ResourceResult<ResourceAllocation> {
        debug!("Processing resource allocation request: {}", request.request_id);

        // Validate request
        self.validate_request(&request).await?;

        // Check resource availability
        let available = self.available_resources.read().await;
        if !self.can_satisfy_request(&request.requirements, &available).await {
            return Err(ResourceError::InsufficientResources {
                requested: request.requirements.clone(),
                available: available.to_requirements(),
            });
        }

        // Check cost constraints
        let cost_info = self.calculate_cost(&request).await?;
        if let Some(max_cost) = request.max_cost_per_hour {
            if cost_info.cost_per_hour > max_cost {
                return Err(ResourceError::CostLimitExceeded {
                    budget: max_cost,
                    current: cost_info.cost_per_hour,
                });
            }
        }

        // Allocate resources
        let allocated_resources = self.allocate_from_pool(&request.requirements).await?;
        let allocation = ResourceAllocation::new(request.request_id, allocated_resources, cost_info);

        // Store allocation
        self.allocations.write().await.insert(allocation.allocation_id, allocation.clone());

        // Update available resources
        self.update_available_resources(&request.requirements, false).await?;

        info!("✅ Successfully allocated resources for request: {}", request.request_id);
        Ok(allocation)
    }

    /// Release a resource allocation
    pub async fn release_allocation(&self, allocation_id: Uuid) -> ResourceResult<()> {
        debug!("Releasing resource allocation: {}", allocation_id);

        let mut allocations = self.allocations.write().await;
        if let Some(mut allocation) = allocations.remove(&allocation_id) {
            allocation.status = AllocationStatus::Completed;
            allocation.end_time = Utc::now();

            // Return resources to pool
            self.update_available_resources(&allocation.allocated_resources.to_requirements(), true).await?;

            info!("✅ Successfully released allocation: {}", allocation_id);
            Ok(())
        } else {
            Err(ResourceError::AllocationFailed {
                message: format!("Allocation not found: {}", allocation_id),
            })
        }
    }

    /// Get allocation status
    pub async fn get_allocation(&self, allocation_id: Uuid) -> Option<ResourceAllocation> {
        self.allocations.read().await.get(&allocation_id).cloned()
    }

    /// Get all active allocations
    pub async fn get_active_allocations(&self) -> Vec<ResourceAllocation> {
        self.allocations.read().await
            .values()
            .filter(|allocation| matches!(allocation.status, AllocationStatus::Active))
            .cloned()
            .collect()
    }

    /// Get allocation status for health checks
    pub async fn get_status(&self) -> Result<types::AllocationStatus> {
        let allocations = self.allocations.read().await;
        let active_count = allocations.values()
            .filter(|allocation| matches!(allocation.status, AllocationStatus::Active))
            .count();

        Ok(types::AllocationStatus {
            total_allocations: allocations.len(),
            active_allocations: active_count,
            available_resources: self.available_resources.read().await.to_requirements(),
            utilization_percentage: self.calculate_utilization().await?,
        })
    }

    /// Health check implementation
    pub async fn health_check(&self) -> Result<()> {
        // Check if we can access allocations
        let _allocations = self.allocations.read().await;

        // Check if we can access available resources
        let _resources = self.available_resources.read().await;

        Ok(())
    }

    /// Readiness check
    pub async fn is_ready(&self) -> bool {
        self.health_check().await.is_ok()
    }

    // Private helper methods

    async fn validate_request(&self, request: &ResourceRequest) -> ResourceResult<()> {
        // Validate GPU requirements
        if request.requirements.gpu_count > self.config.gpu_limits.max_gpus_per_allocation {
            return Err(ResourceError::AllocationFailed {
                message: format!(
                    "GPU count {} exceeds maximum {}",
                    request.requirements.gpu_count,
                    self.config.gpu_limits.max_gpus_per_allocation
                ),
            });
        }

        // Validate memory requirements
        if request.requirements.gpu_memory_gb > self.config.gpu_limits.max_memory_per_gpu_gb {
            return Err(ResourceError::AllocationFailed {
                message: format!(
                    "GPU memory {}GB exceeds maximum {}GB",
                    request.requirements.gpu_memory_gb,
                    self.config.gpu_limits.max_memory_per_gpu_gb
                ),
            });
        }

        // Validate duration
        if request.duration_minutes > self.config.gpu_limits.max_allocation_time_minutes {
            return Err(ResourceError::AllocationFailed {
                message: format!(
                    "Duration {} minutes exceeds maximum {} minutes",
                    request.duration_minutes,
                    self.config.gpu_limits.max_allocation_time_minutes
                ),
            });
        }

        Ok(())
    }

    async fn can_satisfy_request(&self, requirements: &ResourceRequirements, available: &AvailableResources) -> bool {
        requirements.gpu_count <= available.gpu_count &&
        requirements.cpu_cores <= available.cpu_cores &&
        requirements.ram_gb <= available.ram_gb &&
        requirements.storage_gb <= available.storage_gb
    }

    async fn calculate_cost(&self, request: &ResourceRequest) -> ResourceResult<CostInfo> {
        let cost_per_hour = self.config.cost_settings.cost_per_gpu_hour * request.requirements.gpu_count as f64;

        let total_cost = cost_per_hour * (request.duration_minutes as f64 / 60.0);

        let breakdown = CostBreakdown {
            gpu_cost: cost_per_hour * 0.7, // 70% of cost from GPUs
            cpu_cost: cost_per_hour * 0.2, // 20% from CPU
            memory_cost: cost_per_hour * 0.05, // 5% from memory
            storage_cost: cost_per_hour * 0.05, // 5% from storage
        };

        Ok(CostInfo {
            cost_per_hour,
            total_cost,
            currency: "USD".to_string(),
            breakdown,
        })
    }

    async fn allocate_from_pool(&self, requirements: &ResourceRequirements) -> ResourceResult<AllocatedResources> {
        let mut available = self.available_resources.write().await;

        // Allocate GPUs
        let gpu_ids = available.allocate_gpus(requirements.gpu_count)?;

        // Allocate CPU cores
        available.cpu_cores -= requirements.cpu_cores;

        // Allocate RAM
        available.ram_gb -= requirements.ram_gb;

        // Allocate storage
        available.storage_gb -= requirements.storage_gb;

        // Create Kubernetes info (simplified for now)
        let kubernetes_info = KubernetesInfo {
            pod_name: format!("resource-allocation-{}", Uuid::new_v4().simple()),
            node_name: "resource-node-1".to_string(),
            namespace: "default".to_string(),
            limits: ResourceLimits {
                cpu: format!("{}m", requirements.cpu_cores * 1000),
                memory: format!("{}Gi", requirements.ram_gb),
                gpu: Some(requirements.gpu_count.to_string()),
            },
            requests: ResourceLimits {
                cpu: format!("{}m", requirements.cpu_cores * 800), // 80% of limit
                memory: format!("{}Gi", requirements.ram_gb * 8 / 10), // 80% of limit
                gpu: Some(requirements.gpu_count.to_string()),
            },
        };

        Ok(AllocatedResources {
            gpu_ids,
            cpu_cores: requirements.cpu_cores,
            ram_gb: requirements.ram_gb,
            storage_gb: requirements.storage_gb,
            kubernetes_info,
        })
    }

    async fn update_available_resources(&self, requirements: &ResourceRequirements, return_resources: bool) -> Result<()> {
        let mut available = self.available_resources.write().await;

        if return_resources {
            // Return resources to pool
            available.gpu_count += requirements.gpu_count;
            available.cpu_cores += requirements.cpu_cores;
            available.ram_gb += requirements.ram_gb;
            available.storage_gb += requirements.storage_gb;
        } else {
            // Remove resources from pool
            available.gpu_count -= requirements.gpu_count;
            available.cpu_cores -= requirements.cpu_cores;
            available.ram_gb -= requirements.ram_gb;
            available.storage_gb -= requirements.storage_gb;
        }

        Ok(())
    }

    async fn refresh_available_resources(&self) -> Result<()> {
        // In a real implementation, this would query Kubernetes API
        // For now, we'll use default values
        let mut available = self.available_resources.write().await;
        *available = AvailableResources::new().await?;
        Ok(())
    }

    async fn calculate_utilization(&self) -> ResourceResult<f64> {
        let available = self.available_resources.read().await;
        let total_resources = AvailableResources::new().await?;

        let gpu_utilization = if total_resources.gpu_count > 0 {
            (total_resources.gpu_count - available.gpu_count) as f64 / total_resources.gpu_count as f64
        } else {
            0.0
        };

        let cpu_utilization = if total_resources.cpu_cores > 0 {
            (total_resources.cpu_cores - available.cpu_cores) as f64 / total_resources.cpu_cores as f64
        } else {
            0.0
        };

        // Return average utilization
        Ok((gpu_utilization + cpu_utilization) / 2.0)
    }
}

/// Available resources in the system
#[derive(Debug, Clone)]
struct AvailableResources {
    gpu_count: u32,
    cpu_cores: u32,
    ram_gb: u64,
    storage_gb: u64,
    gpu_ids: Vec<String>,
}

impl AvailableResources {
    async fn new() -> Result<Self> {
        // In a real implementation, this would query Kubernetes API
        // For now, return default test values
        Ok(Self {
            gpu_count: 4,
            cpu_cores: 32,
            ram_gb: 128,
            storage_gb: 1000,
            gpu_ids: vec!["gpu-0".to_string(), "gpu-1".to_string(), "gpu-2".to_string(), "gpu-3".to_string()],
        })
    }

    fn to_requirements(&self) -> ResourceRequirements {
        ResourceRequirements {
            gpu_count: self.gpu_count,
            gpu_memory_gb: 24, // Assume 24GB per GPU
            cpu_cores: self.cpu_cores,
            ram_gb: self.ram_gb,
            storage_gb: self.storage_gb,
            special_requirements: Vec::new(),
        }
    }

    fn allocate_gpus(&mut self, count: u32) -> ResourceResult<Vec<String>> {
        if count > self.gpu_count {
            return Err(ResourceError::InsufficientResources {
                requested: ResourceRequirements {
                    gpu_count: count,
                    ..Default::default()
                },
                available: self.to_requirements(),
            });
        }

        let allocated = self.gpu_ids.drain(0..count as usize).collect();
        self.gpu_count -= count;

        Ok(allocated)
    }
}

impl Default for AvailableResources {
    fn default() -> Self {
        Self {
            gpu_count: 0,
            cpu_cores: 0,
            ram_gb: 0,
            storage_gb: 0,
            gpu_ids: Vec::new(),
        }
    }
}

/// Allocation policies for different priority levels
#[derive(Debug, Clone)]
struct AllocationPolicies {
    /// Preemption enabled for critical requests
    enable_preemption: bool,
    /// Resource overcommitment ratios
    overcommit_ratios: HashMap<String, f64>,
    /// Fair sharing policies
    fair_sharing: FairSharingPolicy,
}

impl Default for AllocationPolicies {
    fn default() -> Self {
        let mut overcommit_ratios = HashMap::new();
        overcommit_ratios.insert("gpu".to_string(), 1.0);
        overcommit_ratios.insert("cpu".to_string(), 1.2);
        overcommit_ratios.insert("memory".to_string(), 1.1);

        Self {
            enable_preemption: true,
            overcommit_ratios,
            fair_sharing: FairSharingPolicy::default(),
        }
    }
}

/// Fair sharing policy for resource allocation
#[derive(Debug, Clone)]
struct FairSharingPolicy {
    /// Maximum allocation per layer
    max_per_layer: HashMap<String, ResourceRequirements>,
    /// Priority weights
    priority_weights: HashMap<Priority, f64>,
}

impl Default for FairSharingPolicy {
    fn default() -> Self {
        let mut max_per_layer = HashMap::new();
        max_per_layer.insert("layer4".to_string(), ResourceRequirements {
            gpu_count: 2,
            gpu_memory_gb: 48,
            cpu_cores: 16,
            ram_gb: 64,
            storage_gb: 200,
            special_requirements: Vec::new(),
        });
        max_per_layer.insert("layer5".to_string(), ResourceRequirements {
            gpu_count: 3,
            gpu_memory_gb: 72,
            cpu_cores: 24,
            ram_gb: 96,
            storage_gb: 500,
            special_requirements: Vec::new(),
        });
        max_per_layer.insert("layer7".to_string(), ResourceRequirements {
            gpu_count: 2,
            gpu_memory_gb: 48,
            cpu_cores: 12,
            ram_gb: 48,
            storage_gb: 300,
            special_requirements: Vec::new(),
        });

        let mut priority_weights = HashMap::new();
        priority_weights.insert(Priority::Critical, 1.0);
        priority_weights.insert(Priority::High, 0.8);
        priority_weights.insert(Priority::Normal, 0.6);
        priority_weights.insert(Priority::Low, 0.4);

        Self {
            max_per_layer,
            priority_weights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_allocator_initialization() {
        let config = ResourceConfig::default();
        let allocator = ResourceAllocator::new(config).await;

        assert!(allocator.is_ok());
    }

    #[tokio::test]
    async fn test_resource_allocation() {
        let config = ResourceConfig::default();
        let allocator = ResourceAllocator::new(config).await.unwrap();

        let request = ResourceRequest::new(
            "layer4".to_string(),
            ResourceRequirements {
                gpu_count: 1,
                gpu_memory_gb: 8,
                cpu_cores: 4,
                ram_gb: 16,
                storage_gb: 50,
                special_requirements: Vec::new(),
            },
            Priority::Normal,
        );

        let allocation = allocator.allocate_resources(request).await;
        assert!(allocation.is_ok());
    }

    #[tokio::test]
    async fn test_insufficient_resources() {
        let config = ResourceConfig::default();
        let allocator = ResourceAllocator::new(config).await.unwrap();

        let request = ResourceRequest::new(
            "layer4".to_string(),
            ResourceRequirements {
                gpu_count: 10, // More than available
                gpu_memory_gb: 8,
                cpu_cores: 4,
                ram_gb: 16,
                storage_gb: 50,
                special_requirements: Vec::new(),
            },
            Priority::Normal,
        );

        let allocation = allocator.allocate_resources(request).await;
        assert!(matches!(allocation, Err(ResourceError::InsufficientResources { .. })));
    }
}
