//! # Planner - Main Planning Engine
//!
//! The Planner is the central component of Layer 2 that orchestrates the strategic planning
//! process. It coordinates between task decomposition, resource allocation, risk assessment,
//! and progress tracking to create comprehensive execution plans.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Main planner that orchestrates the planning process
pub struct Planner {
    config: PlanningConfig,
    plans: Arc<RwLock<HashMap<Uuid, Plan>>>,
    task_dependencies: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl Planner {
    /// Create a new planner instance
    pub async fn new() -> Result<Self> {
        Ok(Self {
            config: PlanningConfig::default(),
            plans: Arc::new(RwLock::new(HashMap::new())),
            task_dependencies: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a comprehensive plan from a goal and tasks
    pub async fn create_plan(
        &self,
        goal: Goal,
        tasks: Vec<Task>,
        risks: Vec<Risk>,
    ) -> Result<Plan> {
        info!("Creating plan for goal: {}", goal.id);

        // Validate inputs
        self.validate_plan_inputs(&goal, &tasks, &risks).await?;

        // Calculate timeline
        let timeline = self.calculate_timeline(&tasks, &goal.deadline).await?;

        // Identify critical path
        let critical_path = self.calculate_critical_path(&tasks).await?;

        // Create resource allocations (placeholder - will be filled by ResourceCoordinator)
        let resource_allocations = Vec::new();

        // Create phases
        let phases = self.create_phases(&tasks, &timeline).await?;

        // Create milestones
        let milestones = self.create_milestones(&tasks, &goal).await?;

        let plan = Plan {
            id: Uuid::new_v4(),
            goal,
            tasks,
            risks,
            resource_allocations,
            timeline: Timeline {
                id: Uuid::new_v4(),
                phases,
                milestones,
                critical_path,
                total_duration_hours: timeline.total_duration_hours,
                start_date: timeline.start_date,
                end_date: timeline.end_date,
            },
            status: PlanStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        // Store the plan
        self.plans.write().await.insert(plan.id, plan.clone());

        info!("Plan created successfully: {}", plan.id);
        Ok(plan)
    }

    /// Get a plan by ID
    pub async fn get_plan(&self, plan_id: Uuid) -> Result<Plan> {
        let plans = self.plans.read().await;
        plans.get(&plan_id)
            .cloned()
            .ok_or_else(|| PlanningError::InvalidGoal(format!("Plan not found: {}", plan_id)))
    }

    /// Update plan risks
    pub async fn update_risks(&self, mut plan: Plan, risks: Vec<Risk>) -> Result<Plan> {
        plan.risks = risks;
        plan.updated_at = Utc::now();

        self.plans.write().await.insert(plan.id, plan.clone());
        Ok(plan)
    }

    /// Approve a plan for execution
    pub async fn approve_plan(&self, plan_id: Uuid) -> Result<Plan> {
        let mut plans = self.plans.write().await;
        let plan = plans.get_mut(&plan_id)
            .ok_or_else(|| PlanningError::InvalidGoal(format!("Plan not found: {}", plan_id)))?;

        plan.status = PlanStatus::Approved;
        plan.updated_at = Utc::now();

        info!("Plan approved: {}", plan_id);
        Ok(plan.clone())
    }

    /// Cancel a plan
    pub async fn cancel_plan(&self, plan_id: Uuid, reason: String) -> Result<Plan> {
        let mut plans = self.plans.write().await;
        let plan = plans.get_mut(&plan_id)
            .ok_or_else(|| PlanningError::InvalidGoal(format!("Plan not found: {}", plan_id)))?;

        plan.status = PlanStatus::Cancelled;
        plan.updated_at = Utc::now();
        plan.metadata.insert("cancellation_reason".to_string(), reason);

        info!("Plan cancelled: {}", plan_id);
        Ok(plan.clone())
    }

    /// Validate plan inputs
    async fn validate_plan_inputs(&self, goal: &Goal, tasks: &[Task], risks: &[Risk]) -> Result<()> {
        // Validate goal
        if goal.title.is_empty() {
            return Err(PlanningError::InvalidGoal("Goal title cannot be empty".to_string()));
        }

        if goal.deadline <= Utc::now() {
            return Err(PlanningError::InvalidGoal("Goal deadline must be in the future".to_string()));
        }

        // Validate tasks
        if tasks.is_empty() {
            return Err(PlanningError::InvalidTask("Plan must contain at least one task".to_string()));
        }

        // Check for circular dependencies
        if self.has_circular_dependencies(tasks).await? {
            return Err(PlanningError::InvalidTask("Circular dependencies detected in tasks".to_string()));
        }

        // Validate risks
        for risk in risks {
            if risk.probability < 0.0 || risk.probability > 1.0 {
                return Err(PlanningError::RiskAssessmentFailed(
                    format!("Invalid risk probability: {}", risk.probability)
                ));
            }
        }

        Ok(())
    }

    /// Calculate the timeline for task execution
    async fn calculate_timeline(&self, tasks: &[Task], deadline: &DateTime<Utc>) -> Result<TimelineInfo> {
        let mut task_durations: HashMap<Uuid, f64> = HashMap::new();
        let mut earliest_start: HashMap<Uuid, f64> = HashMap::new();
        let mut latest_start: HashMap<Uuid, f64> = HashMap::new();

        // Calculate earliest start times (forward pass)
        for task in tasks {
            let mut max_dependency_time = 0.0;

            for &dep_id in &task.dependencies {
                if let Some(dep_duration) = task_durations.get(&dep_id) {
                    max_dependency_time = max_dependency_time.max(*dep_duration);
                }
            }

            let duration = task.estimated_duration_hours;
            task_durations.insert(task.id, duration);
            earliest_start.insert(task.id, max_dependency_time);
        }

        // Calculate latest start times (backward pass)
        let total_duration = task_durations.values().sum();
        let project_deadline = deadline.timestamp() as f64 / 3600.0; // Convert to hours

        for task in tasks.iter().rev() {
            let duration = task.estimated_duration_hours;
            let mut min_successor_time = project_deadline - total_duration;

            // Find tasks that depend on this task
            for other_task in tasks {
                if other_task.dependencies.contains(&task.id) {
                    if let Some(successor_start) = earliest_start.get(&other_task.id) {
                        min_successor_time = min_successor_time.min(successor_start - duration);
                    }
                }
            }

            latest_start.insert(task.id, min_successor_time);
        }

        let start_date = Utc::now();
        let end_date = *deadline;

        Ok(TimelineInfo {
            total_duration_hours: total_duration,
            start_date,
            end_date,
            task_earliest_start: earliest_start,
            task_latest_start: latest_start,
        })
    }

    /// Calculate the critical path through tasks
    async fn calculate_critical_path(&self, tasks: &[Task]) -> Result<Vec<Uuid>> {
        let timeline_info = self.calculate_timeline(tasks, &Utc::now()).await?;

        let mut critical_path = Vec::new();

        for task in tasks {
            let earliest = timeline_info.task_earliest_start.get(&task.id).unwrap_or(&0.0);
            let latest = timeline_info.task_latest_start.get(&task.id).unwrap_or(&0.0);

            // Task is on critical path if earliest and latest start times are equal
            if (earliest - latest).abs() < 0.001 {
                critical_path.push(task.id);
            }
        }

        Ok(critical_path)
    }

    /// Check for circular dependencies in tasks
    async fn has_circular_dependencies(&self, tasks: &[Task]) -> Result<bool> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        // Build dependency graph
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for task in tasks {
            graph.insert(task.id, task.dependencies.clone());
        }

        // Check each task for cycles
        for task in tasks {
            if self.has_cycle(&graph, task.id, &mut visited, &mut rec_stack)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Helper function to detect cycles using DFS
    fn has_cycle(
        &self,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        node: Uuid,
        visited: &mut std::collections::HashSet<Uuid>,
        rec_stack: &mut std::collections::HashSet<Uuid>,
    ) -> Result<bool> {
        visited.insert(node);
        rec_stack.insert(node);

        if let Some(dependencies) = graph.get(&node) {
            for &dep in dependencies {
                if !visited.contains(&dep) {
                    if self.has_cycle(graph, dep, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&dep) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(&node);
        Ok(false)
    }

    /// Create execution phases from tasks
    async fn create_phases(&self, tasks: &[Task], timeline: &TimelineInfo) -> Result<Vec<Phase>> {
        // Group tasks by assigned layer or create logical phases
        let mut phases = Vec::new();
        let mut processed_tasks = std::collections::HashSet::new();

        // Phase 1: Discovery and planning tasks
        let mut discovery_tasks = Vec::new();
        for task in tasks {
            if task.assigned_layer.as_ref().map_or(false, |layer| layer.contains("discovery") || layer.contains("planning")) {
                discovery_tasks.push(task.id);
                processed_tasks.insert(task.id);
            }
        }

        if !discovery_tasks.is_empty() {
            phases.push(Phase {
                id: Uuid::new_v4(),
                name: "Discovery & Planning".to_string(),
                description: "Initial discovery and planning activities".to_string(),
                tasks: discovery_tasks,
                start_date: timeline.start_date,
                end_date: timeline.start_date + chrono::Duration::hours(8),
                dependencies: Vec::new(),
            });
        }

        // Phase 2: Core execution tasks
        let mut execution_tasks = Vec::new();
        for task in tasks {
            if !processed_tasks.contains(&task.id) {
                execution_tasks.push(task.id);
                processed_tasks.insert(task.id);
            }
        }

        if !execution_tasks.is_empty() {
            phases.push(Phase {
                id: Uuid::new_v4(),
                name: "Execution".to_string(),
                description: "Main execution activities".to_string(),
                tasks: execution_tasks,
                start_date: timeline.start_date + chrono::Duration::hours(8),
                end_date: timeline.end_date - chrono::Duration::hours(4),
                dependencies: phases.last().map(|p| vec![p.id]).unwrap_or_default(),
            });
        }

        // Phase 3: Validation and completion
        phases.push(Phase {
            id: Uuid::new_v4(),
            name: "Validation & Completion".to_string(),
            description: "Final validation and completion activities".to_string(),
            tasks: Vec::new(),
            start_date: timeline.end_date - chrono::Duration::hours(4),
            end_date: timeline.end_date,
            dependencies: phases.iter().map(|p| p.id).collect(),
        });

        Ok(phases)
    }

    /// Create milestones from tasks and goal
    async fn create_milestones(&self, tasks: &[Task], goal: &Goal) -> Result<Vec<Milestone>> {
        let mut milestones = Vec::new();

        // Goal-based milestones
        milestones.push(Milestone {
            id: Uuid::new_v4(),
            name: format!("{} - Planning Complete", goal.title),
            description: "Initial planning phase completed".to_string(),
            due_date: Utc::now() + chrono::Duration::hours(2),
            tasks: Vec::new(),
            is_critical: true,
        });

        // Task-based milestones (every 25% of tasks)
        let total_tasks = tasks.len();
        let milestone_intervals = if total_tasks > 4 {
            vec![total_tasks / 4, total_tasks / 2, 3 * total_tasks / 4, total_tasks]
        } else {
            vec![total_tasks]
        };

        for (i, &task_count) in milestone_intervals.iter().enumerate() {
            let task_subset: Vec<Uuid> = tasks.iter().take(task_count).map(|t| t.id).collect();

            milestones.push(Milestone {
                id: Uuid::new_v4(),
                name: format!("Milestone {} - {}% Complete", i + 1, (task_count * 100) / total_tasks),
                description: format!("{} out of {} tasks completed", task_count, total_tasks),
                due_date: goal.deadline - chrono::Duration::hours((total_tasks - task_count) as i64),
                tasks: task_subset,
                is_critical: i == milestone_intervals.len() - 1,
            });
        }

        Ok(milestones)
    }

    /// Health check for the planner
    pub async fn health_check(&self) -> Result<()> {
        let plans = self.plans.read().await;
        if plans.len() > 1000 {
            warn!("Large number of plans in memory: {}", plans.len());
        }

        // Check for plans that have been in draft status too long
        let now = Utc::now();
        for plan in plans.values() {
            if matches!(plan.status, PlanStatus::Draft) {
                let age_hours = (now - plan.created_at).num_hours();
                if age_hours > 24 {
                    warn!("Plan {} has been in draft status for {} hours", plan.id, age_hours);
                }
            }
        }

        Ok(())
    }
}

/// Internal timeline calculation result
struct TimelineInfo {
    total_duration_hours: f64,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    task_earliest_start: HashMap<Uuid, f64>,
    task_latest_start: HashMap<Uuid, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_planner_creation() {
        let planner = Planner::new().await;
        assert!(planner.is_ok());
    }

    #[tokio::test]
    async fn test_plan_creation() {
        let planner = Planner::new().await.unwrap();

        let goal = Goal {
            id: Uuid::new_v4(),
            title: "Test Goal".to_string(),
            description: "Test goal description".to_string(),
            priority: Priority::High,
            deadline: Utc::now() + chrono::Duration::hours(24),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let tasks = vec![
            Task {
                id: Uuid::new_v4(),
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
            }
        ];

        let risks = Vec::new();

        let result = planner.create_plan(goal, tasks, risks).await;
        assert!(result.is_ok());
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
}