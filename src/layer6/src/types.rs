//! # Layer 6 Types and Data Structures
//!
//! This module defines all the core types and data structures used throughout Layer 6 (Evolution).
//! These types provide the foundation for advanced evolutionary algorithms, meta-learning,
//! population dynamics, and hyper-heuristic systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Unique identifier for evolutionary algorithms
pub type AlgorithmId = String;

/// Unique identifier for populations
pub type PopulationId = String;

/// Unique identifier for evolution runs
pub type EvolutionRunId = String;

/// Unique identifier for fitness landscapes
pub type LandscapeId = String;

/// Unique identifier for hyper-heuristics
pub type HeuristicId = String;

/// Advanced evolution service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    /// Meta-learning framework configuration
    pub meta_learning: MetaLearningConfig,
    /// Population dynamics configuration
    pub population: PopulationConfig,
    /// Adaptive evolution configuration
    pub adaptive: AdaptiveConfig,
    /// Hyper-heuristics configuration
    pub hyper_heuristics: HyperHeuristicConfig,
    /// Fitness landscape analysis configuration
    pub fitness: FitnessConfig,
    /// Integration hub configuration
    pub integration: IntegrationConfig,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            meta_learning: MetaLearningConfig::default(),
            population: PopulationConfig::default(),
            adaptive: AdaptiveConfig::default(),
            hyper_heuristics: HyperHeuristicConfig::default(),
            fitness: FitnessConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

/// Meta-learning framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningConfig {
    /// Algorithm portfolio size
    pub portfolio_size: usize,
    /// Performance tracking window size
    pub performance_window: usize,
    /// Algorithm selection confidence threshold
    pub selection_threshold: f64,
    /// Learning rate for algorithm adaptation
    pub learning_rate: f64,
    /// Enable online learning
    pub online_learning_enabled: bool,
}

impl Default for MetaLearningConfig {
    fn default() -> Self {
        Self {
            portfolio_size: 10,
            performance_window: 100,
            selection_threshold: 0.8,
            learning_rate: 0.01,
            online_learning_enabled: true,
        }
    }
}

/// Population dynamics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    /// Default population size
    pub default_size: usize,
    /// Maximum population size
    pub max_size: usize,
    /// Minimum population size
    pub min_size: usize,
    /// Migration interval in generations
    pub migration_interval: u32,
    /// Migration rate (0.0 to 1.0)
    pub migration_rate: f64,
    /// Diversity threshold for population management
    pub diversity_threshold: f64,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            default_size: 100,
            max_size: 1000,
            min_size: 20,
            migration_interval: 10,
            migration_rate: 0.1,
            diversity_threshold: 0.7,
        }
    }
}

/// Adaptive evolution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Parameter adaptation rate
    pub adaptation_rate: f64,
    /// Strategy switching threshold
    pub switching_threshold: f64,
    /// Performance monitoring window
    pub monitoring_window: usize,
    /// Enable self-adaptive operators
    pub self_adaptive_enabled: bool,
    /// Convergence detection sensitivity
    pub convergence_sensitivity: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            adaptation_rate: 0.1,
            switching_threshold: 0.05,
            monitoring_window: 50,
            self_adaptive_enabled: true,
            convergence_sensitivity: 0.001,
        }
    }
}

/// Hyper-heuristics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperHeuristicConfig {
    /// Maximum heuristic portfolio size
    pub max_portfolio_size: usize,
    /// Heuristic generation rate
    pub generation_rate: f64,
    /// Selection pressure for heuristics
    pub selection_pressure: f64,
    /// Enable heuristic evolution
    pub heuristic_evolution_enabled: bool,
    /// Complexity penalty factor
    pub complexity_penalty: f64,
}

impl Default for HyperHeuristicConfig {
    fn default() -> Self {
        Self {
            max_portfolio_size: 20,
            generation_rate: 0.1,
            selection_pressure: 2.0,
            heuristic_evolution_enabled: true,
            complexity_penalty: 0.01,
        }
    }
}

/// Fitness landscape analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessConfig {
    /// Sample size for landscape analysis
    pub sample_size: usize,
    /// Analysis depth (number of samples per dimension)
    pub analysis_depth: usize,
    /// Enable multi-objective analysis
    pub multi_objective_enabled: bool,
    /// Landscape correlation threshold
    pub correlation_threshold: f64,
    /// Enable epistasis detection
    pub epistasis_detection_enabled: bool,
}

impl Default for FitnessConfig {
    fn default() -> Self {
        Self {
            sample_size: 1000,
            analysis_depth: 10,
            multi_objective_enabled: true,
            correlation_threshold: 0.5,
            epistasis_detection_enabled: true,
        }
    }
}

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Layer 7 integration timeout in seconds
    pub layer7_timeout_seconds: u64,
    /// Layer 5 feedback polling interval in seconds
    pub layer5_polling_interval_seconds: u64,
    /// Layer 8 resource request timeout in seconds
    pub layer8_timeout_seconds: u64,
    /// Enable bidirectional communication
    pub bidirectional_enabled: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            layer7_timeout_seconds: 30,
            layer5_polling_interval_seconds: 60,
            layer8_timeout_seconds: 15,
            bidirectional_enabled: true,
        }
    }
}

