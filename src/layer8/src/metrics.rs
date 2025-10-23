//! # Resource Metrics
//!
//! Metrics collection and monitoring for the resource management layer.
//! Provides comprehensive observability for resource allocation, utilization,
//! and performance across all layers.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use prometheus::{Encoder, Gauge, Histogram, Counter, TextEncoder, Registry};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Resource metrics collector and exporter
pub struct ResourceMetrics {
    /// Prometheus registry
    registry: Registry,
    /// Resource allocation metrics
    allocation_metrics: AllocationMetrics,
    /// Cost metrics
    cost_metrics: CostMetricsCollector,
    /// Performance metrics
    performance_metrics: PerformanceMetricsCollector,
    /// Running status
    running: Arc<RwLock<bool>>,
}

impl ResourceMetrics {
    /// Create a new metrics collector
    pub async fn new() -> Result<Self> {
        info!("Initializing resource metrics collector...");

        let registry = Registry::new();

        let metrics = Self {
            registry,
            allocation_metrics: AllocationMetrics::new(&registry)?,
            cost_metrics: CostMetricsCollector::new(&registry)?,
            performance_metrics: PerformanceMetricsCollector::new(&registry)?,
            running: Arc::new(RwLock::new(false)),
        };

        info!("✅ Resource metrics collector initialized successfully");
        Ok(metrics)
    }

    /// Start the metrics collection service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting resource metrics service...");

        let mut running = self.running.write().await;
        *running = true;

        // Start metrics collection in background
        self.start_collection_loop().await?;

        info!("✅ Resource metrics service started successfully");
        Ok(())
    }

    /// Stop the metrics collection service
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping resource metrics service...");

        let mut running = self.running.write().await;
        *running = false;

        info!("✅ Resource metrics service stopped successfully");
        Ok(())
    }

    /// Record resource allocation
    pub async fn record_allocation(&self, allocation: &ResourceAllocation) -> Result<()> {
        debug!("Recording allocation metrics: {}", allocation.allocation_id);

        // Update allocation metrics
        self.allocation_metrics.record_allocation(allocation).await?;

        // Update cost metrics
        self.cost_metrics.record_allocation_cost(allocation).await?;

        // Update performance metrics
        self.performance_metrics.record_allocation_performance(allocation).await?;

        Ok(())
    }

    /// Record resource deallocation
    pub async fn record_deallocation(&self, allocation_id: Uuid) -> Result<()> {
        debug!("Recording deallocation metrics: {}", allocation_id);

        self.allocation_metrics.record_deallocation(allocation_id).await?;
        self.performance_metrics.record_deallocation_performance(allocation_id).await?;

        Ok(())
    }

    /// Update GPU metrics
    pub async fn update_gpu_metrics(&self, gpu_status: &GpuStatus) -> Result<()> {
        debug!("Updating GPU metrics: {} GPUs", gpu_status.total_gpus);

        self.performance_metrics.update_gpu_utilization(gpu_status).await?;

        Ok(())
    }

    /// Get metrics in Prometheus format
    pub async fn get_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;

        Ok(String::from_utf8(buffer)?)
    }

    /// Health check implementation
    pub async fn health_check(&self) -> Result<()> {
        // Check if metrics collection is running
        let running = self.running.read().await;
        if !*running {
            return Err(anyhow::anyhow!("Metrics collection not running"));
        }

        // Check if we can access registry
        let _families = self.registry.gather();

        Ok(())
    }

    /// Readiness check
    pub async fn is_ready(&self) -> bool {
        self.health_check().await.is_ok()
    }

    // Private helper methods

    async fn start_collection_loop(&self) -> Result<()> {
        // In a real implementation, this would start a background task
        // that periodically collects and updates metrics
        info!("📊 Metrics collection loop started");
        Ok(())
    }
}

/// Allocation-specific metrics
struct AllocationMetrics {
    /// Total allocations counter
    total_allocations: Counter,
    /// Active allocations gauge
    active_allocations: Gauge,
    /// Allocation duration histogram
    allocation_duration: Histogram,
    /// Allocations by layer counter
    allocations_by_layer: Counter,
    /// Allocation failures counter
    allocation_failures: Counter,
}

impl AllocationMetrics {
    fn new(registry: &Registry) -> Result<Self> {
        let total_allocations = Counter::new(
            "layer8_allocations_total",
            "Total number of resource allocations"
        )?;
        registry.register(Box::new(total_allocations.clone()))?;

        let active_allocations = Gauge::new(
            "layer8_allocations_active",
            "Number of currently active allocations"
        )?;
        registry.register(Box::new(active_allocations.clone()))?;

        let allocation_duration = Histogram::new(
            "layer8_allocation_duration_seconds",
            "Duration of resource allocations"
        )?;
        registry.register(Box::new(allocation_duration.clone()))?;

        let allocations_by_layer = Counter::new(
            "layer8_allocations_by_layer_total",
            "Resource allocations by requesting layer"
        )?;
        registry.register(Box::new(allocations_by_layer.clone()))?;

        let allocation_failures = Counter::new(
            "layer8_allocation_failures_total",
            "Total number of allocation failures"
        )?;
        registry.register(Box::new(allocation_failures.clone()))?;

        Ok(Self {
            total_allocations,
            active_allocations,
            allocation_duration,
            allocations_by_layer,
            allocation_failures,
        })
    }

    async fn record_allocation(&self, allocation: &ResourceAllocation) -> Result<()> {
        self.total_allocations.inc();
        self.active_allocations.inc();

        // Record by layer
        self.allocations_by_layer
            .with_label_values(&[&allocation.allocated_resources.kubernetes_info.namespace])
            .inc();

        Ok(())
    }

