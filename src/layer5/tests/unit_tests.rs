//! Unit tests for Layer 5

#[cfg(test)]
mod tests {
    use super::super::*;
    use chrono::Utc;

    #[test]
    fn test_kpi_batch_creation() {
        let kpi = KpiBatch {
            timestamp: Utc::now(),
            agent_id: uuid::Uuid::new_v4(),
            task_id: uuid::Uuid::new_v4(),
            metrics: [("cpu_usage".to_string(), 0.8), ("memory_usage".to_string(), 0.6)].iter().cloned().collect(),
            metadata: [("version".to_string(), "1.0".to_string())].iter().cloned().collect(),
        };

        assert_eq!(kpi.metrics.len(), 2);
        assert_eq!(kpi.metrics["cpu_usage"], 0.8);
    }

    #[test]
    fn test_optimization_result() {
        let result = OptimizationResult {
            agent_id: uuid::Uuid::new_v4(),
            parameters: [("learning_rate".to_string(), 0.01)].iter().cloned().collect(),
            confidence: 0.95,
            timestamp: Utc::now(),
        };

        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.parameters["learning_rate"], 0.01);
    }

    #[test]
    fn test_pattern_result() {
        let result = PatternResult {
            pattern_type: PatternType::Trend,
            confidence: 0.9,
            metrics: [("slope".to_string(), 0.5)].iter().cloned().collect(),
            timestamp: Utc::now(),
        };

        assert!(matches!(result.pattern_type, PatternType::Trend));
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_experiment_creation() {
        let experiment = Experiment {
            id: uuid::Uuid::new_v4(),
            name: "Test Experiment".to_string(),
            hypothesis: "Increasing learning rate improves performance".to_string(),
            variants: vec![
                ExperimentVariant {
                    name: "Control".to_string(),
                    parameters: [("learning_rate".to_string(), 0.01)].iter().cloned().collect(),
                },
                ExperimentVariant {
                    name: "Treatment".to_string(),
                    parameters: [("learning_rate".to_string(), 0.02)].iter().cloned().collect(),
                },
            ],
            start_time: Utc::now(),
            end_time: None,
            status: ExperimentStatus::Draft,
        };

        assert_eq!(experiment.variants.len(), 2);
        assert!(matches!(experiment.status, ExperimentStatus::Draft));
    }

    #[test]
    fn test_feedback_report() {
        let feedback = FeedbackReport {
            agent_id: uuid::Uuid::new_v4(),
            optimization_id: uuid::Uuid::new_v4(),
            success: true,
            performance_delta: 0.15,
            timestamp: Utc::now(),
        };

        assert!(feedback.success);
        assert_eq!(feedback.performance_delta, 0.15);
    }
}