/// Individual in an evolutionary population
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    /// Unique individual identifier
    pub id: String,
    /// Genome/chromosome representation
    pub genome: Vec<f64>,
    /// Fitness value
    pub fitness: f64,
    /// Multi-objective fitness values
    pub objective_values: Vec<f64>,
    /// Individual age (generations)
    pub age: u32,
    /// Parent information
    pub parents: Option<(String, String)>,
    /// Metadata and annotations
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Population of individuals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Population {
    /// Unique population identifier
    pub id: PopulationId,
    /// Population individuals
    pub individuals: Vec<Individual>,
    /// Population generation number
    pub generation: u32,
    /// Population statistics
    pub statistics: PopulationStatistics,
    /// Subpopulation information (for multi-population algorithms)
    pub subpopulations: Option<Vec<Subpopulation>>,
    /// Migration history
    pub migration_history: Vec<MigrationEvent>,
}

impl Population {
    /// Create a new population with the given individuals
    pub fn new(id: PopulationId, individuals: Vec<Individual>) -> Self {
        let statistics = PopulationStatistics::from_individuals(&individuals);

        Self {
            id,
            individuals,
            generation: 0,
            statistics,
            subpopulations: None,
            migration_history: Vec::new(),
        }
    }

    /// Get the best individual in the population
    pub fn best_individual(&self) -> Option<&Individual> {
        self.individuals.iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
    }

    /// Get the worst individual in the population
    pub fn worst_individual(&self) -> Option<&Individual> {
        self.individuals.iter().min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
    }

    /// Get population diversity measure
    pub fn diversity(&self) -> f64 {
        if self.individuals.len() < 2 {
            return 0.0;
        }

        // Calculate average pairwise distance
        let mut total_distance = 0.0;
        let mut comparisons = 0;

        for (i, ind1) in self.individuals.iter().enumerate() {
            for ind2 in &self.individuals[i + 1..] {
                let distance = euclidean_distance(&ind1.genome, &ind2.genome);
                total_distance += distance;
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            total_distance / comparisons as f64
        } else {
            0.0
        }
    }

    /// Get population size
    pub fn size(&self) -> usize {
        self.individuals.len()
    }
}

/// Population statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationStatistics {
    /// Best fitness in population
    pub best_fitness: f64,
    /// Worst fitness in population
    pub worst_fitness: f64,
    /// Average fitness
    pub average_fitness: f64,
    /// Fitness standard deviation
    pub fitness_std: f64,
    /// Population diversity
    pub diversity: f64,
    /// Convergence measure (0.0 = diverse, 1.0 = converged)
    pub convergence: f64,
}

impl PopulationStatistics {
    /// Calculate statistics from a set of individuals
    pub fn from_individuals(individuals: &[Individual]) -> Self {
        if individuals.is_empty() {
            return Self {
                best_fitness: 0.0,
                worst_fitness: 0.0,
                average_fitness: 0.0,
                fitness_std: 0.0,
                diversity: 0.0,
                convergence: 0.0,
            };
        }

        let fitness_values: Vec<f64> = individuals.iter().map(|ind| ind.fitness).collect();
        let best_fitness = fitness_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let worst_fitness = fitness_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let average_fitness = fitness_values.iter().sum::<f64>() / fitness_values.len() as f64;

        let variance = fitness_values.iter()
            .map(|&f| (f - average_fitness).powi(2))
            .sum::<f64>() / fitness_values.len() as f64;
        let fitness_std = variance.sqrt();

        // Calculate diversity (simplified)
        let diversity = if individuals.len() > 1 {
            let mut total_distance = 0.0;
            let mut comparisons = 0;
            for (i, ind1) in individuals.iter().enumerate() {
                for ind2 in &individuals[i + 1..] {
                    total_distance += euclidean_distance(&ind1.genome, &ind2.genome);
                    comparisons += 1;
                }
            }
            if comparisons > 0 { total_distance / comparisons as f64 } else { 0.0 }
        } else {
            0.0
        };

        // Calculate convergence (simplified)
        let fitness_range = best_fitness - worst_fitness;
        let convergence = if fitness_range > 0.0 {
            fitness_std / fitness_range
        } else {
            0.0
        };

        Self {
            best_fitness,
            worst_fitness,
            average_fitness,
            fitness_std,
            diversity,
            convergence,
        }
    }
}

/// Subpopulation for multi-population algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subpopulation {
    /// Subpopulation identifier
    pub id: String,
    /// Individuals in this subpopulation
    pub individuals: Vec<Individual>,
    /// Subpopulation statistics
    pub statistics: PopulationStatistics,
    /// Migration connections to other subpopulations
    pub connections: Vec<String>,
    /// Subpopulation age
    pub age: u32,
}

/// Migration event between subpopulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEvent {
    /// Source subpopulation
    pub source: String,
    /// Target subpopulation
    pub target: String,
    /// Migrating individuals
    pub individuals: Vec<String>,
    /// Migration timestamp
    pub timestamp: DateTime<Utc>,
    /// Migration reason
    pub reason: MigrationReason,
}

/// Migration reasons
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationReason {
    Scheduled,
    DiversityLow,
    PerformanceStagnation,
    ResourceBalancing,
    Custom(String),
}

/// Evolution algorithm trait
#[async_trait::async_trait]
pub trait EvolutionaryAlgorithm: Send + Sync {
    /// Evolve a population for one generation
    async fn evolve_generation(
        &mut self,
        population: &Population,
        fitness_function: Arc<dyn FitnessFunction>,
    ) -> Result<Population, EvolutionError>;

    /// Get algorithm identifier
    fn get_id(&self) -> AlgorithmId;

    /// Get algorithm name
    fn get_name(&self) -> &str;

