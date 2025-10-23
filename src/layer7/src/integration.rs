//! Integration Layer for Layer 7 Evolution System

use crate::types::*;
use async_channel::{Receiver, Sender};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, warn};

/// Integration Manager handles communication with other layers
pub struct IntegrationManager {
    layer5_client: Client,
    layer4_client: Client,
    layer8_client: Client,
    config: IntegrationConfig,
    feedback_receiver: Arc<Mutex<Option<Receiver<OptimizationFeedback>>>>,
    genome_deployment_sender: Arc<Mutex<Option<Sender<(AgentId, AgentGenome)>>>>,
}

impl IntegrationManager {
    /// Create a new integration manager
    pub async fn new(config: IntegrationConfig) -> Result<Self, IntegrationError> {
        let layer5_client = Client::new();
        let layer4_client = Client::new();
        let layer8_client = Client::new();

        Ok(Self {
            layer5_client,
            layer4_client,
            layer8_client,
            config,
            feedback_receiver: Arc::new(Mutex::new(None)),
            genome_deployment_sender: Arc::new(Mutex::new(None)),
        })
    }

    /// Start integration listeners
    pub async fn start_listeners(&self) -> Result<(), IntegrationError> {
        // Start Layer5 feedback listener
        self.start_layer5_listener().await?;

        // Start Layer4 deployment listener
        self.start_layer4_listener().await?;

        info!("Integration listeners started");
        Ok(())
    }

    /// Stop integration listeners
    pub async fn stop_listeners(&self) -> Result<(), IntegrationError> {
        info!("Integration listeners stopped");
        Ok(())
    }

    /// Receive optimization feedback from Layer5
    pub async fn receive_from_layer5(&self) -> Result<OptimizationFeedback, IntegrationError> {
        // In a real implementation, this would poll Layer5 API or receive via message queue
        info!("Polling Layer5 for optimization feedback");

        let response = self.layer5_client
            .get(&format!("{}/feedback", self.config.layer5_api_url))
            .send()
            .await
            .map_err(|e| IntegrationError::Layer5Api(e.to_string()))?;

        if response.status().is_success() {
            let feedback: OptimizationFeedback = response.json().await
                .map_err(|e| IntegrationError::Layer5Api(e.to_string()))?;
            info!("Received feedback from Layer5 for agent {}", feedback.agent_id);
            Ok(feedback)
        } else {
            Err(IntegrationError::Layer5Api(format!("Layer5 API error: {}", response.status())))
        }
    }

    /// Deploy evolved genome to Layer4
    pub async fn deploy_to_layer4(&self, agent_id: AgentId, genome: AgentGenome) -> Result<(), IntegrationError> {
        info!("Deploying genome {} to Layer4 for agent {}", genome.id, agent_id);

        let payload = serde_json::json!({
            "agent_id": agent_id,
            "genome_id": genome.id,
            "neural_weights": genome.neural_weights,
            "hyperparameters": genome.hyperparameters,
            "architecture": genome.architecture,
            "version": genome.version,
        });

        let response = self.layer4_client
            .post(&format!("{}/deploy-genome", self.config.layer4_api_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| IntegrationError::Layer4Api(e.to_string()))?;

        if response.status().is_success() {
            info!("Successfully deployed genome {} to Layer4", genome.id);
            Ok(())
        } else {
            Err(IntegrationError::Layer4Api(format!("Layer4 API error: {}", response.status())))
        }
    }

    /// Request resources from Layer8 for evolution
    pub async fn request_from_layer8(&self, requirements: ResourceRequirements) -> Result<ResourceAllocation, IntegrationError> {
        info!("Requesting resources from Layer8: CPU={}, GPU={}, Memory={}GB",
              requirements.cpu_cores, requirements.gpu_count, requirements.memory_gb);

        let payload = serde_json::to_string(&requirements)
            .map_err(|e| IntegrationError::Layer8Api(e.to_string()))?;

        let response = self.layer8_client
            .post(&format!("{}/allocate", self.config.layer8_api_url))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| IntegrationError::Layer8Api(e.to_string()))?;

        if response.status().is_success() {
            let allocation: ResourceAllocation = response.json().await
                .map_err(|e| IntegrationError::Layer8Api(e.to_string()))?;
            info!("Received resource allocation {} from Layer8", allocation.allocation_id);
            Ok(allocation)
        } else {
            Err(IntegrationError::Layer8Api(format!("Layer8 API error: {}", response.status())))
        }
    }

