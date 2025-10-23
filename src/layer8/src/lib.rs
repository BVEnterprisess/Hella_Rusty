//! # Layer 8 (Resource Management)
//!
//! Resource management layer responsible for GPU allocation, cost optimization,
//! and compute resource scheduling across the autonomous AI system.
//!
//! ## Core Responsibilities
//!
//! - **GPU Resource Management**: Allocate and optimize GPU resources for ML workloads
//! - **Cost Optimization**: Monitor and optimize compute costs across all layers
//! - **Resource Scheduling**: Intelligent scheduling of compute resources
//! - **Capacity Planning**: Predictive resource allocation based on system needs
//! - **Integration Management**: Coordinate resource requests from Layers 4, 5, 7
//!
//! ## Architecture
//!
//! Layer 8 acts as the central resource coordinator, receiving requests from:
//! - **Layer 4 (Execution)**: Agent runtime resource requirements
//! - **Layer 5 (Refinement)**: ML training and optimization resource needs
//! - **Layer 7 (Evolution)**: Genetic algorithm computation resources
//!
//! ## Key Components
//!
//! - **ResourceAllocator**: Main resource allocation engine
//! - **CostOptimizer**: Cost monitoring and optimization
//! - **GpuManager**: GPU resource management and scheduling
//! - **CapacityPlanner**: Predictive resource planning
//! - **IntegrationManager**: Cross-layer resource coordination

pub mod types;
pub mod resource_allocator;
pub mod cost_optimizer;
pub mod gpu_manager;
pub mod capacity_planner;
pub mod integration;
pub mod metrics;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Main resource management service
pub struct ResourceManager {
    allocator: Arc<RwLock<resource_allocator::ResourceAllocator>>,
    cost_optimizer: Arc<RwLock<cost_optimizer::CostOptimizer>>,
    gpu_manager: Arc<RwLock<gpu_manager::GpuManager>>,
    capacity_planner: Arc<RwLock<capacity_planner::CapacityPlanner>>,
    integration: Arc<RwLock<integration::IntegrationManager>>,
    metrics: Arc<RwLock<metrics::ResourceMetrics>>,
}

impl ResourceManager {
    /// Create a new resource manager instance
    pub async fn new(config: types::ResourceConfig) -> Result<Self> {
        info!("Initializing Layer 8 (Resource Management) service...");

        let allocator = Arc::new(RwLock::new(
            resource_allocator::ResourceAllocator::new(config.clone()).await?
        ));

        let cost_optimizer = Arc::new(RwLock::new(
            cost_optimizer::CostOptimizer::new(config.clone()).await?
        ));

        let gpu_manager = Arc::new(RwLock::new(
            gpu_manager::GpuManager::new(config.clone()).await?
        ));

        let capacity_planner = Arc::new(RwLock::new(
            capacity_planner::CapacityPlanner::new(config.clone()).await?
        ));

        let integration = Arc::new(RwLock::new(
            integration::IntegrationManager::new(config.clone()).await?
        ));

        let metrics = Arc::new(RwLock::new(
            metrics::ResourceMetrics::new().await?
        ));

        info!("✅ Layer 8 (Resource Management) initialized successfully");
        Ok(Self {
            allocator,
            cost_optimizer,
            gpu_manager,
            capacity_planner,
            integration,
            metrics,
        })
    }

    /// Start the resource management service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Layer 8 (Resource Management) service...");

        // Start all sub-components
        self.allocator.write().await.start().await?;
        self.cost_optimizer.write().await.start().await?;
        self.gpu_manager.write().await.start().await?;
        self.capacity_planner.write().await.start().await?;
        self.integration.write().await.start().await?;
        self.metrics.write().await.start().await?;

        info!("✅ Layer 8 (Resource Management) service started successfully");
        Ok(())
    }

    /// Stop the resource management service
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Layer 8 (Resource Management) service...");

        // Stop all sub-components in reverse order
        self.metrics.write().await.stop().await?;
        self.integration.write().await.stop().await?;
        self.capacity_planner.write().await.stop().await?;
        self.gpu_manager.write().await.stop().await?;
        self.cost_optimizer.write().await.stop().await?;
        self.allocator.write().await.stop().await?;

        info!("✅ Layer 8 (Resource Management) service stopped successfully");
        Ok(())
    }

    /// Get current resource allocation status
    pub async fn get_allocation_status(&self) -> Result<types::AllocationStatus> {
        self.allocator.read().await.get_status().await
    }

    /// Get cost optimization metrics
    pub async fn get_cost_metrics(&self) -> Result<types::CostMetrics> {
        self.cost_optimizer.read().await.get_metrics().await
    }

    /// Get GPU utilization status
    pub async fn get_gpu_status(&self) -> Result<types::GpuStatus> {
        self.gpu_manager.read().await.get_status().await
    }

    /// Get capacity planning recommendations
    pub async fn get_capacity_recommendations(&self) -> Result<types::CapacityRecommendations> {
        self.capacity_planner.read().await.get_recommendations().await
    }
}

/// Health check implementation for Layer 8
#[async_trait]
pub trait ResourceHealth {
    async fn health_check(&self) -> Result<types::HealthStatus>;
    async fn readiness_check(&self) -> Result<types::ReadinessStatus>;
    async fn liveness_check(&self) -> Result<types::LivenessStatus>;
}

#[async_trait]
impl ResourceHealth for ResourceManager {
    async fn health_check(&self) -> Result<types::HealthStatus> {
        let mut status = types::HealthStatus::healthy();

        // Check all sub-components
        if let Err(e) = self.allocator.read().await.health_check().await {
            status.add_issue("allocator", e.to_string());
        }

        if let Err(e) = self.gpu_manager.read().await.health_check().await {
            status.add_issue("gpu_manager", e.to_string());
        }

        if let Err(e) = self.integration.read().await.health_check().await {
            status.add_issue("integration", e.to_string());
        }

        Ok(status)
    }

    async fn readiness_check(&self) -> Result<types::ReadinessStatus> {
        let mut ready = true;
        let mut issues = Vec::new();

        // Check if all components are ready
        if !self.allocator.read().await.is_ready().await {
            ready = false;
            issues.push("Resource allocator not ready".to_string());
        }

        if !self.gpu_manager.read().await.is_ready().await {
            ready = false;
            issues.push("GPU manager not ready".to_string());
        }

        Ok(types::ReadinessStatus { ready, issues })
    }

    async fn liveness_check(&self) -> Result<types::LivenessStatus> {
        let alive = self.health_check().await.is_ok();
        Ok(types::LivenessStatus { alive })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_manager_initialization() {
        let config = types::ResourceConfig::default();
        let manager = ResourceManager::new(config).await;

        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = types::ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let health = manager.health_check().await;
        assert!(health.is_ok());
    }
}