//! # Evolution Integration Hub
//!
//! The Evolution Integration Hub manages communication and data sharing between Layer 6 (Evolution)
//! and other layers in the Project Chimera system. It provides a centralized interface
//! for distributing evolution data, receiving optimization feedback, and coordinating
//! with resource management.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Evolution integration hub for inter-layer communication
pub struct EvolutionIntegrationHub {
    config: IntegrationConfig,
    layer_connections: Arc<Mutex<HashMap<LayerId, Box<dyn LayerConnection>>>>,
    evolution_data_queue: Arc<Mutex<Vec<EvolutionData>>>,
    feedback_processor: Arc<Mutex<FeedbackProcessor>>,
    is_running: Arc<Mutex<bool>>,
}

impl EvolutionIntegrationHub {
    /// Create a new evolution integration hub
    pub async fn new(config: IntegrationConfig) -> Result<Self, EvolutionError> {
        let layer_connections = Arc::new(Mutex::new(HashMap::new()));
        let evolution_data_queue = Arc::new(Mutex::new(Vec::new()));
        let feedback_processor = Arc::new(Mutex::new(FeedbackProcessor::new()));

        let mut hub = Self {
            config,
            layer_connections,
            evolution_data_queue,
            feedback_processor,
            is_running: Arc::new(Mutex::new(false)),
        };

        // Initialize layer connections
        hub.initialize_layer_connections().await?;

        Ok(hub)
    }