    /// Release resources back to Layer8
    pub async fn release_to_layer8(&self, allocation_id: Uuid) -> Result<(), IntegrationError> {
        info!("Releasing resource allocation {} to Layer8", allocation_id);

        let response = self.layer8_client
            .delete(&format!("{}/allocation/{}", self.config.layer8_api_url, allocation_id))
            .send()
            .await
            .map_err(|e| IntegrationError::Layer8Api(e.to_string()))?;

        if response.status().is_success() {
            info!("Successfully released resource allocation {}", allocation_id);
            Ok(())
        } else {
            Err(IntegrationError::Layer8Api(format!("Layer8 API error: {}", response.status())))
        }
    }

    /// Send evolution results to Layer5 for validation
    pub async fn send_to_layer5(&self, result: EvolutionResult) -> Result<(), IntegrationError> {
        info!("Sending evolution result to Layer5 for validation");

        let payload = serde_json::to_string(&result)
            .map_err(|e| IntegrationError::Layer5Api(e.to_string()))?;

        let response = self.layer5_client
            .post(&format!("{}/evolution-feedback", self.config.layer5_api_url))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| IntegrationError::Layer5Api(e.to_string()))?;

        if response.status().is_success() {
            info!("Successfully sent evolution result to Layer5");
            Ok(())
        } else {
            Err(IntegrationError::Layer5Api(format!("Layer5 API error: {}", response.status())))
        }
    }

    async fn start_layer5_listener(&self) -> Result<(), IntegrationError> {
        // Start background task to listen for Layer5 feedback
        let layer5_url = self.config.layer5_api_url.clone();
        let client = self.layer5_client.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                match Self::poll_layer5_feedback(&client, &layer5_url).await {
                    Ok(feedback) => {
                        info!("Received optimization feedback from Layer5: agent {}", feedback.agent_id);
                        // In a real implementation, this would be sent to the evolution pipeline
                    }
                    Err(e) => {
                        warn!("Failed to poll Layer5 feedback: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_layer4_listener(&self) -> Result<(), IntegrationError> {
        // Start background task to listen for Layer4 deployment confirmations
        let layer4_url = self.config.layer4_api_url.clone();
        let client = self.layer4_client.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

                match Self::check_layer4_deployment_status(&client, &layer4_url).await {
                    Ok(status) => {
                        info!("Layer4 deployment status: {:?}", status);
                    }
                    Err(e) => {
                        warn!("Failed to check Layer4 deployment status: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn poll_layer5_feedback(client: &Client, layer5_url: &str) -> Result<OptimizationFeedback, IntegrationError> {
        let response = client
            .get(&format!("{}/optimization-feedback", layer5_url))
            .send()
            .await
            .map_err(|e| IntegrationError::Layer5Api(e.to_string()))?;

        if response.status().is_success() {
            let feedback: OptimizationFeedback = response.json().await
                .map_err(|e| IntegrationError::Layer5Api(e.to_string()))?;
            Ok(feedback)
        } else {
            Err(IntegrationError::Layer5Api(format!("Layer5 API error: {}", response.status())))
        }
    }

    async fn check_layer4_deployment_status(client: &Client, layer4_url: &str) -> Result<HashMap<String, String>, IntegrationError> {
        let response = client
            .get(&format!("{}/deployment-status", layer4_url))
            .send()
            .await
            .map_err(|e| IntegrationError::Layer4Api(e.to_string()))?;

        if response.status().is_success() {
            let status: HashMap<String, String> = response.json().await
                .map_err(|e| IntegrationError::Layer4Api(e.to_string()))?;
            Ok(status)
        } else {
            Err(IntegrationError::Layer4Api(format!("Layer4 API error: {}", response.status())))
        }
    }
}