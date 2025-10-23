//! # Planning Metrics - Metrics Collection and Monitoring
//!
//! This module provides comprehensive metrics collection for the Layer 2 planning system.
//! It tracks planning performance, resource utilization, risk metrics, and execution
//! progress to enable monitoring and optimization.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use prometheus::{Counter, Gauge, Histogram, Registry, Encoder, TextEncoder};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Planning metrics collector
pub struct PlanningMetrics {
    registry: Registry,

    // Goal and plan metrics
    goals_received: Counter,
    plans_created: Counter,
    plan_updates: Counter,
    plans_cancelled: Counter,

    // Task metrics
    tasks_created: Counter,
    tasks_completed: Counter,
    tasks_failed: Counter,
    task_completion_duration: Histogram,

    // Risk metrics
    risks_identified: Counter,
    high_risks_identified: Counter,
    risks_mitigated: Counter,
    risk_score_average: Gauge,

    // Resource metrics
    resource_allocations_requested: Counter,
    resource_allocations_successful: Counter,
    resource_allocation_failures: Counter,
    resource_cost_total: Counter,

    // Performance metrics
    planning_duration_seconds: Histogram,
    task_decomposition_duration_seconds: Histogram,
    risk_assessment_duration_seconds: Histogram,
    resource_coordination_duration_seconds: Histogram,

    // Progress metrics
    plan_progress_percentage: Gauge,
    active_plans: Gauge,
    overdue_tasks: Counter,

    // Error metrics
    planning_errors: Counter,
    validation_errors: Counter,
    integration_errors: Counter,
}

impl PlanningMetrics {
    /// Create a new metrics collector
    pub async fn new() -> Result<Self> {
        let registry = Registry::new();

        let metrics = Self {
            registry,

            // Goal and plan metrics
            goals_received: Counter::new("layer2_goals_received_total", "Total goals received for planning")?,
            plans_created: Counter::new("layer2_plans_created_total", "Total plans created")?,
            plan_updates: Counter::new("layer2_plan_updates_total", "Total plan updates")?,
            plans_cancelled: Counter::new("layer2_plans_cancelled_total", "Total plans cancelled")?,

            // Task metrics
            tasks_created: Counter::new("layer2_tasks_created_total", "Total tasks created")?,
            tasks_completed: Counter::new("layer2_tasks_completed_total", "Total tasks completed")?,
            tasks_failed: Counter::new("layer2_tasks_failed_total", "Total tasks failed")?,
            task_completion_duration: Histogram::new(
                "layer2_task_completion_duration_hours",
                "Task completion duration in hours"
            )?,

            // Risk metrics
            risks_identified: Counter::new("layer2_risks_identified_total", "Total risks identified")?,
            high_risks_identified: Counter::new("layer2_high_risks_identified_total", "Total high-impact risks identified")?,
            risks_mitigated: Counter::new("layer2_risks_mitigated_total", "Total risks mitigated")?,
            risk_score_average: Gauge::new("layer2_risk_score_average", "Average risk score across all plans")?,

            // Resource metrics
            resource_allocations_requested: Counter::new("layer2_resource_allocations_requested_total", "Total resource allocations requested")?,
            resource_allocations_successful: Counter::new("layer2_resource_allocations_successful_total", "Total successful resource allocations")?,
            resource_allocation_failures: Counter::new("layer2_resource_allocation_failures_total", "Total resource allocation failures")?,
            resource_cost_total: Counter::new("layer2_resource_cost_total", "Total resource cost in dollars")?,

            // Performance metrics
            planning_duration_seconds: Histogram::new(
                "layer2_planning_duration_seconds",
                "Time taken to create plans in seconds"
            )?,
            task_decomposition_duration_seconds: Histogram::new(
                "layer2_task_decomposition_duration_seconds",
                "Time taken for task decomposition in seconds"
            )?,
            risk_assessment_duration_seconds: Histogram::new(
                "layer2_risk_assessment_duration_seconds",
                "Time taken for risk assessment in seconds"
            )?,
            resource_coordination_duration_seconds: Histogram::new(
                "layer2_resource_coordination_duration_seconds",
                "Time taken for resource coordination in seconds"
            )?,

            // Progress metrics
            plan_progress_percentage: Gauge::new("layer2_plan_progress_percentage", "Average plan progress percentage")?,
            active_plans: Gauge::new("layer2_active_plans", "Number of currently active plans")?,
            overdue_tasks: Counter::new("layer2_overdue_tasks_total", "Total overdue tasks")?,

            // Error metrics
            planning_errors: Counter::new("layer2_planning_errors_total", "Total planning errors")?,
            validation_errors: Counter::new("layer2_validation_errors_total", "Total validation errors")?,
            integration_errors: Counter::new("layer2_integration_errors_total", "Total integration errors")?,
        };

        // Register all metrics
        metrics.register_all().await?;

        info!("Planning metrics initialized");
        Ok(metrics)
    }

