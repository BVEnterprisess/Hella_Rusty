//! # Task Decomposer - Goal Decomposition Engine
//!
//! The Task Decomposer is responsible for breaking down high-level goals into actionable,
//! manageable tasks. It uses various strategies including domain knowledge, pattern
//! recognition, and dependency analysis to create optimal task breakdowns.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Task decomposition strategies
#[derive(Debug, Clone)]
pub enum DecompositionStrategy {
    /// Hierarchical decomposition (top-down)
    Hierarchical,
    /// Functional decomposition (by system components)
    Functional,
    /// Temporal decomposition (by time phases)
    Temporal,
    /// Resource-based decomposition (by required resources)
    ResourceBased,
    /// Risk-based decomposition (by risk mitigation)
    RiskBased,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

/// Task decomposer that breaks down goals into actionable tasks
pub struct TaskDecomposer {
    strategies: Vec<DecompositionStrategy>,
    domain_knowledge: HashMap<String, Vec<TaskTemplate>>,
    max_task_duration_hours: f64,
    min_task_duration_hours: f64,
}

impl TaskDecomposer {
    /// Create a new task decomposer
    pub async fn new() -> Result<Self> {
        let mut decomposer = Self {
            strategies: vec![
                DecompositionStrategy::Hierarchical,
                DecompositionStrategy::Functional,
                DecompositionStrategy::Temporal,
            ],
            domain_knowledge: Self::load_domain_knowledge().await?,
            max_task_duration_hours: 8.0,
            min_task_duration_hours: 0.5,
        };

        info!("Task decomposer initialized with {} strategies", decomposer.strategies.len());
        Ok(decomposer)
    }

    /// Decompose a goal into actionable tasks
    pub async fn decompose_goal(&self, goal: &Goal) -> Result<Vec<Task>> {
        info!("Decomposing goal: {} ({})", goal.title, goal.id);

        let mut all_tasks = Vec::new();

        // Apply each strategy and combine results
        for strategy in &self.strategies {
            let strategy_tasks = self.apply_strategy(strategy, goal).await?;
            all_tasks.extend(strategy_tasks);
        }

        // Remove duplicates and merge similar tasks
        let tasks = self.deduplicate_and_merge_tasks(all_tasks).await?;

        // Validate task dependencies
        self.validate_task_dependencies(&tasks).await?;

        // Optimize task order and dependencies
        let optimized_tasks = self.optimize_task_order(tasks).await?;

        info!("Goal decomposed into {} tasks", optimized_tasks.len());
        Ok(optimized_tasks)
    }

    /// Apply a specific decomposition strategy
    async fn apply_strategy(&self, strategy: &DecompositionStrategy, goal: &Goal) -> Result<Vec<Task>> {
        match strategy {
            DecompositionStrategy::Hierarchical => {
                self.apply_hierarchical_decomposition(goal).await
            }
            DecompositionStrategy::Functional => {
                self.apply_functional_decomposition(goal).await
            }
            DecompositionStrategy::Temporal => {
                self.apply_temporal_decomposition(goal).await
            }
            DecompositionStrategy::ResourceBased => {
                self.apply_resource_based_decomposition(goal).await
            }
            DecompositionStrategy::RiskBased => {
                self.apply_risk_based_decomposition(goal).await
            }
            DecompositionStrategy::Hybrid => {
                self.apply_hybrid_decomposition(goal).await
            }
        }
    }

    /// Apply hierarchical decomposition (top-down approach)
    async fn apply_hierarchical_decomposition(&self, goal: &Goal) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Analyze goal description for key components
        let components = self.extract_goal_components(&goal.description).await?;

