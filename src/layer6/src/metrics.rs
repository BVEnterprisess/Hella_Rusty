//! # Evolution Metrics Collection
//!
//! This module provides comprehensive metrics collection and monitoring capabilities
//! for Layer 6 (Evolution). It tracks performance, convergence, diversity, and
//! operational metrics for all evolution components.

use crate::types::*;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_gauge, register_histogram, Counter, Encoder, Gauge, Histogram,
    Registry, TextEncoder,
};
use std::collections::HashMap;
use tracing::{debug, error, info};

lazy_static! {
    /// Prometheus registry for Layer 6 metrics
    static ref REGISTRY: Registry = Registry::new();

    /// Counter for evolution runs started
    static ref EVOLUTION_RUNS_STARTED: Counter = register_counter!(
        "layer6_evolution_runs_started_total",
        "Total number of evolution runs started"
    ).expect("Can't create evolution_runs_started metric");

    /// Counter for evolution runs completed
    static ref EVOLUTION_RUNS_COMPLETED: Counter = register_counter!(
        "layer6_evolution_runs_completed_total",
        "Total number of evolution runs completed"
    ).expect("Can't create evolution_runs_completed metric");

    /// Counter for algorithm selections
    static ref ALGORITHM_SELECTIONS_TOTAL: Counter = register_counter!(
        "layer6_algorithm_selections_total",
        "Total number of algorithm selections made"
    ).expect("Can't create algorithm_selections_total metric");

    /// Counter for population migrations
    static ref POPULATION_MIGRATIONS_TOTAL: Counter = register_counter!(
        "layer6_population_migrations_total",
        "Total number of population migrations performed"
    ).expect("Can't create population_migrations_total metric");

    /// Counter for heuristic applications
    static ref HEURISTIC_APPLICATIONS_TOTAL: Counter = register_counter!(
        "layer6_heuristic_applications_total",
        "Total number of heuristic applications"
    ).expect("Can't create heuristic_applications_total metric");

    /// Gauge for current active populations
    static ref ACTIVE_POPULATIONS: Gauge = register_gauge!(
        "layer6_active_populations",
        "Number of currently active populations"
    ).expect("Can't create active_populations metric");

    /// Gauge for current algorithm portfolio size
    static ref ALGORITHM_PORTFOLIO_SIZE: Gauge = register_gauge!(
        "layer6_algorithm_portfolio_size",
        "Size of current algorithm portfolio"
    ).expect("Can't create algorithm_portfolio_size metric");

    /// Gauge for current heuristic portfolio size
    static ref HEURISTIC_PORTFOLIO_SIZE: Gauge = register_gauge!(
        "layer6_heuristic_portfolio_size",
        "Size of current heuristic portfolio"
    ).expect("Can't create heuristic_portfolio_size metric");

    /// Histogram for evolution run duration
    static ref EVOLUTION_DURATION_SECONDS: Histogram = register_histogram!(
        "layer6_evolution_duration_seconds",
        "Duration of evolution runs in seconds"
    ).expect("Can't create evolution_duration_seconds metric");

    /// Histogram for algorithm selection time
    static ref ALGORITHM_SELECTION_DURATION_SECONDS: Histogram = register_histogram!(
        "layer6_algorithm_selection_duration_seconds",
        "Time taken for algorithm selection in seconds"
    ).expect("Can't create algorithm_selection_duration_seconds metric");

    /// Histogram for population migration time
    static ref MIGRATION_DURATION_SECONDS: Histogram = register_histogram!(
        "layer6_migration_duration_seconds",
        "Time taken for population migration in seconds"
    ).expect("Can't create migration_duration_seconds metric");

    /// Gauge for average population diversity
    static ref AVERAGE_POPULATION_DIVERSITY: Gauge = register_gauge!(
        "layer6_average_population_diversity",
        "Average diversity across all populations"
    ).expect("Can't create average_population_diversity metric");

    /// Gauge for meta-learning progress
    static ref META_LEARNING_PROGRESS: Gauge = register_gauge!(
        "layer6_meta_learning_progress",
        "Meta-learning progress (0.0 to 1.0)"
    ).expect("Can't create meta_learning_progress metric");

    /// Counter for evolution errors
    static ref EVOLUTION_ERRORS_TOTAL: Counter = register_counter!(
        "layer6_evolution_errors_total",
        "Total number of evolution errors"
    ).expect("Can't create evolution_errors_total metric");

    /// Counter for convergence events
    static ref CONVERGENCE_EVENTS_TOTAL: Counter = register_counter!(
        "layer6_convergence_events_total",
        "Total number of convergence events"
    ).expect("Can't create convergence_events_total metric");

    /// Counter for stagnation events
    static ref STAGNATION_EVENTS_TOTAL: Counter = register_counter!(
        "layer6_stagnation_events_total",
        "Total number of stagnation events"
    ).expect("Can't create stagnation_events_total metric");
}

