//! Agent management and orchestration
//!
//! This module handles the creation, management, and coordination of AI agents
//! within the Chimera platform.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentType {
    General,
    CodeGeneration,
    DataAnalysis,
    Creative,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub agent_name: String,
    pub max_concurrent_requests: usize,
    pub capabilities: Vec<String>,
    pub agent_type: AgentType,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model_path: "models/base".to_string(),
            max_tokens: 512,
            temperature: 0.7,
            system_prompt: "You are a helpful assistant.".to_string(),
            agent_name: "agent".to_string(),
            max_concurrent_requests: 4,
            capabilities: vec!["text_generation".to_string()],
            agent_type: AgentType::General,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub requests_processed: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f32,
    pub last_activity: SystemTime,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            requests_processed: 0,
            average_response_time_ms: 0.0,
            success_rate: 1.0,
            last_activity: SystemTime::now(),
        }
    }
}

#[derive(Clone, Default)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn from_catalog(catalog: HashMap<String, AgentConfig>) -> Self {
        let registry = Self::new();
        for (_name, config) in catalog {
            registry.register_agent(Agent {
                id: uuid::Uuid::new_v4().to_string(),
                name: config.agent_name.clone(),
                agent_type: config.agent_type.clone(),
                status: AgentStatus::Idle,
                capabilities: config.capabilities.clone(),
                config: config.clone(),
                metrics: AgentMetrics::default(),
            });
        }
        registry
    }

    pub fn register_agent(&self, agent: Agent) {
        let mut agents = self.agents.write();
        agents.insert(agent.id.clone(), agent);
    pub fn update_activity(&mut self, id: &str) {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.metrics.last_activity = SystemTime::now();
            agent.metrics.requests_processed += 1;
        }
    }

    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    pub fn update_activity(&self, id: &str) {
        let mut agents = self.agents.write();
        if let Some(agent) = agents.get_mut(id) {
            agent.metrics.last_activity = SystemTime::now();
            agent.metrics.requests_processed += 1;
        }
    }

    pub fn get_agent(&self, id: &str) -> Option<Agent> {
        let agents = self.agents.read();
        agents.get(id).cloned()
    pub fn get_agents_by_type(&self, agent_type: &AgentType) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|agent| {
                std::mem::discriminant(&agent.agent_type) == std::mem::discriminant(agent_type)
            })
            .collect()
    }

    pub fn list_agents(&self) -> Vec<Agent> {
        let agents = self.agents.read();
        agents.values().cloned().collect()
    }

    pub fn get_agents_by_type(&self, agent_type: AgentType) -> Vec<Agent> {
        let agents = self.agents.read();
        agents
            .values()
            .filter(|agent| agent.agent_type == agent_type)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;
    use uuid::Uuid;

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
                agent_name: "test_agent".to_string(),
                max_concurrent_requests: 4,
                capabilities: vec!["text_generation".to_string()],
                agent_type: AgentType::General,
            },
            metrics: AgentMetrics::default(),
            metrics: AgentMetrics {
                requests_processed: 0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                last_activity: SystemTime::now(),
            },
        };

        assert_eq!(agent.name, "test_agent");
        assert!(matches!(agent.agent_type, AgentType::General));
    }

    #[test]
    fn test_agent_manager() {
        let manager = AgentRegistry::new();

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
                agent_name: "test_agent".to_string(),
                max_concurrent_requests: 2,
                capabilities: vec!["code_gen".to_string()],
                agent_type: AgentType::CodeGeneration,
            },
            metrics: AgentMetrics::default(),
            metrics: AgentMetrics {
                requests_processed: 0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                last_activity: SystemTime::now(),
            },
        };

        manager.register_agent(agent);

        assert_eq!(manager.list_agents().len(), 1);
        assert_eq!(manager.get_agent("test-id").unwrap().name, "test_agent");

        manager.update_activity("test-id");
        let agent = manager.get_agent("test-id").unwrap();
        assert!(agent.metrics.requests_processed >= 1);
        assert!(agent
            .metrics
            .last_activity
            .duration_since(UNIX_EPOCH)
            .is_ok());
    }
}
