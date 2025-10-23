//! Common types for Layer 5

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unique identifier for agents
pub type AgentId = Uuid;

/// Unique identifier for tasks
pub type TaskId = Uuid;

/// Unique identifier for experiments
pub type ExperimentId = Uuid;

/// KPI data batch from Layer 4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiBatch {
    pub timestamp: DateTime<Utc>,
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub metrics: HashMap<String, f64>,
    pub metadata: HashMap<String, String>,
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub agent_id: AgentId,
    pub parameters: HashMap<String, f64>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

/// Pattern recognition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternResult {
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub metrics: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Trend,
    Anomaly,
    Seasonality,
    Correlation,
}

/// A/B test experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: ExperimentId,
    pub name: String,
    pub hypothesis: String,
    pub variants: Vec<ExperimentVariant>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: ExperimentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentVariant {
    pub name: String,
    pub parameters: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Draft,
    Running,
    Completed,
    Failed,
}

/// Feedback from Layer 7 on optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackReport {
    pub agent_id: AgentId,
    pub optimization_id: Uuid,
    pub success: bool,
    pub performance_delta: f64,
    pub timestamp: DateTime<Utc>,
}

/// Configuration for Layer 5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer5Config {
    pub ingestion_config: IngestionConfig,
    pub optimization_config: OptimizationConfig,
    pub pattern_config: PatternConfig,
    pub feedback_config: FeedbackConfig,
    pub ab_config: ABConfig,
    pub integration_config: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    pub buffer_size: usize,
    pub batch_timeout_ms: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub learning_rate: f64,
    pub exploration_rate: f64,
    pub model_update_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    pub trend_threshold: f64,
    pub anomaly_threshold: f64,
    pub correlation_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    pub update_interval: u64,
    pub safety_bounds: HashMap<String, (f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABConfig {
    pub min_sample_size: usize,
    pub significance_level: f64,
    pub power_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub redis_url: String,
    pub layer7_api_url: String,
    pub layer8_api_url: String,
}

/// Errors for Layer 5
#[derive(Debug, thiserror::Error)]
pub enum Layer5Error {
    #[error("Ingestion error: {0}")]
    Ingestion(#[from] IngestionError),
    #[error("Optimization error: {0}")]
    Optimization(#[from] OptimizationError),
    #[error("Pattern recognition error: {0}")]
    Pattern(#[from] PatternError),
    #[error("Feedback error: {0}")]
    Feedback(#[from] FeedbackError),
    #[error("AB testing error: {0}")]
    ABTesting(#[from] ABTestingError),
    #[error("Integration error: {0}")]
    Integration(#[from] IntegrationError),
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Processing timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizationError {
    #[error("Model not found")]
    ModelNotFound,
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Training failed")]
    TrainingFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    #[error("Insufficient data")]
    InsufficientData,
    #[error("Pattern not detected")]
    NotDetected,
}

#[derive(Debug, thiserror::Error)]
pub enum FeedbackError {
    #[error("Invalid feedback")]
    InvalidFeedback,
    #[error("Update failed")]
    UpdateFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum ABTestingError {
    #[error("Experiment not found")]
    ExperimentNotFound,
    #[error("Statistical test failed")]
    StatisticalTestFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
}