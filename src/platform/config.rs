use anyhow::{Context, Result};
use config::{Config, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::agents::{AgentConfig, AgentType};

const DEFAULT_CONFIG_PATH: &str = "configs/platform.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlatformConfig {
    pub metadata: MetadataSettings,
    pub observability: ObservabilitySettings,
    pub audit: AuditSettings,
    pub rate_limiting: RateLimitingSettings,
    pub agents: HashMap<String, AgentSettings>,
    pub inference: InferenceSettings,
    pub training: TrainingSettings,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            metadata: MetadataSettings::default(),
            observability: ObservabilitySettings::default(),
            audit: AuditSettings::default(),
            rate_limiting: RateLimitingSettings::default(),
            agents: HashMap::new(),
            inference: InferenceSettings::default(),
            training: TrainingSettings::default(),
        }
    }
}

impl PlatformConfig {
    pub fn load() -> Result<Self> {
        Self::load_from_path(None::<PathBuf>)
    }

    pub fn load_from_path<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let mut builder = Config::builder().add_source(
            File::from(PathBuf::from(DEFAULT_CONFIG_PATH))
                .format(FileFormat::Toml)
                .required(false),
        );

        if let Some(path) = path {
            builder = builder.add_source(
                File::from(path.as_ref())
                    .format(FileFormat::Toml)
                    .required(true),
            );
        }

        builder = builder.add_source(Environment::with_prefix("CHIMERA").separator("__"));

        let config = builder.build()?;
        config
            .try_deserialize()
            .context("invalid platform configuration")
    }

    pub fn agent_catalog(&self) -> HashMap<String, AgentConfig> {
        self.agents
            .iter()
            .map(|(name, settings)| (name.clone(), settings.to_agent_config(name)))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetadataSettings {
    pub name: String,
    pub environment: String,
    pub cluster: String,
    pub node: String,
}

impl Default for MetadataSettings {
    fn default() -> Self {
        Self {
            name: "project-chimera".to_string(),
            environment: "development".to_string(),
            cluster: "local".to_string(),
            node: "node-0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ObservabilitySettings {
    pub log_level: String,
    pub metrics_port: u16,
    pub enable_metrics: bool,
}

impl Default for ObservabilitySettings {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            metrics_port: 9090,
            enable_metrics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditSettings {
    pub log_path: String,
    pub retention_days: u32,
}

impl Default for AuditSettings {
    fn default() -> Self {
        Self {
            log_path: "logs/audit.log".to_string(),
            retention_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitingSettings {
    pub default: RateLimitRule,
    pub endpoints: HashMap<String, RateLimitRule>,
}

impl Default for RateLimitingSettings {
    fn default() -> Self {
        Self {
            default: RateLimitRule::default(),
            endpoints: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitRule {
    pub requests: u32,
    pub window_seconds: u64,
    pub burst: u32,
}

impl Default for RateLimitRule {
    fn default() -> Self {
        Self {
            requests: 1000,
            window_seconds: 60,
            burst: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSettings {
    pub agent_type: AgentType,
    pub model_path: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub system_prompt: String,
    pub capabilities: Vec<String>,
    pub max_concurrent_requests: usize,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            agent_type: AgentType::General,
            model_path: "models/base".to_string(),
            max_tokens: 512,
            temperature: 0.7,
            system_prompt: "You are a helpful assistant.".to_string(),
            capabilities: vec!["text_generation".to_string()],
            max_concurrent_requests: 4,
        }
    }
}

impl AgentSettings {
    fn to_agent_config(&self, name: &str) -> AgentConfig {
        AgentConfig {
            model_path: self.model_path.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            system_prompt: self.system_prompt.clone(),
            agent_name: name.to_string(),
            max_concurrent_requests: self.max_concurrent_requests,
            capabilities: self.capabilities.clone(),
            agent_type: self.agent_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceSettings {
    pub batch_size: usize,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub repetition_penalty: f32,
}

impl Default for InferenceSettings {
    fn default() -> Self {
        Self {
            batch_size: 1,
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            repetition_penalty: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrainingSettings {
    pub base_model: String,
    pub output_dir: String,
    pub learning_rate: f32,
    pub num_epochs: usize,
    pub save_steps: usize,
}

impl Default for TrainingSettings {
    fn default() -> Self {
        Self {
            base_model: "mistralai/Mistral-7B-Instruct-v0.1".to_string(),
            output_dir: "./models".to_string(),
            learning_rate: 1e-4,
            num_epochs: 3,
            save_steps: 500,
        }
    }
}
