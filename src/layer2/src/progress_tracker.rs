//! # Progress Tracker - Plan Execution Monitoring and Adjustment
//!
//! The Progress Tracker monitors the execution of plans and tasks, tracks progress,
//! and provides feedback for plan adjustments. It integrates with execution layers
//! to maintain real-time visibility into plan execution status.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Progress tracker for monitoring plan execution
pub struct ProgressTracker {
    plan_progress: Arc<RwLock<HashMap<Uuid, PlanProgress>>>,
    task_progress: Arc<RwLock<HashMap<Uuid, TaskProgress>>>,
    progress_history: Arc<RwLock<Vec<ProgressSnapshot>>>,
    replanning_thresholds: ReplanningThresholds,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub async fn new() -> Result<Self> {
        Ok(Self {
            plan_progress: Arc::new(RwLock::new(HashMap::new())),
            task_progress: Arc::new(RwLock::new(HashMap::new())),
            progress_history: Arc::new(RwLock::new(Vec::new())),
            replanning_thresholds: ReplanningThresholds::default(),
        })
    }

    /// Update plan progress based on execution feedback
    pub async fn update_progress(&self, mut plan: Plan, feedback: ExecutionFeedback) -> Result<Plan> {
        info!("Updating progress for plan: {} (task: {})", plan.id, feedback.task_id);

        // Update task progress
        let task_progress = TaskProgress {
            task_id: feedback.task_id,
            status: feedback.status.clone(),
            progress_percentage: feedback.progress_percentage,
            actual_duration_hours: feedback.actual_duration_hours,
            actual_cost: feedback.actual_cost,
            started_at: None, // Would be set when task starts
            completed_at: if matches!(feedback.status, TaskStatus::Completed) {
                Some(feedback.timestamp)
            } else {
                None
            },
            notes: feedback.issues.clone(),
            updated_at: feedback.timestamp,
        };

        self.task_progress.write().await.insert(feedback.task_id, task_progress);

        // Update plan-level progress
        let plan_progress = self.calculate_plan_progress(&plan).await?;
        self.plan_progress.write().await.insert(plan.id, plan_progress);

        // Check if replanning is needed
        if self.requires_replanning(&plan, &feedback).await? {
            warn!("Replanning required for plan: {}", plan.id);
            plan.status = PlanStatus::OnHold;
            plan.metadata.insert("replanning_required".to_string(), "true".to_string());
            plan.metadata.insert("replanning_reason".to_string(), self.get_replanning_reason(&feedback));
        }

        // Update plan metrics
        plan.metadata.insert("last_progress_update".to_string(), feedback.timestamp.to_rfc3339());
        plan.updated_at = Utc::now();

        // Store progress snapshot
        let snapshot = ProgressSnapshot {
            plan_id: plan.id,
            timestamp: Utc::now(),
            overall_progress: plan_progress.overall_percentage,
            completed_tasks: plan_progress.completed_tasks,
            total_tasks: plan_progress.total_tasks,
            issues: feedback.issues,
            metrics: feedback.metrics,
        };

        self.progress_history.write().await.push(snapshot);

        info!("Progress updated for plan: {} - {}% complete", plan.id, plan_progress.overall_percentage);
        Ok(plan)
    }

    /// Calculate overall plan progress
    async fn calculate_plan_progress(&self, plan: &Plan) -> Result<PlanProgress> {
        let task_progresses = self.task_progress.read().await;
        let mut completed_tasks = 0;
        let mut total_progress = 0.0;
        let mut total_weight = 0.0;

        for task in &plan.tasks {
            if let Some(progress) = task_progresses.get(&task.id) {
                total_progress += progress.progress_percentage;
                total_weight += 1.0;

                if matches!(progress.status, TaskStatus::Completed) {
                    completed_tasks += 1;
                }
            } else {
                // Task not started yet
                total_weight += 1.0;
            }
        }

        let overall_percentage = if total_weight > 0.0 {
            total_progress / total_weight
        } else {
            0.0
        };

        let status = if overall_percentage >= 100.0 {
            PlanStatus::Completed
        } else if overall_percentage > 0.0 {
            PlanStatus::InProgress
        } else {
            PlanStatus::Draft
        };

        Ok(PlanProgress {
            plan_id: plan.id,
            status,
            overall_percentage,
            completed_tasks,
            total_tasks: plan.tasks.len(),
            on_schedule: self.is_on_schedule(plan).await?,
            within_budget: self.is_within_budget(plan).await?,
            issues: self.collect_current_issues(plan).await?,
            last_updated: Utc::now(),
        })
    }