    /// Get algorithm parameters
    fn get_parameters(&self) -> HashMap<String, f64>;

    /// Set algorithm parameters
    fn set_parameters(&mut self, parameters: HashMap<String, f64>) -> Result<(), EvolutionError>;

    /// Get algorithm capabilities
    fn get_capabilities(&self) -> AlgorithmCapabilities;

    /// Check if algorithm is suitable for a problem
    fn is_suitable_for(&self, problem_characteristics: &ProblemCharacteristics) -> bool;
}

/// Algorithm capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmCapabilities {
    /// Supports multi-objective optimization
    pub multi_objective: bool,
    /// Supports constraint handling
    pub constraint_handling: bool,
    /// Supports large populations
    pub large_population: bool,
    /// Supports high-dimensional problems
    pub high_dimensional: bool,
    /// Supports noisy fitness functions
    pub noisy_fitness: bool,
    /// Parallel processing capability
    pub parallel_processing: bool,
}

/// Problem characteristics for algorithm selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemCharacteristics {
    /// Problem dimensionality
    pub dimensionality: usize,
    /// Problem type
    pub problem_type: ProblemType,
    /// Fitness landscape characteristics
    pub landscape: FitnessLandscapeType,
    /// Multi-objective flag
    pub multi_objective: bool,
    /// Constraint types
    pub constraints: Vec<ConstraintType>,
    /// Expected population size
    pub expected_population_size: usize,
}

impl Default for ProblemCharacteristics {
    fn default() -> Self {
        Self {
            dimensionality: 10,
            problem_type: ProblemType::Continuous,
            landscape: FitnessLandscapeType::Unknown,
            multi_objective: false,
            constraints: Vec::new(),
            expected_population_size: 100,
        }
    }
}

/// Problem types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProblemType {
    Continuous,
    Discrete,
    Mixed,
    Combinatorial,
    Dynamic,
    Custom(String),
}

/// Fitness landscape types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FitnessLandscapeType {
    Unimodal,
    Multimodal,
    Deceptive,
    Neutral,
    Rugged,
    Unknown,
}

/// Constraint types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintType {
    Equality,
    Inequality,
    Boundary,
    Custom(String),
}

/// Fitness function trait
#[async_trait::async_trait]
pub trait FitnessFunction: Send + Sync {
    /// Evaluate fitness of an individual
    async fn evaluate(&self, individual: &Individual) -> Result<FitnessResult, EvolutionError>;

    /// Evaluate fitness of multiple individuals
    async fn evaluate_batch(&self, individuals: &[Individual]) -> Result<Vec<FitnessResult>, EvolutionError>;

    /// Get fitness function properties
    fn get_properties(&self) -> FitnessProperties;

    /// Check if fitness function is multi-objective
    fn is_multi_objective(&self) -> bool {
        self.get_properties().multi_objective
    }

    /// Get number of objectives
    fn num_objectives(&self) -> usize {
        self.get_properties().num_objectives
    }
}

/// Fitness evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessResult {
    /// Fitness value(s)
    pub fitness: f64,
    /// Multi-objective fitness values
    pub objective_values: Vec<f64>,
    /// Constraint violations
    pub constraint_violations: Vec<ConstraintViolation>,
    /// Evaluation metadata
    pub metadata: HashMap<String, String>,
    /// Evaluation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Constraint violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Violation magnitude
    pub violation: f64,
    /// Constraint identifier
    pub constraint_id: String,
}

/// Fitness function properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessProperties {
    /// Multi-objective optimization
    pub multi_objective: bool,
    /// Number of objectives
    pub num_objectives: usize,
    /// Problem bounds
    pub bounds: Option<(Vec<f64>, Vec<f64>)>,
    /// Constraint count
    pub constraint_count: usize,
    /// Expected fitness range
    pub expected_range: Option<(f64, f64)>,
}

/// Evolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    /// Best individual found
    pub best_individual: Individual,
    /// Best fitness achieved
    pub best_fitness: f64,
    /// Multi-objective best values
    pub best_objective_values: Vec<f64>,
    /// Final population
    pub final_population: Population,
    /// Evolution statistics
    pub statistics: EvolutionStatistics,
    /// Algorithm used
    pub algorithm_used: AlgorithmId,
    /// Number of generations
    pub generations: u32,
    /// Total evaluations
    pub total_evaluations: u64,
    /// Evolution duration
    pub duration_seconds: f64,
}

impl EvolutionResult {
    /// Create a new evolution result
    pub fn new(
        best_individual: Individual,
        final_population: Population,
        algorithm_used: AlgorithmId,
        generations: u32,
        total_evaluations: u64,
        duration_seconds: f64,
    ) -> Self {
        let best_fitness = best_individual.fitness;
        let best_objective_values = best_individual.objective_values.clone();
        let statistics = EvolutionStatistics::from_result(&best_individual, &final_population, generations, total_evaluations);

        Self {
            best_individual,
            best_fitness,
            best_objective_values,
            final_population,
            statistics,
            algorithm_used,
            generations,
            total_evaluations,
            duration_seconds,
        }
    }
}

/// Evolution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionStatistics {
    /// Convergence achieved
    pub converged: bool,
    /// Final population diversity
    pub final_diversity: f64,
    /// Best fitness improvement over generations
    pub improvement_rate: f64,
    /// Success rate (improvements per generation)
    pub success_rate: f64,
    /// Average generation time
    pub avg_generation_time_seconds: f64,
    /// Fitness variance over time
    pub fitness_variance: f64,
}

