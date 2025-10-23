//! # Population Dynamics
//!
//! The Population Dynamics module manages advanced population structures, migration strategies,
//! diversity maintenance, and convergence detection for Layer 6 (Evolution). It provides
//! sophisticated population management capabilities for multi-population evolutionary algorithms.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Population manager for advanced population dynamics
pub struct PopulationManager {
    config: PopulationConfig,
    populations: Arc<Mutex<HashMap<PopulationId, Population>>>,
    migration_engine: Arc<Mutex<MigrationEngine>>,
    diversity_manager: Arc<Mutex<DiversityManager>>,
    convergence_detector: Arc<Mutex<ConvergenceDetector>>,
    is_running: Arc<Mutex<bool>>,
}

impl PopulationManager {
    /// Create a new population manager
    pub async fn new(config: PopulationConfig) -> Result<Self, EvolutionError> {
        let populations = Arc::new(Mutex::new(HashMap::new()));
        let migration_engine = Arc::new(Mutex::new(MigrationEngine::new()));
        let diversity_manager = Arc::new(Mutex::new(DiversityManager::new()));
        let convergence_detector = Arc::new(Mutex::new(ConvergenceDetector::new()));

        Ok(Self {
            config,
            populations,
            migration_engine,
            diversity_manager,
            convergence_detector,
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start the population manager
    pub async fn start(&mut self) -> Result<(), EvolutionError> {
        info!("Starting Population Manager");
        *self.is_running.lock().await = true;

        // Start migration engine
        let migration_engine = self.migration_engine.clone();
        let migration_interval = self.config.migration_interval;
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // Check every minute

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !*is_running.lock().await {
                            break;
                        }

                        if let Err(e) = migration_engine.lock().await.perform_migration_cycle().await {
                            error!("Migration cycle failed: {}", e);
                        }
                    }
                }
            }
        });

        info!("Population Manager started successfully");
        Ok(())
    }

    /// Stop the population manager
    pub async fn stop(&mut self) -> Result<(), EvolutionError> {
        info!("Stopping Population Manager");
        *self.is_running.lock().await = false;
        info!("Population Manager stopped successfully");
        Ok(())
    }

    /// Initialize a new population
    pub async fn initialize_population(
        &self,
        initial_population: Population,
        landscape_analysis: &LandscapeAnalysis,
    ) -> Result<Population, EvolutionError> {
        debug!("Initializing population: {}", initial_population.id);

        let mut population = initial_population;
        population.generation = 0;

        // Analyze population structure based on landscape
        let structure_analysis = self.analyze_optimal_structure(landscape_analysis).await?;

        // Apply population structure optimizations
        if structure_analysis.recommend_multi_population {
            self.convert_to_multi_population(&mut population, &structure_analysis).await?;
        }

        // Initialize diversity management
        let mut diversity_manager = self.diversity_manager.lock().await;
        diversity_manager.initialize_for_population(&population).await?;

        // Initialize convergence detection
        let mut convergence_detector = self.convergence_detector.lock().await;
        convergence_detector.initialize_for_population(&population).await?;

        // Store the population
        self.populations.lock().await.insert(population.id.clone(), population.clone());

        info!("Population {} initialized with {} individuals", population.id, population.size());
        Ok(population)
    }

    /// Create initial population for a problem
    pub async fn create_initial_population(
        &self,
        problem: &TestProblem,
    ) -> Result<Population, EvolutionError> {
        debug!("Creating initial population for problem: {}", problem.id);

        let population_size = self.config.default_size;
        let mut individuals = Vec::new();

        // Create individuals with random genomes within problem bounds
        for i in 0..population_size {
            let genome = if let Some((lower_bounds, upper_bounds)) = &problem.bounds {
                // Generate within bounds
                lower_bounds.iter()
                    .zip(upper_bounds.iter())
                    .map(|(lower, upper)| {
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        rng.gen_range(*lower..=*upper)
                    })
                    .collect()
            } else {
                // Generate standard normal distribution
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..problem.dimensionality).map(|_| rng.gen_range(0.0..1.0)).collect()
            };

            let individual = Individual {
                id: format!("initial-{}-{}", problem.id, i),
                genome,
                fitness: 0.0, // Will be evaluated by fitness function
                objective_values: Vec::new(),
                age: 0,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            };

            individuals.push(individual);
        }

        let population = Population::new(
            format!("initial-population-{}", problem.id),
            individuals,
        );

        info!("Created initial population with {} individuals", population.size());
        Ok(population)
    }

    /// Evolve population for one generation
    pub async fn evolve_generation(
        &self,
        population: &mut Population,
        fitness_function: Arc<dyn FitnessFunction>,
    ) -> Result<(), EvolutionError> {
        debug!("Evolving population {} for generation {}", population.id, population.generation);

        // Evaluate fitness if needed
        if population.individuals.iter().any(|ind| ind.fitness == 0.0) {
            self.evaluate_population_fitness(population, fitness_function).await?;
        }

        // Update population statistics
        population.statistics = PopulationStatistics::from_individuals(&population.individuals);
        population.generation += 1;

        // Check for convergence
        let mut convergence_detector = self.convergence_detector.lock().await;
        if convergence_detector.check_convergence(population).await? {
            info!("Population {} has converged", population.id);
        }

        // Manage diversity
        let mut diversity_manager = self.diversity_manager.lock().await;
        diversity_manager.update_diversity(population).await?;

        // Perform migration if applicable
        let mut migration_engine = self.migration_engine.lock().await;
        migration_engine.check_and_perform_migration(population).await?;

        Ok(())
    }

    /// Optimize population parameters based on performance
    pub async fn optimize_parameters(
        &self,
        performance_analysis: &PerformanceAnalysis,
    ) -> Result<PopulationParameters, EvolutionError> {
        debug!("Optimizing population parameters");

        let diversity_manager = self.diversity_manager.lock().await;
        let convergence_detector = self.convergence_detector.lock().await;

        let optimal_size = self.calculate_optimal_population_size(performance_analysis).await?;
        let migration_topology = self.select_optimal_migration_topology(performance_analysis).await?;
        let diversity_params = diversity_manager.optimize_diversity_parameters(performance_analysis).await?;
        let convergence_params = convergence_detector.optimize_convergence_parameters(performance_analysis).await?;

        Ok(PopulationParameters {
            optimal_size,
            migration_topology,
            diversity_params,
            convergence_params,
        })
    }

    /// Get current population state
    pub async fn get_state(&self) -> Result<PopulationState, EvolutionError> {
        let populations = self.populations.lock().await;
        let migration_engine = self.migration_engine.lock().await;
        let diversity_manager = self.diversity_manager.lock().await;

        let mut population_infos = HashMap::new();
        for (id, population) in populations.iter() {
            population_infos.insert(id.clone(), PopulationInfo {
                id: id.clone(),
                size: population.size(),
                statistics: population.statistics.clone(),
                connections: population.subpopulations.as_ref()
                    .map(|subs| subs.iter().flat_map(|sub| sub.connections.clone()).collect())
                    .unwrap_or_default(),
                age: population.generation,
            });
        }

        let migration_events = migration_engine.get_recent_events().await?;
        let global_diversity = diversity_manager.get_global_diversity().await?;
        let health_score = self.calculate_population_health_score(&population_infos).await?;

        Ok(PopulationState {
            populations: population_infos,
            migration_events,
            global_diversity,
            health_score,
        })
    }

    /// Get population health status
    pub async fn health_check(&self) -> Result<ComponentHealth, EvolutionError> {
        let is_running = *self.is_running.lock().await;
        let populations_count = self.populations.lock().await.len();
        let diversity_manager = self.diversity_manager.lock().await;

        let status = if is_running && populations_count > 0 {
            ServiceStatus::Healthy
        } else if is_running {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "population-manager".to_string(),
            status,
            check_duration_ms: 0,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("populations_count".to_string(), populations_count as f64);
                metrics.insert("global_diversity".to_string(), diversity_manager.get_global_diversity().await?);
                metrics.insert("migration_interval".to_string(), self.config.migration_interval as f64);
                metrics.insert("migration_rate".to_string(), self.config.migration_rate);
                metrics
            },
        })
    }

    /// Evaluate fitness for all individuals in population
    async fn evaluate_population_fitness(
        &self,
        population: &mut Population,
        fitness_function: Arc<dyn FitnessFunction>,
    ) -> Result<(), EvolutionError> {
        debug!("Evaluating fitness for {} individuals", population.individuals.len());

        // Evaluate in batches for efficiency
        const BATCH_SIZE: usize = 100;
        for chunk in population.individuals.chunks_mut(BATCH_SIZE) {
            let individuals: Vec<Individual> = chunk.iter().cloned().collect();
            let fitness_results = fitness_function.evaluate_batch(&individuals).await?;

            for (individual, fitness_result) in chunk.iter_mut().zip(fitness_results) {
                individual.fitness = fitness_result.fitness;
                individual.objective_values = fitness_result.objective_values.clone();
            }
        }

        Ok(())
    }

    /// Analyze optimal population structure for a landscape
    async fn analyze_optimal_structure(
        &self,
        landscape_analysis: &LandscapeAnalysis,
    ) -> Result<StructureAnalysis, EvolutionError> {
        let recommend_multi_population = match landscape_analysis.landscape_type {
            FitnessLandscapeType::Multimodal => true,
            FitnessLandscapeType::Deceptive => true,
            FitnessLandscapeType::Rugged => true,
            _ => false,
        };

        let recommended_subpopulations = if recommend_multi_population {
            match landscape_analysis.characteristics.modality as usize {
                0..=2 => 2,
                3..=5 => 3,
                6..=10 => 4,
                _ => 5,
            }
        } else {
            1
        };

        let migration_topology = if recommend_multi_population {
            MigrationTopology::Ring
        } else {
            MigrationTopology::Complete
        };

        Ok(StructureAnalysis {
            recommend_multi_population,
            recommended_subpopulations,
            migration_topology,
            diversity_target: 0.7,
            convergence_threshold: 0.001,
        })
    }

    /// Convert single population to multi-population structure
    async fn convert_to_multi_population(
        &self,
        population: &mut Population,
        structure_analysis: &StructureAnalysis,
    ) -> Result<(), EvolutionError> {
        if !structure_analysis.recommend_multi_population {
            return Ok(());
        }

        let subpopulation_size = population.size() / structure_analysis.recommended_subpopulations;
        let mut subpopulations = Vec::new();

        for i in 0..structure_analysis.recommended_subpopulations {
            let start_idx = i * subpopulation_size;
            let end_idx = if i == structure_analysis.recommended_subpopulations - 1 {
                population.size()
            } else {
                (i + 1) * subpopulation_size
            };

            let individuals = population.individuals[start_idx..end_idx].to_vec();
            let subpopulation = Subpopulation {
                id: format!("{}-subpop-{}", population.id, i),
                individuals,
                statistics: PopulationStatistics::from_individuals(&population.individuals[start_idx..end_idx]),
                connections: self.generate_subpopulation_connections(i, structure_analysis.recommended_subpopulations).await?,
                age: 0,
            };

            subpopulations.push(subpopulation);
        }

        population.subpopulations = Some(subpopulations);
        info!("Converted population to multi-population structure with {} subpopulations", structure_analysis.recommended_subpopulations);
        Ok(())
    }

    /// Generate connections between subpopulations
    async fn generate_subpopulation_connections(
        &self,
        current_idx: usize,
        total_subpopulations: usize,
    ) -> Result<Vec<String>, EvolutionError> {
        let mut connections = Vec::new();

        match total_subpopulations {
            2 => {
                // For 2 subpopulations, connect to the other one
                let other_idx = 1 - current_idx;
                connections.push(format!("subpop-{}", other_idx));
            }
            3 => {
                // Ring topology for 3 subpopulations
                let next_idx = (current_idx + 1) % 3;
                connections.push(format!("subpop-{}", next_idx));
            }
            _ => {
                // Ring topology for more subpopulations
                let next_idx = (current_idx + 1) % total_subpopulations;
                let prev_idx = if current_idx == 0 { total_subpopulations - 1 } else { current_idx - 1 };
                connections.push(format!("subpop-{}", next_idx));
                connections.push(format!("subpop-{}", prev_idx));
            }
        }

        Ok(connections)
    }

    /// Calculate optimal population size based on performance analysis
    async fn calculate_optimal_population_size(
        &self,
        performance_analysis: &PerformanceAnalysis,
    ) -> Result<usize, EvolutionError> {
        // Analyze performance trends to determine optimal size
        let base_size = self.config.default_size;

        // Adjust based on convergence trends
        let convergence_trend = &performance_analysis.trends.convergence_time_trend;
        let size_adjustment = match convergence_trend.direction {
            TrendDirection::Increasing => -50, // Decrease size if convergence is getting slower
            TrendDirection::Decreasing => 50,  // Increase size if convergence is getting faster
            _ => 0,
        };

        let optimal_size = (base_size as i32 + size_adjustment).max(self.config.min_size as i32).min(self.config.max_size as i32) as usize;

        Ok(optimal_size)
    }

    /// Select optimal migration topology based on performance
    async fn select_optimal_migration_topology(
        &self,
        performance_analysis: &PerformanceAnalysis,
    ) -> Result<MigrationTopology, EvolutionError> {
        // Analyze which topology performs best based on historical data
        let diversity_benefit = performance_analysis.opportunities.iter()
            .find(|opp| matches!(opp.opportunity_type, OpportunityType::AlgorithmImprovement))
            .map(|opp| opp.potential_improvement)
            .unwrap_or(0.0);

        let topology = if diversity_benefit > 0.2 {
            MigrationTopology::Ring // Better for maintaining diversity
        } else if diversity_benefit > 0.1 {
            MigrationTopology::Star // Good balance
        } else {
            MigrationTopology::Complete // Best for convergence
        };

        Ok(topology)
    }

    /// Calculate overall population health score
    async fn calculate_population_health_score(
        &self,
        population_infos: &HashMap<PopulationId, PopulationInfo>,
    ) -> Result<f64, EvolutionError> {
        if population_infos.is_empty() {
            return Ok(0.0);
        }

        let mut total_health = 0.0;
        let mut count = 0;

        for info in population_infos.values() {
            // Health based on diversity and convergence
            let diversity_score = info.statistics.diversity;
            let convergence_score = 1.0 - info.statistics.convergence; // Lower convergence = higher diversity
            let health = (diversity_score + convergence_score) / 2.0;

            total_health += health;
            count += 1;
        }

        Ok(total_health / count as f64)
    }
}

