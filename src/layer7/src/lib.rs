//! # Chimera Layer 7 - Evolution
//!
//! Layer 7 serves as the agent genome evolution and adaptation engine for Project Chimera.
//! It receives optimization feedback from Layer 5 (Refinement) and evolves agent genomes
//! through genetic algorithms, enabling autonomous agent improvement and adaptation.

pub mod types;
pub mod genome_manager;
pub mod evolution_engine;
pub mod genetic_operators;
pub mod fitness_evaluator;
pub mod integration;
pub mod evolution_pipeline;

// Re-export main types for convenience
pub use types::*;
pub use genome_manager::*;
pub use evolution_engine::*;
pub use genetic_operators::*;
pub use fitness_evaluator::*;
pub use integration::*;
pub use evolution_pipeline::*;

/// Initialize the Layer 7 evolution system with configuration
pub async fn init_layer7(config: Layer7Config) -> Result<Layer7System, Layer7Error> {
    // Initialize logging
    tracing_subscriber::init();

    // Initialize genome manager
    let genome_manager = GenomeManager::new(config.genome_config).await?;

    // Initialize evolution engine
    let evolution_engine = EvolutionEngine::new(config.evolution_config).await?;

    // Initialize fitness evaluator
    let fitness_evaluator = FitnessEvaluator::new(config.fitness_config).await?;

    // Initialize genetic operators
    let genetic_operators = GeneticOperators::new(config.operators_config).await?;

    // Initialize integrations
    let integration_manager = IntegrationManager::new(config.integration_config).await?;

    // Initialize evolution pipeline
    let evolution_pipeline = EvolutionPipeline::new(
        genome_manager.clone(),
        evolution_engine.clone(),
        fitness_evaluator.clone(),
        genetic_operators.clone(),
        integration_manager.clone(),
    ).await?;

    Ok(Layer7System {
        genome_manager,
        evolution_engine,
        fitness_evaluator,
        genetic_operators,
        integration_manager,
        evolution_pipeline,
    })
}

/// Main Layer 7 evolution system
pub struct Layer7System {
    pub genome_manager: GenomeManager,
    pub evolution_engine: EvolutionEngine,
    pub fitness_evaluator: FitnessEvaluator,
    pub genetic_operators: GeneticOperators,
    pub integration_manager: IntegrationManager,
    pub evolution_pipeline: EvolutionPipeline,
}

impl Layer7System {
    /// Start the evolution system
    pub async fn start(&self) -> Result<(), Layer7Error> {
        info!("Starting Layer 7 Evolution System");

        // Start integration listeners
        self.integration_manager.start_listeners().await?;

        // Start evolution pipeline
        self.evolution_pipeline.start().await?;

        info!("Layer 7 Evolution System started successfully");
        Ok(())
    }

    /// Shutdown the evolution system
    pub async fn shutdown(&self) -> Result<(), Layer7Error> {
        info!("Shutting down Layer 7 Evolution System");

        // Stop evolution pipeline
        self.evolution_pipeline.stop().await?;

        // Stop integration listeners
        self.integration_manager.stop_listeners().await?;

        info!("Layer 7 Evolution System shutdown complete");
        Ok(())
    }

    /// Process optimization feedback from Layer 5
    pub async fn process_optimization_feedback(&self, feedback: OptimizationFeedback) -> Result<EvolutionResult, Layer7Error> {
        self.evolution_pipeline.process_feedback(feedback).await
    }

    /// Deploy evolved genome to Layer 4
    pub async fn deploy_genome(&self, agent_id: AgentId, genome: AgentGenome) -> Result<(), Layer7Error> {
        self.integration_manager.deploy_to_layer4(agent_id, genome).await
    }

    /// Request resources from Layer 8 for evolution
    pub async fn request_evolution_resources(&self, requirements: ResourceRequirements) -> Result<ResourceAllocation, Layer7Error> {
        self.integration_manager.request_from_layer8(requirements).await
    }
}