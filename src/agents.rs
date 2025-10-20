//! Agent management and orchestration
//!
//! This module handles the creation, management, and coordination of AI agents
//! within the Chimera platform.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    General,
    CodeGeneration,
    DataAnalysis,
    Creative,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Busy,
    Idle,
    Error,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub config: AgentConfig,
    pub metrics: AgentMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model_path: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub requests_processed: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f32,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

pub struct AgentManager {
    agents: HashMap<String, Agent>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    pub fn list_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    pub fn get_agents_by_type(&self, agent_type: &AgentType) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|agent| std::mem::discriminant(&agent.agent_type) == std::mem::discriminant(agent_type))
            .collect()
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent {
            id: Uuid::new_v4().to_string(),
            name: "test_agent".to_string(),
            agent_type: AgentType::General,
            status: AgentStatus::Idle,
            capabilities: vec!["text_generation".to_string()],
            config: AgentConfig {
                model_path: "models/test".to_string(),
                max_tokens: 512,
                temperature: 0.7,
                system_prompt: "You are a helpful assistant.".to_string(),
            },
            metrics: AgentMetrics {
                requests_processed: 0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                last_activity: chrono::Utc::now(),
            },
        };

        assert_eq!(agent.name, "test_agent");
        assert!(matches!(agent.agent_type, AgentType::General));
    }

    #[test]
    fn test_agent_manager() {
        let mut manager = AgentManager::new();

        let agent = Agent {
            id: "test-id".to_string(),
            name: "test_agent".to_string(),
            agent_type: AgentType::CodeGeneration,
            status: AgentStatus::Active,
            capabilities: vec!["code_gen".to_string()],
            config: AgentConfig {
                model_path: "models/codellama".to_string(),
                max_tokens: 1024,
                temperature: 0.3,
                system_prompt: "You are a code generation assistant.".to_string(),
            },
            metrics: AgentMetrics {
                requests_processed: 0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                last_activity: chrono::Utc::now(),
            },
        };

        manager.register_agent(agent);

        assert_eq!(manager.list_agents().len(), 1);
        assert_eq!(manager.get_agent("test-id").unwrap().name, "test_agent");
    }
}