impl EvolutionStatistics {
    /// Calculate statistics from evolution result
    pub fn from_result(
        best_individual: &Individual,
        final_population: &Population,
        generations: u32,
        total_evaluations: u64,
    ) -> Self {
        let converged = final_population.statistics.convergence > 0.95;
        let final_diversity = final_population.diversity();
        let improvement_rate = if generations > 0 {
            best_individual.fitness / generations as f64
        } else {
            0.0
        };
        let success_rate = if generations > 0 {
            1.0 / generations as f64 // Simplified
        } else {
            0.0
        };
        let avg_generation_time_seconds = if generations > 0 {
            0.1 // Placeholder - would be calculated from actual timing
        } else {
            0.0
        };
        let fitness_variance = final_population.statistics.fitness_std;

        Self {
            converged,
            final_diversity,
            improvement_rate,
            success_rate,
            avg_generation_time_seconds,
            fitness_variance,
        }
    }
}

/// Evolution run configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRunConfig {
    /// Maximum generations
    pub max_generations: u32,
    /// Target fitness
    pub target_fitness: Option<f64>,
    /// Population size
    pub population_size: usize,
    /// Enable parallel evaluation
    pub parallel_evaluation: bool,
    /// Use hyper-heuristics
    pub use_hyper_heuristics: bool,
    /// Enable adaptive strategies
    pub adaptive_enabled: bool,
    /// Convergence threshold
    pub convergence_threshold: f64,
    /// Stagnation limit (generations without improvement)
    pub stagnation_limit: u32,
}

impl Default for EvolutionRunConfig {
    fn default() -> Self {
        Self {
            max_generations: 1000,
            target_fitness: None,
            population_size: 100,
            parallel_evaluation: true,
            use_hyper_heuristics: true,
            adaptive_enabled: true,
            convergence_threshold: 0.001,
            stagnation_limit: 50,
        }
    }
}

/// Complete evolution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionState {
    /// Meta-learning state
    pub meta_learning: MetaLearningState,
    /// Population state
    pub population: PopulationState,
    /// Adaptive evolution state
    pub adaptive: AdaptiveState,
    /// Hyper-heuristics state
    pub hyper_heuristics: HyperHeuristicState,
    /// Fitness analysis state
    pub fitness_analysis: FitnessAnalysisState,
    /// State timestamp
    pub timestamp: DateTime<Utc>,
}

/// Meta-learning state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningState {
    /// Available algorithms
    pub algorithms: HashMap<AlgorithmId, AlgorithmInfo>,
    /// Algorithm performance history
    pub performance_history: Vec<AlgorithmPerformance>,
    /// Current algorithm recommendations
    pub recommendations: HashMap<ProblemId, AlgorithmRecommendation>,
    /// Learning progress
    pub learning_progress: f64,
}

/// Algorithm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmInfo {
    /// Algorithm identifier
    pub id: AlgorithmId,
    /// Algorithm name
    pub name: String,
    /// Algorithm capabilities
    pub capabilities: AlgorithmCapabilities,
    /// Performance metrics
    pub performance_metrics: AlgorithmMetrics,
    /// Usage statistics
    pub usage_stats: UsageStatistics,
}

/// Algorithm performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmMetrics {
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average convergence time
    pub avg_convergence_time: f64,
    /// Average solution quality
    pub avg_solution_quality: f64,
    /// Resource efficiency
    pub resource_efficiency: f64,
    /// Robustness score
    pub robustness: f64,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Total runs
    pub total_runs: u64,
    /// Successful runs
    pub successful_runs: u64,
    /// Average run time
    pub avg_run_time_seconds: f64,
    /// Last used timestamp
    pub last_used: DateTime<Utc>,
}

/// Algorithm performance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmPerformance {
    /// Algorithm used
    pub algorithm_id: AlgorithmId,
    /// Problem characteristics
    pub problem_characteristics: ProblemCharacteristics,
    /// Performance metrics
    pub metrics: AlgorithmMetrics,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Algorithm recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmRecommendation {
    /// Recommended algorithm
    pub algorithm_id: AlgorithmId,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Expected performance
    pub expected_performance: f64,
    /// Reasoning
    pub reasoning: String,
}

/// Population state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationState {
    /// Active populations
    pub populations: HashMap<PopulationId, PopulationInfo>,
    /// Migration events
    pub migration_events: Vec<MigrationEvent>,
    /// Global diversity metrics
    pub global_diversity: f64,
    /// Population health
    pub health_score: f64,
}

/// Population information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationInfo {
    /// Population identifier
    pub id: PopulationId,
    /// Population size
    pub size: usize,
    /// Population statistics
    pub statistics: PopulationStatistics,
    /// Migration connections
    pub connections: Vec<String>,
    /// Age in generations
    pub age: u32,
}

/// Adaptive evolution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveState {
    /// Current strategy
    pub current_strategy: StrategyInfo,
    /// Strategy performance
    pub strategy_performance: StrategyPerformance,
    /// Parameter adaptation state
    pub parameter_state: ParameterAdaptationState,
    /// Strategy switching history
    pub switching_history: Vec<StrategySwitch>,
}

/// Strategy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInfo {
    /// Strategy identifier
    pub id: String,
    /// Strategy name
    pub name: String,
    /// Strategy parameters
    pub parameters: HashMap<String, f64>,
    /// Adaptation status
    pub adaptation_status: AdaptationStatus,
}