        for (i, component) in components.iter().enumerate() {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("{}. {}", i + 1, component),
                description: format!("Execute {} for goal: {}", component, goal.title),
                priority: self.calculate_task_priority(goal.priority, i, components.len()),
                estimated_duration_hours: self.estimate_task_duration(component).await?,
                resource_requirements: self.estimate_resource_requirements(component).await?,
                dependencies: if i > 0 { vec![tasks[i-1].id] } else { Vec::new() },
                constraints: self.inherit_constraints(&goal.constraints, component),
                success_criteria: self.create_task_success_criteria(component),
                assigned_layer: self.assign_task_to_layer(component),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("decomposition_strategy".to_string(), "hierarchical".to_string()),
                    ("component".to_string(), component.clone()),
                ]),
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Apply functional decomposition (by system components)
    async fn apply_functional_decomposition(&self, goal: &Goal) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Identify functional areas from goal
        let functions = self.identify_functions(&goal.description).await?;

        for function in functions {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Implement {}", function),
                description: format!("Implement {} functionality for {}", function, goal.title),
                priority: goal.priority,
                estimated_duration_hours: self.estimate_function_duration(&function).await?,
                resource_requirements: self.get_function_resource_requirements(&function).await?,
                dependencies: Vec::new(), // Will be calculated later
                constraints: goal.constraints.clone(),
                success_criteria: self.create_function_success_criteria(&function),
                assigned_layer: self.assign_function_to_layer(&function),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("decomposition_strategy".to_string(), "functional".to_string()),
                    ("function".to_string(), function.clone()),
                ]),
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Apply temporal decomposition (by time phases)
    async fn apply_temporal_decomposition(&self, goal: &Goal) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Define standard project phases
        let phases = vec![
            "Planning & Analysis",
            "Design & Architecture",
            "Implementation",
            "Testing & Validation",
            "Deployment & Integration",
        ];

        let total_duration = (goal.deadline - Utc::now()).num_hours() as f64;
        let phase_duration = total_duration / phases.len() as f64;

        for (i, phase) in phases.iter().enumerate() {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("{} Phase", phase),
                description: format!("Complete {} phase for {}", phase, goal.title),
                priority: if i == phases.len() - 1 { Priority::Critical } else { goal.priority },
                estimated_duration_hours: phase_duration,
                resource_requirements: self.get_phase_resource_requirements(phase).await?,
                dependencies: if i > 0 { vec![tasks[i-1].id] } else { Vec::new() },
                constraints: goal.constraints.clone(),
                success_criteria: self.create_phase_success_criteria(phase),
                assigned_layer: self.assign_phase_to_layer(phase),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("decomposition_strategy".to_string(), "temporal".to_string()),
                    ("phase".to_string(), phase.to_string()),
                ]),
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Apply resource-based decomposition
    async fn apply_resource_based_decomposition(&self, goal: &Goal) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Group by resource types
        let resource_types = self.identify_resource_types(&goal.description).await?;

        for resource_type in resource_types {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Resource Setup: {}", resource_type),
                description: format!("Set up and configure {} resources", resource_type),
                priority: Priority::High,
                estimated_duration_hours: 2.0,
                resource_requirements: vec![ResourceRequirement {
                    resource_type: resource_type.clone(),
                    quantity: 1.0,
                    unit: "instance".to_string(),
                    max_cost_per_hour: None,
                    preferred_providers: Vec::new(),
                }],
                dependencies: Vec::new(),
                constraints: goal.constraints.clone(),
                success_criteria: self.create_resource_success_criteria(&resource_type),
                assigned_layer: Some("layer8".to_string()), // Resource management layer
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("decomposition_strategy".to_string(), "resource_based".to_string()),
                    ("resource_type".to_string(), resource_type.clone()),
                ]),
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Apply risk-based decomposition
    async fn apply_risk_based_decomposition(&self, goal: &Goal) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Create risk mitigation tasks
        let risk_mitigations = vec![
            "Risk Assessment",
            "Contingency Planning",
            "Risk Monitoring Setup",
            "Mitigation Implementation",
        ];

        for mitigation in risk_mitigations {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Risk Mitigation: {}", mitigation),
                description: format!("Implement {} for goal risks", mitigation),
                priority: Priority::High,
                estimated_duration_hours: 1.0,
                resource_requirements: Vec::new(),
                dependencies: Vec::new(),
                constraints: goal.constraints.clone(),
                success_criteria: self.create_risk_success_criteria(mitigation),
                assigned_layer: Some("layer3".to_string()), // Validation layer
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("decomposition_strategy".to_string(), "risk_based".to_string()),
                    ("risk_mitigation".to_string(), mitigation.to_string()),
                ]),
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Apply hybrid decomposition combining multiple strategies
    async fn apply_hybrid_decomposition(&self, goal: &Goal) -> Result<Vec<Task>> {
        let mut all_tasks = Vec::new();

        // Get tasks from multiple strategies
        let hierarchical_tasks = self.apply_hierarchical_decomposition(goal).await?;
        let functional_tasks = self.apply_functional_decomposition(goal).await?;
        let temporal_tasks = self.apply_temporal_decomposition(goal).await?;

        // Combine and deduplicate
        all_tasks.extend(hierarchical_tasks);
        all_tasks.extend(functional_tasks);
        all_tasks.extend(temporal_tasks);

        self.deduplicate_and_merge_tasks(all_tasks).await
    }

    /// Extract components from goal description
    async fn extract_goal_components(&self, description: &str) -> Result<Vec<String>> {
        let mut components = Vec::new();

        // Simple keyword-based extraction (could be enhanced with NLP)
        let keywords = [
            "implement", "create", "build", "develop", "design", "analyze",
            "test", "deploy", "integrate", "optimize", "configure", "setup"
        ];

        let desc_lower = description.to_lowercase();
        for keyword in &keywords {
            if desc_lower.contains(&keyword.to_lowercase()) {
                components.push(keyword.to_uppercase().to_string());
            }
        }

        // If no keywords found, create generic components
        if components.is_empty() {
            components = vec![
                "Analysis".to_string(),
                "Implementation".to_string(),
                "Testing".to_string(),
                "Deployment".to_string(),
            ];
        }

        Ok(components)
    }

    /// Identify functional areas from description
    async fn identify_functions(&self, description: &str) -> Result<Vec<String>> {
        let functions = vec![
            "Data Processing".to_string(),
            "API Development".to_string(),
            "User Interface".to_string(),
            "Database Operations".to_string(),
            "Security Implementation".to_string(),
            "Performance Optimization".to_string(),
        ];

        let desc_lower = description.to_lowercase();
        let mut identified = Vec::new();

        for function in functions {
            let func_lower = function.to_lowercase();
            if desc_lower.contains(&func_lower) {
                identified.push(function);
            }
        }

        if identified.is_empty() {
            identified.push("Core Functionality".to_string());
        }

        Ok(identified)
    }

    /// Identify resource types from description
    async fn identify_resource_types(&self, description: &str) -> Result<Vec<String>> {
        let resource_types = vec![
            "GPU".to_string(),
            "CPU".to_string(),
            "Memory".to_string(),
            "Storage".to_string(),
            "Network".to_string(),
        ];

        let desc_lower = description.to_lowercase();
        let mut identified = Vec::new();

        for resource_type in resource_types {
            if desc_lower.contains(&resource_type.to_lowercase()) {
                identified.push(resource_type);
            }
        }

        if identified.is_empty() {
            identified.push("Compute".to_string());
        }

        Ok(identified)
    }

    /// Calculate task priority based on position and goal priority
    fn calculate_task_priority(&self, goal_priority: Priority, position: usize, total: usize) -> Priority {
        match goal_priority {
            Priority::Critical => Priority::High,
            Priority::High => Priority::Medium,
            Priority::Medium => Priority::Low,
            Priority::Low => Priority::Low,
        }
    }

    /// Estimate task duration based on component
    async fn estimate_task_duration(&self, component: &str) -> Result<f64> {
        let base_duration = match component.to_lowercase().as_str() {
            "analysis" => 2.0,
            "design" => 4.0,
            "implementation" => 6.0,
            "testing" => 3.0,
            "deployment" => 2.0,
            _ => 4.0,
        };

        Ok(base_duration.min(self.max_task_duration_hours).max(self.min_task_duration_hours))
    }

    /// Estimate resource requirements for a component
    async fn estimate_resource_requirements(&self, component: &str) -> Result<Vec<ResourceRequirement>> {
        let requirements = match component.to_lowercase().as_str() {
            "gpu" | "training" | "inference" => vec![
                ResourceRequirement {
                    resource_type: "GPU".to_string(),
                    quantity: 1.0,
                    unit: "GPU".to_string(),
                    max_cost_per_hour: Some(2.0),
                    preferred_providers: vec!["AWS".to_string(), "GCP".to_string()],
                }
            ],
            "cpu" | "processing" => vec![
                ResourceRequirement {
                    resource_type: "CPU".to_string(),
                    quantity: 4.0,
                    unit: "cores".to_string(),
                    max_cost_per_hour: Some(0.5),
                    preferred_providers: vec!["AWS".to_string()],
                }
            ],
            _ => vec![
                ResourceRequirement {
                    resource_type: "CPU".to_string(),
                    quantity: 2.0,
                    unit: "cores".to_string(),
                    max_cost_per_hour: Some(0.25),
                    preferred_providers: Vec::new(),
                }
            ],
        };

        Ok(requirements)
    }

    /// Load domain knowledge for task templates
    async fn load_domain_knowledge() -> Result<HashMap<String, Vec<TaskTemplate>>> {
        // This would typically load from a configuration file or database
        let mut knowledge = HashMap::new();

        knowledge.insert("machine_learning".to_string(), vec![
            TaskTemplate {
                name: "Data Preprocessing".to_string(),
                description: "Clean and prepare data for training".to_string(),
                estimated_duration_hours: 4.0,
                resource_requirements: vec!["CPU".to_string()],
                dependencies: Vec::new(),
            },
            TaskTemplate {
                name: "Model Training".to_string(),
                description: "Train the machine learning model".to_string(),
                estimated_duration_hours: 8.0,
                resource_requirements: vec!["GPU".to_string()],
                dependencies: vec!["Data Preprocessing".to_string()],
            },
        ]);

        Ok(knowledge)
    }

    /// Deduplicate and merge similar tasks
    async fn deduplicate_and_merge_tasks(&self, tasks: Vec<Task>) -> Result<Vec<Task>> {
        let mut unique_tasks = Vec::new();
        let mut seen_titles = std::collections::HashSet::new();

        for task in tasks {
            if !seen_titles.contains(&task.title) {
                unique_tasks.push(task);
                seen_titles.insert(task.title.clone());
            } else {
                // Merge with existing task
                if let Some(existing) = unique_tasks.iter_mut().find(|t| t.title == task.title) {
                    existing.estimated_duration_hours += task.estimated_duration_hours;
                    existing.resource_requirements.extend(task.resource_requirements);
                }
            }
        }

        Ok(unique_tasks)
    }

    /// Validate task dependencies
    async fn validate_task_dependencies(&self, tasks: &[Task]) -> Result<()> {
        for task in tasks {
            for dep_id in &task.dependencies {
                if !tasks.iter().any(|t| t.id == *dep_id) {
                    return Err(PlanningError::InvalidTask(
                        format!("Task {} has invalid dependency: {}", task.id, dep_id)
                    ));
                }
            }
        }
        Ok(())
    }

    /// Optimize task order and dependencies
    async fn optimize_task_order(&self, mut tasks: Vec<Task>) -> Result<Vec<Task>> {
        // Simple topological sort based on dependencies
        let mut result = Vec::new();
        let mut remaining: std::collections::HashMap<Uuid, Task> = tasks.into_iter().map(|t| (t.id, t)).collect();
        let mut processed = std::collections::HashSet::new();

        while !remaining.is_empty() {
            let mut made_progress = false;

            for (id, task) in &remaining.clone() {
                let can_process = task.dependencies.iter().all(|dep_id| processed.contains(dep_id));

                if can_process {
                    result.push(task.clone());
                    remaining.remove(id);
                    processed.insert(*id);
                    made_progress = true;
                }
            }

            if !made_progress {
                return Err(PlanningError::InvalidTask(
                    "Circular dependency detected during optimization".to_string()
                ));
            }
        }

        Ok(result)
    }

    /// Health check for the task decomposer
    pub async fn health_check(&self) -> Result<()> {
        if self.strategies.is_empty() {
            return Err(PlanningError::ConfigurationError(
                "No decomposition strategies configured".to_string()
            ));
        }

        if self.domain_knowledge.is_empty() {
            warn!("No domain knowledge loaded - using generic decomposition");
        }

        Ok(())
    }

    // Helper methods for various estimations and assignments
    async fn estimate_function_duration(&self, function: &str) -> Result<f64> {
        Ok(match function.to_lowercase().as_str() {
            "data processing" => 6.0,
            "api development" => 8.0,
            "user interface" => 12.0,
            "database operations" => 4.0,
            "security implementation" => 6.0,
            "performance optimization" => 4.0,
            _ => 6.0,
        })
    }

    async fn get_function_resource_requirements(&self, function: &str) -> Result<Vec<ResourceRequirement>> {
        Ok(match function.to_lowercase().as_str() {
            "data processing" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 8.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(1.0),
                preferred_providers: Vec::new(),
            }],
            "api development" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 4.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.5),
                preferred_providers: Vec::new(),
            }],
            _ => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 2.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.25),
                preferred_providers: Vec::new(),
            }],
        })
    }

    async fn get_phase_resource_requirements(&self, phase: &str) -> Result<Vec<ResourceRequirement>> {
        Ok(match phase {
            "Planning & Analysis" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 2.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.25),
                preferred_providers: Vec::new(),
            }],
            "Design & Architecture" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 4.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.5),
                preferred_providers: Vec::new(),
            }],
            "Implementation" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 8.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(1.0),
                preferred_providers: Vec::new(),
            }],
            "Testing & Validation" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 4.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.5),
                preferred_providers: Vec::new(),
            }],
            "Deployment & Integration" => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 2.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.25),
                preferred_providers: Vec::new(),
            }],
            _ => vec![ResourceRequirement {
                resource_type: "CPU".to_string(),
                quantity: 2.0,
                unit: "cores".to_string(),
                max_cost_per_hour: Some(0.25),
                preferred_providers: Vec::new(),
            }],
        })
    }

    fn inherit_constraints(&self, goal_constraints: &[Constraint], component: &str) -> Vec<Constraint> {
        goal_constraints.iter()
            .filter(|c| self.constraint_applies_to_component(c, component))
            .cloned()
            .collect()
    }

    fn constraint_applies_to_component(&self, constraint: &Constraint, component: &str) -> bool {
        // Simple logic - could be enhanced
        match constraint.constraint_type {
            ConstraintType::Resource => component.to_lowercase().contains("resource"),
            ConstraintType::Time => true,
            ConstraintType::Quality => component.to_lowercase().contains("test") || component.to_lowercase().contains("validation"),
            _ => true,
        }
    }

    fn create_task_success_criteria(&self, component: &str) -> Vec<SuccessCriterion> {
        vec![SuccessCriterion {
            id: Uuid::new_v4(),
            description: format!("{} completed successfully", component),
            metric: "completion".to_string(),
            target_value: 1.0,
            comparison_operator: ComparisonOperator::Equals,
            weight: 1.0,
        }]
    }

    fn create_function_success_criteria(&self, function: &str) -> Vec<SuccessCriterion> {
        vec![SuccessCriterion {
            id: Uuid::new_v4(),
            description: format!("{} implemented and tested", function),
            metric: "functionality".to_string(),
            target_value: 1.0,
            comparison_operator: ComparisonOperator::Equals,
            weight: 1.0,
        }]
    }

    fn create_phase_success_criteria(&self, phase: &str) -> Vec<SuccessCriterion> {
        vec![SuccessCriterion {
            id: Uuid::new_v4(),
            description: format!("{} phase completed", phase),
            metric: "phase_completion".to_string(),
            target_value: 1.0,
            comparison_operator: ComparisonOperator::Equals,
            weight: 1.0,
        }]
    }

    fn create_resource_success_criteria(&self, resource_type: &str) -> Vec<SuccessCriterion> {
        vec![SuccessCriterion {
            id: Uuid::new_v4(),
            description: format!("{} resources allocated and configured", resource_type),
            metric: "resource_allocation".to_string(),
            target_value: 1.0,
            comparison_operator: ComparisonOperator::Equals,
            weight: 1.0,
        }]
    }

    fn create_risk_success_criteria(&self, mitigation: &str) -> Vec<SuccessCriterion> {
        vec![SuccessCriterion {
            id: Uuid::new_v4(),
            description: format!("{} implemented", mitigation),
            metric: "risk_mitigation".to_string(),
            target_value: 1.0,
            comparison_operator: ComparisonOperator::Equals,
            weight: 1.0,
        }]
    }

    fn assign_task_to_layer(&self, component: &str) -> Option<String> {
        match component.to_lowercase().as_str() {
            "analysis" | "planning" => Some("layer2".to_string()),
            "design" | "architecture" => Some("layer2".to_string()),
            "implementation" | "development" => Some("layer4".to_string()),
            "testing" | "validation" => Some("layer3".to_string()),
            "deployment" | "integration" => Some("layer4".to_string()),
            "resource" | "gpu" | "cpu" => Some("layer8".to_string()),
            _ => Some("layer4".to_string()),
        }
    }

    fn assign_function_to_layer(&self, function: &str) -> Option<String> {
        match function.to_lowercase().as_str() {
            "data processing" => Some("layer4".to_string()),
            "api development" => Some("layer4".to_string()),
            "user interface" => Some("layer4".to_string()),
            "database operations" => Some("layer4".to_string()),
            "security implementation" => Some("layer3".to_string()),
            "performance optimization" => Some("layer5".to_string()),
            _ => Some("layer4".to_string()),
        }
    }

    fn assign_phase_to_layer(&self, phase: &str) -> Option<String> {
        match phase {
            "Planning & Analysis" => Some("layer2".to_string()),
            "Design & Architecture" => Some("layer2".to_string()),
            "Implementation" => Some("layer4".to_string()),
            "Testing & Validation" => Some("layer3".to_string()),
            "Deployment & Integration" => Some("layer4".to_string()),
            _ => Some("layer4".to_string()),
        }
    }
}

/// Task template for domain knowledge
#[derive(Debug, Clone)]
struct TaskTemplate {
    name: String,
    description: String,
    estimated_duration_hours: f64,
    resource_requirements: Vec<String>,
    dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_decomposer_creation() {
        let decomposer = TaskDecomposer::new().await;
        assert!(decomposer.is_ok());
    }

    #[tokio::test]
    async fn test_goal_decomposition() {
        let decomposer = TaskDecomposer::new().await.unwrap();

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

        let tasks = decomposer.decompose_goal(&goal).await;
        assert!(tasks.is_ok());
        let tasks = tasks.unwrap();
        assert!(!tasks.is_empty());
    }

    #[tokio::test]
    async fn test_component_extraction() {
        let decomposer = TaskDecomposer::new().await.unwrap();
        let components = decomposer.extract_goal_components("Implement and test the system").await.unwrap();
        assert!(!components.is_empty());
    }
}