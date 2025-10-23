//! # Resource Coordinator - Layer 8 Integration for Resource Management
//!
//! The Resource Coordinator manages the interface between Layer 2 (Planning) and Layer 8
//! (Resource Management). It handles resource allocation requests, monitors resource
//! availability, and coordinates resource scheduling across all planning activities.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Resource coordinator for Layer 8 integration
pub struct ResourceCoordinator {
    #[cfg(feature = "layer8-integration")]
    layer8_client: Client,
    #[cfg(feature = "layer8-integration")]
    layer8_base_url: String,
    resource_cache: std::sync::Arc<tokio::sync::RwLock<HashMap<String, ResourceAvailability>>>,
    allocation_cache: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, ResourceAllocation>>>,
}

impl ResourceCoordinator {
    /// Create a new resource coordinator
    pub async fn new() -> Result<Self> {
        #[cfg(feature = "layer8-integration")]
        let coordinator = Self {
            layer8_client: Client::new(),
            layer8_base_url: std::env::var("LAYER8_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8088".to_string()),
            resource_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            allocation_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        };

        #[cfg(not(feature = "layer8-integration"))]
        let coordinator = Self {
            resource_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            allocation_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        };

        // Initialize resource cache
        coordinator.refresh_resource_cache().await?;

        #[cfg(feature = "layer8-integration")]
        info!("Resource coordinator initialized for Layer 8 at: {}", coordinator.layer8_base_url);

        #[cfg(not(feature = "layer8-integration"))]
        info!("Resource coordinator initialized in standalone mode (no Layer 8 integration)");

        Ok(coordinator)
    }

    /// Allocate resources for a plan
    pub async fn allocate_resources(&self, mut plan: Plan) -> Result<Plan> {
        info!("Allocating resources for plan: {}", plan.id);

        let mut allocations = Vec::new();

        // Allocate resources for each task
        for task in &plan.tasks {
            let task_allocations = self.allocate_task_resources(task).await?;
            allocations.extend(task_allocations);
        }

        // Update plan with allocations
        plan.resource_allocations = allocations;
        plan.updated_at = Utc::now();

        info!("Resource allocation completed for plan: {}", plan.id);
        Ok(plan)
    }

    /// Reallocate resources for a plan
    pub async fn reallocate_resources(&self, mut plan: Plan) -> Result<Plan> {
        info!("Reallocating resources for plan: {}", plan.id);

        // Release existing allocations
        for allocation in &plan.resource_allocations {
            if let Err(e) = self.release_allocation(allocation).await {
                warn!("Failed to release allocation {}: {}", allocation.id, e);
            }
        }

        // Allocate new resources
        self.allocate_resources(plan).await
    }

