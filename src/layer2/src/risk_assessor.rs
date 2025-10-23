//! # Risk Assessor - Risk Analysis and Mitigation for Planning
//!
//! The Risk Assessor identifies, analyzes, and proposes mitigation strategies for risks
//! associated with plans and tasks. It provides comprehensive risk management throughout
//! the planning lifecycle.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Risk assessor for comprehensive risk management
pub struct RiskAssessor {
    risk_templates: HashMap<String, RiskTemplate>,
    risk_weights: RiskWeights,
    mitigation_strategies: HashMap<String, MitigationStrategy>,
}

impl RiskAssessor {
    /// Create a new risk assessor
    pub async fn new() -> Result<Self> {
        let assessor = Self {
            risk_templates: Self::load_risk_templates().await?,
            risk_weights: RiskWeights::default(),
            mitigation_strategies: Self::load_mitigation_strategies().await?,
        };

        info!("Risk assessor initialized with {} risk templates", assessor.risk_templates.len());
        Ok(assessor)
    }

    /// Assess risks for a goal
    pub async fn assess_risks(&self, goal: &Goal) -> Result<Vec<Risk>> {
        info!("Assessing risks for goal: {}", goal.id);

        let mut risks = Vec::new();

        // Technical risks
        risks.extend(self.assess_technical_risks(goal).await?);

        // Resource risks
        risks.extend(self.assess_resource_risks(goal).await?);

        // Timeline risks
        risks.extend(self.assess_timeline_risks(goal).await?);

        // Integration risks
        risks.extend(self.assess_integration_risks(goal).await?);

        // External risks
        risks.extend(self.assess_external_risks(goal).await?);

        // Sort risks by risk score (probability * impact)
        risks.sort_by(|a, b| {
            let score_a = self.calculate_risk_score(a);
            let score_b = self.calculate_risk_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        info!("Risk assessment completed: {} risks identified", risks.len());
        Ok(risks)
    }

    /// Reassess risks for an updated plan
    pub async fn reassess_risks(&self, plan: &Plan) -> Result<Vec<Risk>> {
        info!("Reassessing risks for plan: {}", plan.id);

        let mut risks = Vec::new();

        // Reassess based on current plan state
        risks.extend(self.reassess_plan_risks(plan).await?);

        // Check for new risks based on progress
        risks.extend(self.assess_progress_based_risks(plan).await?);

        // Update risk statuses based on mitigation progress
        for risk in &mut risks {
            risk.updated_at = Utc::now();
        }

        Ok(risks)
    }

    /// Assess technical risks
    async fn assess_technical_risks(&self, goal: &Goal) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        let description = goal.description.to_lowercase();

        // Complexity risk
        if description.contains("complex") || description.contains("sophisticated") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Technical Complexity".to_string(),
                description: "High technical complexity may impact implementation".to_string(),
                probability: 0.7,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Break down into smaller, manageable components".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Technology risk
        if description.contains("new") || description.contains("untested") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Technology Maturity".to_string(),
                description: "Use of new or untested technology increases uncertainty".to_string(),
                probability: 0.6,
                impact: ImpactLevel::Medium,
                mitigation_strategy: Some("Conduct technology validation and proof of concept".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Integration risk
        if description.contains("integrate") || description.contains("multiple") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Integration Complexity".to_string(),
                description: "Complex integrations may cause delays or failures".to_string(),
                probability: 0.5,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Plan integration testing early and allocate buffer time".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(risks)
    }

    /// Assess resource risks
    async fn assess_resource_risks(&self, goal: &Goal) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        // GPU availability risk
        if goal.description.to_lowercase().contains("gpu") || goal.description.to_lowercase().contains("training") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "GPU Resource Availability".to_string(),
                description: "Limited GPU availability may delay training tasks".to_string(),
                probability: 0.4,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Reserve GPU resources in advance and consider cloud alternatives".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Team availability risk
        if goal.description.to_lowercase().contains("team") || goal.description.to_lowercase().contains("staff") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Team Availability".to_string(),
                description: "Team members may not be available when needed".to_string(),
                probability: 0.3,
                impact: ImpactLevel::Medium,
                mitigation_strategy: Some("Cross-train team members and maintain resource buffer".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(risks)
    }

    /// Assess timeline risks
    async fn assess_timeline_risks(&self, goal: &Goal) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        let time_to_deadline = goal.deadline - Utc::now();
        let hours_to_deadline = time_to_deadline.num_hours();

        // Tight deadline risk
        if hours_to_deadline < 48 {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Tight Timeline".to_string(),
                description: "Very tight deadline increases risk of delays".to_string(),
                probability: 0.8,
                impact: ImpactLevel::VeryHigh,
                mitigation_strategy: Some("Consider deadline extension or scope reduction".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        } else if hours_to_deadline < 168 { // 1 week
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Compressed Timeline".to_string(),
                description: "Compressed timeline may not allow for unexpected delays".to_string(),
                probability: 0.5,
                impact: ImpactLevel::Medium,
                mitigation_strategy: Some("Add buffer time and prioritize critical path tasks".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(risks)
    }

    /// Assess integration risks
    async fn assess_integration_risks(&self, goal: &Goal) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        // Cross-layer integration risk
        if goal.description.to_lowercase().contains("layer") || goal.description.to_lowercase().contains("integration") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Cross-Layer Integration".to_string(),
                description: "Integration between different system layers may be complex".to_string(),
                probability: 0.6,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Plan integration testing and use well-defined APIs".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // External system integration risk
        if goal.description.to_lowercase().contains("external") || goal.description.to_lowercase().contains("third-party") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "External Dependencies".to_string(),
                description: "Dependence on external systems increases integration risk".to_string(),
                probability: 0.4,
                impact: ImpactLevel::Medium,
                mitigation_strategy: Some("Identify and test external dependencies early".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(risks)
    }

    /// Assess external risks
    async fn assess_external_risks(&self, goal: &Goal) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        // Security risk
        if goal.description.to_lowercase().contains("security") || goal.description.to_lowercase().contains("sensitive") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Security Vulnerabilities".to_string(),
                description: "Security requirements may introduce complexity and delays".to_string(),
                probability: 0.5,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Follow security best practices and conduct security reviews".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Compliance risk
        if goal.description.to_lowercase().contains("compliance") || goal.description.to_lowercase().contains("regulation") {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Regulatory Compliance".to_string(),
                description: "Compliance requirements may impact timeline and implementation".to_string(),
                probability: 0.3,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Ensure compliance requirements are understood and planned for".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(risks)
    }

    /// Reassess risks based on current plan state
    async fn reassess_plan_risks(&self, plan: &Plan) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        // Check for resource allocation risks
        let unallocated_tasks: Vec<_> = plan.tasks.iter()
            .filter(|task| task.resource_requirements.len() > 0 && plan.resource_allocations.iter().all(|alloc| alloc.task_id != task.id))
            .collect();

        if !unallocated_tasks.is_empty() {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Resource Allocation Gap".to_string(),
                description: format!("{} tasks have unallocated resources", unallocated_tasks.len()),
                probability: 0.8,
                impact: ImpactLevel::High,
                mitigation_strategy: Some("Allocate resources for all tasks before execution".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Check for dependency risks
        let circular_deps = self.detect_circular_dependencies(&plan.tasks).await?;
        if !circular_deps.is_empty() {
            risks.push(Risk {
                id: Uuid::new_v4(),
                title: "Circular Dependencies".to_string(),
                description: "Circular dependencies detected in task graph".to_string(),
                probability: 1.0,
                impact: ImpactLevel::VeryHigh,
                mitigation_strategy: Some("Resolve circular dependencies in task structure".to_string()),
                owner: None,
                status: RiskStatus::Identified,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(risks)
    }

    /// Assess risks based on execution progress
    async fn assess_progress_based_risks(&self, plan: &Plan) -> Result<Vec<Risk>> {
        let mut risks = Vec::new();

        // Check for tasks that are significantly behind schedule
        let now = Utc::now();
        for task in &plan.tasks {
            let task_age = now - task.created_at;
            let expected_progress = (task_age.num_hours() as f64 / task.estimated_duration_hours) * 100.0;

            if expected_progress > 50.0 {
                risks.push(Risk {
                    id: Uuid::new_v4(),
                    title: "Task Delay Risk".to_string(),
                    description: format!("Task '{}' may be behind schedule", task.title),
                    probability: 0.6,
                    impact: ImpactLevel::Medium,
                    mitigation_strategy: Some("Monitor task progress and provide additional resources if needed".to_string()),
                    owner: None,
                    status: RiskStatus::Identified,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
        }

        Ok(risks)
    }

    /// Detect circular dependencies in tasks
    async fn detect_circular_dependencies(&self, tasks: &[Task]) -> Result<Vec<Vec<Uuid>>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        // Build dependency graph
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for task in tasks {
            graph.insert(task.id, task.dependencies.clone());
        }

        // Check each task for cycles
        for task in tasks {
            if self.has_cycle(&graph, task.id, &mut visited, &mut rec_stack, &mut Vec::new)? {
                // Extract cycle (simplified)
                cycles.push(vec![task.id]);
            }
        }

        Ok(cycles)
    }

    /// Helper function to detect cycles using DFS
    fn has_cycle(
        &self,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        node: Uuid,
        visited: &mut std::collections::HashSet<Uuid>,
        rec_stack: &mut std::collections::HashSet<Uuid>,
        path: &mut Vec<Uuid>,
    ) -> Result<bool> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(dependencies) = graph.get(&node) {
            for &dep in dependencies {
                if !visited.contains(&dep) {
                    if self.has_cycle(graph, dep, visited, rec_stack, path)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&dep) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(&node);
        path.pop();
        Ok(false)
    }

    /// Calculate risk score (probability * impact weight)
    fn calculate_risk_score(&self, risk: &Risk) -> f64 {
        let impact_weight = match risk.impact {
            ImpactLevel::VeryLow => 1.0,
            ImpactLevel::Low => 2.0,
            ImpactLevel::Medium => 3.0,
            ImpactLevel::High => 4.0,
            ImpactLevel::VeryHigh => 5.0,
        };

        risk.probability * impact_weight
    }

    /// Load risk templates from configuration
    async fn load_risk_templates() -> Result<HashMap<String, RiskTemplate>> {
        let mut templates = HashMap::new();

        templates.insert("technical_complexity".to_string(), RiskTemplate {
            name: "Technical Complexity".to_string(),
            description: "High technical complexity may impact implementation".to_string(),
            base_probability: 0.6,
            base_impact: ImpactLevel::High,
            factors: vec!["complexity".to_string(), "sophisticated".to_string()],
        });

        templates.insert("resource_availability".to_string(), RiskTemplate {
            name: "Resource Availability".to_string(),
            description: "Required resources may not be available when needed".to_string(),
            base_probability: 0.4,
            base_impact: ImpactLevel::High,
            factors: vec!["gpu".to_string(), "specialized".to_string(), "limited".to_string()],
        });

        Ok(templates)
    }

    /// Load mitigation strategies
    async fn load_mitigation_strategies() -> Result<HashMap<String, MitigationStrategy>> {
        let mut strategies = HashMap::new();

        strategies.insert("resource_allocation".to_string(), MitigationStrategy {
            name: "Resource Allocation".to_string(),
            description: "Allocate resources in advance and maintain buffer capacity".to_string(),
            cost: 0.1,
            effectiveness: 0.8,
            timeline_impact: 0.05,
        });

        strategies.insert("parallel_execution".to_string(), MitigationStrategy {
            name: "Parallel Execution".to_string(),
            description: "Execute independent tasks in parallel to reduce timeline risk".to_string(),
            cost: 0.2,
            effectiveness: 0.7,
            timeline_impact: -0.1,
        });

        Ok(strategies)
    }

    /// Health check for the risk assessor
    pub async fn health_check(&self) -> Result<()> {
        if self.risk_templates.is_empty() {
            warn!("No risk templates loaded");
        }

        if self.mitigation_strategies.is_empty() {
            warn!("No mitigation strategies loaded");
        }

        Ok(())
    }
}

/// Risk template for common risk patterns
#[derive(Debug, Clone)]
struct RiskTemplate {
    name: String,
    description: String,
    base_probability: f64,
    base_impact: ImpactLevel,
    factors: Vec<String>,
}

/// Mitigation strategy information
#[derive(Debug, Clone)]
struct MitigationStrategy {
    name: String,
    description: String,
    cost: f64,           // Relative cost (0.0 to 1.0)
    effectiveness: f64,  // Effectiveness (0.0 to 1.0)
    timeline_impact: f64, // Impact on timeline (-1.0 to 1.0, negative means reduction)
}

/// Risk weights for different assessment factors
#[derive(Debug, Clone)]
struct RiskWeights {
    technical_weight: f64,
    resource_weight: f64,
    timeline_weight: f64,
    integration_weight: f64,
    external_weight: f64,
}

impl Default for RiskWeights {
    fn default() -> Self {
        Self {
            technical_weight: 0.3,
            resource_weight: 0.25,
            timeline_weight: 0.2,
            integration_weight: 0.15,
            external_weight: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_assessor_creation() {
        let assessor = RiskAssessor::new().await;
        assert!(assessor.is_ok());
    }

    #[tokio::test]
    async fn test_risk_assessment() {
        let assessor = RiskAssessor::new().await.unwrap();

        let goal = Goal {
            id: Uuid::new_v4(),
            title: "Complex ML Training".to_string(),
            description: "Implement complex machine learning training with GPU requirements".to_string(),
            priority: Priority::High,
            deadline: Utc::now() + chrono::Duration::hours(48),
            constraints: Vec::new(),
            success_criteria: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let risks = assessor.assess_risks(&goal).await;
        assert!(risks.is_ok());
        let risks = risks.unwrap();
        assert!(!risks.is_empty());
    }

    #[test]
    fn test_risk_score_calculation() {
        let assessor = RiskAssessor::new().unwrap();

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
}