/// Strategy performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    /// Performance score
    pub score: f64,
    /// Improvement rate
    pub improvement_rate: f64,
    /// Stability measure
    pub stability: f64,
    /// Resource efficiency
    pub efficiency: f64,
}

/// Parameter adaptation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterAdaptationState {
    /// Current parameters
    pub current_parameters: HashMap<String, f64>,
    /// Parameter history
    pub parameter_history: Vec<ParameterSnapshot>,
    /// Adaptation progress
    pub adaptation_progress: f64,
    /// Learning rate
    pub learning_rate: f64,
}

/// Parameter snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSnapshot {
    /// Parameters at this snapshot
    pub parameters: HashMap<String, f64>,
    /// Performance at this snapshot
    pub performance: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Strategy switching event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySwitch {
    /// From strategy
    pub from_strategy: String,
    /// To strategy
    pub to_strategy: String,
    /// Switch reason
    pub reason: SwitchReason,
    /// Performance improvement
    pub performance_improvement: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Strategy switching reasons
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwitchReason {
    Performance,
    Convergence,
    Stagnation,
    ResourceEfficiency,
    ProblemChange,
    Custom(String),
}

/// Adaptation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdaptationStatus {
    Stable,
    Adapting,
    Converging,
    Diverging,
    Unknown,
}

/// Hyper-heuristics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperHeuristicState {
    /// Active heuristic portfolio
    pub portfolio: HashMap<HeuristicId, HeuristicInfo>,
    /// Heuristic performance
    pub performance: HeuristicPerformance,
    /// Generation state
    pub generation_state: GenerationState,
    /// Selection state
    pub selection_state: SelectionState,
}

/// Heuristic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicInfo {
    /// Heuristic identifier
    pub id: HeuristicId,
    /// Heuristic type
    pub heuristic_type: HeuristicType,
    /// Performance metrics
    pub performance: HeuristicMetrics,
    /// Complexity score
    pub complexity: f64,
    /// Usage count
    pub usage_count: u64,
}

/// Heuristic types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HeuristicType {
    Selection,
    Crossover,
    Mutation,
    Replacement,
    Custom(String),
}

/// Heuristic performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicMetrics {
    /// Success rate
    pub success_rate: f64,
    /// Improvement contribution
    pub improvement_contribution: f64,
    /// Diversity impact
    pub diversity_impact: f64,
    /// Computational cost
    pub computational_cost: f64,
}

/// Heuristic performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicPerformance {
    /// Best performing heuristic
    pub best_heuristic: Option<HeuristicId>,
    /// Average portfolio performance
    pub average_performance: f64,
    /// Portfolio diversity
    pub portfolio_diversity: f64,
    /// Generation success rate
    pub generation_success_rate: f64,
}

/// Generation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationState {
    /// Generation attempts
    pub generation_attempts: u64,
    /// Successful generations
    pub successful_generations: u64,
    /// Average generation time
    pub avg_generation_time_seconds: f64,
    /// Generation diversity
    pub generation_diversity: f64,
}

/// Selection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionState {
    /// Selection pressure
    pub selection_pressure: f64,
    /// Selection diversity
    pub selection_diversity: f64,
    /// Elite preservation rate
    pub elite_preservation_rate: f64,
    /// Selection stability
    pub selection_stability: f64,
}

/// Fitness analysis state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessAnalysisState {
    /// Analyzed landscapes
    pub analyzed_landscapes: HashMap<LandscapeId, LandscapeAnalysis>,
    /// Analysis performance
    pub analysis_performance: AnalysisPerformance,
    /// Prediction accuracy
    pub prediction_accuracy: f64,
    /// Analysis cache hit rate
    pub cache_hit_rate: f64,
}

/// Fitness landscape analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandscapeAnalysis {
    /// Landscape identifier
    pub id: LandscapeId,
    /// Landscape type
    pub landscape_type: FitnessLandscapeType,
    /// Landscape characteristics
    pub characteristics: LandscapeCharacteristics,
    /// Algorithm recommendations
    pub algorithm_recommendations: Vec<AlgorithmRecommendation>,
    /// Analysis confidence
    pub confidence: f64,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
}

/// Landscape characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandscapeCharacteristics {
    /// Modality (number of local optima)
    pub modality: f64,
    /// Global structure
    pub global_structure: GlobalStructure,
    /// Local structure
    pub local_structure: LocalStructure,
    /// Deceptiveness measure
    pub deceptiveness: f64,
    /// Neutrality measure
    pub neutrality: f64,
}

/// Global structure of fitness landscape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStructure {
    /// Global correlation
    pub global_correlation: f64,
    /// Fitness distance correlation
    pub fitness_distance_correlation: f64,
    /// Epistasis measure
    pub epistasis: f64,
    /// Ruggedness
    pub ruggedness: f64,
}

/// Local structure of fitness landscape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStructure {
    /// Local optima density
    pub local_optima_density: f64,
    /// Basin size distribution
    pub basin_sizes: Vec<f64>,
    /// Gradient information
    pub gradient_info: GradientInfo,
    /// Neighborhood structure
    pub neighborhood_structure: NeighborhoodStructure,
}

/// Gradient information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientInfo {
    /// Average gradient magnitude
    pub avg_gradient_magnitude: f64,
    /// Gradient variance
    pub gradient_variance: f64,
    /// Gradient direction consistency
    pub direction_consistency: f64,
}

/// Neighborhood structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodStructure {
    /// Average neighborhood fitness
    pub avg_neighborhood_fitness: f64,
    /// Neighborhood diversity
    pub neighborhood_diversity: f64,
    /// Connectivity
    pub connectivity: f64,
}

