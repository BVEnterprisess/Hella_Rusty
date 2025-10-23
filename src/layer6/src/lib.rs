//! # Layer 6 (Evolution) - Advanced Evolutionary Algorithms and Meta-Learning
//!
//! Layer 6 implements advanced evolutionary algorithms and meta-learning capabilities that build
//! upon the basic genetic algorithms in Layer 7. It provides sophisticated evolution strategies,
//! population dynamics, adaptive learning mechanisms, and hyper-heuristic systems for autonomous
//! AI system improvement.
//!
//! ## Core Components
//!
//! - **Meta-Learning Framework**: Algorithm selection and adaptive learning
//! - **Advanced Population Dynamics**: Multi-population and migration strategies
//! - **Adaptive Evolution Strategies**: Self-adaptive parameter control and strategy switching
//! - **Hyper-Heuristic System**: High-level heuristic selection and generation
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Layer 6 - Evolution                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┤
//! │  │ Meta-       │  │ Population  │  │ Adaptive   │  │ Hyper-  │
//! │  │ Learning    │  │ Dynamics    │  │ Evolution  │  │ Heuristic│
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┤
//! │  │ Fitness     │  │ Evolution   │  │ Diversity  │  │ Algorithm│
//! │  │ Landscapes  │  │ Strategies  │  │ Management │  │ Selection│
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod types;
pub mod meta_learning;
pub mod population_dynamics;
pub mod adaptive_evolution;
pub mod hyper_heuristics;
pub mod fitness_landscape;
pub mod integration;
pub mod metrics;

pub use types::*;
pub use meta_learning::*;
pub use population_dynamics::*;
pub use adaptive_evolution::*;
pub use hyper_heuristics::*;
pub use fitness_landscape::*;
pub use integration::*;
pub use metrics::*;

/// Main advanced evolution service that orchestrates all Layer 6 components
pub struct AdvancedEvolutionService {
    meta_learning: MetaLearningFramework,
    population_manager: PopulationManager,
    adaptive_strategy: AdaptiveEvolutionStrategy,
    hyper_heuristics: HyperHeuristicSystem,
    fitness_analyzer: FitnessLandscapeAnalyzer,
    integration_hub: EvolutionIntegrationHub,
    config: EvolutionConfig,
}

impl AdvancedEvolutionService {
    /// Create a new advanced evolution service with the given configuration
    pub async fn new(config: EvolutionConfig) -> Result<Self, EvolutionError> {
        let meta_learning = MetaLearningFramework::new(config.meta_learning.clone()).await?;
        let population_manager = PopulationManager::new(config.population.clone()).await?;
        let adaptive_strategy = AdaptiveEvolutionStrategy::new(config.adaptive.clone()).await?;
        let hyper_heuristics = HyperHeuristicSystem::new(config.hyper_heuristics.clone()).await?;
        let fitness_analyzer = FitnessLandscapeAnalyzer::new(config.fitness.clone()).await?;
        let integration_hub = EvolutionIntegrationHub::new(config.integration.clone()).await?;

        Ok(Self {
            meta_learning,
            population_manager,
            adaptive_strategy,
            hyper_heuristics,
            fitness_analyzer,
            integration_hub,
            config,
        })
    }

    /// Start the advanced evolution service and all its components
    pub async fn start(&mut self) -> Result<(), EvolutionError> {
        tracing::info!("Starting Layer 6 Advanced Evolution Service");

        // Start all components
        self.meta_learning.start().await?;
        self.population_manager.start().await?;
        self.adaptive_strategy.start().await?;
        self.hyper_heuristics.start().await?;
        self.fitness_analyzer.start().await?;
        self.integration_hub.start().await?;

        tracing::info!("Layer 6 Advanced Evolution Service started successfully");
        Ok(())
    }

    /// Stop the advanced evolution service and all its components
    pub async fn stop(&mut self) -> Result<(), EvolutionError> {
        tracing::info!("Stopping Layer 6 Advanced Evolution Service");

        // Stop all components in reverse order
        self.integration_hub.stop().await?;
        self.fitness_analyzer.stop().await?;
        self.hyper_heuristics.stop().await?;
        self.adaptive_strategy.stop().await?;
        self.population_manager.stop().await?;
        self.meta_learning.stop().await?;

        tracing::info!("Layer 6 Advanced Evolution Service stopped successfully");
        Ok(())
    }

    /// Evolve a population using advanced algorithms
    pub async fn evolve_population(
        &mut self,
        initial_population: Population,
        fitness_function: Arc<dyn FitnessFunction>,
        evolution_config: EvolutionRunConfig,
    ) -> Result<EvolutionResult, EvolutionError> {
        tracing::info!(
            "Starting advanced evolution with {} individuals",
            initial_population.individuals.len()
        );

        // Analyze fitness landscape
        let landscape_analysis = self.fitness_analyzer
            .analyze_landscape(&initial_population, fitness_function.clone())
            .await?;

        // Select optimal algorithm based on landscape characteristics
        let selected_algorithm = self.meta_learning
            .select_algorithm(&landscape_analysis, &evolution_config)
            .await?;

        // Configure adaptive strategy based on algorithm and landscape
        self.adaptive_strategy
            .configure_for_algorithm(&selected_algorithm, &landscape_analysis)
            .await?;

        // Initialize population dynamics
        let managed_population = self.population_manager
            .initialize_population(initial_population, &landscape_analysis)
            .await?;

        // Generate hyper-heuristics if beneficial
        let heuristic_portfolio = if evolution_config.use_hyper_heuristics {
            Some(self.hyper_heuristics
                .generate_portfolio(&landscape_analysis, &evolution_config)
                .await?)
        } else {
            None
        };

        // Run evolution with adaptive strategy
        let evolution_result = self.adaptive_strategy
            .evolve(
                managed_population,
                fitness_function,
                selected_algorithm,
                heuristic_portfolio,
                evolution_config,
            )
            .await?;

        // Analyze final results
        let final_analysis = self.fitness_analyzer
            .analyze_results(&evolution_result)
            .await?;

        // Update meta-learning with results
        self.meta_learning
            .update_with_results(&evolution_result, &final_analysis)
            .await?;

        tracing::info!(
            "Advanced evolution completed: best fitness = {:.6}, generations = {}",
            evolution_result.best_fitness,
            evolution_result.generations
        );

        Ok(evolution_result)
    }

