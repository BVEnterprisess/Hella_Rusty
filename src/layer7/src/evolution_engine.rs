//! Evolution Engine for Layer 7 - Core Genetic Algorithm Implementation

use crate::types::*;
use crate::genome_manager::GenomeManager;
use crate::genetic_operators::GeneticOperators;
use std::collections::HashMap;
use tracing::{info, debug};

/// Evolution Engine manages populations and genetic algorithm execution
pub struct EvolutionEngine {
    populations: HashMap<EvolutionExperimentId, EvolutionPopulation>,
    genetic_operators: GeneticOperators,
    config: EvolutionConfig,
}

impl EvolutionEngine {
    /// Create a new evolution engine
    pub async fn new(config: EvolutionConfig) -> Result<Self, EvolutionError> {
        let genetic_operators = GeneticOperators::new(GeneticOperatorConfig {
            selection_method: SelectionMethod::Tournament(3),
            crossover_method: CrossoverMethod::SinglePoint,
            mutation_method: MutationMethod::Gaussian(0.1),
            crossover_rate: 0.8,
            mutation_rate: 0.1,
        }).await?;

        Ok(Self {
            populations: HashMap::new(),
            genetic_operators,
            config,
        })
    }

    /// Initialize a new population for an experiment
    pub async fn initialize_population(&mut self, experiment_id: EvolutionExperimentId) -> Result<EvolutionPopulation, EvolutionError> {
        info!("Initializing population for experiment {}", experiment_id);

        // Create initial random genomes (placeholder implementation)
        let mut genomes = Vec::new();
        let mut fitness_scores = HashMap::new();

        for i in 0..50 { // Population size
            let genome = AgentGenome {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(), // Would be set based on experiment target agents
                version: 1,
                neural_weights: self.generate_random_weights(1000), // Example size
                hyperparameters: self.generate_random_hyperparameters(),
                architecture: NetworkArchitecture {
                    layers: vec![
                        LayerConfig {
                            layer_type: LayerType::Dense,
                            size: 512,
                            activation: "relu".to_string(),
                        },
                        LayerConfig {
                            layer_type: LayerType::Dense,
                            size: 256,
                            activation: "relu".to_string(),
                        },
                        LayerConfig {
                            layer_type: LayerType::Dense,
                            size: 128,
                            activation: "softmax".to_string(),
                        },
                    ],
                    activation_functions: vec!["relu".to_string(), "softmax".to_string()],
                    input_size: 784,
                    output_size: 10,
                },
                metadata: GenomeMetadata {
                    fitness_score: 0.0,
                    generation: 0,
                    mutation_rate: 0.01,
                    crossover_method: "initial".to_string(),
                    training_data_hash: "".to_string(),
                    validation_accuracy: 0.0,
                },
                created_at: Utc::now(),
                parent_genomes: Vec::new(),
            };

            genomes.push(genome);
            fitness_scores.insert(genome.agent_id, 0.0); // Initial fitness
        }

        let population = EvolutionPopulation {
            id: Uuid::new_v4(),
            generation: 0,
            genomes,
            fitness_scores,
            diversity_metrics: DiversityMetrics {
                genetic_diversity: 0.8,
                phenotypic_diversity: 0.7,
                fitness_variance: 0.1,
                population_entropy: 4.2,
            },
            created_at: Utc::now(),
            target_improvement: 0.05,
        };

        self.populations.insert(experiment_id, population.clone());
        info!("Initialized population with {} genomes for experiment {}", population.genomes.len(), experiment_id);

        Ok(population)
    }