/// Metrics collector for Layer 6 evolution
pub struct EvolutionMetricsCollector {
    registry: Registry,
}

impl EvolutionMetricsCollector {
    /// Create a new evolution metrics collector
    pub fn new() -> Result<Self, EvolutionError> {
        Ok(Self {
            registry: REGISTRY.clone(),
        })
    }

    /// Record evolution run start
    pub fn record_evolution_start(&self, algorithm_id: &str) {
        EVOLUTION_RUNS_STARTED.inc();
        debug!("Recorded evolution run start with algorithm: {}", algorithm_id);
    }

    /// Record evolution run completion
    pub fn record_evolution_completion(&self, duration_seconds: f64, success: bool) {
        EVOLUTION_RUNS_COMPLETED.inc();
        EVOLUTION_DURATION_SECONDS.observe(duration_seconds);

        if !success {
            EVOLUTION_ERRORS_TOTAL.inc();
        }

        debug!("Recorded evolution completion: {:.3}s, success: {}", duration_seconds, success);
    }

    /// Record algorithm selection
    pub fn record_algorithm_selection(&self, duration_seconds: f64, algorithm_id: &str) {
        ALGORITHM_SELECTIONS_TOTAL.inc();
        ALGORITHM_SELECTION_DURATION_SECONDS.observe(duration_seconds);

        debug!("Recorded algorithm selection: {} in {:.3}s", algorithm_id, duration_seconds);
    }

    /// Record population migration
    pub fn record_population_migration(&self, duration_seconds: f64, individuals_migrated: u32) {
        POPULATION_MIGRATIONS_TOTAL.inc();
        MIGRATION_DURATION_SECONDS.observe(duration_seconds);

        debug!("Recorded population migration: {} individuals in {:.3}s", individuals_migrated, duration_seconds);
    }

    /// Record heuristic application
    pub fn record_heuristic_application(&self, heuristic_id: &str, heuristic_type: &HeuristicType) {
        HEURISTIC_APPLICATIONS_TOTAL.inc();

        debug!("Recorded heuristic application: {} ({:?})", heuristic_id, heuristic_type);
    }

    /// Update active populations count
    pub fn update_active_populations(&self, count: f64) {
        ACTIVE_POPULATIONS.set(count);
        debug!("Updated active populations: {}", count);
    }

    /// Update algorithm portfolio size
    pub fn update_algorithm_portfolio_size(&self, size: f64) {
        ALGORITHM_PORTFOLIO_SIZE.set(size);
        debug!("Updated algorithm portfolio size: {}", size);
    }

    /// Update heuristic portfolio size
    pub fn update_heuristic_portfolio_size(&self, size: f64) {
        HEURISTIC_PORTFOLIO_SIZE.set(size);
        debug!("Updated heuristic portfolio size: {}", size);
    }

    /// Update average population diversity
    pub fn update_average_diversity(&self, diversity: f64) {
        AVERAGE_POPULATION_DIVERSITY.set(diversity);
        debug!("Updated average population diversity: {:.3}", diversity);
    }

    /// Update meta-learning progress
    pub fn update_meta_learning_progress(&self, progress: f64) {
        META_LEARNING_PROGRESS.set(progress);
        debug!("Updated meta-learning progress: {:.3}", progress);
    }

    /// Record convergence event
    pub fn record_convergence(&self, generation: u32, final_fitness: f64) {
        CONVERGENCE_EVENTS_TOTAL.inc();
        debug!("Recorded convergence at generation {} with fitness {:.6}", generation, final_fitness);
    }

    /// Record stagnation event
    pub fn record_stagnation(&self, generations_stagnant: u32) {
        STAGNATION_EVENTS_TOTAL.inc();
        debug!("Recorded stagnation after {} generations", generations_stagnant);
    }

