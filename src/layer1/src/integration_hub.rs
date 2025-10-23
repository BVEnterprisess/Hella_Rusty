//! # Integration Hub
//!
//! The Integration Hub manages communication and data sharing between Layer 1 (Discovery)
//! and other layers in the Project Chimera system. It provides a centralized interface
//! for distributing discovery data and receiving requests from other layers.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Integration hub for inter-layer communication
pub struct IntegrationHub {
    config: IntegrationConfig,
    layer_connections: Arc<Mutex<HashMap<LayerId, Box<dyn LayerConnection>>>>,
    outgoing_queue: Arc<Mutex<Vec<DiscoveryData>>>,
    event_router: Arc<Mutex<EventRouter>>,
    is_running: Arc<Mutex<bool>>,
}

impl IntegrationHub {
    /// Create a new integration hub
    pub async fn new(config: IntegrationConfig) -> Result<Self, DiscoveryError> {
        let layer_connections = Arc::new(Mutex::new(HashMap::new()));
        let outgoing_queue = Arc::new(Mutex::new(Vec::new()));
        let event_router = Arc::new(Mutex::new(EventRouter::new()));

        let mut hub = Self {
            config,
            layer_connections,
            outgoing_queue,
            event_router,
            is_running: Arc::new(Mutex::new(false)),
        };

        // Initialize layer connections
        hub.initialize_layer_connections().await?;

        Ok(hub)
    }

    /// Start the integration hub
    pub async fn start(&mut self) -> Result<(), DiscoveryError> {
        info!("Starting Integration Hub");
        *self.is_running.lock().await = true;

        // Start message processing loop
        let config = self.config.clone();
        let outgoing_queue = self.outgoing_queue.clone();
        let layer_connections = self.layer_connections.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !*is_running.lock().await {
                            break;
                        }

                        if let Err(e) = Self::process_outgoing_messages(
                            &config,
                            &outgoing_queue,
                            &layer_connections,
                        ).await {
                            error!("Message processing failed: {}", e);
                        }
                    }
                }
            }
        });

        info!("Integration Hub started successfully");
        Ok(())
    }

    /// Stop the integration hub
    pub async fn stop(&mut self) -> Result<(), DiscoveryError> {
        info!("Stopping Integration Hub");
        *self.is_running.lock().await = false;
        info!("Integration Hub stopped successfully");
        Ok(())
    }

    /// Distribute discovery data to other layers
    pub async fn distribute_discovery_data(&self, data: DiscoveryData) -> Result<(), DiscoveryError> {
        debug!("Distributing discovery data: {:?}", data);

        // Add to outgoing queue
        self.outgoing_queue.lock().await.push(data.clone());

        // Route based on data type
        let event_router = self.event_router.lock().await;
        event_router.route_event(data).await?;

        info!("Discovery data distributed successfully");
        Ok(())
    }

    /// Send data to a specific layer
    pub async fn send_to_layer(&self, layer_id: LayerId, data: DiscoveryData) -> Result<(), DiscoveryError> {
        let connections = self.layer_connections.lock().await;

        if let Some(connection) = connections.get(&layer_id) {
            timeout(
                Duration::from_secs(self.config.layer_timeout_seconds),
                connection.send_data(data),
            )
            .await??;
        } else {
            return Err(DiscoveryError::IntegrationError(
                format!("No connection available for layer: {}", layer_id)
            ));
        }

        Ok(())
    }

    /// Receive data from a specific layer
    pub async fn receive_from_layer(&self, layer_id: LayerId) -> Result<DiscoveryData, DiscoveryError> {
        let connections = self.layer_connections.lock().await;

        if let Some(connection) = connections.get(&layer_id) {
            timeout(
                Duration::from_secs(self.config.layer_timeout_seconds),
                connection.receive_data(),
            )
            .await??;
        } else {
            return Err(DiscoveryError::IntegrationError(
                format!("No connection available for layer: {}", layer_id)
            ));
        }

        // This would return actual data in a real implementation
        Err(DiscoveryError::IntegrationError("Not implemented".to_string()))
    }

    /// Get hub health status
    pub async fn health_check(&self) -> Result<ComponentHealth, DiscoveryError> {
        let is_running = *self.is_running.lock().await;
        let connections_count = self.layer_connections.lock().await.len();
        let queue_size = self.outgoing_queue.lock().await.len();

        let status = if is_running && connections_count > 0 {
            ServiceStatus::Healthy
        } else if is_running {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "integration-hub".to_string(),
            status,
            check_duration_ms: 0,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("connections_count".to_string(), connections_count as f64);
                metrics.insert("queue_size".to_string(), queue_size as f64);
                metrics.insert("timeout_seconds".to_string(), self.config.layer_timeout_seconds as f64);
                metrics.insert("max_retry_attempts".to_string(), self.config.max_retry_attempts as f64);
                metrics
            },
        })
    }

    /// Initialize connections to other layers
    async fn initialize_layer_connections(&mut self) -> Result<(), DiscoveryError> {
        // Layer 2 (Planning) connection
        self.layer_connections.lock().await.insert(
            "layer2".to_string(),
            Box::new(PlanningLayerConnection::new()),
        );

        // Layer 3 (Validation) connection
        self.layer_connections.lock().await.insert(
            "layer3".to_string(),
            Box::new(ValidationLayerConnection::new()),
        );

        // Layer 4 (Execution) connection
        self.layer_connections.lock().await.insert(
            "layer4".to_string(),
            Box::new(ExecutionLayerConnection::new()),
        );

        info!("Initialized connections to {} layers", 3);
        Ok(())
    }

    /// Process outgoing messages to other layers
    async fn process_outgoing_messages(
        config: &IntegrationConfig,
        outgoing_queue: &Arc<Mutex<Vec<DiscoveryData>>>,
        layer_connections: &Arc<Mutex<HashMap<LayerId, Box<dyn LayerConnection>>>>,
    ) -> Result<(), DiscoveryError> {
        let mut messages = outgoing_queue.lock().await;
        if messages.is_empty() {
            return Ok(());
        }

        let mut messages_to_remove = Vec::new();
        let connections = layer_connections.lock().await;

        for (index, message) in messages.iter().enumerate() {
            let mut delivered = false;

            // Try to deliver to each layer
            for (layer_id, connection) in connections.iter() {
                match timeout(
                    Duration::from_secs(config.layer_timeout_seconds),
                    connection.send_data(message.clone()),
                ).await {
                    Ok(Ok(())) => {
                        debug!("Message delivered to layer: {}", layer_id);
                        delivered = true;
                        break;
                    }
                    Ok(Err(e)) => {
                        warn!("Failed to deliver message to layer {}: {}", layer_id, e);
                    }
                    Err(_) => {
                        warn!("Timeout delivering message to layer: {}", layer_id);
                    }
                }
            }

            if delivered {
                messages_to_remove.push(index);
            }
        }

        // Remove successfully delivered messages (in reverse order to maintain indices)
        for &index in messages_to_remove.iter().rev() {
            messages.remove(index);
        }

        Ok(())
    }
}