/// Analysis performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPerformance {
    /// Analysis accuracy
    pub accuracy: f64,
    /// Analysis speed
    pub speed_samples_per_second: f64,
    /// Memory usage
    pub memory_usage_mb: f64,
    /// Cache efficiency
    pub cache_efficiency: f64,
}

/// Test problem for algorithm evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProblem {
    /// Problem identifier
    pub id: String,
    /// Problem name
    pub name: String,
    /// Problem type
    pub problem_type: ProblemType,
    /// Dimensionality
    pub dimensionality: usize,
    /// Fitness function
    pub fitness_function: Arc<dyn FitnessFunction>,
    /// Problem bounds
    pub bounds: Option<(Vec<f64>, Vec<f64>)>,
    /// Known optimal solution (if available)
    pub optimal_solution: Option<Vec<f64>>,
    /// Optimal fitness value
    pub optimal_fitness: Option<f64>,
}

/// Comprehensive analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveAnalysis {
    /// Analysis identifier
    pub id: String,
    /// Test problems analyzed
    pub problems_analyzed: Vec<ProblemAnalysisResult>,
    /// Algorithm portfolio performance
    pub portfolio_performance: PortfolioPerformance,
    /// Meta-learning insights
    pub meta_insights: MetaLearningInsights,
    /// Recommendations
    pub recommendations: AnalysisRecommendations,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
}

/// Problem analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemAnalysisResult {
    /// Problem identifier
    pub problem_id: String,
    /// Landscape analysis
    pub landscape_analysis: LandscapeAnalysis,
    /// Algorithm recommendations
    pub algorithm_recommendations: Vec<AlgorithmRecommendation>,
    /// Expected performance
    pub expected_performance: ExpectedPerformance,
}

/// Portfolio performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPerformance {
    /// Overall portfolio success rate
    pub success_rate: f64,
    /// Average performance across problems
    pub average_performance: f64,
    /// Portfolio robustness
    pub robustness: f64,
    /// Algorithm diversity benefit
    pub diversity_benefit: f64,
}

/// Meta-learning insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningInsights {
    /// Algorithm selection accuracy
    pub selection_accuracy: f64,
    /// Learning progress
    pub learning_progress: f64,
    /// Adaptation effectiveness
    pub adaptation_effectiveness: f64,
    /// Knowledge transfer efficiency
    pub knowledge_transfer: f64,
}

/// Analysis recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecommendations {
    /// Recommended algorithm portfolio updates
    pub portfolio_updates: Vec<PortfolioUpdate>,
    /// Parameter optimization suggestions
    pub parameter_suggestions: Vec<ParameterSuggestion>,
    /// Strategy improvements
    pub strategy_improvements: Vec<StrategyImprovement>,
    /// Integration recommendations
    pub integration_recommendations: Vec<IntegrationRecommendation>,
}

/// Portfolio update recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioUpdate {
    /// Algorithm to add
    pub add_algorithm: Option<AlgorithmId>,
    /// Algorithm to remove
    pub remove_algorithm: Option<AlgorithmId>,
    /// Algorithm to modify
    pub modify_algorithm: Option<(AlgorithmId, HashMap<String, f64>)>,
    /// Reasoning
    pub reasoning: String,
}

/// Parameter optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSuggestion {
    /// Parameter name
    pub parameter_name: String,
    /// Suggested value
    pub suggested_value: f64,
    /// Expected improvement
    pub expected_improvement: f64,
    /// Confidence level
    pub confidence: f64,
}

/// Strategy improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyImprovement {
    /// Strategy to improve
    pub strategy_id: String,
    /// Improvement type
    pub improvement_type: ImprovementType,
    /// Description
    pub description: String,
    /// Expected benefit
    pub expected_benefit: f64,
}

/// Improvement types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImprovementType {
    ParameterTuning,
    AlgorithmEnhancement,
    IntegrationImprovement,
    PerformanceOptimization,
    Custom(String),
}

/// Integration recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRecommendation {
    /// Target layer
    pub target_layer: String,
    /// Recommendation type
    pub recommendation_type: IntegrationType,
    /// Description
    pub description: String,
    /// Priority
    pub priority: Priority,
}

/// Integration types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrationType {
    DataSharing,
    FeedbackLoop,
    ResourceCoordination,
    PerformanceMonitoring,
    Custom(String),
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Expected performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedPerformance {
    /// Expected success rate
    pub success_rate: f64,
    /// Expected convergence time
    pub convergence_time: f64,
    /// Expected solution quality
    pub solution_quality: f64,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
}

/// Optimized parameters result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedParameters {
    /// Population parameters
    pub population: PopulationParameters,
    /// Adaptive strategy parameters
    pub adaptive_strategy: AdaptiveStrategyParameters,
    /// Hyper-heuristic parameters
    pub hyper_heuristics: HyperHeuristicParameters,
    /// Performance analysis
    pub performance_analysis: PerformanceAnalysis,
    /// Optimization timestamp
    pub timestamp: DateTime<Utc>,
}

/// Population parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationParameters {
    /// Optimal population size
    pub optimal_size: usize,
    /// Migration topology
    pub migration_topology: MigrationTopology,
    /// Diversity management parameters
    pub diversity_params: DiversityParameters,
    /// Convergence parameters
    pub convergence_params: ConvergenceParameters,
}

/// Migration topology types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationTopology {
    Ring,
    Star,
    Complete,
    Grid,
    Custom(String),
}

