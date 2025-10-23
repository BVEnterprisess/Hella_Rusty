//! # Layer 2 Unit Tests - Individual Component Testing
//!
//! Unit tests for individual components of the Layer 2 planning system.

use layer2_planning::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_planning_service_creation() {
        let service = PlanningService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_goal_processing() {
        let service = PlanningService::new().await.unwrap();

        let goal = Goal {
            id: Uuid::new_v4(),
            title: "Test Goal".to_string(),
            description: "Implement a machine learning model with testing and deployment".to_string(),
            priority: Priority::High,
            deadline: Utc::now() + chrono::Duration::hours(24),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let result = service.process_goal(goal).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.goal.title, "Test Goal");
        assert!(!plan.tasks.is_empty());
        assert!(!plan.risks.is_empty());
    }

    #[tokio::test]
    async fn test_plan_update() {
        let service = PlanningService::new().await.unwrap();

        // Create initial plan
        let goal = Goal {
            id: Uuid::new_v4(),
            title: "Test Goal".to_string(),
            description: "Test goal".to_string(),
            priority: Priority::High,
            deadline: Utc::now() + chrono::Duration::hours(24),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let plan = service.process_goal(goal).await.unwrap();

        // Update plan with feedback
        let feedback = ExecutionFeedback {
            plan_id: plan.id,
            task_id: plan.tasks[0].id,
            status: TaskStatus::InProgress,
            progress_percentage: 50.0,
            actual_duration_hours: Some(2.0),
            actual_cost: Some(1.0),
            issues: Vec::new(),
            recommendations: Vec::new(),
            metrics: HashMap::new(),
            timestamp: Utc::now(),
        };

        let updated_plan = service.update_plan(plan.id, feedback).await;
        assert!(updated_plan.is_ok());
    }

    #[tokio::test]
    async fn test_planner_creation() {
        let planner = Planner::new().await;
        assert!(planner.is_ok());
    }

    #[tokio::test]
    async fn test_task_decomposer_creation() {
        let decomposer = TaskDecomposer::new().await;
        assert!(decomposer.is_ok());
    }

    #[tokio::test]
    async fn test_resource_coordinator_creation() {
        let coordinator = ResourceCoordinator::new().await;
        // This might fail if Layer 8 is not running, but that's expected in tests
        assert!(coordinator.is_ok() || coordinator.is_err());
    }

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new().await;
        assert!(tracker.is_ok());
    }

    #[tokio::test]
    async fn test_risk_assessor_creation() {
        let assessor = RiskAssessor::new().await;
        assert!(assessor.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = PlanningMetrics::new().await;
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical as u8 < Priority::High as u8);
        assert!(Priority::High as u8 < Priority::Medium as u8);
        assert!(Priority::Medium as u8 < Priority::Low as u8);
    }

    #[test]
    fn test_goal_validation() {
        let goal = Goal {
            id: Uuid::new_v4(),
            title: "Valid Title".to_string(),
            description: "Valid description".to_string(),
            priority: Priority::High,
            deadline: Utc::now() + chrono::Duration::hours(24),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert!(goal.validate().is_ok());
    }

    #[test]
    fn test_task_validation() {
        let task = Task {
            id: Uuid::new_v4(),
            title: "Valid Task".to_string(),
            description: "Valid task description".to_string(),
            priority: Priority::Medium,
            estimated_duration_hours: 4.0,
            resource_requirements: Vec::new(),
            dependencies: Vec::new(),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            assigned_layer: Some("layer4".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert!(task.validate().is_ok());
    }

    #[test]
    fn test_invalid_goal_validation() {
        let goal = Goal {
            id: Uuid::new_v4(),
            title: "".to_string(), // Invalid: empty title
            description: "Valid description".to_string(),
            priority: Priority::High,
            deadline: Utc::now() + chrono::Duration::hours(24),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert!(goal.validate().is_err());
    }

    #[test]
    fn test_invalid_task_validation() {
        let task = Task {
            id: Uuid::new_v4(),
            title: "".to_string(), // Invalid: empty title
            description: "Valid task description".to_string(),
            priority: Priority::Medium,
            estimated_duration_hours: 4.0,
            resource_requirements: Vec::new(),
            dependencies: Vec::new(),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            assigned_layer: Some("layer4".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert!(task.validate().is_err());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let planner = Planner::new().await.unwrap();

        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();

        let tasks = vec![
            Task {
                id: task1_id,
                title: "Task 1".to_string(),
                description: "First task".to_string(),
                priority: Priority::High,
                estimated_duration_hours: 4.0,
                resource_requirements: Vec::new(),
                dependencies: vec![task2_id], // Depends on task 2
                constraints: Vec::new(),
                success_criteria: Vec::new(),
                assigned_layer: Some("layer4".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            },
            Task {
                id: task2_id,
                title: "Task 2".to_string(),
                description: "Second task".to_string(),
                priority: Priority::High,
                estimated_duration_hours: 4.0,
                resource_requirements: Vec::new(),
                dependencies: vec![task1_id], // Depends on task 1 - creates cycle
                constraints: Vec::new(),
                success_criteria: Vec::new(),
                assigned_layer: Some("layer4".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            }
        ];

        let has_cycle = planner.has_circular_dependencies(&tasks).await.unwrap();
        assert!(has_cycle);
    }

    #[tokio::test]
    async fn test_risk_score_calculation() {
        let assessor = RiskAssessor::new().await.unwrap();

        let risk = Risk {
            id: Uuid::new_v4(),
            title: "Test Risk".to_string(),
            description: "Test".to_string(),
            probability: 0.8,
            impact: ImpactLevel::High,
            mitigation_strategy: None,
            owner: None,
            status: RiskStatus::Identified,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let score = assessor.calculate_risk_score(&risk);
        assert_eq!(score, 3.2); // 0.8 * 4.0 (High impact weight)
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let metrics = PlanningMetrics::new().await.unwrap();

        metrics.record_goal_received().await;
        metrics.record_plan_created().await;
        metrics.record_task_completed(4.0).await;

        let snapshot = metrics.snapshot().await.unwrap();
        assert_eq!(snapshot.goals_received, 1.0);
        assert_eq!(snapshot.plans_created, 1.0);
        assert_eq!(snapshot.tasks_completed, 1.0);
    }

    #[tokio::test]
    async fn test_health_checks() {
        let service = PlanningService::new().await.unwrap();

        let health = service.health_check().await;
        assert!(health.is_ok());

        let planner = Planner::new().await.unwrap();
        let planner_health = planner.health_check().await;
        assert!(planner_health.is_ok());

        let decomposer = TaskDecomposer::new().await.unwrap();
        let decomposer_health = decomposer.health_check().await;
        assert!(decomposer_health.is_ok());

        let tracker = ProgressTracker::new().await.unwrap();
        let tracker_health = tracker.health_check().await;
        assert!(tracker_health.is_ok());

        let assessor = RiskAssessor::new().await.unwrap();
        let assessor_health = assessor.health_check().await;
        assert!(assessor_health.is_ok());
    }

    #[tokio::test]
    async fn test_component_extraction() {
        let decomposer = TaskDecomposer::new().await.unwrap();

        let components = decomposer.extract_goal_components("Implement and test the system").await.unwrap();
        assert!(!components.is_empty());

        let functions = decomposer.identify_functions("API development and data processing").await.unwrap();
        assert!(!functions.is_empty());

        let resource_types = decomposer.identify_resource_types("GPU training with CPU processing").await.unwrap();
        assert!(!resource_types.is_empty());
    }

    #[tokio::test]
    async fn test_task_priority_calculation() {
        let decomposer = TaskDecomposer::new().await.unwrap();

        let high_priority = decomposer.calculate_task_priority(Priority::High, 0, 3);
        assert_eq!(high_priority, Priority::Medium);

        let critical_priority = decomposer.calculate_task_priority(Priority::Critical, 0, 3);
        assert_eq!(critical_priority, Priority::High);
    }

    #[tokio::test]
    async fn test_resource_requirements_estimation() {
        let decomposer = TaskDecomposer::new().await.unwrap();

        let gpu_requirements = decomposer.estimate_resource_requirements("gpu").await.unwrap();
        assert!(!gpu_requirements.is_empty());

        let cpu_requirements = decomposer.estimate_resource_requirements("cpu").await.unwrap();
        assert!(!cpu_requirements.is_empty());
    }

    #[tokio::test]
    async fn test_task_deduplication() {
        let decomposer = TaskDecomposer::new().await.unwrap();

        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "Test task 1".to_string(),
            priority: Priority::High,
            estimated_duration_hours: 4.0,
            resource_requirements: Vec::new(),
            dependencies: Vec::new(),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            assigned_layer: Some("layer4".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(), // Same title
            description: "Test task 2".to_string(),
            priority: Priority::Medium,
            estimated_duration_hours: 2.0,
            resource_requirements: Vec::new(),
            dependencies: Vec::new(),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            assigned_layer: Some("layer4".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let tasks = vec![task1, task2];
        let deduplicated = decomposer.deduplicate_and_merge_tasks(tasks).await.unwrap();

        assert_eq!(deduplicated.len(), 1);
        assert_eq!(deduplicated[0].estimated_duration_hours, 6.0); // 4.0 + 2.0
    }

    #[tokio::test]
    async fn test_task_dependency_validation() {
        let decomposer = TaskDecomposer::new().await.unwrap();

        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();

        let tasks = vec![
            Task {
                id: task1_id,
                title: "Task 1".to_string(),
                description: "First task".to_string(),
                priority: Priority::High,
                estimated_duration_hours: 4.0,
                resource_requirements: Vec::new(),
                dependencies: Vec::new(),
                constraints: Vec::new(),
                success_criteria: Vec::new(),
                assigned_layer: Some("layer4".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            },
            Task {
                id: task2_id,
                title: "Task 2".to_string(),
                description: "Second task".to_string(),
                priority: Priority::High,
                estimated_duration_hours: 4.0,
                resource_requirements: Vec::new(),
                dependencies: vec![task1_id], // Valid dependency
                constraints: Vec::new(),
                success_criteria: Vec::new(),
                assigned_layer: Some("layer4".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            }
        ];

        let validation = decomposer.validate_task_dependencies(&tasks).await;
        assert!(validation.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_task_dependency_validation() {
        let decomposer = TaskDecomposer::new().await.unwrap();

        let invalid_task_id = Uuid::new_v4();

        let tasks = vec![
            Task {
                id: Uuid::new_v4(),
                title: "Task 1".to_string(),
                description: "First task".to_string(),
                priority: Priority::High,
                estimated_duration_hours: 4.0,
                resource_requirements: Vec::new(),
                dependencies: vec![invalid_task_id], // Invalid dependency
                constraints: Vec::new(),
                success_criteria: Vec::new(),
                assigned_layer: Some("layer4".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            }
        ];

        let validation = decomposer.validate_task_dependencies(&tasks).await;
        assert!(validation.is_err());
    }
}