/// Layer identifier type
pub type LayerId = String;

/// Trait for layer connections
#[async_trait]
pub trait LayerConnection: Send + Sync {
    /// Send data to the layer
    async fn send_data(&self, data: DiscoveryData) -> Result<(), IntegrationError>;

    /// Receive data from the layer
    async fn receive_data(&self) -> Result<DiscoveryData, IntegrationError>;

    /// Get the layer identifier
    fn get_layer_id(&self) -> LayerId;

    /// Check if connection is healthy
    async fn health_check(&self) -> Result<(), IntegrationError> {
        Ok(())
    }
}

/// Event router for distributing discovery data
struct EventRouter {
    routes: HashMap<String, Vec<LayerId>>,
}

impl EventRouter {
    fn new() -> Self {
        let mut routes = HashMap::new();

        // Route system state updates to planning and validation layers
        routes.insert("SystemStateUpdate".to_string(), vec![
            "layer2".to_string(), // Planning needs system state for resource allocation
            "layer3".to_string(), // Validation needs system state for compliance checks
        ]);

        // Route health check results to validation layer
        routes.insert("HealthCheckResults".to_string(), vec![
            "layer3".to_string(), // Validation needs health data for system integrity checks
        ]);

        // Route alerts to all layers
        routes.insert("Alert".to_string(), vec![
            "layer2".to_string(), // Planning needs to know about system issues
            "layer3".to_string(), // Validation needs alert data for compliance
            "layer4".to_string(), // Execution needs to adapt to system issues
        ]);

        // Route performance updates to execution layer
        routes.insert("PerformanceUpdate".to_string(), vec![
            "layer4".to_string(), // Execution needs performance data for optimization
        ]);

        Self { routes }
    }

    async fn route_event(&self, data: DiscoveryData) -> Result<(), DiscoveryError> {
        let event_type = match &data {
            DiscoveryData::SystemStateUpdate(_) => "SystemStateUpdate",
            DiscoveryData::HealthCheckResults(_) => "HealthCheckResults",
            DiscoveryData::Alert(_) => "Alert",
            DiscoveryData::PerformanceUpdate(_) => "PerformanceUpdate",
            DiscoveryData::FullScanResult(_) => "FullScanResult",
            DiscoveryData::DataBatch(_) => "DataBatch",
        };

        if let Some(target_layers) = self.routes.get(event_type) {
            debug!("Routing {} event to {} layers", event_type, target_layers.len());
            // In a real implementation, this would send the data to each target layer
        } else {
            debug!("No routing rules for event type: {}", event_type);
        }

        Ok(())
    }
}