    /// Register all metrics with the registry
    async fn register_all(&self) -> Result<()> {
        macro_rules! register_metric {
            ($metric:expr, $name:expr) => {
                self.registry.register(Box::new($metric.clone()))?;
            };
        }

        register_metric!(self.goals_received, "goals_received");
        register_metric!(self.plans_created, "plans_created");
        register_metric!(self.plan_updates, "plan_updates");
        register_metric!(self.plans_cancelled, "plans_cancelled");

        register_metric!(self.tasks_created, "tasks_created");
        register_metric!(self.tasks_completed, "tasks_completed");
        register_metric!(self.tasks_failed, "tasks_failed");
        register_metric!(self.task_completion_duration, "task_completion_duration");

        register_metric!(self.risks_identified, "risks_identified");
        register_metric!(self.high_risks_identified, "high_risks_identified");
        register_metric!(self.risks_mitigated, "risks_mitigated");
        register_metric!(self.risk_score_average, "risk_score_average");

        register_metric!(self.resource_allocations_requested, "resource_allocations_requested");
        register_metric!(self.resource_allocations_successful, "resource_allocations_successful");
        register_metric!(self.resource_allocation_failures, "resource_allocation_failures");
        register_metric!(self.resource_cost_total, "resource_cost_total");

        register_metric!(self.planning_duration_seconds, "planning_duration_seconds");
        register_metric!(self.task_decomposition_duration_seconds, "task_decomposition_duration_seconds");
        register_metric!(self.risk_assessment_duration_seconds, "risk_assessment_duration_seconds");
        register_metric!(self.resource_coordination_duration_seconds, "resource_coordination_duration_seconds");

        register_metric!(self.plan_progress_percentage, "plan_progress_percentage");
        register_metric!(self.active_plans, "active_plans");
        register_metric!(self.overdue_tasks, "overdue_tasks");

        register_metric!(self.planning_errors, "planning_errors");
        register_metric!(self.validation_errors, "validation_errors");
        register_metric!(self.integration_errors, "integration_errors");

        Ok(())
    }

    /// Get metrics snapshot
    pub async fn snapshot(&self) -> Result<PlanningMetricsSnapshot> {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&self.registry.gather(), &mut buffer)?;

        let metrics_text = String::from_utf8(buffer)?;

        Ok(PlanningMetricsSnapshot {
            timestamp: Utc::now(),
            metrics_text,
            goals_received: self.goals_received.get(),
            plans_created: self.plans_created.get(),
            plans_active: self.active_plans.get(),
            tasks_completed: self.tasks_completed.get(),
            tasks_failed: self.tasks_failed.get(),
            risks_identified: self.risks_identified.get(),
            high_risks_identified: self.high_risks_identified.get(),
            resource_allocations_successful: self.resource_allocations_successful.get(),
            resource_allocation_failures: self.resource_allocation_failures.get(),
            average_risk_score: self.risk_score_average.get(),
            average_plan_progress: self.plan_progress_percentage.get(),
        })
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&self.registry.gather(), &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Update metrics based on planning activity
    pub async fn record_goal_received(&self) {
        self.goals_received.inc();
        self.active_plans.inc();
    }

    /// Update metrics when a plan is created
    pub async fn record_plan_created(&self) {
        self.plans_created.inc();
    }

    /// Update metrics when a plan is updated
    pub async fn record_plan_updated(&self) {
        self.plan_updates.inc();
    }

    /// Update metrics when a plan is cancelled
    pub async fn record_plan_cancelled(&self) {
        self.plans_cancelled.inc();
        self.active_plans.dec();
    }

    /// Update metrics for planning performance
    pub async fn record_planning_duration(&self, duration_seconds: f64) {
        self.planning_duration_seconds.observe(duration_seconds);
    }

    /// Update metrics when a task is created
    pub async fn record_task_created(&self) {
        self.tasks_created.inc();
    }

    /// Update metrics when a task is completed
    pub async fn record_task_completed(&self, duration_hours: f64) {
        self.tasks_completed.inc();
        self.task_completion_duration.observe(duration_hours);
    }

    /// Update metrics when a task fails
    pub async fn record_task_failed(&self) {
        self.tasks_failed.inc();
    }