    /// Check if replanning is required based on thresholds
    async fn requires_replanning(&self, plan: &Plan, feedback: &ExecutionFeedback) -> Result<bool> {
        // Check if task is significantly behind schedule
        if let Some(actual_duration) = feedback.actual_duration_hours {
            let task = plan.tasks.iter().find(|t| t.id == feedback.task_id);
            if let Some(task) = task {
                let expected_duration = task.estimated_duration_hours;
                let duration_variance = (actual_duration - expected_duration) / expected_duration;

                if duration_variance > self.replanning_thresholds.duration_variance_threshold {
                    return Ok(true);
                }
            }
        }

        // Check if cost is significantly over budget
        if let Some(actual_cost) = feedback.actual_cost {
            let task = plan.tasks.iter().find(|t| t.id == feedback.task_id);
            if let Some(task) = task {
                let estimated_cost = task.estimated_duration_hours * 0.5; // Simplified cost calculation
                let cost_variance = (actual_cost - estimated_cost) / estimated_cost;

                if cost_variance > self.replanning_thresholds.cost_variance_threshold {
                    return Ok(true);
                }
            }
        }

        // Check for critical issues
        if feedback.issues.iter().any(|issue| issue.to_lowercase().contains("critical") ||
                                             issue.to_lowercase().contains("blocker")) {
            return Ok(true);
        }

        // Check if too many tasks are blocked
        let task_progresses = self.task_progress.read().await;
        let blocked_tasks = task_progresses.values()
            .filter(|p| matches!(p.status, TaskStatus::Blocked))
            .count();

        if blocked_tasks as f64 > plan.tasks.len() as f64 * self.replanning_thresholds.blocked_task_threshold {
            return Ok(true);
        }

        Ok(false)
    }

    /// Get replanning reason
    fn get_replanning_reason(&self, feedback: &ExecutionFeedback) -> String {
        if let Some(actual_duration) = feedback.actual_duration_hours {
            return format!("Task duration exceeded threshold: {:.1}h", actual_duration);
        }

        if let Some(actual_cost) = feedback.actual_cost {
            return format!("Task cost exceeded threshold: ${:.2}", actual_cost);
        }

        if feedback.issues.iter().any(|issue| issue.to_lowercase().contains("critical")) {
            return "Critical issues detected".to_string();
        }

        "Multiple factors require replanning".to_string()
    }

    /// Check if plan is on schedule
    async fn is_on_schedule(&self, plan: &Plan) -> Result<bool> {
        let now = Utc::now();
        let plan_age = now - plan.created_at;
        let plan_age_hours = plan_age.num_hours() as f64;

        let progress = self.calculate_plan_progress(plan).await?;
        let expected_progress = (plan_age_hours / plan.timeline.total_duration_hours) * 100.0;

        Ok(progress.overall_percentage >= expected_progress * 0.9) // Within 10% of expected
    }

    /// Check if plan is within budget
    async fn is_within_budget(&self, plan: &Plan) -> Result<bool> {
        let task_progresses = self.task_progress.read().await;
        let mut actual_cost = 0.0;
        let mut estimated_cost = 0.0;

        for task in &plan.tasks {
            if let Some(progress) = task_progresses.get(&task.id) {
                if let Some(cost) = progress.actual_cost {
                    actual_cost += cost;
                }
            }
            estimated_cost += task.estimated_duration_hours * 0.5; // Simplified cost calculation
        }

        Ok(actual_cost <= estimated_cost * 1.1) // Within 10% of budget
    }

    /// Collect current issues from all tasks
    async fn collect_current_issues(&self, plan: &Plan) -> Result<Vec<String>> {
        let task_progresses = self.task_progress.read().await;
        let mut issues = Vec::new();

        for task in &plan.tasks {
            if let Some(progress) = task_progresses.get(&task.id) {
                issues.extend(progress.notes.iter().cloned());

                if matches!(progress.status, TaskStatus::Blocked) {
                    issues.push(format!("Task {} is blocked", task.title));
                }
            }
        }

        Ok(issues)
    }

    /// Check if risk reassessment is needed
    pub async fn requires_risk_reassessment(&self, plan: &Plan) -> Result<bool> {
        let progress = self.calculate_plan_progress(plan).await?;

        // Reassess if behind schedule
        if !progress.on_schedule {
            return Ok(true);
        }

        // Reassess if over budget
        if !progress.within_budget {
            return Ok(true);
        }

        // Reassess if many issues
        if progress.issues.len() > 5 {
            return Ok(true);
        }

        Ok(false)
    }

    /// Get progress for a specific plan
    pub async fn get_plan_progress(&self, plan_id: Uuid) -> Result<Option<PlanProgress>> {
        let progress = self.plan_progress.read().await;
        Ok(progress.get(&plan_id).cloned())
    }

    /// Get progress for a specific task
    pub async fn get_task_progress(&self, task_id: Uuid) -> Result<Option<TaskProgress>> {
        let progress = self.task_progress.read().await;
        Ok(progress.get(&task_id).cloned())
    }

    /// Get progress history for a plan
    pub async fn get_progress_history(&self, plan_id: Uuid) -> Result<Vec<ProgressSnapshot>> {
        let history = self.progress_history.read().await;
        Ok(history.iter()
            .filter(|snapshot| snapshot.plan_id == plan_id)
            .cloned()
            .collect())
    }