    /// Record evolution error
    pub fn record_evolution_error(&self, error_type: &EvolutionErrorType) {
        EVOLUTION_ERRORS_TOTAL.inc();

        match error_type {
            EvolutionErrorType::MetaLearningError(_) => {
                debug!("Recorded meta-learning error");
            }
            EvolutionErrorType::PopulationError(_) => {
                debug!("Recorded population error");
            }
            EvolutionErrorType::AdaptiveError(_) => {
                debug!("Recorded adaptive strategy error");
            }
            EvolutionErrorType::HyperHeuristicError(_) => {
                debug!("Recorded hyper-heuristic error");
            }
            EvolutionErrorType::FitnessAnalysisError(_) => {
                debug!("Recorded fitness analysis error");
            }
            EvolutionErrorType::IntegrationError(_) => {
                debug!("Recorded integration error");
            }
            EvolutionErrorType::AlgorithmError(_) => {
                debug!("Recorded algorithm error");
            }
            EvolutionErrorType::ConfigurationError(_) => {
                debug!("Recorded configuration error");
            }
            EvolutionErrorType::ResourceError(_) => {
                debug!("Recorded resource error");
            }
            EvolutionErrorType::ConvergenceError(_) => {
                debug!("Recorded convergence error");
            }
            EvolutionErrorType::ValidationError(_) => {
                debug!("Recorded validation error");
            }
            EvolutionErrorType::InternalError(_) => {
                debug!("Recorded internal error");
            }
        }
    }

    /// Get current metrics as Prometheus format
    pub fn get_metrics(&self) -> Result<String, EvolutionError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let result = encoder.encode_to_string(&metric_families)?;

        Ok(result)
    }

    /// Get metrics summary as a hashmap
    pub fn get_metrics_summary(&self) -> Result<HashMap<String, f64>, EvolutionError> {
        let mut summary = HashMap::new();

        // Get counter values
        summary.insert("evolution_runs_started_total".to_string(), EVOLUTION_RUNS_STARTED.get());
        summary.insert("evolution_runs_completed_total".to_string(), EVOLUTION_RUNS_COMPLETED.get());
        summary.insert("algorithm_selections_total".to_string(), ALGORITHM_SELECTIONS_TOTAL.get());
        summary.insert("population_migrations_total".to_string(), POPULATION_MIGRATIONS_TOTAL.get());
        summary.insert("heuristic_applications_total".to_string(), HEURISTIC_APPLICATIONS_TOTAL.get());
        summary.insert("evolution_errors_total".to_string(), EVOLUTION_ERRORS_TOTAL.get());
        summary.insert("convergence_events_total".to_string(), CONVERGENCE_EVENTS_TOTAL.get());
        summary.insert("stagnation_events_total".to_string(), STAGNATION_EVENTS_TOTAL.get());

        // Get gauge values
        summary.insert("active_populations".to_string(), ACTIVE_POPULATIONS.get());
        summary.insert("algorithm_portfolio_size".to_string(), ALGORITHM_PORTFOLIO_SIZE.get());
        summary.insert("heuristic_portfolio_size".to_string(), HEURISTIC_PORTFOLIO_SIZE.get());
        summary.insert("average_population_diversity".to_string(), AVERAGE_POPULATION_DIVERSITY.get());
        summary.insert("meta_learning_progress".to_string(), META_LEARNING_PROGRESS.get());

        Ok(summary)
    }

    /// Reset all metrics (for testing)
    pub fn reset_metrics(&self) {
        EVOLUTION_RUNS_STARTED.reset();
        EVOLUTION_RUNS_COMPLETED.reset();
        ALGORITHM_SELECTIONS_TOTAL.reset();
        POPULATION_MIGRATIONS_TOTAL.reset();
        HEURISTIC_APPLICATIONS_TOTAL.reset();
        EVOLUTION_ERRORS_TOTAL.reset();
        CONVERGENCE_EVENTS_TOTAL.reset();
        STAGNATION_EVENTS_TOTAL.reset();
        ACTIVE_POPULATIONS.set(0.0);
        ALGORITHM_PORTFOLIO_SIZE.set(0.0);
        HEURISTIC_PORTFOLIO_SIZE.set(0.0);
        AVERAGE_POPULATION_DIVERSITY.set(0.0);
        META_LEARNING_PROGRESS.set(0.0);

        debug!("Reset all evolution metrics");
    }
}

impl Default for EvolutionMetricsCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create evolution metrics collector")
    }
}

