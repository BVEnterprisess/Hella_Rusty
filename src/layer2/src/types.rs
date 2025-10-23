//! # Layer 2 Types - Core Data Structures for Planning System
//!
//! This module defines the core data structures used throughout the Layer 2 planning system.
//! These types represent goals, plans, tasks, and related concepts in the strategic planning
//! and task decomposition process.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use validator::Validate;

/// Priority levels for goals and tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical = 1,
    High = 2,
    Medium = 3,
    Low = 4,
}

/// Resource requirements for tasks
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResourceRequirement {
    pub resource_type: String,
    pub quantity: f64,
    pub unit: String,
    pub max_cost_per_hour: Option<f64>,
    pub preferred_providers: Vec<String>,
}

/// Constraints that must be satisfied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Uuid,
    pub description: String,
    pub constraint_type: ConstraintType,
    pub parameters: HashMap<String, String>,
    pub is_mandatory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Time,
    Resource,
    Dependency,
    Quality,
    Security,
    Cost,
}

/// Success criteria for goals and tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub id: Uuid,
    pub description: String,
    pub metric: String,
    pub target_value: f64,
    pub comparison_operator: ComparisonOperator,
    pub weight: f64, // For weighted scoring
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    WithinRange,
}

/// High-level goal definition
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Goal {
    pub id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    pub priority: Priority,
    pub deadline: DateTime<Utc>,
    pub constraints: Vec<Constraint>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Individual task within a plan
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Task {
    pub id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    pub priority: Priority,
    pub estimated_duration_hours: f64,
    pub resource_requirements: Vec<ResourceRequirement>,
    pub dependencies: Vec<Uuid>, // Task IDs this task depends on
    pub constraints: Vec<Constraint>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub assigned_layer: Option<String>, // Which layer should execute this task
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Comprehensive plan containing goals and tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub goal: Goal,
    pub tasks: Vec<Task>,
    pub risks: Vec<Risk>,
    pub resource_allocations: Vec<ResourceAllocation>,
    pub timeline: Timeline,
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Risk assessment for plans and tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub probability: f64, // 0.0 to 1.0
    pub impact: ImpactLevel,
    pub mitigation_strategy: Option<String>,
    pub owner: Option<String>,
    pub status: RiskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskStatus {
    Identified,
    Mitigated,
    Accepted,
    Transferred,
}

/// Resource allocation for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub id: Uuid,
    pub task_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
    pub quantity: f64,
    pub unit: String,
    pub cost_per_hour: f64,
    pub allocated_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
    pub status: AllocationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    Allocated,
    InUse,
    Released,
    Failed,
}

/// Timeline for plan execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: Uuid,
    pub phases: Vec<Phase>,
    pub milestones: Vec<Milestone>,
    pub critical_path: Vec<Uuid>, // Task IDs in critical path
    pub total_duration_hours: f64,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tasks: Vec<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub dependencies: Vec<Uuid>, // Phase IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub due_date: DateTime<Utc>,
    pub tasks: Vec<Uuid>,
    pub is_critical: bool,
}

/// Plan execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    Draft,
    Approved,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
    Failed,
}

/// Progress tracking for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub progress_percentage: f64,
    pub actual_duration_hours: Option<f64>,
    pub actual_cost: Option<f64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
    Failed,
}

/// Feedback from execution layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFeedback {
    pub plan_id: Uuid,
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub progress_percentage: f64,
    pub actual_duration_hours: Option<f64>,
    pub actual_cost: Option<f64>,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub metrics: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

/// Health status for service components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { issues: Vec<String> },
    Unhealthy { issues: Vec<String> },
}

/// Planning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConfig {
    pub max_concurrent_tasks: usize,
    pub default_task_timeout_hours: f64,
    pub risk_threshold_probability: f64,
    pub risk_threshold_impact: ImpactLevel,
    pub resource_buffer_percentage: f64,
    pub planning_horizon_days: u32,
    pub auto_approve_low_risk_plans: bool,
    pub enable_continuous_replanning: bool,
    pub replanning_interval_minutes: u32,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            default_task_timeout_hours: 8.0,
            risk_threshold_probability: 0.7,
            risk_threshold_impact: ImpactLevel::High,
            resource_buffer_percentage: 0.1,
            planning_horizon_days: 30,
            auto_approve_low_risk_plans: true,
            enable_continuous_replanning: true,
            replanning_interval_minutes: 60,
        }
    }
}

/// Error types for the planning system
#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Invalid goal: {0}")]
    InvalidGoal(String),

    #[error("Invalid task: {0}")]
    InvalidTask(String),

    #[error("Resource allocation failed: {0}")]
    ResourceAllocationFailed(String),

    #[error("Risk assessment failed: {0}")]
    RiskAssessmentFailed(String),

    #[error("Plan validation failed: {0}")]
    PlanValidationFailed(String),

    #[error("Integration error: {0}")]
    IntegrationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for planning operations
pub type PlanningResult<T> = Result<T, PlanningError>;

#[cfg(test)]
mod tests {
    use super::*;

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
}