    /// Generate progress report
    pub async fn generate_progress_report(&self, plan_id: Uuid) -> Result<ProgressReport> {
        let plan_progress = self.get_plan_progress(plan_id).await?
            .ok_or_else(|| PlanningError::InvalidGoal(format!("Plan not found: {}", plan_id)))?;

        let task_progresses: Vec<TaskProgress> = {
            let tasks = self.task_progress.read().await;
            tasks.values()
                .filter(|p| plan_id.to_string().contains(&p.task_id.to_string())) // Simplified filtering
                .cloned()
                .collect()
        };

        let history = self.get_progress_history(plan_id).await?;

        Ok(ProgressReport {
            plan_id,
            generated_at: Utc::now(),
            plan_progress,
            task_progresses,
            history,
            recommendations: self.generate_recommendations(&plan_progress, &task_progresses).await?,
        })
    }

    /// Generate recommendations based on progress
    async fn generate_recommendations(
        &self,
        plan_progress: &PlanProgress,
        task_progresses: &[TaskProgress],
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        if !plan_progress.on_schedule {
            recommendations.push("Plan is behind schedule. Consider resource reallocation or timeline adjustment.".to_string());
        }

        if !plan_progress.within_budget {
            recommendations.push("Plan is over budget. Review resource usage and consider cost optimization.".to_string());
        }

        let blocked_tasks: Vec<_> = task_progresses.iter()
            .filter(|p| matches!(p.status, TaskStatus::Blocked))
            .collect();

        if !blocked_tasks.is_empty() {
            recommendations.push(format!("{} tasks are blocked. Address blocking issues to maintain progress.", blocked_tasks.len()));
        }

        let failed_tasks: Vec<_> = task_progresses.iter()
            .filter(|p| matches!(p.status, TaskStatus::Failed))
            .collect();

        if !failed_tasks.is_empty() {
            recommendations.push(format!("{} tasks have failed. Review failures and consider retry or alternative approaches.", failed_tasks.len()));
        }

        if plan_progress.overall_percentage > 80.0 && plan_progress.issues.is_empty() {
            recommendations.push("Plan is on track for successful completion.".to_string());
        }

        Ok(recommendations)
    }

    /// Health check for the progress tracker
    pub async fn health_check(&self) -> Result<()> {
        let plan_count = self.plan_progress.read().await.len();
        let task_count = self.task_progress.read().await.len();
        let history_count = self.progress_history.read().await.len();

        if plan_count > 1000 {
            warn!("Large number of plans being tracked: {}", plan_count);
        }

        if task_count > 10000 {
            warn!("Large number of tasks being tracked: {}", task_count);
        }

        if history_count > 50000 {
            warn!("Large progress history: {} snapshots", history_count);
        }

        Ok(())
    }
}

/// Plan-level progress information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanProgress {
    pub plan_id: Uuid,
    pub status: PlanStatus,
    pub overall_percentage: f64,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub on_schedule: bool,
    pub within_budget: bool,
    pub issues: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Progress snapshot for historical tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressSnapshot {
    pub plan_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub overall_progress: f64,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub issues: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

/// Comprehensive progress report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressReport {
    pub plan_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub plan_progress: PlanProgress,
    pub task_progresses: Vec<TaskProgress>,
    pub history: Vec<ProgressSnapshot>,
    pub recommendations: Vec<String>,
}

/// Replanning thresholds configuration
#[derive(Debug, Clone)]
struct ReplanningThresholds {
    duration_variance_threshold: f64, // e.g., 0.5 = 50% over estimate
    cost_variance_threshold: f64,     // e.g., 0.3 = 30% over budget
    blocked_task_threshold: f64,      // e.g., 0.2 = 20% of tasks blocked
}

impl Default for ReplanningThresholds {
    fn default() -> Self {
        Self {
            duration_variance_threshold: 0.5,
            cost_variance_threshold: 0.3,
            blocked_task_threshold: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new().await;
        assert!(tracker.is_ok());
    }

    #[tokio::test]
    async fn test_progress_update() {
        let tracker = ProgressTracker::new().await.unwrap();

        let plan = Plan {
            id: Uuid::new_v4(),
            goal: Goal {
                id: Uuid::new_v4(),
                title: "Test Goal".to_string(),
                description: "Test".to_string(),
                priority: Priority::High,
                deadline: Utc::now() + chrono::Duration::hours(24),
                constraints: Vec::new(),
                success_criteria: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            },
            tasks: vec![Task {
                id: Uuid::new_v4(),
                title: "Test Task".to_string(),
                description: "Test".to_string(),
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
            }],
            risks: Vec::new(),
            resource_allocations: Vec::new(),
            timeline: Timeline {
                id: Uuid::new_v4(),
                phases: Vec::new(),
                milestones: Vec::new(),
                critical_path: Vec::new(),
                total_duration_hours: 4.0,
                start_date: Utc::now(),
                end_date: Utc::now() + chrono::Duration::hours(4),
            },
            status: PlanStatus::InProgress,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

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

        let updated_plan = tracker.update_progress(plan, feedback).await;
        assert!(updated_plan.is_ok());
    }
}