    /// Create next generation from current population
    pub async fn create_next_generation(
        &self,
        population: EvolutionPopulation,
        fitness_scores: HashMap<AgentId, f64>,
    ) -> Result<EvolutionPopulation, EvolutionError> {
        info!("Creating next generation for population (gen {})", population.generation + 1);

        // Select parents using genetic operators
        let parents = self.genetic_operators.select_parents(&population).await?;

        // Create offspring through crossover and mutation
        let mut offspring = Vec::new();
        let mut new_fitness_scores = HashMap::new();

        for i in 0..parents.len()/2 {
            let parent1 = &parents[i*2];
            let parent2 = &parents[i*2 + 1];

            // Perform crossover
            let (child1, child2) = self.genetic_operators.crossover(parent1, parent2).await?;

            // Mutate offspring
            let mut_child1 = self.genetic_operators.mutate(&child1).await?;
            let mut_child2 = self.genetic_operators.mutate(&child2).await?;

            offspring.push(mut_child1);
            offspring.push(mut_child2);

            // Assign initial fitness scores (will be evaluated)
            new_fitness_scores.insert(child1.agent_id, 0.0);
            new_fitness_scores.insert(child2.agent_id, 0.0);
        }

        // Calculate diversity metrics
        let diversity_metrics = self.calculate_diversity_metrics(&offspring, &new_fitness_scores)?;

        let next_population = EvolutionPopulation {
            id: Uuid::new_v4(),
            generation: population.generation + 1,
            genomes: offspring,
            fitness_scores: new_fitness_scores,
            diversity_metrics,
            created_at: Utc::now(),
            target_improvement: population.target_improvement,
        };

        info!("Created next generation with {} genomes", next_population.genomes.len());
        Ok(next_population)
    }

    /// Generate random neural network weights
    fn generate_random_weights(&self, size: usize) -> Vec<f32> {
        use rand::prelude::*;
        let mut rng = thread_rng();

        (0..size).map(|_| rng.gen::<f32>() * 2.0 - 1.0).collect() // Random weights between -1 and 1
    }

    /// Generate random hyperparameters
    fn generate_random_hyperparameters(&self) -> HashMap<String, f64> {
        use rand::prelude::*;
        let mut rng = thread_rng();

        HashMap::from([
            ("learning_rate".to_string(), rng.gen::<f64>() * 0.1), // 0.0 to 0.1
            ("momentum".to_string(), rng.gen::<f64>()), // 0.0 to 1.0
            ("dropout_rate".to_string(), rng.gen::<f64>() * 0.5), // 0.0 to 0.5
            ("batch_size".to_string(), (rng.gen::<f64>() * 100.0 + 16.0).round()), // 16 to 116
        ])
    }

    /// Calculate diversity metrics for a population
    fn calculate_diversity_metrics(
        &self,
        genomes: &[AgentGenome],
        fitness_scores: &HashMap<AgentId, f64>,
    ) -> Result<DiversityMetrics, EvolutionError> {
        if genomes.is_empty() {
            return Ok(DiversityMetrics {
                genetic_diversity: 0.0,
                phenotypic_diversity: 0.0,
                fitness_variance: 0.0,
                population_entropy: 0.0,
            });
        }

        // Calculate genetic diversity (simplified)
        let genetic_diversity = self.calculate_genetic_diversity(genomes);

        // Calculate phenotypic diversity (based on fitness)
        let fitness_values: Vec<f64> = fitness_scores.values().cloned().collect();
        let phenotypic_diversity = if fitness_values.len() > 1 {
            let mean = fitness_values.iter().sum::<f64>() / fitness_values.len() as f64;
            let variance = fitness_values.iter()
                .map(|f| (f - mean).powi(2))
                .sum::<f64>() / fitness_values.len() as f64;
            variance.sqrt() / mean.max(1.0) // Normalized standard deviation
        } else {
            0.0
        };

        // Calculate fitness variance
        let fitness_variance = if fitness_values.len() > 1 {
            let mean = fitness_values.iter().sum::<f64>() / fitness_values.len() as f64;
            fitness_values.iter()
                .map(|f| (f - mean).powi(2))
                .sum::<f64>() / (fitness_values.len() - 1) as f64
        } else {
            0.0
        };

        // Calculate population entropy (simplified)
        let population_entropy = (genomes.len() as f64).ln().max(1.0);

        Ok(DiversityMetrics {
            genetic_diversity,
            phenotypic_diversity,
            fitness_variance,
            population_entropy,
        })
    }