/// Evolution error types for metrics
#[derive(Debug, Clone)]
pub enum EvolutionErrorType {
    MetaLearningError(String),
    PopulationError(String),
    AdaptiveError(String),
    HyperHeuristicError(String),
    FitnessAnalysisError(String),
    IntegrationError(String),
    AlgorithmError(String),
    ConfigurationError(String),
    ResourceError(String),
    ConvergenceError(String),
    ValidationError(String),
    InternalError(String),
}

/// Performance timer for evolution operations
pub struct EvolutionPerformanceTimer {
    start_time: std::time::Instant,
    operation_name: String,
}

impl EvolutionPerformanceTimer {
    /// Start a new performance timer
    pub fn start(operation_name: String) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            operation_name,
        }
    }

    /// Stop the timer and return the duration in seconds
    pub fn stop(self) -> f64 {
        let duration = self.start_time.elapsed();
        duration.as_secs_f64()
    }

    /// Stop the timer and record the duration in a histogram
    pub fn stop_and_record(self, histogram: &Histogram) -> f64 {
        let duration = self.stop();
        histogram.observe(duration);
        duration
    }
}

/// Scoped metrics recorder for evolution operations
pub struct ScopedEvolutionMetricsRecorder {
    timer: EvolutionPerformanceTimer,
    histogram: Histogram,
}

impl ScopedEvolutionMetricsRecorder {
    /// Create a new scoped metrics recorder
    pub fn new(operation_name: String, histogram: Histogram) -> Self {
        Self {
            timer: EvolutionPerformanceTimer::start(operation_name),
            histogram,
        }
    }
}

impl Drop for ScopedEvolutionMetricsRecorder {
    fn drop(&mut self) {
        let duration = self.timer.stop();
        self.histogram.observe(duration);
    }
}

/// Create a scoped metrics recorder for evolution operations
#[macro_export]
macro_rules! record_evolution_duration {
    ($histogram:expr, $operation:expr) => {
        let _recorder = ScopedEvolutionMetricsRecorder::new($operation.to_string(), $histogram);
    };
}

/// Evolution metrics utilities
pub struct EvolutionMetricsUtils;

impl EvolutionMetricsUtils {
    /// Calculate evolution success rate
    pub fn calculate_success_rate(completed_runs: u64, total_runs: u64) -> f64 {
        if total_runs == 0 {
            0.0
        } else {
            completed_runs as f64 / total_runs as f64
        }
    }

    /// Calculate convergence efficiency
    pub fn calculate_convergence_efficiency(
        successful_convergences: u64,
        total_convergences: u64,
        avg_convergence_time: f64,
    ) -> f64 {
        let success_rate = Self::calculate_success_rate(successful_convergences, total_convergences);
        let time_efficiency = if avg_convergence_time > 0.0 {
            1.0 / (1.0 + avg_convergence_time / 100.0) // Normalize to 0-1 range
        } else {
            1.0
        };

        (success_rate + time_efficiency) / 2.0
    }

