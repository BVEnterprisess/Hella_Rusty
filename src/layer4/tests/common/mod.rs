//! Common test utilities and helpers for Layer 4 tests
//!
//! This module provides shared testing infrastructure including:
//! - Mock WASM binaries
//! - Test fixtures and factories
//! - Assertion helpers
//! - Setup/teardown utilities

use chimera_layer4::*;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Create a valid mock WASM binary for testing
pub fn mock_wasm_binary() -> Vec<u8> {
    // Valid WASM magic number and version
    let mut binary = vec![
        0x00, 0x61, 0x73, 0x6D, // Magic number "\0asm"
        0x01, 0x00, 0x00, 0x00, // Version 1
    ];
    
    // Add some padding to make it a realistic size
    binary.extend_from_slice(&[0u8; 100]);
    
    binary
}

/// Create a test agent configuration with sensible defaults
pub fn test_agent_config() -> AgentConfig {
    AgentConfig {
        agent_id: Uuid::new_v4(),
        agent_type: "test_agent".to_string(),
        resource_quota: ResourceQuota {
            max_cpu_cores: 0.5,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        environment: HashMap::new(),
        parameters: HashMap::new(),
    }
}

/// Create a test task with specified priority
pub fn test_task_with_priority(priority: Priority) -> Task {
    Task {
        id: Uuid::new_v4(),
        priority,
        payload: serde_json::json!({"action": "test", "data": "test_data"}),
        created_at: SystemTime::now(),
        deadline: None,
        resource_quota: ResourceQuota::default(),
        source_layer: "test".to_string(),
        target_agent_type: "test_agent".to_string(),
        metadata: HashMap::new(),
    }
}

/// Create multiple test tasks with varying priorities
pub fn test_tasks(count: usize) -> Vec<Task> {
    let priorities = [
        Priority::Critical,
        Priority::High,
        Priority::Normal,
        Priority::Low,
        Priority::Background,
    ];
    
    (0..count)
        .map(|i| test_task_with_priority(priorities[i % priorities.len()]))
        .collect()
}

/// Create a test Layer4Config for testing
pub fn test_layer4_config() -> Layer4Config {
    Layer4Config {
        max_agents: 5,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.5,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 100,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 1,
        agent_timeout_secs: 5,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9091, // Use different port for tests
        debug_mode: true,
    }
}

/// Assert that an ExecutionResult represents success
pub fn assert_execution_success(result: &ExecutionResult) {
    assert!(result.success, "Expected successful execution, got error: {:?}", result.error);
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);
    assert!(result.execution_time_ms > 0, "Expected non-zero execution time");
}

/// Assert that an ExecutionResult represents failure
pub fn assert_execution_failure(result: &ExecutionResult) {
    assert!(!result.success, "Expected failed execution");
    assert!(result.error.is_some(), "Expected error message");
}

/// Wait for a condition with timeout
pub async fn wait_for<F>(mut condition: F, timeout_secs: u64) -> bool
where
    F: FnMut() -> bool,
{
    use tokio::time::{sleep, Duration};
    
    let start = SystemTime::now();
    let timeout = Duration::from_secs(timeout_secs);
    
    loop {
        if condition() {
            return true;
        }
        
        if start.elapsed().unwrap() > timeout {
            return false;
        }
        
        sleep(Duration::from_millis(100)).await;
    }
}

/// Create a test scheduler configuration
pub fn test_scheduler_config() -> SchedulerConfig {
    SchedulerConfig {
        max_queue_size: 100,
        max_retries: 3,
        retry_base_delay_secs: 1,
        retry_max_delay_secs: 10,
        retry_backoff_multiplier: 2.0,
        task_timeout_secs: 30,
        enable_preemption: true,
        dead_letter_queue_size: 50,
    }
}

/// Create a test executor configuration
pub fn test_executor_config() -> ExecutorConfig {
    ExecutorConfig {
        max_agents: 5,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.5,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        heartbeat_interval_secs: 1,
        agent_timeout_secs: 5,
        debug_mode: true,
    }
}

/// Create a test metrics configuration
pub fn test_metrics_config() -> MetricsConfig {
    MetricsConfig {
        prometheus_port: 9092, // Different port for tests
        collection_interval_secs: 1,
        enable_detailed_metrics: true,
        retention_secs: 60,
        enable_export: false, // Disable HTTP server in tests
    }
}

/// Macro to create parameterized tests
#[macro_export]
macro_rules! parameterized_test {
    ($test_name:ident, $test_fn:expr, $($input:expr),+) => {
        #[cfg(test)]
        mod $test_name {
            use super::*;
            
            $(
                #[tokio::test]
                async fn test() {
                    $test_fn($input).await;
                }
            )+
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_wasm_binary_valid() {
        let binary = mock_wasm_binary();
        
        // Check magic number
        assert_eq!(&binary[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        
        // Check version
        assert_eq!(&binary[4..8], &[0x01, 0x00, 0x00, 0x00]);
        
        // Check minimum size
        assert!(binary.len() > 8);
    }
    
    #[test]
    fn test_agent_config_creation() {
        let config = test_agent_config();
        
        assert_eq!(config.agent_type, "test_agent");
        assert_eq!(config.resource_quota.max_cpu_cores, 0.5);
        assert_eq!(config.resource_quota.max_memory_mb, 256);
    }
    
    #[test]
    fn test_task_creation_with_priority() {
        let task = test_task_with_priority(Priority::Critical);
        
        assert_eq!(task.priority, Priority::Critical);
        assert_eq!(task.target_agent_type, "test_agent");
    }
    
    #[test]
    fn test_multiple_tasks_creation() {
        let tasks = test_tasks(10);
        
        assert_eq!(tasks.len(), 10);
        
        // Verify priorities are distributed
        let priorities: Vec<_> = tasks.iter().map(|t| t.priority).collect();
        assert!(priorities.contains(&Priority::Critical));
        assert!(priorities.contains(&Priority::High));
        assert!(priorities.contains(&Priority::Normal));
    }
}