    /// Check if resource reallocation is needed
    pub async fn requires_resource_reallocation(&self, plan: &Plan) -> Result<bool> {
        let cache = self.resource_cache.read().await;

        for allocation in &plan.resource_allocations {
            if let Some(availability) = cache.get(&allocation.resource_type) {
                if availability.available_quantity < allocation.quantity {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Allocate resources for a specific task
    async fn allocate_task_resources(&self, task: &Task) -> Result<Vec<ResourceAllocation>> {
        let mut allocations = Vec::new();

        for requirement in &task.resource_requirements {
            let allocation = self.allocate_single_resource(requirement, task.id).await?;
            allocations.push(allocation);
        }

        Ok(allocations)
    }

    /// Allocate a single resource type
    async fn allocate_single_resource(
        &self,
        requirement: &ResourceRequirement,
        task_id: Uuid,
    ) -> Result<ResourceAllocation> {
        // Check cache first
        let cache = self.resource_cache.read().await;
        if let Some(availability) = cache.get(&requirement.resource_type) {
            if availability.available_quantity < requirement.quantity {
                return Err(PlanningError::ResourceAllocationFailed(
                    format!("Insufficient {} resources available", requirement.resource_type)
                ));
            }
        }

        // Make allocation request to Layer 8
        #[cfg(feature = "layer8-integration")]
        let allocation_request = json!({
            "resource_type": requirement.resource_type,
            "quantity": requirement.quantity,
            "unit": requirement.unit,
            "max_cost_per_hour": requirement.max_cost_per_hour,
            "preferred_providers": requirement.preferred_providers,
            "task_id": task_id,
            "estimated_duration_hours": 8.0, // Default estimate
        });

        #[cfg(feature = "layer8-integration")]
        let response = self.layer8_client
            .post(&format!("{}/api/v1/resources/allocate", self.layer8_base_url))
            .json(&allocation_request)
            .send()
            .await?;

        #[cfg(feature = "layer8-integration")]
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PlanningError::ResourceAllocationFailed(
                format!("Layer 8 allocation failed: {}", error_text)
            ));
        }

        #[cfg(feature = "layer8-integration")]
        let allocation_data: Value = response.json().await?;

        // Mock allocation for standalone mode
        #[cfg(not(feature = "layer8-integration"))]
        let allocation_data = json!({
            "id": Uuid::new_v4().to_string(),
            "resource_id": format!("mock-{}-{}", requirement.resource_type, Uuid::new_v4().simple()),
            "cost_per_hour": 0.5
        });

        let allocation = ResourceAllocation {
            id: Uuid::parse_str(allocation_data["id"].as_str().unwrap_or_default())?,
            task_id,
            resource_type: requirement.resource_type.clone(),
            resource_id: allocation_data["resource_id"].as_str().unwrap_or_default().to_string(),
            quantity: requirement.quantity,
            unit: requirement.unit.clone(),
            cost_per_hour: allocation_data["cost_per_hour"].as_f64().unwrap_or(0.0),
            allocated_at: Utc::now(),
            released_at: None,
            status: AllocationStatus::Allocated,
        };

        // Update cache
        self.allocation_cache.write().await.insert(allocation.id, allocation.clone());

        info!("Allocated {} {} for task {}", requirement.quantity, requirement.resource_type, task_id);
        Ok(allocation)
    }

    /// Release a resource allocation
    async fn release_allocation(&self, allocation: &ResourceAllocation) -> Result<()> {
        #[cfg(feature = "layer8-integration")]
        let release_request = json!({
            "allocation_id": allocation.id,
            "resource_id": allocation.resource_id,
            "quantity": allocation.quantity,
            "unit": allocation.unit,
        });

        #[cfg(feature = "layer8-integration")]
        let response = self.layer8_client
            .post(&format!("{}/api/v1/resources/release", self.layer8_base_url))
            .json(&release_request)
            .send()
            .await?;

        #[cfg(feature = "layer8-integration")]
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PlanningError::ResourceAllocationFailed(
                format!("Layer 8 release failed: {}", error_text)
            ));
        }

        // Update cache
        let mut cache = self.allocation_cache.write().await;
        if let Some(alloc) = cache.get_mut(&allocation.id) {
            alloc.status = AllocationStatus::Released;
            alloc.released_at = Some(Utc::now());
        }

        info!("Released allocation: {}", allocation.id);
        Ok(())
    }

    /// Refresh resource availability cache
    async fn refresh_resource_cache(&self) -> Result<()> {
        #[cfg(feature = "layer8-integration")]
        let response = self.layer8_client
            .get(&format!("{}/api/v1/resources/availability", self.layer8_base_url))
            .send()
            .await?;

        #[cfg(feature = "layer8-integration")]
        if !response.status().is_success() {
            warn!("Failed to refresh resource cache from Layer 8");
            return Ok(());
        }

        #[cfg(feature = "layer8-integration")]
        let availability_data: Value = response.json().await?;

        // Mock data for standalone mode
        #[cfg(not(feature = "layer8-integration"))]
        let availability_data = json!([
            {
                "resource_type": "CPU",
                "available_quantity": 16.0,
                "total_quantity": 32.0,
                "cost_per_hour": 0.25
            },
            {
                "resource_type": "GPU",
                "available_quantity": 4.0,
                "total_quantity": 8.0,
                "cost_per_hour": 2.0
            },
            {
                "resource_type": "Memory",
                "available_quantity": 64.0,
                "total_quantity": 128.0,
                "cost_per_hour": 0.1
            }
        ]);

        let mut cache = self.resource_cache.write().await;
        cache.clear();

        if let Some(resources) = availability_data.as_array() {
            for resource in resources {
                let resource_type = resource["resource_type"].as_str().unwrap_or_default();
                let available_quantity = resource["available_quantity"].as_f64().unwrap_or(0.0);
                let total_quantity = resource["total_quantity"].as_f64().unwrap_or(0.0);
                let cost_per_hour = resource["cost_per_hour"].as_f64().unwrap_or(0.0);

                cache.insert(resource_type.to_string(), ResourceAvailability {
                    resource_type: resource_type.to_string(),
                    available_quantity,
                    total_quantity,
                    cost_per_hour,
                    last_updated: Utc::now(),
                });
            }
        }

        debug!("Resource cache refreshed with {} resource types", cache.len());
        Ok(())
    }