/// Structure analysis for population optimization
struct StructureAnalysis {
    recommend_multi_population: bool,
    recommended_subpopulations: usize,
    migration_topology: MigrationTopology,
    diversity_target: f64,
    convergence_threshold: f64,
}

/// Migration engine for inter-population migration
struct MigrationEngine {
    migration_events: Vec<MigrationEvent>,
    topology_cache: HashMap<PopulationId, MigrationTopology>,
}

impl MigrationEngine {
    fn new() -> Self {
        Self {
            migration_events: Vec::new(),
            topology_cache: HashMap::new(),
        }
    }

    async fn perform_migration_cycle(&mut self) -> Result<(), EvolutionError> {
        debug!("Performing migration cycle");
        // Implementation would perform actual migration between populations
        Ok(())
    }

    async fn check_and_perform_migration(&mut self, population: &mut Population) -> Result<(), EvolutionError> {
        if let Some(subpopulations) = &population.subpopulations {
            if population.generation % 10 == 0 { // Migrate every 10 generations
                self.perform_migration(population, subpopulations).await?;
            }
        }
        Ok(())
    }

    async fn perform_migration(
        &mut self,
        population: &mut Population,
        subpopulations: &[Subpopulation],
    ) -> Result<(), EvolutionError> {
        debug!("Performing migration between {} subpopulations", subpopulations.len());

        // Simple migration: exchange best individuals between connected subpopulations
        for (i, subpop) in subpopulations.iter().enumerate() {
            for connection in &subpop.connections {
                if let Some(target_idx) = self.parse_subpopulation_id(connection) {
                    if target_idx < subpopulations.len() && target_idx != i {
                        // Perform migration between subpopulation i and target_idx
                        let migration_event = MigrationEvent {
                            source: subpop.id.clone(),
                            target: subpopulations[target_idx].id.clone(),
                            individuals: vec!["best-individual".to_string()], // Would contain actual individual IDs
                            timestamp: Utc::now(),
                            reason: MigrationReason::Scheduled,
                        };

                        self.migration_events.push(migration_event);
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_subpopulation_id(&self, connection: &str) -> Option<usize> {
        if let Some(idx_str) = connection.strip_prefix("subpop-") {
            idx_str.parse().ok()
        } else {
            None
        }
    }

    async fn get_recent_events(&self) -> Result<Vec<MigrationEvent>, EvolutionError> {
        Ok(self.migration_events.clone())
    }
}

/// Diversity manager for population diversity optimization
struct DiversityManager {
    diversity_targets: HashMap<PopulationId, f64>,
    diversity_history: HashMap<PopulationId, Vec<f64>>,
    diversity_measures: Vec<DiversityMeasure>,
}

impl DiversityManager {
    fn new() -> Self {
        Self {
            diversity_targets: HashMap::new(),
            diversity_history: HashMap::new(),
            diversity_measures: vec![DiversityMeasure::Genotypic, DiversityMeasure::Phenotypic],
        }
    }

    async fn initialize_for_population(&mut self, population: &Population) -> Result<(), EvolutionError> {
        self.diversity_targets.insert(population.id.clone(), 0.7);
        self.diversity_history.insert(population.id.clone(), vec![population.diversity()]);
        Ok(())
    }

    async fn update_diversity(&mut self, population: &mut Population) -> Result<(), EvolutionError> {
        let current_diversity = population.diversity();

        // Update diversity history
        if let Some(history) = self.diversity_history.get_mut(&population.id) {
            history.push(current_diversity);

            // Keep only recent history
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // Check if diversity management is needed
        if let Some(target) = self.diversity_targets.get(&population.id) {
            if current_diversity < target * 0.8 {
                // Diversity too low - inject diversity
                self.inject_diversity(population).await?;
            } else if current_diversity > target * 1.2 {
                // Diversity too high - may need to reduce
                self.reduce_diversity(population).await?;
            }
        }

        Ok(())
    }

    async fn inject_diversity(&mut self, population: &mut Population) -> Result<(), EvolutionError> {
        debug!("Injecting diversity into population {}", population.id);

        // Simple diversity injection: mutate some individuals
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let individuals_to_mutate = (population.size() as f64 * 0.1) as usize; // Mutate 10% of population
        for i in 0..individuals_to_mutate {
            if i < population.individuals.len() {
                let individual = &mut population.individuals[i];
                // Simple mutation: add small random values
                for gene in &mut individual.genome {
                    *gene += rng.gen_range(-0.1..=0.1);
                }
                individual.fitness = 0.0; // Reset fitness for re-evaluation
            }
        }

        Ok(())
    }

    async fn reduce_diversity(&mut self, population: &mut Population) -> Result<(), EvolutionError> {
        debug!("Reducing diversity in population {}", population.id);

        // Simple diversity reduction: replace worst individuals with copies of best
        if let Some(best_individual) = population.best_individual() {
            let worst_count = (population.size() as f64 * 0.05) as usize; // Replace 5% of worst

            // Sort individuals by fitness (worst first)
            population.individuals.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());

            for i in 0..worst_count {
                if i < population.individuals.len() {
                    population.individuals[i] = best_individual.clone();
                    population.individuals[i].id = format!("diversity-reduced-{}", i);
                    population.individuals[i].fitness = 0.0; // Reset fitness
                }
            }

            // Re-sort by fitness (best first)
            population.individuals.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        }

        Ok(())
    }

    async fn optimize_diversity_parameters(
        &self,
        performance_analysis: &PerformanceAnalysis,
    ) -> Result<DiversityParameters, EvolutionError> {
        // Analyze performance to optimize diversity parameters
        let diversity_trend = &performance_analysis.trends.solution_quality_trend;

        let target_diversity = match diversity_trend.direction {
            TrendDirection::Increasing => 0.8, // Higher diversity for improving performance
            TrendDirection::Decreasing => 0.6,  // Lower diversity for declining performance
            _ => 0.7,
        };

        let adjustment_rate = 0.1; // Gradual adjustment

        let measures = vec![DiversityMeasure::Genotypic, DiversityMeasure::Phenotypic];

        let mut thresholds = HashMap::new();
        thresholds.insert("min_diversity".to_string(), 0.3);
        thresholds.insert("max_diversity".to_string(), 0.9);

        Ok(DiversityParameters {
            target_diversity,
            adjustment_rate,
            measures,
            thresholds,
        })
    }

    async fn get_global_diversity(&self) -> Result<f64, EvolutionError> {
        let total_populations = self.diversity_history.len();
        if total_populations == 0 {
            return Ok(0.0);
        }

        let total_diversity: f64 = self.diversity_history.values()
            .map(|history| history.last().unwrap_or(&0.0))
            .sum();

        Ok(total_diversity / total_populations as f64)
    }
}

/// Convergence detector for population convergence analysis
struct ConvergenceDetector {
    convergence_history: HashMap<PopulationId, Vec<f64>>,
    stagnation_thresholds: HashMap<PopulationId, u32>,
}

impl ConvergenceDetector {
    fn new() -> Self {
        Self {
            convergence_history: HashMap::new(),
            stagnation_thresholds: HashMap::new(),
        }
    }

    async fn initialize_for_population(&mut self, population: &Population) -> Result<(), EvolutionError> {
        self.convergence_history.insert(population.id.clone(), vec![population.statistics.convergence]);
        self.stagnation_thresholds.insert(population.id.clone(), 0);
        Ok(())
    }

    async fn check_convergence(&mut self, population: &Population) -> Result<bool, EvolutionError> {
        let current_convergence = population.statistics.convergence;

        if let Some(history) = self.convergence_history.get_mut(&population.id) {
            history.push(current_convergence);

            // Keep only recent history
            if history.len() > 50 {
                history.remove(0);
            }

            // Check for convergence
            let is_converged = current_convergence > 0.95;

            // Check for stagnation (no improvement in recent generations)
            if history.len() >= 10 {
                let recent_avg: f64 = history[history.len() - 10..].iter().sum::<f64>() / 10.0;
                let older_avg: f64 = if history.len() >= 20 {
                    history[history.len() - 20..history.len() - 10].iter().sum::<f64>() / 10.0
                } else {
                    recent_avg
                };

                if (recent_avg - older_avg).abs() < 0.001 {
                    if let Some(stagnation_count) = self.stagnation_thresholds.get_mut(&population.id) {
                        *stagnation_count += 1;
                    }
                } else {
                    if let Some(stagnation_count) = self.stagnation_thresholds.get_mut(&population.id) {
                        *stagnation_count = 0;
                    }
                }
            }

            Ok(is_converged)
        } else {
            Ok(false)
        }
    }

    async fn optimize_convergence_parameters(
        &self,
        performance_analysis: &PerformanceAnalysis,
    ) -> Result<ConvergenceParameters, EvolutionError> {
        let convergence_trend = &performance_analysis.trends.convergence_time_trend;

        let threshold = match convergence_trend.direction {
            TrendDirection::Increasing => 0.001, // Stricter threshold for slower convergence
            TrendDirection::Decreasing => 0.01,  // More lenient threshold for faster convergence
            _ => 0.005,
        };

        let stagnation_window = 20; // Generations without improvement

        let restart_criteria = RestartCriteria {
            max_generations_no_improvement: 50,
            min_diversity_threshold: 0.3,
            restart_probability: 0.1,
        };

        Ok(ConvergenceParameters {
            threshold,
            stagnation_window,
            restart_criteria,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_population_manager_creation() {
        let config = PopulationConfig::default();
        let manager = PopulationManager::new(config).await;
        assert!(manager.is_ok());
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
        assert!(population.diversity() > 0.0);
    }

    #[test]
    fn test_population_statistics() {
        let individuals = vec![
            Individual {
                id: "ind1".to_string(),
                genome: vec![1.0, 2.0],
                fitness: 1.0,
                objective_values: vec![1.0],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
            Individual {
                id: "ind2".to_string(),
                genome: vec![2.0, 3.0],
                fitness: 0.5,
                objective_values: vec![0.5],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
            Individual {
                id: "ind3".to_string(),
                genome: vec![1.5, 2.5],
                fitness: 0.75,
                objective_values: vec![0.75],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
        ];

        let stats = PopulationStatistics::from_individuals(&individuals);

        assert_eq!(stats.best_fitness, 1.0);
        assert_eq!(stats.worst_fitness, 0.5);
        assert!((stats.average_fitness - 0.75).abs() < 0.001);
        assert!(stats.fitness_std > 0.0);
        assert!(stats.diversity > 0.0);
        assert!(stats.convergence >= 0.0 && stats.convergence <= 1.0);
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
    fn test_migration_topology() {
        assert_eq!(MigrationTopology::Ring, MigrationTopology::Ring);
        assert_eq!(MigrationTopology::Star, MigrationTopology::Star);
        assert_eq!(MigrationTopology::Complete, MigrationTopology::Complete);
        assert_eq!(MigrationTopology::Grid, MigrationTopology::Grid);
    }

    #[test]
    fn test_diversity_measures() {
        assert_eq!(DiversityMeasure::Genotypic, DiversityMeasure::Genotypic);
        assert_eq!(DiversityMeasure::Phenotypic, DiversityMeasure::Phenotypic);
        assert_eq!(DiversityMeasure::Behavioral, DiversityMeasure::Behavioral);
    }

    #[test]
    fn test_migration_reasons() {
        assert_eq!(MigrationReason::Scheduled, MigrationReason::Scheduled);
        assert_eq!(MigrationReason::DiversityLow, MigrationReason::DiversityLow);
        assert_eq!(MigrationReason::PerformanceStagnation, MigrationReason::PerformanceStagnation);
        assert_eq!(MigrationReason::ResourceBalancing, MigrationReason::ResourceBalancing);
    }
}