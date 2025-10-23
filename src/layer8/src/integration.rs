//! # Integration Manager
//!
//! Handles communication and integration with other layers (4, 5, 7)
//! for resource allocation requests and status updates.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Integration manager for cross-layer communication
pub struct IntegrationManager {
    /// HTTP client for API calls
    client: Client,
    /// Layer 4 integration endpoints
    layer4_endpoints: LayerEndpoints,
    /// Layer 5 integration endpoints
    layer5_endpoints: LayerEndpoints,
    /// Layer 7 integration endpoints
    layer7_endpoints: LayerEndpoints,
    /// Integration status
    status: Arc<RwLock<IntegrationStatus>>,
}

impl IntegrationManager {
    /// Create a new integration manager
    pub async fn new(config: ResourceConfig) -> Result<Self> {
        info!("Initializing integration manager...");

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.integration.timeouts.request_timeout_seconds))
            .build()?;

        let manager = Self {
            client,
            layer4_endpoints: config.integration.layer4_endpoints,
            layer5_endpoints: config.integration.layer5_endpoints,
            layer7_endpoints: config.integration.layer7_endpoints,
            status: Arc::new(RwLock::new(IntegrationStatus::default())),
        };

        info!("✅ Integration manager initialized successfully");
        Ok(manager)
    }

    /// Start the integration manager
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting integration manager...");

        // Test connectivity to all layers
        self.test_layer_connectivity().await?;

        info!("✅ Integration manager started successfully");
        Ok(())
    }

    /// Stop the integration manager
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping integration manager...");

        // Update status to stopped
        let mut status = self.status.write().await;
        status.overall_status = LayerStatus::Stopped;

        info!("✅ Integration manager stopped successfully");
        Ok(())
    }

    /// Request resources from Layer 7 (Evolution)
    pub async fn request_evolution_resources(&self, requirements: ResourceRequirements) -> ResourceResult<ResourceAllocation> {
        debug!("Requesting evolution resources from Layer 7: {:?}", requirements);

        let request = ResourceRequest::new(
            "layer7".to_string(),
            requirements,
            Priority::High,
        );

        // Send request to Layer 7
        let response = self.send_resource_request(&self.layer7_endpoints.resource_endpoint, &request).await?;

        // Parse response
        let allocation: ResourceAllocation = response.json().await
            .map_err(|e| ResourceError::IntegrationError {
                layer: "layer7".to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        info!("✅ Successfully requested evolution resources: {}", allocation.allocation_id);
        Ok(allocation)
    }

    /// Notify Layer 5 (Refinement) of resource availability
    pub async fn notify_refinement_resources(&self, available_resources: ResourceRequirements) -> Result<()> {
        debug!("Notifying Layer 5 of available resources: {:?}", available_resources);

        let notification = ResourceAvailabilityNotification {
            available_resources,
            timestamp: Utc::now(),
            source_layer: "layer8".to_string(),
        };

        self.send_notification(&self.layer5_endpoints.resource_endpoint, &notification).await?;

        info!("✅ Successfully notified Layer 5 of resource availability");
        Ok(())
    }

    /// Get resource requirements from Layer 4 (Execution)
    pub async fn get_execution_requirements(&self) -> ResourceResult<Vec<ResourceRequest>> {
        debug!("Getting resource requirements from Layer 4");

        let response = self.client
            .get(&self.layer4_endpoints.resource_endpoint)
            .send()
            .await
            .map_err(|e| ResourceError::IntegrationError {
                layer: "layer4".to_string(),
                message: format!("Failed to get requirements: {}", e),
            })?;

        let requests: Vec<ResourceRequest> = response.json().await
            .map_err(|e| ResourceError::IntegrationError {
                layer: "layer4".to_string(),
                message: format!("Failed to parse requirements: {}", e),
            })?;

        info!("✅ Successfully retrieved {} resource requirements from Layer 4", requests.len());
        Ok(requests)
    }

    /// Health check implementation
    pub async fn health_check(&self) -> Result<()> {
        // Test connectivity to all layers
        self.test_layer_connectivity().await?;

        // Update status
        let mut status = self.status.write().await;
        status.last_health_check = Utc::now();
        status.overall_status = LayerStatus::Healthy;

        Ok(())
    }

    /// Readiness check
    pub async fn is_ready(&self) -> bool {
        let status = self.status.read().await;
        matches!(status.overall_status, LayerStatus::Healthy)
    }

    /// Get integration status
    pub async fn get_status(&self) -> IntegrationStatus {
        self.status.read().await.clone()
    }

    // Private helper methods

    async fn test_layer_connectivity(&self) -> Result<()> {
        let mut status = self.status.write().await;

        // Test Layer 4 connectivity
        match self.client.get(&self.layer4_endpoints.health_endpoint).send().await {
            Ok(_) => status.layer4_status = LayerStatus::Healthy,
            Err(_) => {
                status.layer4_status = LayerStatus::Unhealthy;
                warn!("Layer 4 health check failed");
            }
        }

        // Test Layer 5 connectivity
        match self.client.get(&self.layer5_endpoints.health_endpoint).send().await {
            Ok(_) => status.layer5_status = LayerStatus::Healthy,
            Err(_) => {
                status.layer5_status = LayerStatus::Unhealthy;
                warn!("Layer 5 health check failed");
            }
        }

        // Test Layer 7 connectivity
        match self.client.get(&self.layer7_endpoints.health_endpoint).send().await {
            Ok(_) => status.layer7_status = LayerStatus::Healthy,
            Err(_) => {
                status.layer7_status = LayerStatus::Unhealthy;
                warn!("Layer 7 health check failed");
            }
        }

        // Update overall status
        status.overall_status = if matches!(status.layer4_status, LayerStatus::Healthy) &&
                                   matches!(status.layer5_status, LayerStatus::Healthy) &&
                                   matches!(status.layer7_status, LayerStatus::Healthy) {
            LayerStatus::Healthy
        } else {
            LayerStatus::Degraded
        };

        Ok(())
    }

    async fn send_resource_request(&self, endpoint: &str, request: &ResourceRequest) -> Result<reqwest::Response> {
        self.client
            .post(endpoint)
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send resource request: {}", e))
    }

    async fn send_notification(&self, endpoint: &str, notification: &ResourceAvailabilityNotification) -> Result<reqwest::Response> {
        self.client
            .post(endpoint)
            .json(notification)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send notification: {}", e))
    }
}

/// Integration status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    /// Overall integration status
    pub overall_status: LayerStatus,
    /// Layer 4 integration status
    pub layer4_status: LayerStatus,
    /// Layer 5 integration status
    pub layer5_status: LayerStatus,
    /// Layer 7 integration status
    pub layer7_status: LayerStatus,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
    /// Connection statistics
    pub connection_stats: ConnectionStats,
}