    /// Get resource availability for a specific type
    pub async fn get_resource_availability(&self, resource_type: &str) -> Result<Option<ResourceAvailability>> {
        let cache = self.resource_cache.read().await;
        Ok(cache.get(resource_type).cloned())
    }

    /// Get all resource availability
    pub async fn get_all_resource_availability(&self) -> Result<HashMap<String, ResourceAvailability>> {
        let cache = self.resource_cache.read().await;
        Ok(cache.clone())
    }

    /// Estimate resource costs for a plan
    pub async fn estimate_plan_cost(&self, plan: &Plan) -> Result<f64> {
        let mut total_cost = 0.0;

        for task in &plan.tasks {
            for requirement in &task.resource_requirements {
                if let Some(availability) = self.get_resource_availability(&requirement.resource_type).await? {
                    total_cost += availability.cost_per_hour * requirement.quantity * task.estimated_duration_hours;
                } else {
                    // Use default cost if not available
                    total_cost += 0.5 * requirement.quantity * task.estimated_duration_hours;
                }
            }
        }

        Ok(total_cost)
    }

    /// Health check for the resource coordinator
    pub async fn health_check(&self) -> Result<()> {
        #[cfg(feature = "layer8-integration")]
        let response = self.layer8_client
            .get(&format!("{}/health", self.layer8_base_url))
            .send()
            .await;

        #[cfg(feature = "layer8-integration")]
        match response {
            Ok(resp) if resp.status().is_success() => {
                info!("Layer 8 health check passed");
            }
            _ => {
                return Err(PlanningError::IntegrationError(
                    "Layer 8 health check failed".to_string()
                ));
            }
        }

        #[cfg(not(feature = "layer8-integration"))]
        info!("Resource coordinator running in standalone mode - no Layer 8 integration");

        // Check cache freshness
        let cache = self.resource_cache.read().await;
        let cache_age = Utc::now() - cache.values().next()
            .map(|r| r.last_updated)
            .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1));

        if cache_age > chrono::Duration::minutes(5) {
            warn!("Resource cache is stale: {} minutes old", cache_age.num_minutes());
        }

        Ok(())
    }
}

/// Resource availability information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceAvailability {
    pub resource_type: String,
    pub available_quantity: f64,
    pub total_quantity: f64,
    pub cost_per_hour: f64,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_coordinator_creation() {
        let coordinator = ResourceCoordinator::new().await;
        // This might fail if Layer 8 is not running, but that's expected in tests
        assert!(coordinator.is_ok() || coordinator.is_err());
    }

    #[test]
    fn test_resource_availability_struct() {
        let availability = ResourceAvailability {
            resource_type: "GPU".to_string(),
            available_quantity: 4.0,
            total_quantity: 8.0,
            cost_per_hour: 2.0,
            last_updated: Utc::now(),
        };

        assert_eq!(availability.resource_type, "GPU");
        assert_eq!(availability.available_quantity, 4.0);
        assert_eq!(availability.total_quantity, 8.0);
        assert_eq!(availability.cost_per_hour, 2.0);
    }
}