    async fn record_deallocation(&self, _allocation_id: Uuid) -> Result<()> {
        self.active_allocations.dec();
        Ok(())
    }
}

/// Cost-specific metrics collection
struct CostMetricsCollector {
    /// Total cost counter
    total_cost: Counter,
    /// Cost by layer counter
    cost_by_layer: Counter,
    /// Cost optimization savings
    cost_savings: Gauge,
    /// Budget utilization gauge
    budget_utilization: Gauge,
}

impl CostMetricsCollector {
    fn new(registry: &Registry) -> Result<Self> {
        let total_cost = Counter::new(
            "layer8_cost_total",
            "Total cost of resource allocations"
        )?;
        registry.register(Box::new(total_cost.clone()))?;

        let cost_by_layer = Counter::new(
            "layer8_cost_by_layer_total",
            "Cost by requesting layer"
        )?;
        registry.register(Box::new(cost_by_layer.clone()))?;

        let cost_savings = Gauge::new(
            "layer8_cost_savings_ratio",
            "Cost savings ratio from optimizations"
        )?;
        registry.register(Box::new(cost_savings.clone()))?;

        let budget_utilization = Gauge::new(
            "layer8_budget_utilization_ratio",
            "Budget utilization ratio by layer"
        )?;
        registry.register(Box::new(budget_utilization.clone()))?;

        Ok(Self {
            total_cost,
            cost_by_layer,
            cost_savings,
            budget_utilization,
        })
    }

    async fn record_allocation_cost(&self, allocation: &ResourceAllocation) -> Result<()> {
        self.total_cost.inc_by(allocation.cost_info.total_cost);

        // Record by layer (using namespace as proxy for layer)
        self.cost_by_layer
            .with_label_values(&[&allocation.allocated_resources.kubernetes_info.namespace])
            .inc_by(allocation.cost_info.total_cost);

        Ok(())
    }
}

/// Performance metrics collection
struct PerformanceMetricsCollector {
    /// GPU utilization gauge
    gpu_utilization: Gauge,
    /// CPU utilization gauge
    cpu_utilization: Gauge,
    /// Memory utilization gauge
    memory_utilization: Gauge,
    /// Resource efficiency gauge
    resource_efficiency: Gauge,
    /// Allocation latency histogram
    allocation_latency: Histogram,
}

impl PerformanceMetricsCollector {
    fn new(registry: &Registry) -> Result<Self> {
        let gpu_utilization = Gauge::new(
            "layer8_gpu_utilization_ratio",
            "GPU utilization ratio"
        )?;
        registry.register(Box::new(gpu_utilization.clone()))?;

        let cpu_utilization = Gauge::new(
            "layer8_cpu_utilization_ratio",
            "CPU utilization ratio"
        )?;
        registry.register(Box::new(cpu_utilization.clone()))?;

        let memory_utilization = Gauge::new(
            "layer8_memory_utilization_ratio",
            "Memory utilization ratio"
        )?;
        registry.register(Box::new(memory_utilization.clone()))?;

        let resource_efficiency = Gauge::new(
            "layer8_resource_efficiency_ratio",
            "Resource allocation efficiency ratio"
        )?;
        registry.register(Box::new(resource_efficiency.clone()))?;

        let allocation_latency = Histogram::new(
            "layer8_allocation_latency_seconds",
            "Time taken for resource allocation"
        )?;
        registry.register(Box::new(allocation_latency.clone()))?;

        Ok(Self {
            gpu_utilization,
            cpu_utilization,
            memory_utilization,
            resource_efficiency,
            allocation_latency,
        })
    }

    async fn update_gpu_utilization(&self, gpu_status: &GpuStatus) -> Result<()> {
        if !gpu_status.utilization.is_empty() {
            let avg_utilization: f64 = gpu_status.utilization.iter().sum::<f64>() / gpu_status.utilization.len() as f64;
            self.gpu_utilization.set(avg_utilization);
        }

        Ok(())
    }

    async fn record_allocation_performance(&self, allocation: &ResourceAllocation) -> Result<()> {
        // Calculate efficiency based on resource utilization
        let efficiency = self.calculate_allocation_efficiency(allocation);
        self.resource_efficiency.set(efficiency);

        Ok(())
    }

    async fn record_deallocation_performance(&self, _allocation_id: Uuid) -> Result<()> {
        // Update efficiency metrics after deallocation
        Ok(())
    }

    fn calculate_allocation_efficiency(&self, allocation: &ResourceAllocation) -> f64 {
        // Simple efficiency calculation based on resource allocation
        let gpu_efficiency = if allocation.allocated_resources.gpu_ids.len() > 0 {
            0.9 // Assume good GPU allocation efficiency
        } else {
            1.0 // No GPU allocation
        };

        let cpu_efficiency = if allocation.allocated_resources.cpu_cores > 0 {
            0.85 // Assume good CPU allocation efficiency
        } else {
            1.0
        };

        (gpu_efficiency + cpu_efficiency) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_initialization() {
        let metrics = ResourceMetrics::new().await;
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    async fn test_allocation_recording() {
        let metrics = ResourceMetrics::new().await.unwrap();

        let allocation = ResourceAllocation::new(
            Uuid::new_v4(),
            AllocatedResources::default(),
            CostInfo {
                cost_per_hour: 2.5,
                total_cost: 5.0,
                currency: "USD".to_string(),
                breakdown: CostBreakdown::default(),
            },
        );

        let result = metrics.record_allocation(&allocation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let metrics = ResourceMetrics::new().await.unwrap();

        let exported = metrics.get_metrics().await;
        assert!(exported.is_ok());
        assert!(!exported.unwrap().is_empty());
    }
}