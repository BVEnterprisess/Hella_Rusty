//! Core type definitions for Layer 7 Evolution System

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unique identifier for agents
pub type AgentId = Uuid;

/// Unique identifier for genomes
pub type GenomeId = Uuid;

/// Unique identifier for evolution experiments
pub type EvolutionExperimentId = Uuid;

/// Agent genome containing neural network weights and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGenome {
    pub id: GenomeId,
    pub agent_id: AgentId,
    pub version: u64,
    pub neural_weights: Vec<f32>,
    pub hyperparameters: HashMap<String, f64>,
    pub architecture: NetworkArchitecture,
    pub metadata: GenomeMetadata,
    pub created_at: DateTime<Utc>,
    pub parent_genomes: Vec<GenomeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkArchitecture {
    pub layers: Vec<LayerConfig>,
    pub activation_functions: Vec<String>,
    pub input_size: usize,
    pub output_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub layer_type: LayerType,
    pub size: usize,
    pub activation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerType {
    Dense,
    Conv1D,
    Conv2D,
    LSTM,
    Attention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeMetadata {
    pub fitness_score: f64,
    pub generation: u64,
    pub mutation_rate: f64,
    pub crossover_method: String,
    pub training_data_hash: String,
    pub validation_accuracy: f64,
}

/// Evolution population containing multiple genomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPopulation {
    pub id: Uuid,
    pub generation: u64,
    pub genomes: Vec<AgentGenome>,
    pub fitness_scores: HashMap<AgentId, f64>,
    pub diversity_metrics: DiversityMetrics,
    pub created_at: DateTime<Utc>,
    pub target_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityMetrics {
    pub genetic_diversity: f64,
    pub phenotypic_diversity: f64,
    pub fitness_variance: f64,
    pub population_entropy: f64,
}

/// Fitness evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessResult {
    pub agent_id: AgentId,
    pub genome_id: GenomeId,
    pub fitness_score: f64,
    pub performance_metrics: HashMap<String, f64>,
    pub validation_score: f64,
    pub evaluated_at: DateTime<Utc>,
}

/// Optimization feedback from Layer 5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationFeedback {
    pub agent_id: AgentId,
    pub optimization_id: Uuid,
    pub performance_improvement: f64,
    pub confidence: f64,
    pub parameters: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
    pub validation_results: ValidationResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub latency_ms: f64,
}

/// Evolution experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionExperiment {
    pub id: EvolutionExperimentId,
    pub name: String,
    pub description: String,
    pub target_agents: Vec<AgentId>,
    pub fitness_function: FitnessFunction,
    pub genetic_operators: GeneticOperatorConfig,
    pub population_config: PopulationConfig,
    pub termination_conditions: TerminationConditions,
    pub status: ExperimentStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessFunction {
    pub function_type: FitnessFunctionType,
    pub weights: HashMap<String, f64>,
    pub constraints: HashMap<String, (f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FitnessFunctionType {
    WeightedSum,
    MultiObjective,
    ParetoOptimal,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticOperatorConfig {
    pub selection_method: SelectionMethod,
    pub crossover_method: CrossoverMethod,
    pub mutation_method: MutationMethod,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    Tournament(usize),
    RouletteWheel,
    RankBased,
    Elitism(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossoverMethod {
    SinglePoint,
    MultiPoint(usize),
    Uniform,
    Arithmetic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationMethod {
    Gaussian(f64),
    Polynomial(f64),
    Uniform(f64, f64),
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub size: usize,
    pub elite_size: usize,
    pub tournament_size: usize,
    pub max_generations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationConditions {
    pub max_generations: u64,
    pub target_fitness: f64,
    pub fitness_stagnation_generations: u64,
    pub time_limit_hours: u64,
    pub improvement_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Created,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub experiment_id: EvolutionExperimentId,
    pub best_genome: AgentGenome,
    pub best_fitness: f64,
    pub improvement_achieved: f64,
    pub generations_completed: u64,
    pub total_evaluations: u64,
    pub timestamp: DateTime<Utc>,
}

/// Resource requirements for evolution simulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_count: u32,
    pub storage_gb: u32,
    pub network_bandwidth_mbps: u32,
    pub duration_hours: u32,
    pub priority: ResourcePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: Uuid,
    pub requirements: ResourceRequirements,
    pub allocated_resources: AllocatedResources,
    pub status: AllocationStatus,
    pub allocated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    pub compute_nodes: Vec<String>,
    pub gpu_devices: Vec<String>,
    pub storage_paths: Vec<String>,
    pub network_interfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    Pending,
    Allocated,
    Active,
    Released,
    Failed,
}

/// Configuration for Layer 7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer7Config {
    pub genome_config: GenomeConfig,
    pub evolution_config: EvolutionConfig,
    pub fitness_config: FitnessConfig,
    pub operators_config: GeneticOperatorConfig,
    pub integration_config: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeConfig {
    pub max_genome_size: usize,
    pub compression_enabled: bool,
    pub versioning_enabled: bool,
    pub backup_generations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub max_generations: u64,
    pub convergence_threshold: f64,
    pub parallel_evaluations: bool,
    pub checkpoint_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessConfig {
    pub evaluation_timeout_seconds: u64,
    pub max_concurrent_evaluations: usize,
    pub validation_split: f64,
    pub cross_validation_folds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub layer5_api_url: String,
    pub layer4_api_url: String,
    pub layer8_api_url: String,
    pub redis_url: String,
}

/// Errors for Layer 7
#[derive(Debug, thiserror::Error)]
pub enum Layer7Error {
    #[error("Genome management error: {0}")]
    Genome(#[from] GenomeError),
    #[error("Evolution engine error: {0}")]
    Evolution(#[from] EvolutionError),
    #[error("Fitness evaluation error: {0}")]
    Fitness(#[from] FitnessError),
    #[error("Genetic operators error: {0}")]
    Operators(#[from] OperatorsError),
    #[error("Integration error: {0}")]
    Integration(#[from] IntegrationError),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GenomeError {
    #[error("Genome not found: {0}")]
    NotFound(GenomeId),
    #[error("Invalid genome data: {0}")]
    InvalidData(String),
    #[error("Genome corrupted: {0}")]
    Corrupted(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EvolutionError {
    #[error("Population initialization failed")]
    PopulationInitFailed,
    #[error("Convergence not reached after {0} generations")]
    ConvergenceFailed(u64),
    #[error("Insufficient fitness improvement: {0}")]
    InsufficientImprovement(f64),
    #[error("Evolution timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum FitnessError {
    #[error("Fitness evaluation failed for agent {0}")]
    EvaluationFailed(AgentId),
    #[error("Invalid fitness score: {0}")]
    InvalidScore(f64),
    #[error("Evaluation timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum OperatorsError {
    #[error("Selection operator failed")]
    SelectionFailed,
    #[error("Crossover operator failed")]
    CrossoverFailed,
    #[error("Mutation operator failed")]
    MutationFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Layer5 API error: {0}")]
    Layer5Api(String),
    #[error("Layer4 API error: {0}")]
    Layer4Api(String),
    #[error("Layer8 API error: {0}")]
    Layer8Api(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}