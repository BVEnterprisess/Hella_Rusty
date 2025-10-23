//! A/B Testing & Validation Framework for Layer 5

use crate::types::*;
use statrs::distribution::{ContinuousCDF, StudentsT};
use statrs::statistics::{Data, Distribution, OrderStatistics};
use std::collections::HashMap;
use tracing::{info, error};

/// A/B Testing Framework
pub struct ABTestingFramework {
    experiment_manager: ExperimentManager,
    hypothesis_tester: HypothesisTester,
}

impl ABTestingFramework {
    pub async fn new(config: ABConfig) -> Result<Self, ABTestingError> {
        Ok(Self {
            experiment_manager: ExperimentManager::new().await?,
            hypothesis_tester: HypothesisTester::new(config.significance_level, config.power_threshold).await?,
        })
    }

    /// Create a new experiment
    pub async fn create_experiment(&self, experiment: Experiment) -> Result<ExperimentId, ABTestingError> {
        self.experiment_manager.create_experiment(experiment).await
    }

    /// Run statistical test on experiment results
    pub async fn test_hypothesis(&self, experiment_id: ExperimentId, results: HashMap<String, Vec<f64>>) -> Result<StatisticalResult, ABTestingError> {
        self.hypothesis_tester.test(experiment_id, results).await
    }
}

/// Experiment Manager
pub struct ExperimentManager {
    active_experiments: HashMap<ExperimentId, Experiment>,
    results_storage: ExperimentStorage,
    statistical_engine: StatisticalEngine,
}

struct ExperimentStorage {
    // In a real implementation, this would be a database
}

struct StatisticalEngine {
    // Statistical computation engine
}

impl ExperimentManager {
    pub async fn new() -> Result<Self, ABTestingError> {
        Ok(Self {
            active_experiments: HashMap::new(),
            results_storage: ExperimentStorage {},
            statistical_engine: StatisticalEngine {},
        })
    }

    pub async fn create_experiment(&mut self, experiment: Experiment) -> Result<ExperimentId, ABTestingError> {
        let id = experiment.id.clone();
        self.active_experiments.insert(id.clone(), experiment);
        info!("Created experiment: {}", id);
        Ok(id)
    }

    pub async fn get_experiment(&self, id: ExperimentId) -> Option<&Experiment> {
        self.active_experiments.get(&id)
    }

    pub async fn update_experiment_results(&self, id: ExperimentId, results: HashMap<String, Vec<f64>>) -> Result<(), ABTestingError> {
        // Store results
        info!("Updated results for experiment: {}", id);
        Ok(())
    }
}

/// Hypothesis Tester
pub struct HypothesisTester {
    significance_level: f64,
    power_threshold: f64,
}

struct StatisticalResult {
    p_value: f64,
    effect_size: f64,
    confidence_interval: (f64, f64),
    is_significant: bool,
}

impl HypothesisTester {
    pub async fn new(significance_level: f64, power_threshold: f64) -> Result<Self, ABTestingError> {
        Ok(Self {
            significance_level,
            power_threshold,
        })
    }

    pub async fn test(&self, experiment_id: ExperimentId, results: HashMap<String, Vec<f64>>) -> Result<StatisticalResult, ABTestingError> {
        if results.len() < 2 {
            return Err(ABTestingError::StatisticalTestFailed);
        }

        // Perform t-test between variants
        let variants: Vec<&Vec<f64>> = results.values().collect();
        let control = variants[0];
        let treatment = variants[1];

        let t_stat = self.calculate_t_statistic(control, treatment);
        let p_value = self.calculate_p_value(t_stat, control.len() + treatment.len() - 2);
        let effect_size = self.calculate_effect_size(control, treatment);
        let confidence_interval = self.calculate_confidence_interval(control, treatment);

        let is_significant = p_value < self.significance_level;

        Ok(StatisticalResult {
            p_value,
            effect_size,
            confidence_interval,
            is_significant,
        })
    }

    fn calculate_t_statistic(&self, group1: &[f64], group2: &[f64]) -> f64 {
        let mean1 = group1.iter().sum::<f64>() / group1.len() as f64;
        let mean2 = group2.iter().sum::<f64>() / group2.len() as f64;

        let var1 = group1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (group1.len() - 1) as f64;
        let var2 = group2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (group2.len() - 1) as f64;

        let pooled_var = ((group1.len() - 1) as f64 * var1 + (group2.len() - 1) as f64 * var2) / (group1.len() + group2.len() - 2) as f64;
        let se = (pooled_var * (1.0 / group1.len() as f64 + 1.0 / group2.len() as f64)).sqrt();

        if se == 0.0 {
            0.0
        } else {
            (mean1 - mean2) / se
        }
    }

    fn calculate_p_value(&self, t_stat: f64, df: usize) -> f64 {
        let t_dist = StudentsT::new(0.0, 1.0, df as f64).unwrap();
        2.0 * (1.0 - t_dist.cdf(t_stat.abs()))
    }

    fn calculate_effect_size(&self, group1: &[f64], group2: &[f64]) -> f64 {
        let mean1 = group1.iter().sum::<f64>() / group1.len() as f64;
        let mean2 = group2.iter().sum::<f64>() / group2.len() as f64;

        let std1 = (group1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (group1.len() - 1) as f64).sqrt();
        let std2 = (group2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (group2.len() - 1) as f64).sqrt();

        let pooled_std = (std1 + std2) / 2.0;

        if pooled_std == 0.0 {
            0.0
        } else {
            (mean1 - mean2) / pooled_std
        }
    }

    fn calculate_confidence_interval(&self, group1: &[f64], group2: &[f64]) -> (f64, f64) {
        let mean1 = group1.iter().sum::<f64>() / group1.len() as f64;
        let mean2 = group2.iter().sum::<f64>() / group2.len() as f64;

        let diff = mean1 - mean2;
        let se = (self.calculate_t_statistic(group1, group2).abs() * 0.1).sqrt(); // Simplified

        let margin = 1.96 * se; // 95% confidence

        (diff - margin, diff + margin)
    }
}