/// Diversity parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityParameters {
    /// Target diversity level
    pub target_diversity: f64,
    /// Diversity adjustment rate
    pub adjustment_rate: f64,
    /// Diversity measures
    pub measures: Vec<DiversityMeasure>,
    /// Diversity thresholds
    pub thresholds: HashMap<String, f64>,
}

/// Diversity measures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiversityMeasure {
    Genotypic,
    Phenotypic,
    Behavioral,
    Custom(String),
}

/// Convergence parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceParameters {
    /// Convergence threshold
    pub threshold: f64,
    /// Stagnation detection window
    pub stagnation_window: u32,
    /// Restart criteria
    pub restart_criteria: RestartCriteria,
}

/// Restart criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartCriteria {
    /// Maximum generations without improvement
    pub max_generations_no_improvement: u32,
    /// Minimum diversity threshold for restart
    pub min_diversity_threshold: f64,
    /// Restart probability
    pub restart_probability: f64,
}

/// Adaptive strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveStrategyParameters {
    /// Learning rates
    pub learning_rates: HashMap<String, f64>,
    /// Adaptation schedules
    pub adaptation_schedules: HashMap<String, AdaptationSchedule>,
    /// Strategy switching parameters
    pub switching_params: StrategySwitchingParameters,
}

/// Adaptation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationSchedule {
    /// Schedule type
    pub schedule_type: ScheduleType,
    /// Schedule parameters
    pub parameters: HashMap<String, f64>,
    /// Initial value
    pub initial_value: f64,
    /// Final value
    pub final_value: f64,
}

/// Schedule types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleType {
    Linear,
    Exponential,
    Logarithmic,
    Step,
    Custom(String),
}

/// Strategy switching parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySwitchingParameters {
    /// Switching threshold
    pub threshold: f64,
    /// Switching delay (generations)
    pub delay: u32,
    /// Performance window for switching decision
    pub performance_window: u32,
    /// Switching cost penalty
    pub cost_penalty: f64,
}

/// Hyper-heuristic parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperHeuristicParameters {
    /// Portfolio size limits
    pub portfolio_limits: PortfolioLimits,
    /// Generation parameters
    pub generation_params: GenerationParameters,
    /// Selection parameters
    pub selection_params: SelectionParameters,
}

/// Portfolio limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioLimits {
    /// Maximum portfolio size
    pub max_size: usize,
    /// Minimum portfolio size
    pub min_size: usize,
    /// Complexity limits
    pub complexity_limits: ComplexityLimits,
}

/// Complexity limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityLimits {
    /// Maximum heuristic complexity
    pub max_complexity: f64,
    /// Complexity penalty factor
    pub penalty_factor: f64,
    /// Complexity diversity requirement
    pub diversity_requirement: f64,
}

/// Generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParameters {
    /// Generation rate
    pub rate: f64,
    /// Generation diversity target
    pub diversity_target: f64,
    /// Generation quality threshold
    pub quality_threshold: f64,
    /// Generation attempts limit
    pub attempts_limit: u32,
}

/// Selection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionParameters {
    /// Selection pressure
    pub pressure: f64,
    /// Elite size
    pub elite_size: usize,
    /// Tournament size
    pub tournament_size: usize,
    /// Selection stability factor
    pub stability_factor: f64,
}

/// Performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Historical performance trends
    pub trends: PerformanceTrends,
    /// Bottleneck identification
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Optimization opportunities
    pub opportunities: Vec<OptimizationOpportunity>,
    /// Resource utilization analysis
    pub resource_analysis: ResourceAnalysis,
}

/// Performance trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Success rate trend
    pub success_rate_trend: Trend,
    /// Convergence time trend
    pub convergence_time_trend: Trend,
    /// Solution quality trend
    pub solution_quality_trend: Trend,
    /// Resource efficiency trend
    pub resource_efficiency_trend: Trend,
}

/// Trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend magnitude
    pub magnitude: f64,
    /// Trend stability
    pub stability: f64,
    /// Data points
    pub data_points: Vec<(DateTime<Utc>, f64)>,
}

/// Trend directions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Bottleneck type
    pub bottleneck_type: BottleneckType,
    /// Impact magnitude
    pub impact: f64,
    /// Affected components
    pub affected_components: Vec<String>,
    /// Mitigation suggestions
    pub mitigation_suggestions: Vec<String>,
}

/// Bottleneck types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BottleneckType {
    Computational,
    Memory,
    Communication,
    Algorithmic,
    Integration,
    Custom(String),
}

/// Optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    /// Opportunity type
    pub opportunity_type: OpportunityType,
    /// Potential improvement
    pub potential_improvement: f64,
    /// Implementation complexity
    pub implementation_complexity: Complexity,
    /// Priority score
    pub priority_score: f64,
}

/// Opportunity types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpportunityType {
    AlgorithmImprovement,
    ParameterOptimization,
    ResourceOptimization,
    IntegrationEnhancement,
    Custom(String),
}

/// Complexity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Complexity {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Resource analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    /// CPU utilization patterns
    pub cpu_utilization: UtilizationPattern,
    /// Memory utilization patterns
    pub memory_utilization: UtilizationPattern,
    /// Network utilization patterns
    pub network_utilization: UtilizationPattern,
    /// Efficiency metrics
    pub efficiency_metrics: EfficiencyMetrics,
}