    /// Update metrics when risks are identified
    pub async fn record_risks_identified(&self, risks: &[Risk]) {
        self.risks_identified.inc_by(risks.len() as f64);

        let high_risks = risks.iter().filter(|r| matches!(r.impact, ImpactLevel::High | ImpactLevel::VeryHigh)).count();
        self.high_risks_identified.inc_by(high_risks as f64);

        // Update average risk score
        if !risks.is_empty() {
            let total_score: f64 = risks.iter().map(|r| r.probability * self.impact_to_weight(&r.impact)).sum();
            let average_score = total_score / risks.len() as f64;
            self.risk_score_average.set(average_score);
        }
    }

    /// Update metrics when risks are mitigated
    pub async fn record_risks_mitigated(&self, count: usize) {
        self.risks_mitigated.inc_by(count as f64);
    }

    /// Update metrics for resource allocation
    pub async fn record_resource_allocation(&self, success: bool, cost: f64) {
        if success {
            self.resource_allocations_successful.inc();
        } else {
            self.resource_allocation_failures.inc();
        }
        self.resource_cost_total.inc_by(cost);
    }

    /// Update metrics for planning performance
    pub async fn record_planning_duration(&self, duration_seconds: f64) {
        self.planning_duration_seconds.observe(duration_seconds);
    }

    /// Update metrics for task decomposition performance
    pub async fn record_task_decomposition_duration(&self, duration_seconds: f64) {
        self.task_decomposition_duration_seconds.observe(duration_seconds);
    }

    /// Update metrics for risk assessment performance
    pub async fn record_risk_assessment_duration(&self, duration_seconds: f64) {
        self.risk_assessment_duration_seconds.observe(duration_seconds);
    }

    /// Update metrics for resource coordination performance
    pub async fn record_resource_coordination_duration(&self, duration_seconds: f64) {
        self.resource_coordination_duration_seconds.observe(duration_seconds);
    }

    /// Update progress metrics
    pub async fn update_progress_metrics(&self, progress_percentage: f64, overdue_count: u64) {
        self.plan_progress_percentage.set(progress_percentage);
        self.overdue_tasks.inc_by(overdue_count);
    }

    /// Update error metrics
    pub async fn record_error(&self, error_type: &str) {
        match error_type {
            "planning" => self.planning_errors.inc(),
            "validation" => self.validation_errors.inc(),
            "integration" => self.integration_errors.inc(),
            _ => warn!("Unknown error type: {}", error_type),
        }
    }

    /// Convert impact level to numeric weight
    fn impact_to_weight(&self, impact: &ImpactLevel) -> f64 {
        match impact {
            ImpactLevel::VeryLow => 1.0,
            ImpactLevel::Low => 2.0,
            ImpactLevel::Medium => 3.0,
            ImpactLevel::High => 4.0,
            ImpactLevel::VeryHigh => 5.0,
        }
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanningMetricsSnapshot {
    pub timestamp: chrono::DateTime<Utc>,
    pub metrics_text: String,
    pub goals_received: f64,
    pub plans_created: f64,
    pub plans_active: f64,
    pub tasks_completed: f64,
    pub tasks_failed: f64,
    pub risks_identified: f64,
    pub high_risks_identified: f64,
    pub resource_allocations_successful: f64,
    pub resource_allocation_failures: f64,
    pub average_risk_score: f64,
    pub average_plan_progress: f64,
}

impl Default for PlanningMetricsSnapshot {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            metrics_text: String::new(),
            goals_received: 0.0,
            plans_created: 0.0,
            plans_active: 0.0,
            tasks_completed: 0.0,
            tasks_failed: 0.0,
            risks_identified: 0.0,
            high_risks_identified: 0.0,
            resource_allocations_successful: 0.0,
            resource_allocation_failures: 0.0,
            average_risk_score: 0.0,
            average_plan_progress: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = PlanningMetrics::new().await;
        assert!(metrics.is_ok());
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
    async fn test_risk_metrics() {
        let metrics = PlanningMetrics::new().await.unwrap();

        let risks = vec![
            Risk {
                id: Uuid::new_v4(),
                title: "Test Risk 1".to_string(),
                description: "Test".to_string(),
                probability: 0.8,
                impact: ImpactLevel::High,
                mitigation_strategy: None,
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Risk {
                id: Uuid::new_v4(),
                title: "Test Risk 2".to_string(),
                description: "Test".to_string(),
                probability: 0.6,
                impact: ImpactLevel::Medium,
                mitigation_strategy: None,
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        metrics.record_risks_identified(&risks).await;

        let snapshot = metrics.snapshot().await.unwrap();
        assert_eq!(snapshot.risks_identified, 2.0);
        assert_eq!(snapshot.high_risks_identified, 1.0);
        assert_eq!(snapshot.average_risk_score, 3.4); // (0.8*4 + 0.6*3) / 2
    }
}