    /// Calculate algorithm portfolio diversity
    pub fn calculate_portfolio_diversity(algorithms: &[AlgorithmInfo]) -> f64 {
        if algorithms.len() < 2 {
            return 0.0;
        }

        // Calculate diversity based on algorithm capabilities
        let mut total_capability_differences = 0.0;
        let mut comparisons = 0;

        for (i, alg1) in algorithms.iter().enumerate() {
            for alg2 in &algorithms[i + 1..] {
                let capability_diff = Self::calculate_capability_difference(&alg1.capabilities, &alg2.capabilities);
                total_capability_differences += capability_diff;
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            total_capability_differences / comparisons as f64
        } else {
            0.0
        }
    }

    /// Calculate difference between algorithm capabilities
    fn calculate_capability_difference(cap1: &AlgorithmCapabilities, cap2: &AlgorithmCapabilities) -> f64 {
        let mut differences = 0.0;

        differences += if cap1.multi_objective != cap2.multi_objective { 1.0 } else { 0.0 };
        differences += if cap1.constraint_handling != cap2.constraint_handling { 1.0 } else { 0.0 };
        differences += if cap1.large_population != cap2.large_population { 1.0 } else { 0.0 };
        differences += if cap1.high_dimensional != cap2.high_dimensional { 1.0 } else { 0.0 };
        differences += if cap1.noisy_fitness != cap2.noisy_fitness { 1.0 } else { 0.0 };
        differences += if cap1.parallel_processing != cap2.parallel_processing { 1.0 } else { 0.0 };

        differences / 6.0 // Normalize to 0-1 range
    }

    /// Calculate meta-learning effectiveness
    pub fn calculate_meta_learning_effectiveness(
        selection_accuracy: f64,
        adaptation_effectiveness: f64,
        knowledge_transfer: f64,
    ) -> f64 {
        (selection_accuracy * 0.4 + adaptation_effectiveness * 0.3 + knowledge_transfer * 0.3).min(1.0).max(0.0)
    }

    /// Format evolution metrics for logging
    pub fn format_evolution_metrics_for_log(metrics: &HashMap<String, f64>) -> String {
        let mut parts = Vec::new();
        for (key, value) in metrics.iter() {
            parts.push(format!("{}={:.3}", key, value));
        }
        parts.join(", ")
    }

    /// Calculate evolution efficiency score
    pub fn calculate_evolution_efficiency(
        success_rate: f64,
        avg_convergence_time: f64,
        resource_efficiency: f64,
        solution_quality: f64,
    ) -> f64 {
        let time_score = if avg_convergence_time < 100.0 {
            1.0
        } else if avg_convergence_time < 500.0 {
            0.8
        } else if avg_convergence_time < 1000.0 {
            0.6
        } else {
            0.4
        };

        (success_rate * 0.3 + time_score * 0.3 + resource_efficiency * 0.2 + solution_quality * 0.2).min(1.0).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_metrics_collector_creation() {
        let collector = EvolutionMetricsCollector::new();
        assert!(collector.is_ok());
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
    fn test_portfolio_diversity_calculation() {
        let algorithms = vec![
            AlgorithmInfo {
                id: "alg1".to_string(),
                name: "Algorithm 1".to_string(),
                capabilities: AlgorithmCapabilities {
                    multi_objective: false,
                    constraint_handling: true,
                    large_population: true,
                    high_dimensional: false,
                    noisy_fitness: true,
                    parallel_processing: true,
                },
                performance_metrics: AlgorithmMetrics {
                    success_rate: 0.8,
                    avg_convergence_time: 100.0,
                    avg_solution_quality: 0.9,
                    resource_efficiency: 0.8,
                    robustness: 0.7,
                },
                usage_stats: UsageStatistics {
                    total_runs: 100,
                    successful_runs: 80,
                    avg_run_time_seconds: 50.0,
                    last_used: Utc::now(),
                },
            },
            AlgorithmInfo {
                id: "alg2".to_string(),
                name: "Algorithm 2".to_string(),
                capabilities: AlgorithmCapabilities {
                    multi_objective: true,
                    constraint_handling: false,
                    large_population: false,
                    high_dimensional: true,
                    noisy_fitness: false,
                    parallel_processing: false,
                },
                performance_metrics: AlgorithmMetrics {
                    success_rate: 0.7,
                    avg_convergence_time: 150.0,
                    avg_solution_quality: 0.85,
                    resource_efficiency: 0.6,
                    robustness: 0.8,
                },
                usage_stats: UsageStatistics {
                    total_runs: 80,
                    successful_runs: 56,
                    avg_run_time_seconds: 75.0,
                    last_used: Utc::now(),
                },
            },
        ];

        let diversity = EvolutionMetricsUtils::calculate_portfolio_diversity(&algorithms);
        assert!(diversity > 0.5); // Should be diverse due to different capabilities
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
    fn test_evolution_error_types() {
        let meta_error = EvolutionErrorType::MetaLearningError("Test error".to_string());
        let population_error = EvolutionErrorType::PopulationError("Test error".to_string());
        let adaptive_error = EvolutionErrorType::AdaptiveError("Test error".to_string());

        // Test that we can match on different error types
        match meta_error {
            EvolutionErrorType::MetaLearningError(_) => assert!(true),
            _ => assert!(false),
        }

        match population_error {
            EvolutionErrorType::PopulationError(_) => assert!(true),
            _ => assert!(false),
        }

        match adaptive_error {
            EvolutionErrorType::AdaptiveError(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_performance_timer() {
        let timer = EvolutionPerformanceTimer::start("test_evolution_operation".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.stop();

        assert!(duration >= 0.01);
        assert!(duration < 1.0); // Should be much less than 1 second
    }
}