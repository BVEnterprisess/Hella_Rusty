//! Integration Layer for Layer 5

use crate::types::*;
use redis::{aio::ConnectionManager, AsyncCommands};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{info, error};

/// Integration Manager
pub struct IntegrationManager {
    redis_client: ConnectionManager,
    layer7_client: Client,
    layer8_client: Client,
    config: IntegrationConfig,
}

impl IntegrationManager {
    pub async fn new(config: IntegrationConfig) -> Result<Self, IntegrationError> {
        let redis_client = redis::Client::open(&config.redis_url)
            .map_err(|e| IntegrationError::ConnectionFailed(e.to_string()))?
            .get_connection_manager()
            .await
            .map_err(|e| IntegrationError::ConnectionFailed(e.to_string()))?;

        let layer7_client = Client::new();
        let layer8_client = Client::new();

        Ok(Self {
            redis_client,
            layer7_client,
            layer8_client,
            config,
        })
    }

    /// Consume KPI data from Layer 4 via Redis streams
    pub async fn consume_kpi_from_layer4(&self) -> Result<KpiBatch, IntegrationError> {
        let mut conn = self.redis_client.clone();

        // Read from Redis stream
        let results: HashMap<String, HashMap<String, String>> = conn.xread(&["kpi_stream"], &["$"]).await
            .map_err(|e| IntegrationError::ConnectionFailed(e.to_string()))?;

        if let Some(stream_data) = results.get("kpi_stream") {
            if let Some((id, fields)) = stream_data.iter().next() {
                let kpi = self.parse_kpi_from_redis(fields)?;
                info!("Consumed KPI from Layer 4: agent {}", kpi.agent_id);
                return Ok(kpi);
            }
        }

        Err(IntegrationError::ApiError("No KPI data available".to_string()))
    }

    /// Send optimization results to Layer 7
    pub async fn send_to_layer7(&self, optimization: OptimizationResult) -> Result<(), IntegrationError> {
        let payload = serde_json::to_string(&optimization)
            .map_err(|e| IntegrationError::ApiError(e.to_string()))?;

        let response = self.layer7_client
            .post(&format!("{}/optimization", self.config.layer7_api_url))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| IntegrationError::ConnectionFailed(e.to_string()))?;

        if response.status().is_success() {
            info!("Sent optimization to Layer 7 for agent {}", optimization.agent_id);
            Ok(())
        } else {
            Err(IntegrationError::ApiError(format!("Layer 7 API error: {}", response.status())))
        }
    }

    /// Send resource recommendations to Layer 8
    pub async fn send_to_layer8(&self, recommendations: ResourceRecommendations) -> Result<(), IntegrationError> {
        let payload = serde_json::to_string(&recommendations)
            .map_err(|e| IntegrationError::ApiError(e.to_string()))?;

        let response = self.layer8_client
            .post(&format!("{}/recommendations", self.config.layer8_api_url))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| IntegrationError::ConnectionFailed(e.to_string()))?;

        if response.status().is_success() {
            info!("Sent resource recommendations to Layer 8");
            Ok(())
        } else {
            Err(IntegrationError::ApiError(format!("Layer 8 API error: {}", response.status())))
        }
    }

    /// Receive feedback from Layer 7
    pub async fn receive_feedback_from_layer7(&self) -> Result<FeedbackReport, IntegrationError> {
        let response = self.layer7_client
            .get(&format!("{}/feedback", self.config.layer7_api_url))
            .send()
            .await
            .map_err(|e| IntegrationError::ConnectionFailed(e.to_string()))?;

        if response.status().is_success() {
            let feedback: FeedbackReport = response.json().await
                .map_err(|e| IntegrationError::ApiError(e.to_string()))?;
            info!("Received feedback from Layer 7 for agent {}", feedback.agent_id);
            Ok(feedback)
        } else {
            Err(IntegrationError::ApiError(format!("Layer 7 API error: {}", response.status())))
        }
    }

    fn parse_kpi_from_redis(&self, fields: &HashMap<String, String>) -> Result<KpiBatch, IntegrationError> {
        let timestamp_str = fields.get("timestamp").ok_or_else(|| IntegrationError::ApiError("Missing timestamp".to_string()))?;
        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|e| IntegrationError::ApiError(e.to_string()))?
            .with_timezone(&chrono::Utc);

        let agent_id_str = fields.get("agent_id").ok_or_else(|| IntegrationError::ApiError("Missing agent_id".to_string()))?;
        let agent_id = uuid::Uuid::parse_str(agent_id_str)
            .map_err(|e| IntegrationError::ApiError(e.to_string()))?;

        let task_id_str = fields.get("task_id").ok_or_else(|| IntegrationError::ApiError("Missing task_id".to_string()))?;
        let task_id = uuid::Uuid::parse_str(task_id_str)
            .map_err(|e| IntegrationError::ApiError(e.to_string()))?;

        let metrics_str = fields.get("metrics").ok_or_else(|| IntegrationError::ApiError("Missing metrics".to_string()))?;
        let metrics: HashMap<String, f64> = serde_json::from_str(metrics_str)
            .map_err(|e| IntegrationError::ApiError(e.to_string()))?;

        let metadata_str = fields.get("metadata").unwrap_or(&"{}".to_string());
        let metadata: HashMap<String, String> = serde_json::from_str(metadata_str)
            .unwrap_or_default();

        Ok(KpiBatch {
            timestamp,
            agent_id,
            task_id,
            metrics,
            metadata,
        })
    }
}

/// Resource Recommendations for Layer 8
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceRecommendations {
    pub agent_id: AgentId,
    pub cpu_allocation: f64,
    pub memory_allocation: f64,
    pub gpu_allocation: Option<f64>,
    pub reasoning: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}