/// Connection to Layer 2 (Planning)
struct PlanningLayerConnection {
    layer_id: LayerId,
}

impl PlanningLayerConnection {
    fn new() -> Self {
        Self {
            layer_id: "layer2".to_string(),
        }
    }
}

#[async_trait]
impl LayerConnection for PlanningLayerConnection {
    async fn send_data(&self, data: DiscoveryData) -> Result<(), IntegrationError> {
        // In a real implementation, this would send data to Layer 2 via message queue or API
        debug!("Sending data to Layer 2 (Planning): {:?}", data);
        Ok(())
    }

    async fn receive_data(&self) -> Result<DiscoveryData, IntegrationError> {
        // In a real implementation, this would receive data from Layer 2
        Err(IntegrationError::ConnectionLost("Not implemented".to_string()))
    }

    fn get_layer_id(&self) -> LayerId {
        self.layer_id.clone()
    }
}

/// Connection to Layer 3 (Validation)
struct ValidationLayerConnection {
    layer_id: LayerId,
}

impl ValidationLayerConnection {
    fn new() -> Self {
        Self {
            layer_id: "layer3".to_string(),
        }
    }
}

#[async_trait]
impl LayerConnection for ValidationLayerConnection {
    async fn send_data(&self, data: DiscoveryData) -> Result<(), IntegrationError> {
        // In a real implementation, this would send data to Layer 3 via message queue or API
        debug!("Sending data to Layer 3 (Validation): {:?}", data);
        Ok(())
    }

    async fn receive_data(&self) -> Result<DiscoveryData, IntegrationError> {
        // In a real implementation, this would receive data from Layer 3
        Err(IntegrationError::ConnectionLost("Not implemented".to_string()))
    }

    fn get_layer_id(&self) -> LayerId {
        self.layer_id.clone()
    }
}

/// Connection to Layer 4 (Execution)
struct ExecutionLayerConnection {
    layer_id: LayerId,
}

impl ExecutionLayerConnection {
    fn new() -> Self {
        Self {
            layer_id: "layer4".to_string(),
        }
    }
}

#[async_trait]
impl LayerConnection for ExecutionLayerConnection {
    async fn send_data(&self, data: DiscoveryData) -> Result<(), IntegrationError> {
        // In a real implementation, this would send data to Layer 4 via message queue or API
        debug!("Sending data to Layer 4 (Execution): {:?}", data);
        Ok(())
    }

    async fn receive_data(&self) -> Result<DiscoveryData, IntegrationError> {
        // In a real implementation, this would receive data from Layer 4
        Err(IntegrationError::ConnectionLost("Not implemented".to_string()))
    }

    fn get_layer_id(&self) -> LayerId {
        self.layer_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_hub_creation() {
        let config = IntegrationConfig::default();
        let hub = IntegrationHub::new(config).await;
        assert!(hub.is_ok());
    }

    #[tokio::test]
    async fn test_event_router() {
        let router = EventRouter::new();

        let system_state = DiscoveryData::SystemStateUpdate(SystemState {
            environmental: EnvironmentalState {
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
                    total_cpu_cores: 0,
                    total_memory_mb: 0,
                    total_disk_gb: 0,
                    available: ResourceAvailability {
                        cpu_cores: 0,
                        memory_mb: 0,
                        disk_gb: 0,
                        network_mbps: 0,
                    },
                },
                last_scan: Utc::now(),
            },
            monitoring: MonitoringState {
                health_checks: HashMap::new(),
                performance_metrics: PerformanceMetrics {
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
                },
                alerts: Vec::new(),
                last_update: Utc::now(),
            },
            collection: CollectionState {
                data_sources: HashMap::new(),
                recent_batches: Vec::new(),
                statistics: CollectionStatistics {
                    total_data_points: 0,
                    data_points_per_second: 0.0,
                    success_rate: 1.0,
                    avg_latency_ms: 0.0,
                    quality_score: 1.0,
                },
                last_collection: Utc::now(),
            },
            timestamp: Utc::now(),
        });

        let result = router.route_event(system_state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_layer_connections() {
        let planning_connection = PlanningLayerConnection::new();
        assert_eq!(planning_connection.get_layer_id(), "layer2");

        let validation_connection = ValidationLayerConnection::new();
        assert_eq!(validation_connection.get_layer_id(), "layer3");

        let execution_connection = ExecutionLayerConnection::new();
        assert_eq!(execution_connection.get_layer_id(), "layer4");
    }
}