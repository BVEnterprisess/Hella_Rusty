//! Simple test to validate Layer 4 basic functionality
//!
//! This is a minimal test that validates the core Layer 4 functionality
//! without complex dependencies or compilation issues.

use std::time::{SystemTime, Duration};
use std::thread;

/// Simple test for basic Layer 4 types
#[cfg(test)]
mod tests {
    use chimera_layer4::types::*;
    use chimera_layer4::utils;
    use std::collections::HashMap;

    #[test]
    fn test_basic_types() {
        // Test priority enum
        assert_eq!(Priority::Critical as u8, 100);
        assert_eq!(Priority::High as u8, 75);
        assert_eq!(Priority::Normal as u8, 50);
        assert_eq!(Priority::Low as u8, 25);
        assert_eq!(Priority::Background as u8, 1);

        // Test priority ordering
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);

        // Test resource quota
        let quota = ResourceQuota {
            max_cpu_cores: 2.0,
            max_memory_mb: 1024,
            max_execution_time_secs: 300,
            max_network_mbps: Some(50),
        };

        assert_eq!(quota.max_cpu_cores, 2.0);
        assert_eq!(quota.max_memory_mb, 1024);

        // Test task creation
        let task = Task {
            id: utils::generate_task_id(),
            priority: Priority::High,
            payload: serde_json::json!({"action": "test"}),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(300)),
            resource_quota: quota,
            source_layer: "test_layer".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        };

        assert!(!task.id.is_nil());
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.source_layer, "test_layer");

        // Test execution result
        let execution_result = ExecutionResult {
            task_id: task.id,
            success: true,
            output: serde_json::json!({"result": "success"}),
            execution_time_ms: 150,
            resource_usage: ResourceUsage {
                cpu_seconds: 0.1,
                memory_peak_mb: 64.0,
                network_tx_bytes: 512,
                network_rx_bytes: 256,
                disk_io_ops: 10,
                gpu_utilization: None,
            },
            error: None,
            completed_at: SystemTime::now(),
        };

        assert!(execution_result.success);
        assert_eq!(execution_result.execution_time_ms, 150);

        // Test Layer4Config default
        let layer4_config = Layer4Config::default();
        assert_eq!(layer4_config.max_agents, 10);
        assert_eq!(layer4_config.metrics_port, 9090);

        // Test system health
        let system_health = SystemHealth {
            status: HealthStatus::Healthy,
            active_agents: 5,
            pending_tasks: 10,
            uptime_seconds: 3600,
            resource_utilization: ResourceUtilization {
                cpu_usage: 0.15,
                memory_usage: 0.25,
                disk_usage: 0.10,
                network_usage: 0.05,
            },
            last_check: SystemTime::now(),
        };

        assert_eq!(system_health.status, HealthStatus::Healthy);
        assert_eq!(system_health.active_agents, 5);

        println!("✅ Basic types test passed!");
    }

    #[test]
    fn test_utility_functions() {
        // Test ID generation
        let task_id = utils::generate_task_id();
        let agent_id = utils::generate_agent_id();

        assert!(!task_id.is_nil());
        assert!(!agent_id.is_nil());
        assert_ne!(task_id, agent_id);

        // Test timestamp generation
        let timestamp1 = utils::current_timestamp_secs();
        std::thread::sleep(Duration::from_millis(10));
        let timestamp2 = utils::current_timestamp_secs();

        assert!(timestamp2 >= timestamp1);

        // Test WASM validation
        let valid_wasm = vec![
            0x00, 0x61, 0x73, 0x6D, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version
        ];

        let result = utils::validate_wasm_binary(&valid_wasm);
        assert!(result.is_ok());

        // Test invalid WASM
        let invalid_wasm = vec![0x00, 0x01, 0x02, 0x03];
        let result = utils::validate_wasm_binary(&invalid_wasm);
        assert!(result.is_err());

        // Test default resource quota
        let quota = utils::default_resource_quota();
        assert_eq!(quota.max_cpu_cores, 1.0);
        assert_eq!(quota.max_memory_mb, 512);

        // Test default task creation
        let task = utils::default_task();
        assert_eq!(task.priority, Priority::Normal);
        assert_eq!(task.source_layer, "test");
        assert_eq!(task.target_agent_type, "test_agent");

        println!("✅ Utility functions test passed!");
    }

    #[test]
    fn test_version_info() {
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
        assert!(env!("CARGO_PKG_VERSION").contains('.'));
        // Skip BUILD_INFO test for now as it's not available in this context

        println!("✅ Version info test passed!");
    }
}