    /// Get current evolution state from all components
    pub async fn get_evolution_state(&self) -> Result<EvolutionState, EvolutionError> {
        let meta_learning_state = self.meta_learning.get_state().await?;
        let population_state = self.population_manager.get_state().await?;
        let adaptive_state = self.adaptive_strategy.get_state().await?;
        let hyper_heuristic_state = self.hyper_heuristics.get_state().await?;
        let fitness_state = self.fitness_analyzer.get_state().await?;

        Ok(EvolutionState {
            meta_learning: meta_learning_state,
            population: population_state,
            adaptive: adaptive_state,
            hyper_heuristics: hyper_heuristic_state,
            fitness_analysis: fitness_state,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Optimize evolution parameters based on historical performance
    pub async fn optimize_parameters(
        &mut self,
        historical_results: Vec<EvolutionResult>,
    ) -> Result<OptimizedParameters, EvolutionError> {
        tracing::info!("Optimizing evolution parameters based on {} historical results", historical_results.len());

        // Analyze historical performance patterns
        let performance_analysis = self.meta_learning
            .analyze_historical_performance(historical_results.clone())
            .await?;

        // Optimize population parameters
        let population_params = self.population_manager
            .optimize_parameters(&performance_analysis)
            .await?;

        // Optimize adaptive strategy parameters
        let strategy_params = self.adaptive_strategy
            .optimize_parameters(&performance_analysis)
            .await?;

        // Optimize hyper-heuristic parameters
        let heuristic_params = self.hyper_heuristics
            .optimize_parameters(&performance_analysis)
            .await?;

        let optimized = OptimizedParameters {
            population: population_params,
            adaptive_strategy: strategy_params,
            hyper_heuristics: heuristic_params,
            performance_analysis,
            timestamp: chrono::Utc::now(),
        };

        tracing::info!("Parameter optimization completed");
        Ok(optimized)
    }

    /// Get service health status
    pub async fn health_check(&self) -> Result<ServiceHealth, EvolutionError> {
        let meta_learning_health = self.meta_learning.health_check().await?;
        let population_health = self.population_manager.health_check().await?;
        let adaptive_health = self.adaptive_strategy.health_check().await?;
        let hyper_heuristic_health = self.hyper_heuristics.health_check().await?;
        let fitness_health = self.fitness_analyzer.health_check().await?;
        let integration_health = self.integration_hub.health_check().await?;

        let components = vec![
            meta_learning_health,
            population_health,
            adaptive_health,
            hyper_heuristic_health,
            fitness_health,
            integration_health,
        ];

        let overall_status = if components.iter().all(|c| c.status == ServiceStatus::Healthy) {
            ServiceStatus::Healthy
        } else if components.iter().any(|c| c.status == ServiceStatus::Unhealthy) {
            ServiceStatus::Unhealthy
        } else {
            ServiceStatus::Degraded
        };

        Ok(ServiceHealth {
            service: "layer6-evolution".to_string(),
            status: overall_status,
            components,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Trigger a comprehensive evolution analysis
    pub async fn trigger_comprehensive_analysis(
        &mut self,
        test_problems: Vec<TestProblem>,
    ) -> Result<ComprehensiveAnalysis, EvolutionError> {
        tracing::info!("Starting comprehensive evolution analysis with {} test problems", test_problems.len());

        let mut results = Vec::new();

        for problem in test_problems {
            // Create initial population for this problem
            let initial_population = self.population_manager
                .create_initial_population(&problem)
                .await?;

            // Run evolution analysis
            let analysis_result = self.fitness_analyzer
                .comprehensive_analysis(&initial_population, problem.fitness_function.clone())
                .await?;

            results.push(ProblemAnalysisResult {
                problem_id: problem.id,
                landscape_analysis: analysis_result.landscape,
                algorithm_recommendations: analysis_result.recommendations,
                expected_performance: analysis_result.expected_performance,
            });
        }

        // Generate comprehensive report
        let comprehensive = self.meta_learning
            .generate_comprehensive_report(results.clone())
            .await?;

        // Update algorithm portfolio based on analysis
        self.meta_learning
            .update_portfolio_from_analysis(&comprehensive)
            .await?;

        tracing::info!("Comprehensive evolution analysis completed");
        Ok(comprehensive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_evolution_service_creation() {
        let config = EvolutionConfig::default();
        let service = AdvancedEvolutionService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = EvolutionConfig::default();
        let service = AdvancedEvolutionService::new(config).await.unwrap();
        let health = service.health_check().await.unwrap();
        assert_eq!(health.service, "layer6-evolution");
    }

    #[tokio::test]
    async fn test_evolution_state() {
        let config = EvolutionConfig::default();
        let service = AdvancedEvolutionService::new(config).await.unwrap();
        let state = service.get_evolution_state().await.unwrap();
        assert!(state.timestamp <= chrono::Utc::now());
    }
}