/// Utilization pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationPattern {
    /// Average utilization
    pub average: f64,
    /// Peak utilization
    pub peak: f64,
    /// Utilization variance
    pub variance: f64,
    /// Bottleneck periods
    pub bottleneck_periods: Vec<TimePeriod>,
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Start time
    pub start: DateTime<Utc>,
    /// End time
    pub end: DateTime<Utc>,
    /// Utilization during period
    pub utilization: f64,
}

/// Efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Computational efficiency
    pub computational_efficiency: f64,
    /// Memory efficiency
    pub memory_efficiency: f64,
    /// Communication efficiency
    pub communication_efficiency: f64,
    /// Overall efficiency score
    pub overall_efficiency: f64,
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service name
    pub service: String,
    /// Overall service status
    pub status: ServiceStatus,
    /// Component health details
    pub components: Vec<ComponentHealth>,
    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Starting,
    Stopping,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Component status
    pub status: ServiceStatus,
    /// Health check duration in milliseconds
    pub check_duration_ms: u64,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Component metrics
    pub metrics: HashMap<String, f64>,
}

/// Evolution service errors
#[derive(Error, Debug)]
pub enum EvolutionError {
    #[error("Meta-learning error: {0}")]
    MetaLearningError(String),

    #[error("Population error: {0}")]
    PopulationError(String),

    #[error("Adaptive strategy error: {0}")]
    AdaptiveError(String),

    #[error("Hyper-heuristic error: {0}")]
    HyperHeuristicError(String),

    #[error("Fitness analysis error: {0}")]
    FitnessAnalysisError(String),

    #[error("Integration error: {0}")]
    IntegrationError(String),

    #[error("Algorithm error: {0}")]
    AlgorithmError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Resource error: {0}")]
    ResourceError(String),

    #[error("Convergence error: {0}")]
    ConvergenceError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Utility functions
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return f64::INFINITY;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Calculate problem identifier from characteristics
pub fn calculate_problem_id(characteristics: &ProblemCharacteristics) -> String {
    format!(
        "problem_{}_{}_{}_{}",
        characteristics.dimensionality,
        characteristics.problem_type,
        characteristics.landscape,
        characteristics.multi_objective
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_config_default() {
        let config = EvolutionConfig::default();
        assert_eq!(config.meta_learning.portfolio_size, 10);
        assert_eq!(config.population.default_size, 100);
        assert_eq!(config.adaptive.adaptation_rate, 0.1);
        assert_eq!(config.hyper_heuristics.max_portfolio_size, 20);
    }

    #[test]
    fn test_individual_creation() {
        let individual = Individual {
            id: "test-individual".to_string(),
            genome: vec![1.0, 2.0, 3.0],
            fitness: 0.95,
            objective_values: vec![0.95, 0.8],
            age: 5,
            parents: Some(("parent1".to_string(), "parent2".to_string())),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        assert_eq!(individual.id, "test-individual");
        assert_eq!(individual.genome.len(), 3);
        assert_eq!(individual.fitness, 0.95);
        assert_eq!(individual.objective_values.len(), 2);
        assert_eq!(individual.age, 5);
    }

    #[test]
    fn test_population_creation() {
        let individuals = vec![
            Individual {
                id: "ind1".to_string(),
                genome: vec![1.0, 2.0],
                fitness: 0.9,
                objective_values: vec![0.9],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
            Individual {
                id: "ind2".to_string(),
                genome: vec![2.0, 3.0],
                fitness: 0.8,
                objective_values: vec![0.8],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
        ];

        let population = Population::new("test-population".to_string(), individuals);

        assert_eq!(population.id, "test-population");
        assert_eq!(population.size(), 2);
        assert_eq!(population.generation, 0);
        assert!(population.best_individual().unwrap().fitness > 0.8);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        let distance = euclidean_distance(&a, &b);
        assert!((distance - 5.0).abs() < 0.001); // 3-4-5 triangle

        let c = vec![1.0, 1.0];
        let d = vec![1.0, 1.0];
        let distance2 = euclidean_distance(&c, &d);
        assert!((distance2 - 0.0).abs() < 0.001); // Same point
    }

    #[test]
    fn test_problem_characteristics() {
        let characteristics = ProblemCharacteristics {
            dimensionality: 10,
            problem_type: ProblemType::Continuous,
            landscape: FitnessLandscapeType::Multimodal,
            multi_objective: true,
            constraints: vec![ConstraintType::Boundary, ConstraintType::Equality],
            expected_population_size: 200,
        };

        let problem_id = calculate_problem_id(&characteristics);
        assert!(problem_id.contains("10"));
        assert!(problem_id.contains("Continuous"));
        assert!(problem_id.contains("Multimodal"));
        assert!(problem_id.contains("true"));
    }

    #[test]
    fn test_evolution_result_creation() {
        let best_individual = Individual {
            id: "best".to_string(),
            genome: vec![1.0, 2.0],
            fitness: 0.99,
            objective_values: vec![0.99, 0.95],
            age: 10,
            parents: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        let individuals = vec![best_individual.clone()];
        let final_population = Population::new("final".to_string(), individuals);

        let result = EvolutionResult::new(
            best_individual,
            final_population,
            "test-algorithm".to_string(),
            100,
            10000,
            45.5,
        );

        assert_eq!(result.algorithm_used, "test-algorithm");
        assert_eq!(result.generations, 100);
        assert_eq!(result.total_evaluations, 10000);
        assert!((result.duration_seconds - 45.5).abs() < 0.001);
        assert!(result.best_fitness > 0.9);
    }
}