impl Default for IntegrationStatus {
    fn default() -> Self {
        Self {
            overall_status: LayerStatus::Initializing,
            layer4_status: LayerStatus::Initializing,
            layer5_status: LayerStatus::Initializing,
            layer7_status: LayerStatus::Initializing,
            last_health_check: Utc::now(),
            connection_stats: ConnectionStats::default(),
        }
    }
}

/// Layer status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayerStatus {
    /// Layer is initializing
    Initializing,
    /// Layer is healthy and operational
    Healthy,
    /// Layer is degraded but functional
    Degraded,
    /// Layer is unhealthy
    Unhealthy,
    /// Layer is stopped
    Stopped,
}

impl Default for LayerStatus {
    fn default() -> Self {
        LayerStatus::Initializing
    }
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Total requests sent
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Last successful connection
    pub last_successful_connection: Option<DateTime<Utc>>,
    /// Last failed connection
    pub last_failed_connection: Option<DateTime<Utc>>,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            last_successful_connection: None,
            last_failed_connection: None,
        }
    }
}

/// Resource availability notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailabilityNotification {
    /// Available resources
    pub available_resources: ResourceRequirements,
    /// Notification timestamp
    pub timestamp: DateTime<Utc>,
    /// Source layer
    pub source_layer: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_manager_initialization() {
        let config = ResourceConfig::default();
        let manager = IntegrationManager::new(config).await;

        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_layer_status_transitions() {
        let status = IntegrationStatus::default();

        assert!(matches!(status.overall_status, LayerStatus::Initializing));

        // Test status updates would go here
    }
}