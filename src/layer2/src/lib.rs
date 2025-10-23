//! # Layer 2 (Planning) - Strategic Planning and Task Decomposition
//!
//! Layer 2 is responsible for strategic planning, goal decomposition, and task orchestration
//! across the Project Chimera autonomous AI system. It breaks down high-level objectives
//! into actionable tasks and coordinates resource allocation with Layer 8.
//!
//! ## Core Responsibilities
//!
//! - **Strategic Planning**: Long-term goal analysis and planning
//! - **Task Decomposition**: Breaking complex goals into manageable tasks
//! - **Resource Coordination**: Working with Layer 8 for resource allocation
//! - **Progress Tracking**: Monitoring task execution and adjusting plans
//! - **Risk Assessment**: Identifying and mitigating planning risks
//!
//! ## Architecture
//!
//! The planning system consists of several key components:
//!
//! - **Planner**: Main planning engine and coordination
//! - **TaskDecomposer**: Breaks down goals into actionable tasks
//! - **ResourceCoordinator**: Coordinates with Layer 8 for resource allocation
//! - **ProgressTracker**: Monitors execution and adjusts plans
//! - **RiskAssessor**: Identifies and manages planning risks

pub mod planner;
pub mod task_decomposer;
pub mod resource_coordinator;
pub mod progress_tracker;
pub mod risk_assessor;
pub mod types;
pub mod metrics;

pub use planner::Planner;
pub use task_decomposer::TaskDecomposer;
pub use resource_coordinator::ResourceCoordinator;
pub use progress_tracker::ProgressTracker;
pub use risk_assessor::RiskAssessor;
pub use types::*;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Main planning service that orchestrates all planning activities
pub struct PlanningService {
    planner: Arc<Planner>,
    task_decomposer: Arc<TaskDecomposer>,
    resource_coordinator: Arc<ResourceCoordinator>,
    progress_tracker: Arc<ProgressTracker>,
    risk_assessor: Arc<RiskAssessor>,
    metrics: Arc<metrics::PlanningMetrics>,
}

impl PlanningService {
    /// Create a new planning service
    pub async fn new() -> Result<Self> {
        let planner = Arc::new(Planner::new().await?);
        let task_decomposer = Arc::new(TaskDecomposer::new().await?);
        let resource_coordinator = Arc::new(ResourceCoordinator::new().await?);
        let progress_tracker = Arc::new(ProgressTracker::new().await?);
        let risk_assessor = Arc::new(RiskAssessor::new().await?);
        let metrics = Arc::new(metrics::PlanningMetrics::new().await?);

        Ok(Self {
            planner,
            task_decomposer,
            resource_coordinator,
            progress_tracker,
            risk_assessor,
            metrics,
        })
    }

    /// Process a high-level goal and create a comprehensive plan
    pub async fn process_goal(&self, goal: Goal) -> Result<Plan> {
        info!("Processing goal: {}", goal.id);

        // Record metrics
        self.metrics.record_goal_received().await;
        let start_time = std::time::Instant::now();

        // Assess risks first
        let risks = self.risk_assessor.assess_risks(&goal).await?;

        // Decompose goal into tasks
        let tasks = self.task_decomposer.decompose_goal(&goal).await?;

        // Create initial plan
        let mut plan = self.planner.create_plan(goal, tasks, risks).await?;

        // Coordinate resources
        plan = self.resource_coordinator.allocate_resources(plan).await?;

        // Record completion metrics
        let duration = start_time.elapsed();
        self.metrics.record_planning_duration(duration.as_secs_f64()).await;
        self.metrics.record_plan_created().await;

        info!("Plan created successfully: {}", plan.id);
        Ok(plan)
    }

    /// Update plan based on execution feedback
    pub async fn update_plan(&self, plan_id: Uuid, feedback: ExecutionFeedback) -> Result<Plan> {
        debug!("Updating plan: {}", plan_id);

        self.metrics.record_plan_updated().await;

        // Get current plan
        let current_plan = self.planner.get_plan(plan_id).await?;

        // Update progress
        let updated_plan = self.progress_tracker.update_progress(current_plan, feedback).await?;

        // Reassess risks if needed
        if self.progress_tracker.requires_risk_reassessment(&updated_plan).await? {
            let risks = self.risk_assessor.reassess_risks(&updated_plan).await?;
            let updated_plan = self.planner.update_risks(updated_plan, risks).await?;
        }

        // Reallocate resources if needed
        if self.resource_coordinator.requires_resource_reallocation(&updated_plan).await? {
            let updated_plan = self.resource_coordinator.reallocate_resources(updated_plan).await?;
        }

        info!("Plan updated successfully: {}", plan_id);
        Ok(updated_plan)
    }

    /// Get planning metrics
    pub async fn get_metrics(&self) -> Result<metrics::PlanningMetricsSnapshot> {
        self.metrics.snapshot().await
    }

    /// Health check for the planning service
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut issues = Vec::new();

        // Check all components
        if let Err(e) = self.planner.health_check().await {
            issues.push(format!("Planner: {}", e));
        }

        if let Err(e) = self.task_decomposer.health_check().await {
            issues.push(format!("Task Decomposer: {}", e));
        }

        if let Err(e) = self.resource_coordinator.health_check().await {
            issues.push(format!("Resource Coordinator: {}", e));
        }

        if let Err(e) = self.progress_tracker.health_check().await {
            issues.push(format!("Progress Tracker: {}", e));
        }

        if let Err(e) = self.risk_assessor.health_check().await {
            issues.push(format!("Risk Assessor: {}", e));
        }

        if issues.is_empty() {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Degraded { issues })
        }
    }
}

impl Default for PlanningService {
    fn default() -> Self {
        Self::new().expect("Failed to create PlanningService")
    }
}

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
            description: "A test goal for planning".to_string(),
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
    }
}