    /// Start the evolution integration hub
    pub async fn start(&mut self) -> Result<(), EvolutionError> {
        info!("Starting Evolution Integration Hub");
        *self.is_running.lock().await = true;

        // Start data distribution loop
        let config = self.config.clone();
        let evolution_data_queue = self.evolution_data_queue.clone();
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

                        if let Err(e) = Self::distribute_evolution_data(
                            &config,
                            &evolution_data_queue,
                            &layer_connections,
                        ).await {
                            error!("Evolution data distribution failed: {}", e);
                        }
                    }
                }
            }
        });

        // Start feedback processing
        let feedback_processor = self.feedback_processor.clone();
        let layer5_polling_interval = self.config.layer5_polling_interval_seconds;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(layer5_polling_interval));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = feedback_processor.lock().await.process_feedback().await {
                            error!("Feedback processing failed: {}", e);
                        }
                    }
                }
            }
        });

        info!("Evolution Integration Hub started successfully");
        Ok(())
    }

    /// Stop the evolution integration hub
    pub async fn stop(&mut self) -> Result<(), EvolutionError> {
        info!("Stopping Evolution Integration Hub");
        *self.is_running.lock().await = false;
        info!("Evolution Integration Hub stopped successfully");
        Ok(())
    }

    /// Send evolution data to other layers
    pub async fn send_evolution_data(&self, data: EvolutionData) -> Result<(), EvolutionError> {
        debug!("Sending evolution data: {:?}", data);

        // Add to queue for distribution
        self.evolution_data_queue.lock().await.push(data.clone());

        // Route based on data type
        self.route_evolution_data(data).await?;

        info!("Evolution data sent successfully");
        Ok(())
    }

    /// Send data to a specific layer
    pub async fn send_to_layer(&self, layer_id: LayerId, data: EvolutionData) -> Result<(), EvolutionError> {
        let connections = self.layer_connections.lock().await;

        if let Some(connection) = connections.get(&layer_id) {
            timeout(
                Duration::from_secs(self.config.layer7_timeout_seconds),
                connection.send_data(data),
            )
            .await??;
        } else {
            return Err(EvolutionError::IntegrationError(
                format!("No connection available for layer: {}", layer_id)
            ));
        }

        Ok(())
    }

    /// Receive data from a specific layer
    pub async fn receive_from_layer(&self, layer_id: LayerId) -> Result<EvolutionData, EvolutionError> {
        let connections = self.layer_connections.lock().await;

        if let Some(connection) = connections.get(&layer_id) {
            timeout(
                Duration::from_secs(self.config.layer7_timeout_seconds),
                connection.receive_data(),
            )
            .await??;
        } else {
            return Err(EvolutionError::IntegrationError(
                format!("No connection available for layer: {}", layer_id)
            ));
        }

        // This would return actual data in a real implementation
        Err(EvolutionError::IntegrationError("Not implemented".to_string()))
    }

    /// Get hub health status
    pub async fn health_check(&self) -> Result<ComponentHealth, EvolutionError> {
        let is_running = *self.is_running.lock().await;
        let connections_count = self.layer_connections.lock().await.len();
        let queue_size = self.evolution_data_queue.lock().await.len();

        let status = if is_running && connections_count > 0 {
            ServiceStatus::Healthy
        } else if is_running {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "evolution-integration-hub".to_string(),
            status,
            check_duration_ms: 0,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("connections_count".to_string(), connections_count as f64);
                metrics.insert("queue_size".to_string(), queue_size as f64);
                metrics.insert("layer7_timeout_seconds".to_string(), self.config.layer7_timeout_seconds as f64);
                metrics.insert("layer5_polling_interval_seconds".to_string(), self.config.layer5_polling_interval_seconds as f64);
                metrics.insert("bidirectional_enabled".to_string(), if self.config.bidirectional_enabled { 1.0 } else { 0.0 });
                metrics
            },
        })
    }

    /// Initialize connections to other layers
    async fn initialize_layer_connections(&mut self) -> Result<(), EvolutionError> {
        // Layer 7 (Basic Evolution) connection
        self.layer_connections.lock().await.insert(
            "layer7".to_string(),
            Box::new(BasicEvolutionConnection::new()),
        );

        // Layer 5 (Refinement) connection
        self.layer_connections.lock().await.insert(
            "layer5".to_string(),
            Box::new(RefinementConnection::new()),
        );

        // Layer 8 (Resource Management) connection
        self.layer_connections.lock().await.insert(
            "layer8".to_string(),
            Box::new(ResourceManagementConnection::new()),
        );

        info!("Initialized connections to {} layers", 3);
        Ok(())
    }

    /// Route evolution data based on type
    async fn route_evolution_data(&self, data: EvolutionData) -> Result<(), EvolutionError> {
        match &data {
            EvolutionData::EvolutionResult(_) => {
                // Send evolution results to Layer 5 for optimization feedback
                self.send_to_layer("layer5".to_string(), data).await?;
            }
            EvolutionData::PopulationUpdate(_) => {
                // Send population updates to Layer 7 for basic evolution coordination
                self.send_to_layer("layer7".to_string(), data).await?;
            }
            EvolutionData::AlgorithmRecommendation(_) => {
                // Send algorithm recommendations to Layer 7
                self.send_to_layer("layer7".to_string(), data).await?;
            }
            EvolutionData::ResourceRequest(_) => {
                // Send resource requests to Layer 8
                self.send_to_layer("layer8".to_string(), data).await?;
            }
            EvolutionData::PerformanceFeedback(_) => {
                // Send performance feedback to Layer 5
                self.send_to_layer("layer5".to_string(), data).await?;
            }
        }

        Ok(())
    }

    /// Distribute evolution data to connected layers
    async fn distribute_evolution_data(
        config: &IntegrationConfig,
        evolution_data_queue: &Arc<Mutex<Vec<EvolutionData>>>,
        layer_connections: &Arc<Mutex<HashMap<LayerId, Box<dyn LayerConnection>>>>,
    ) -> Result<(), EvolutionError> {
        let mut data_items = evolution_data_queue.lock().await;
        if data_items.is_empty() {
            return Ok(());
        }

        let mut items_to_remove = Vec::new();
        let connections = layer_connections.lock().await;

        for (index, data_item) in data_items.iter().enumerate() {
            let mut delivered = false;

            // Try to deliver to appropriate layers based on data type
            let target_layers = Self::get_target_layers(data_item);

            for layer_id in target_layers {
                if let Some(connection) = connections.get(&layer_id) {
                    match timeout(
                        Duration::from_secs(config.layer7_timeout_seconds),
                        connection.send_data(data_item.clone()),
                    ).await {
                        Ok(Ok(())) => {
                            debug!("Evolution data delivered to layer: {}", layer_id);
                            delivered = true;
                            break;
                        }
                        Ok(Err(e)) => {
                            warn!("Failed to deliver evolution data to layer {}: {}", layer_id, e);
                        }
                        Err(_) => {
                            warn!("Timeout delivering evolution data to layer: {}", layer_id);
                        }
                    }
                }
            }

            if delivered {
                items_to_remove.push(index);
            }
        }

        // Remove successfully delivered items (in reverse order to maintain indices)
        for &index in items_to_remove.iter().rev() {
            data_items.remove(index);
        }

        Ok(())
    }

    /// Get target layers for evolution data type
    fn get_target_layers(data: &EvolutionData) -> Vec<LayerId> {
        match data {
            EvolutionData::EvolutionResult(_) => vec!["layer5".to_string()], // Refinement layer
            EvolutionData::PopulationUpdate(_) => vec!["layer7".to_string()], // Basic evolution
            EvolutionData::AlgorithmRecommendation(_) => vec!["layer7".to_string()], // Basic evolution
            EvolutionData::ResourceRequest(_) => vec!["layer8".to_string()], // Resource management
            EvolutionData::PerformanceFeedback(_) => vec!["layer5".to_string()], // Refinement layer
        }
    }
}

