//! # Layer 6 Unit Tests
//!
//! Comprehensive unit tests for all Layer 6 (Evolution) components.

use layer6_evolution::*;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

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
        assert!(matches!(health.status, ServiceStatus::Healthy | ServiceStatus::Degraded));
    }

    #[tokio::test]
    async fn test_evolution_state() {
        let config = EvolutionConfig::default();
        let service = AdvancedEvolutionService::new(config).await.unwrap();
        let state = service.get_evolution_state().await.unwrap();
        assert!(state.timestamp <= chrono::Utc::now());
    }

    #[test]
    fn test_evolution_config_default() {
        let config = EvolutionConfig::default();
        assert_eq!(config.meta_learning.portfolio_size, 10);
        assert_eq!(config.population.default_size, 100);
        assert_eq!(config.adaptive.adaptation_rate, 0.1);
        assert_eq!(config.hyper_heuristics.max_portfolio_size, 20);
        assert_eq!(config.fitness.sample_size, 1000);
        assert_eq!(config.integration.layer7_timeout_seconds, 30);
    }

    #[test]
    fn test_individual_creation() {
        let individual = Individual {
            id: "test-individual".to_string(),
            genome: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            fitness: 0.95,
            objective_values: vec![0.95, 0.8, 0.7],
            age: 10,
            parents: Some(("parent1".to_string(), "parent2".to_string())),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("generation".to_string(), "10".to_string());
                metadata.insert("algorithm".to_string(), "test-algorithm".to_string());
                metadata
            },
            created_at: Utc::now(),
        };

        assert_eq!(individual.id, "test-individual");
        assert_eq!(individual.genome.len(), 5);
        assert_eq!(individual.fitness, 0.95);
        assert_eq!(individual.objective_values.len(), 3);
        assert_eq!(individual.age, 10);
        assert!(individual.parents.is_some());
        assert_eq!(individual.metadata.len(), 2);
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
            Individual {
                id: "ind3".to_string(),
                genome: vec![1.5, 2.5],
                fitness: 0.85,
                objective_values: vec![0.85],
                age: 1,
                parents: None,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
        ];

        let population = Population::new("test-population".to_string(), individuals);

        assert_eq!(population.id, "test-population");
        assert_eq!(population.size(), 3);
        assert_eq!(population.generation, 0);
        assert!(population.best_individual().unwrap().fitness > 0.8);
        assert!(population.worst_individual().unwrap().fitness < 0.9);
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
        assert!(result.statistics.converged);
        assert!(result.statistics.final_diversity > 0.0);
    }

    #[test]
    fn test_algorithm_capabilities() {
        let capabilities = AlgorithmCapabilities {
            multi_objective: true,
            constraint_handling: true,
            large_population: true,
            high_dimensional: true,
            noisy_fitness: false,
            parallel_processing: true,
        };

        assert!(capabilities.multi_objective);
        assert!(capabilities.constraint_handling);
        assert!(capabilities.large_population);
        assert!(capabilities.high_dimensional);
        assert!(!capabilities.noisy_fitness);
        assert!(capabilities.parallel_processing);
    }

    #[test]
    fn test_problem_characteristics() {
        let characteristics = ProblemCharacteristics {
            dimensionality: 50,
            problem_type: ProblemType::Continuous,
            landscape: FitnessLandscapeType::Multimodal,
            multi_objective: true,
            constraints: vec![ConstraintType::Boundary, ConstraintType::Equality],
            expected_population_size: 200,
        };

        assert_eq!(characteristics.dimensionality, 50);
        assert_eq!(characteristics.problem_type, ProblemType::Continuous);
        assert_eq!(characteristics.landscape, FitnessLandscapeType::Multimodal);
        assert!(characteristics.multi_objective);
        assert_eq!(characteristics.constraints.len(), 2);
        assert_eq!(characteristics.expected_population_size, 200);
    }

    #[test]
    fn test_fitness_landscape_types() {
        assert_eq!(FitnessLandscapeType::Unimodal, FitnessLandscapeType::Unimodal);
        assert_eq!(FitnessLandscapeType::Multimodal, FitnessLandscapeType::Multimodal);
        assert_eq!(FitnessLandscapeType::Deceptive, FitnessLandscapeType::Deceptive);
        assert_eq!(FitnessLandscapeType::Rugged, FitnessLandscapeType::Rugged);
        assert_eq!(FitnessLandscapeType::Neutral, FitnessLandscapeType::Neutral);
        assert_eq!(FitnessLandscapeType::Unknown, FitnessLandscapeType::Unknown);
    }

    #[test]
    fn test_problem_types() {
        assert_eq!(ProblemType::Continuous, ProblemType::Continuous);
        assert_eq!(ProblemType::Discrete, ProblemType::Discrete);
        assert_eq!(ProblemType::Mixed, ProblemType::Mixed);
        assert_eq!(ProblemType::Combinatorial, ProblemType::Combinatorial);
        assert_eq!(ProblemType::Dynamic, ProblemType::Dynamic);
    }

    #[test]
    fn test_constraint_types() {
        assert_eq!(ConstraintType::Equality, ConstraintType::Equality);
        assert_eq!(ConstraintType::Inequality, ConstraintType::Inequality);
        assert_eq!(ConstraintType::Boundary, ConstraintType::Boundary);
    }

    #[test]
    fn test_heuristic_types() {
        assert_eq!(HeuristicType::Selection, HeuristicType::Selection);
        assert_eq!(HeuristicType::Crossover, HeuristicType::Crossover);
        assert_eq!(HeuristicType::Mutation, HeuristicType::Mutation);
        assert_eq!(HeuristicType::Replacement, HeuristicType::Replacement);
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

    #[test]
    fn test_adaptation_status() {
        assert_eq!(AdaptationStatus::Stable, AdaptationStatus::Stable);
        assert_eq!(AdaptationStatus::Adapting, AdaptationStatus::Adapting);
        assert_eq!(AdaptationStatus::Converging, AdaptationStatus::Converging);
        assert_eq!(AdaptationStatus::Diverging, AdaptationStatus::Diverging);
        assert_eq!(AdaptationStatus::Unknown, AdaptationStatus::Unknown);
    }

    #[test]
    fn test_switch_reasons() {
        assert_eq!(SwitchReason::Performance, SwitchReason::Performance);
        assert_eq!(SwitchReason::Convergence, SwitchReason::Convergence);
        assert_eq!(SwitchReason::Stagnation, SwitchReason::Stagnation);
        assert_eq!(SwitchReason::ResourceEfficiency, SwitchReason::ResourceEfficiency);
        assert_eq!(SwitchReason::ProblemChange, SwitchReason::ProblemChange);
    }

    #[test]
    fn test_trend_directions() {
        assert_eq!(TrendDirection::Increasing, TrendDirection::Increasing);
        assert_eq!(TrendDirection::Decreasing, TrendDirection::Decreasing);
        assert_eq!(TrendDirection::Stable, TrendDirection::Stable);
        assert_eq!(TrendDirection::Volatile, TrendDirection::Volatile);
    }

    #[test]
    fn test_priority_levels() {
        assert_eq!(Priority::Low, Priority::Low);
        assert_eq!(Priority::Medium, Priority::Medium);
        assert_eq!(Priority::High, Priority::High);
        assert_eq!(Priority::Critical, Priority::Critical);
    }

    #[test]
    fn test_improvement_types() {
        assert_eq!(ImprovementType::ParameterTuning, ImprovementType::ParameterTuning);
        assert_eq!(ImprovementType::AlgorithmEnhancement, ImprovementType::AlgorithmEnhancement);
        assert_eq!(ImprovementType::IntegrationImprovement, ImprovementType::IntegrationImprovement);
        assert_eq!(ImprovementType::PerformanceOptimization, ImprovementType::PerformanceOptimization);
    }

    #[test]
    fn test_complexity_levels() {
        assert_eq!(Complexity::Low, Complexity::Low);
        assert_eq!(Complexity::Medium, Complexity::Medium);
        assert_eq!(Complexity::High, Complexity::High);
        assert_eq!(Complexity::VeryHigh, Complexity::VeryHigh);
    }

    #[test]
    fn test_bottleneck_types() {
        assert_eq!(BottleneckType::Computational, BottleneckType::Computational);
        assert_eq!(BottleneckType::Memory, BottleneckType::Memory);
        assert_eq!(BottleneckType::Communication, BottleneckType::Communication);
        assert_eq!(BottleneckType::Algorithmic, BottleneckType::Algorithmic);
        assert_eq!(BottleneckType::Integration, BottleneckType::Integration);
    }

    #[test]
    fn test_opportunity_types() {
        assert_eq!(OpportunityType::AlgorithmImprovement, OpportunityType::AlgorithmImprovement);
        assert_eq!(OpportunityType::ParameterOptimization, OpportunityType::ParameterOptimization);
        assert_eq!(OpportunityType::ResourceOptimization, OpportunityType::ResourceOptimization);
        assert_eq!(OpportunityType::IntegrationEnhancement, OpportunityType::IntegrationEnhancement);
    }

    #[test]
    fn test_schedule_types() {
        assert_eq!(ScheduleType::Linear, ScheduleType::Linear);
        assert_eq!(ScheduleType::Exponential, ScheduleType::Exponential);
        assert_eq!(ScheduleType::Logarithmic, ScheduleType::Logarithmic);
        assert_eq!(ScheduleType::Step, ScheduleType::Step);
    }

    #[test]
    fn test_evolution_error_types() {
        let meta_error = EvolutionError::MetaLearningError("Test error".to_string());
        let population_error = EvolutionError::PopulationError("Test error".to_string());
        let adaptive_error = EvolutionError::AdaptiveError("Test error".to_string());
        let hyper_error = EvolutionError::HyperHeuristicError("Test error".to_string());
        let fitness_error = EvolutionError::FitnessAnalysisError("Test error".to_string());
        let integration_error = EvolutionError::IntegrationError("Test error".to_string());
        let algorithm_error = EvolutionError::AlgorithmError("Test error".to_string());
        let config_error = EvolutionError::ConfigurationError("Test error".to_string());
        let resource_error = EvolutionError::ResourceError("Test error".to_string());
        let convergence_error = EvolutionError::ConvergenceError("Test error".to_string());
        let validation_error = EvolutionError::ValidationError("Test error".to_string());
        let internal_error = EvolutionError::InternalError("Test error".to_string());

        // Test that we can match on different error types
        match meta_error {
            EvolutionError::MetaLearningError(_) => assert!(true),
            _ => assert!(false),
        }

        match population_error {
            EvolutionError::PopulationError(_) => assert!(true),
            _ => assert!(false),
        }

        match adaptive_error {
            EvolutionError::AdaptiveError(_) => assert!(true),
            _ => assert!(false),
        }

        match hyper_error {
            EvolutionError::HyperHeuristicError(_) => assert!(true),
            _ => assert!(false),
        }

        match fitness_error {
            EvolutionError::FitnessAnalysisError(_) => assert!(true),
            _ => assert!(false),
        }

        match integration_error {
            EvolutionError::IntegrationError(_) => assert!(true),
            _ => assert!(false),
        }

        match algorithm_error {
            EvolutionError::AlgorithmError(_) => assert!(true),
            _ => assert!(false),
        }

        match config_error {
            EvolutionError::ConfigurationError(_) => assert!(true),
            _ => assert!(false),
        }

        match resource_error {
            EvolutionError::ResourceError(_) => assert!(true),
            _ => assert!(false),
        }

        match convergence_error {
            EvolutionError::ConvergenceError(_) => assert!(true),
            _ => assert!(false),
        }

        match validation_error {
            EvolutionError::ValidationError(_) => assert!(true),
            _ => assert!(false),
        }

        match internal_error {
            EvolutionError::InternalError(_) => assert!(true),
            _ => assert!(false),
        }
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

        let e = vec![0.0, 0.0];
        let f = vec![1.0, 0.0];
        let distance3 = euclidean_distance(&e, &f);
        assert!((distance3 - 1.0).abs() < 0.001); // Unit distance
    }

    #[test]
    fn test_calculate_problem_id() {
        let characteristics = ProblemCharacteristics {
            dimensionality: 10,
            problem_type: ProblemType::Continuous,
            landscape: FitnessLandscapeType::Multimodal,
            multi_objective: false,
            constraints: Vec::new(),
            expected_population_size: 100,
        };

        let problem_id = calculate_problem_id(&characteristics);
        assert!(problem_id.contains("10"));
        assert!(problem_id.contains("Continuous"));
        assert!(problem_id.contains("Multimodal"));
        assert!(problem_id.contains("false"));
    }

    #[test]
    fn test_success_rate_calculation() {
        assert_eq!(EvolutionMetricsUtils::calculate_success_rate(80, 100), 0.8);
        assert_eq!(EvolutionMetricsUtils::calculate_success_rate(0, 100), 0.0);
        assert_eq!(EvolutionMetricsUtils::calculate_success_rate(100, 0), 0.0);
        assert_eq!(EvolutionMetricsUtils::calculate_success_rate(0, 0), 0.0);
    }

    #[test]
    fn test_convergence_efficiency_calculation() {
        let efficiency = EvolutionMetricsUtils::calculate_convergence_efficiency(90, 100, 50.0);
        assert!(efficiency > 0.8);

        let efficiency2 = EvolutionMetricsUtils::calculate_convergence_efficiency(10, 100, 1000.0);
        assert!(efficiency2 < 0.5);
    }

    #[test]
    fn test_meta_learning_effectiveness_calculation() {
        let effectiveness = EvolutionMetricsUtils::calculate_meta_learning_effectiveness(0.9, 0.8, 0.7);
        assert!((effectiveness - 0.82).abs() < 0.01);

        let effectiveness2 = EvolutionMetricsUtils::calculate_meta_learning_effectiveness(0.5, 0.3, 0.2);
        assert!(effectiveness2 < 0.4);
    }

    #[test]
    fn test_evolution_efficiency_calculation() {
        let efficiency = EvolutionMetricsUtils::calculate_evolution_efficiency(0.9, 50.0, 0.8, 0.95);
        assert!(efficiency > 0.8);

        let efficiency2 = EvolutionMetricsUtils::calculate_evolution_efficiency(0.5, 1000.0, 0.3, 0.4);
        assert!(efficiency2 < 0.5);
    }

    #[test]
    fn test_metrics_formatting() {
        let mut metrics = HashMap::new();
        metrics.insert("evolution_runs".to_string(), 100.0);
        metrics.insert("success_rate".to_string(), 0.85);
        metrics.insert("avg_convergence_time".to_string(), 125.5);
        metrics.insert("diversity".to_string(), 0.7);

        let formatted = EvolutionMetricsUtils::format_evolution_metrics_for_log(&metrics);
        assert!(formatted.contains("evolution_runs=100"));
        assert!(formatted.contains("success_rate=0.85"));
        assert!(formatted.contains("avg_convergence_time=125.5"));
        assert!(formatted.contains("diversity=0.7"));
    }

    #[test]
    fn test_landscape_characteristics() {
        let characteristics = LandscapeCharacteristics {
            modality: 3.5,
            global_structure: GlobalStructure {
                global_correlation: 0.7,
                fitness_distance_correlation: 0.6,
                epistasis: 0.2,
                ruggedness: 0.4,
            },
            local_structure: LocalStructure {
                local_optima_density: 0.15,
                basin_sizes: vec![10.0, 5.0, 2.0],
                gradient_info: GradientInfo {
                    avg_gradient_magnitude: 0.5,
                    gradient_variance: 0.1,
                    direction_consistency: 0.8,
                },
                neighborhood_structure: NeighborhoodStructure {
                    avg_neighborhood_fitness: 0.6,
                    neighborhood_diversity: 0.4,
                    connectivity: 0.9,
                },
            },
            deceptiveness: 0.2,
            neutrality: 0.1,
        };

        assert_eq!(characteristics.modality, 3.5);
        assert_eq!(characteristics.global_structure.global_correlation, 0.7);
        assert_eq!(characteristics.local_structure.local_optima_density, 0.15);
        assert_eq!(characteristics.deceptiveness, 0.2);
        assert_eq!(characteristics.neutrality, 0.1);
    }

    #[test]
    fn test_expected_performance() {
        let performance = ExpectedPerformance {
            success_rate: 0.9,
            convergence_time: 250.0,
            solution_quality: 0.95,
            confidence_interval: (0.85, 1.05),
        };

        assert_eq!(performance.success_rate, 0.9);
        assert_eq!(performance.convergence_time, 250.0);
        assert_eq!(performance.solution_quality, 0.95);
        assert_eq!(performance.confidence_interval.0, 0.85);
        assert_eq!(performance.confidence_interval.1, 1.05);
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
        assert_eq!(request.expected_duration_minutes, 60);
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
    fn test_evolution_data_types() {
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

        // Test that we can match on evolution data types
        match evolution_result {
            EvolutionData::EvolutionResult(_) => assert!(true),
            _ => assert!(false),
        }

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

        match resource_request {
            EvolutionData::ResourceRequest(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_service_status() {
        assert_eq!(ServiceStatus::Healthy, ServiceStatus::Healthy);
        assert_eq!(ServiceStatus::Degraded, ServiceStatus::Degraded);
        assert_eq!(ServiceStatus::Unhealthy, ServiceStatus::Unhealthy);
        assert_eq!(ServiceStatus::Starting, ServiceStatus::Starting);
        assert_eq!(ServiceStatus::Stopping, ServiceStatus::Stopping);
    }

    #[test]
    fn test_component_health() {
        let health = ComponentHealth {
            name: "test-component".to_string(),
            status: ServiceStatus::Healthy,
            check_duration_ms: 150,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("test_metric".to_string(), 42.0);
                metrics
            },
        };

        assert_eq!(health.name, "test-component");
        assert_eq!(health.status, ServiceStatus::Healthy);
        assert_eq!(health.check_duration_ms, 150);
        assert!(health.error_message.is_none());
        assert_eq!(health.metrics.len(), 1);
    }

    #[test]
    fn test_service_health() {
        let health = ServiceHealth {
            service: "test-service".to_string(),
            status: ServiceStatus::Healthy,
            components: vec![
                ComponentHealth {
                    name: "component1".to_string(),
                    status: ServiceStatus::Healthy,
                    check_duration_ms: 100,
                    error_message: None,
                    metrics: HashMap::new(),
                },
                ComponentHealth {
                    name: "component2".to_string(),
                    status: ServiceStatus::Healthy,
                    check_duration_ms: 200,
                    error_message: None,
                    metrics: HashMap::new(),
                },
            ],
            timestamp: Utc::now(),
        };

        assert_eq!(health.service, "test-service");
        assert_eq!(health.status, ServiceStatus::Healthy);
        assert_eq!(health.components.len(), 2);
    }
}