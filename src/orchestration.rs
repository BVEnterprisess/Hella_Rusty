//! High-level task orchestration and workflow management
//!
//! Coordinates multiple agents to work together on complex tasks,
//! manages task dependencies, and handles result aggregation.

use crate::agents::{Agent, AgentManager, AgentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: String,
    pub input: serde_json::Value,
    pub status: TaskStatus,
    pub assigned_agent: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: SystemTime,
    pub completed_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

pub struct TaskOrchestrator {
    agent_manager: AgentManager,
    pending_tasks: HashMap<String, Task>,
    active_tasks: HashMap<String, Task>,
}

impl TaskOrchestrator {
    pub fn new(agent_manager: AgentManager) -> Self {
        Self {
            agent_manager,
            pending_tasks: HashMap::new(),
            active_tasks: HashMap::new(),
        }
    }

    pub fn submit_task(&mut self, task_type: String, input: serde_json::Value) -> String {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            task_type,
            input,
            status: TaskStatus::Pending,
            assigned_agent: None,
            result: None,
            error: None,
            created_at: SystemTime::now(),
            completed_at: None,
        };

        self.pending_tasks.insert(task_id.clone(), task);
        task_id
    }

    pub fn get_task_status(&self, task_id: &str) -> Option<&Task> {
        self.pending_tasks
            .get(task_id)
            .or_else(|| self.active_tasks.get(task_id))
    }

    pub async fn process_tasks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Move pending tasks to active if agents are available
        let mut tasks_to_activate = Vec::new();

        for (task_id, task) in &self.pending_tasks {
            if let Some(agent) = self.find_suitable_agent(&task.task_type) {
                tasks_to_activate.push((task_id.clone(), task.clone(), agent.id.clone()));
            }
        }

        // Activate tasks
        for (task_id, mut task, agent_id) in tasks_to_activate {
            task.status = TaskStatus::Assigned;
            task.assigned_agent = Some(agent_id.clone());

            self.pending_tasks.remove(&task_id);
            self.active_tasks.insert(task_id.clone(), task);

            println!("Assigned task {} to agent {}", task_id, agent_id);
        }

        Ok(())
    }

    fn find_suitable_agent(&self, task_type: &str) -> Option<&Agent> {
        // Simple agent selection logic
        // In a real implementation, this would be more sophisticated

        // First try to find agents by type
        match task_type {
            "code_generation" => self
                .agent_manager
                .get_agents_by_type(&AgentType::CodeGeneration)
                .first()
                .copied(),
            "data_analysis" => self
                .agent_manager
                .get_agents_by_type(&AgentType::DataAnalysis)
                .first()
                .copied(),
            "creative" => self
                .agent_manager
                .get_agents_by_type(&AgentType::Creative)
                .first()
                .copied(),
            _ => self
                .agent_manager
                .get_agents_by_type(&AgentType::General)
                .first()
                .copied(),
        }
    }

    pub fn complete_task(&mut self, task_id: &str, result: serde_json::Value) {
        if let Some(task) = self.active_tasks.get_mut(task_id) {
            task.status = TaskStatus::Completed;
            task.result = Some(result);
            task.completed_at = Some(SystemTime::now());
        }
    }

    pub fn fail_task(&mut self, task_id: &str, error: String) {
        if let Some(task) = self.active_tasks.get_mut(task_id) {
            task.status = TaskStatus::Failed;
            task.error = Some(error);
            task.completed_at = Some(SystemTime::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::{AgentConfig, AgentMetrics};
    use std::time::SystemTime;

    fn create_test_agent(id: &str, agent_type: AgentType) -> Agent {
        Agent {
            id: id.to_string(),
            name: format!("agent_{}", id),
            agent_type,
            status: crate::agents::AgentStatus::Idle,
            capabilities: vec!["test".to_string()],
            config: AgentConfig {
                model_path: "test_model".to_string(),
                max_tokens: 512,
                temperature: 0.7,
                system_prompt: "Test prompt".to_string(),
            },
            metrics: AgentMetrics {
                requests_processed: 0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                last_activity: SystemTime::now(),
            },
        }
    }

    #[test]
    fn test_task_submission() {
        let mut manager = AgentManager::new();
        manager.register_agent(create_test_agent("1", AgentType::General));

        let mut orchestrator = TaskOrchestrator::new(manager);
        let task_id = orchestrator.submit_task(
            "test_task".to_string(),
            serde_json::json!({"input": "test"}),
        );

        assert!(!task_id.is_empty());
        assert_eq!(orchestrator.pending_tasks.len(), 1);
    }
}