/// Evolution data types for inter-layer communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionData {
    /// Complete evolution result
    EvolutionResult(EvolutionResult),
    /// Population state update
    PopulationUpdate(PopulationState),
    /// Algorithm recommendation
    AlgorithmRecommendation(AlgorithmRecommendation),
    /// Resource allocation request
    ResourceRequest(ResourceRequest),
    /// Performance feedback from other layers
    PerformanceFeedback(PerformanceFeedback),
}

/// Resource request for evolution computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// Request identifier
    pub request_id: String,
    /// Requested CPU cores
    pub cpu_cores: u32,
    /// Requested memory in MB
    pub memory_mb: u64,
    /// Requested GPU units
    pub gpu_units: u32,
    /// Expected duration in minutes
    pub expected_duration_minutes: u32,
    /// Priority level
    pub priority: Priority,
    /// Justification
    pub justification: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Performance feedback from other layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceFeedback {
    /// Source layer
    pub source_layer: String,
    /// Feedback type
    pub feedback_type: FeedbackType,
    /// Performance metrics
    pub metrics: HashMap<String, f64>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Feedback types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeedbackType {
    Optimization,
    Validation,
    Resource,
    Integration,
    Custom(String),
}

/// Feedback processor for handling incoming feedback
struct FeedbackProcessor {
    feedback_history: Vec<PerformanceFeedback>,
    feedback_aggregation: HashMap<String, AggregatedFeedback>,
}

impl FeedbackProcessor {
    fn new() -> Self {
        Self {
            feedback_history: Vec::new(),
            feedback_aggregation: HashMap::new(),
        }
    }

    async fn process_feedback(&mut self) -> Result<(), EvolutionError> {
        debug!("Processing evolution feedback");
        // Implementation would process and aggregate feedback from other layers
        Ok(())
    }
}

/// Aggregated feedback from multiple sources
struct AggregatedFeedback {
    total_feedback_count: u64,
    average_metrics: HashMap<String, f64>,
    common_recommendations: Vec<String>,
    last_updated: DateTime<Utc>,
}

/// Connection to Layer 7 (Basic Evolution)
struct BasicEvolutionConnection {
    layer_id: LayerId,
}

impl BasicEvolutionConnection {
    fn new() -> Self {
        Self {
            layer_id: "layer7".to_string(),
        }
    }
}

#[async_trait]
impl LayerConnection for BasicEvolutionConnection {
    async fn send_data(&self, data: EvolutionData) -> Result<(), IntegrationError> {
        // In a real implementation, this would send data to Layer 7 via message queue or API
        debug!("Sending evolution data to Layer 7 (Basic Evolution): {:?}", data);
        Ok(())
    }

    async fn receive_data(&self) -> Result<EvolutionData, IntegrationError> {
        // In a real implementation, this would receive data from Layer 7
        Err(IntegrationError::ConnectionLost("Not implemented".to_string()))
    }

    fn get_layer_id(&self) -> LayerId {
        self.layer_id.clone()
    }
}

/// Connection to Layer 5 (Refinement)
struct RefinementConnection {
    layer_id: LayerId,
}

impl RefinementConnection {
    fn new() -> Self {
        Self {
            layer_id: "layer5".to_string(),
        }
    }
}

#[async_trait]
impl LayerConnection for RefinementConnection {
    async fn send_data(&self, data: EvolutionData) -> Result<(), IntegrationError> {
        // In a real implementation, this would send data to Layer 5 via message queue or API
        debug!("Sending evolution data to Layer 5 (Refinement): {:?}", data);
        Ok(())
    }

    async fn receive_data(&self) -> Result<EvolutionData, IntegrationError> {
        // In a real implementation, this would receive data from Layer 5
        Err(IntegrationError::ConnectionLost("Not implemented".to_string()))
    }

    fn get_layer_id(&self) -> LayerId {
        self.layer_id.clone()
    }
}

/// Connection to Layer 8 (Resource Management)
struct ResourceManagementConnection {
    layer_id: LayerId,
}

impl ResourceManagementConnection {
    fn new() -> Self {
        Self {
            layer_id: "layer8".to_string(),
        }
    }
}

#[async_trait]
impl LayerConnection for ResourceManagementConnection {
    async fn send_data(&self, data: EvolutionData) -> Result<(), IntegrationError> {
        // In a real implementation, this would send data to Layer 8 via message queue or API
        debug!("Sending evolution data to Layer 8 (Resource Management): {:?}", data);
        Ok(())
    }

    async fn receive_data(&self) -> Result<EvolutionData, IntegrationError> {
        // In a real implementation, this would receive data from Layer 8
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
    async fn test_evolution_integration_hub_creation() {
        let config = IntegrationConfig::default();
        let hub = EvolutionIntegrationHub::new(config).await;
        assert!(hub.is_ok());
    }

    #[test]
    fn test_evolution_data_types() {
        let evolution_result = EvolutionData::EvolutionResult(EvolutionResult {
            best_individual: Individual {
                id: "test".to_string(),
                genome: vec![1.0, 2.0],
                fitness: 0.9,
                objective_values: vec![0.9],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
            best_fitness: 0.9,
            best_objective_values: vec![0.9],
            final_population: Population::new("test".to_string(), vec![]),
            statistics: EvolutionStatistics {
                converged: true,
                final_diversity: 0.5,
                improvement_rate: 0.1,
                success_rate: 0.9,
                avg_generation_time_seconds: 0.5,
                fitness_variance: 0.1,
            },
            algorithm_used: "test-algorithm".to_string(),
            generations: 100,
            total_evaluations: 10000,
            duration_seconds: 50.0,
        });

        // Test that we can create and match on evolution data types
        match evolution_result {
            EvolutionData::EvolutionResult(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_resource_request() {
        let request = ResourceRequest {
            request_id: "req-123".to_string(),
            cpu_cores: 8,
            memory_mb: 16384,
            gpu_units: 2,
            expected_duration_minutes: 60,
            priority: Priority::High,
            justification: "Advanced evolution computation required".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(request.request_id, "req-123");
        assert_eq!(request.cpu_cores, 8);
        assert_eq!(request.memory_mb, 16384);
        assert_eq!(request.gpu_units, 2);
        assert_eq!(request.priority, Priority::High);
    }

    #[test]
    fn test_performance_feedback() {
        let feedback = PerformanceFeedback {
            source_layer: "layer5".to_string(),
            feedback_type: FeedbackType::Optimization,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("optimization_accuracy".to_string(), 0.85);
                metrics.insert("improvement_rate".to_string(), 0.15);
                metrics
            },
            recommendations: vec![
                "Increase population diversity".to_string(),
                "Try alternative algorithms".to_string(),
            ],
            timestamp: Utc::now(),
        };

        assert_eq!(feedback.source_layer, "layer5");
        assert_eq!(feedback.feedback_type, FeedbackType::Optimization);
        assert_eq!(feedback.metrics.len(), 2);
        assert_eq!(feedback.recommendations.len(), 2);
    }

    #[test]
    fn test_feedback_types() {
        assert_eq!(FeedbackType::Optimization, FeedbackType::Optimization);
        assert_eq!(FeedbackType::Validation, FeedbackType::Validation);
        assert_eq!(FeedbackType::Resource, FeedbackType::Resource);
        assert_eq!(FeedbackType::Integration, FeedbackType::Integration);
    }

    #[test]
    fn test_target_layers_routing() {
        let evolution_result = EvolutionData::EvolutionResult(EvolutionResult {
            best_individual: Individual {
                id: "test".to_string(),
                genome: vec![1.0],
                fitness: 0.9,
                objective_values: vec![0.9],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
            best_fitness: 0.9,
            best_objective_values: vec![0.9],
            final_population: Population::new("test".to_string(), vec![]),
            statistics: EvolutionStatistics {
                converged: true,
                final_diversity: 0.5,
                improvement_rate: 0.1,
                success_rate: 0.9,
                avg_generation_time_seconds: 0.5,
                fitness_variance: 0.1,
            },
            algorithm_used: "test".to_string(),
            generations: 100,
            total_evaluations: 10000,
            duration_seconds: 50.0,
        });

        let target_layers = EvolutionIntegrationHub::get_target_layers(&evolution_result);
        assert!(target_layers.contains(&"layer5".to_string()));

        let resource_request = EvolutionData::ResourceRequest(ResourceRequest {
            request_id: "test".to_string(),
            cpu_cores: 4,
            memory_mb: 8192,
            gpu_units: 1,
            expected_duration_minutes: 30,
            priority: Priority::Medium,
            justification: "Test request".to_string(),
            timestamp: Utc::now(),
        });

        let target_layers2 = EvolutionIntegrationHub::get_target_layers(&resource_request);
        assert!(target_layers2.contains(&"layer8".to_string()));
    }
}