    /// Calculate genetic diversity between genomes
    fn calculate_genetic_diversity(&self, genomes: &[AgentGenome]) -> f64 {
        if genomes.len() < 2 {
            return 0.0;
        }

        let mut total_distance = 0.0;
        let mut comparisons = 0;

        for i in 0..genomes.len() {
            for j in i+1..genomes.len() {
                let distance = self.calculate_genome_distance(&genomes[i], &genomes[j]);
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

    /// Calculate distance between two genomes
    fn calculate_genome_distance(&self, genome1: &AgentGenome, genome2: &AgentGenome) -> f64 {
        if genome1.neural_weights.len() != genome2.neural_weights.len() {
            return 1.0; // Maximum distance for different architectures
        }

        let mut total_diff = 0.0;
        for i in 0..genome1.neural_weights.len() {
            let diff = (genome1.neural_weights[i] - genome2.neural_weights[i]).abs();
            total_diff += diff;
        }

        // Normalize by genome size
        total_diff / genome1.neural_weights.len() as f64
    }

    /// Get population for an experiment
    pub fn get_population(&self, experiment_id: EvolutionExperimentId) -> Option<&EvolutionPopulation> {
        self.populations.get(&experiment_id)
    }

    /// Update population for an experiment
    pub fn update_population(&mut self, experiment_id: EvolutionExperimentId, population: EvolutionPopulation) {
        self.populations.insert(experiment_id, population);
    }

    /// Get evolution statistics
    pub fn get_evolution_stats(&self) -> HashMap<EvolutionExperimentId, EvolutionStats> {
        let mut stats = HashMap::new();

        for (experiment_id, population) in &self.populations {
            let max_fitness = population.fitness_scores.values().cloned().fold(0.0, f64::max);
            let avg_fitness = population.fitness_scores.values().sum::<f64>() / population.fitness_scores.len() as f64;
            let min_fitness = population.fitness_scores.values().cloned().fold(f64::INFINITY, f64::min);

            stats.insert(*experiment_id, EvolutionStats {
                experiment_id: *experiment_id,
                generation: population.generation,
                population_size: population.genomes.len(),
                max_fitness,
                avg_fitness,
                min_fitness,
                genetic_diversity: population.diversity_metrics.genetic_diversity,
                phenotypic_diversity: population.diversity_metrics.phenotypic_diversity,
                fitness_variance: population.diversity_metrics.fitness_variance,
                convergence_rate: self.calculate_convergence_rate(population),
            });
        }

        stats
    }

    /// Calculate convergence rate for a population
    fn calculate_convergence_rate(&self, population: &EvolutionPopulation) -> f64 {
        // Simple convergence metric based on fitness variance
        let fitness_values: Vec<f64> = population.fitness_scores.values().cloned().collect();

        if fitness_values.len() < 2 {
            return 0.0;
        }

        let mean = fitness_values.iter().sum::<f64>() / fitness_values.len() as f64;
        let variance = fitness_values.iter()
            .map(|f| (f - mean).powi(2))
            .sum::<f64>() / fitness_values.len() as f64;

        // Lower variance means higher convergence
        1.0 / (1.0 + variance)
    }
}

/// Evolution statistics for monitoring
#[derive(Debug, Clone)]
pub struct EvolutionStats {
    pub experiment_id: EvolutionExperimentId,
    pub generation: u64,
    pub population_size: usize,
    pub max_fitness: f64,
    pub avg_fitness: f64,
    pub min_fitness: f64,
    pub genetic_diversity: f64,
    pub phenotypic_diversity: f64,
    pub fitness_variance: f64,
    pub convergence_rate: f64,
}