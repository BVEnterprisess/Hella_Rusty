//! Configuration management utilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub ai: AIConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub model_path: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_port: u16,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/chimera".to_string(),
                max_connections: 20,
                timeout_seconds: 30,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 10,
            },
            ai: AIConfig {
                model_path: "./models/default".to_string(),
                max_tokens: 512,
                temperature: 0.7,
                batch_size: 1,
            },
            monitoring: MonitoringConfig {
                prometheus_port: 9090,
                log_level: "info".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080),
                workers: env::var("WORKERS").unwrap_or_else(|_| "4".to_string()).parse().unwrap_or(4),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/chimera".to_string()),
                max_connections: env::var("DB_MAX_CONNECTIONS").unwrap_or_else(|_| "20".to_string()).parse().unwrap_or(20),
                timeout_seconds: env::var("DB_TIMEOUT").unwrap_or_else(|_| "30".to_string()).parse().unwrap_or(30),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_connections: env::var("REDIS_MAX_CONNECTIONS").unwrap_or_else(|_| "10".to_string()).parse().unwrap_or(10),
            },
            ai: AIConfig {
                model_path: env::var("MODEL_PATH").unwrap_or_else(|_| "./models/default".to_string()),
                max_tokens: env::var("MAX_TOKENS").unwrap_or_else(|_| "512".to_string()).parse().unwrap_or(512),
                temperature: env::var("TEMPERATURE").unwrap_or_else(|_| "0.7".to_string()).parse().unwrap_or(0.7),
                batch_size: env::var("BATCH_SIZE").unwrap_or_else(|_| "1".to_string()).parse().unwrap_or(1),
            },
            monitoring: MonitoringConfig {
                prometheus_port: env::var("PROMETHEUS_PORT").unwrap_or_else(|_| "9090".to_string()).parse().unwrap_or(9090),
                log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            },
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }

        if self.database.max_connections == 0 {
            errors.push("Database max connections must be greater than 0".to_string());
        }

        if self.ai.max_tokens == 0 {
            errors.push("Max tokens must be greater than 0".to_string());
        }

        if self.ai.temperature < 0.0 || self.ai.temperature > 2.0 {
            errors.push("Temperature must be between 0.0 and 2.0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.ai.temperature, 0.7);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;

        let result = config.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Server port")));
    }

    #[test]
    fn test_env_config() {
        env::set_var("PORT", "9000");
        env::set_var("TEMPERATURE", "0.5");

        let config = AppConfig::from_env();
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.ai.temperature, 0.5);

        // Clean up
        env::remove_var("PORT");
        env::remove_var("TEMPERATURE");
    }
}