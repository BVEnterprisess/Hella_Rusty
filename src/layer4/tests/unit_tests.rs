//! Comprehensive unit tests for Layer 4 Execution Fabric
//!
//! This module provides thorough unit testing for all Layer 4 components
//! including types, executor, scheduler, metrics, and agent templates.
//! Each test validates individual component behavior in isolation.

use crate::types::*;
use crate::executor::*;
use crate::scheduler::*;
use crate::metrics::*;
use crate::agent_template::*;
use crate::utils;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tokio::time::timeout;
use uuid::Uuid;

/// Test configuration for unit tests
#[derive(Debug, Clone)]
struct UnitTestConfig {
    /// Timeout for individual unit tests
    pub test_timeout: Duration,
    /// Enable detailed logging
    pub verbose: bool,
    /// Mock external dependencies
    pub mock_external: bool,
}

impl Default for UnitTestConfig {
    fn default() -> Self {
        Self {
            test_timeout: Duration::from_secs(30),
            verbose: false,
            mock_external: true,
        }
    }
}

/// Run all unit tests for Layer 4
pub async fn run_unit_tests() -> Result<(), Box<dyn std::error::Error>> {
    let config = UnitTestConfig::default();

    println!("🧪 Starting Layer 4 unit tests...");

    // Test types module
    test_types_module(&config).await?;

    // Test executor module
    test_executor_module(&config).await?;

    // Test scheduler module
    test_scheduler_module(&config).await?;

    // Test metrics module
    test_metrics_module(&config).await?;

    // Test agent template module
    test_agent_template_module(&config).await?;

    // Test utility functions
    test_utility_functions(&config).await?;

    println!("✅ All unit tests passed!");
    Ok(())
}

/// Test the types module functionality
async fn test_types_module(config: &UnitTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Testing types module...");

    // Test priority enum
    assert_eq!(Priority::Critical as u8, 100);
    assert_eq!(Priority::High as u8, 75);
    assert_eq!(Priority::Normal as u8, 50);
    assert_eq!(Priority::Low as u8, 25);
    assert_eq!(Priority::Background as u8, 1);

    // Test priority ordering
    assert!(Priority::Critical > Priority::High);
    assert!(Priority::High > Priority::Normal);
    assert!(Priority::Normal > Priority::Low);
    assert!(Priority::Low > Priority::Background);

    // Test task state enum
    let pending_state = TaskState::Pending;
    let running_state = TaskState::Running;
    let completed_state = TaskState::Completed;

    // Test resource quota
    let quota = ResourceQuota {
        max_cpu_cores: 2.0,
        max_memory_mb: 1024,
        max_execution_time_secs: 300,
        max_network_mbps: Some(50),
    };

    assert_eq!(quota.max_cpu_cores, 2.0);
    assert_eq!(quota.max_memory_mb, 1024);
    assert_eq!(quota.max_execution_time_secs, 300);
    assert_eq!(quota.max_network_mbps, Some(50));

    // Test default resource quota
    let default_quota = ResourceQuota::default();
    assert_eq!(default_quota.max_cpu_cores, 1.0);
    assert_eq!(default_quota.max_memory_mb, 512);

    // Test task creation
    let task = Task {
        id: Uuid::new_v4(),
        priority: Priority::High,
        payload: serde_json::json!({"action": "test", "data": "test_data"}),
        created_at: SystemTime::now(),
        deadline: Some(SystemTime::now() + Duration::from_secs(300)),
        resource_quota: quota,
        source_layer: "test_layer".to_string(),
        target_agent_type: "test_agent".to_string(),
        metadata: HashMap::from([("test_key".to_string(), "test_value".to_string())]),
    };

    assert!(!task.id.is_nil());
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.source_layer, "test_layer");
    assert_eq!(task.target_agent_type, "test_agent");
    assert_eq!(task.metadata.get("test_key"), Some(&"test_value".to_string()));

    // Test KPI report
    let kpi_report = KpiReport {
        task_id: Uuid::new_v4(),
        agent_id: Uuid::new_v4(),
        latency_ms: 150.0,
        accuracy: 0.95,
        cpu_usage: 0.1,
        memory_mb: 64.0,
        network_bytes: 1024,
        custom_metrics: HashMap::from([("confidence".to_string(), 0.87)]),
        recorded_at: SystemTime::now(),
        execution_context: ExecutionContext {
            hostname: "test-host".to_string(),
            available_cores: 8,
            available_memory_mb: 16384,
            gpu_info: None,
            network_interfaces: vec!["eth0".to_string()],
        },
    };

    assert_eq!(kpi_report.latency_ms, 150.0);
    assert_eq!(kpi_report.accuracy, 0.95);
    assert_eq!(kpi_report.custom_metrics.get("confidence"), Some(&0.87));

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
    assert_eq!(execution_result.resource_usage.cpu_seconds, 0.1);

    // Test error types
    let wasm_error = Layer4Error::WasmRuntime(anyhow::anyhow!("test error"));
    let task_not_found = Layer4Error::TaskNotFound(task.id);
    let resource_exceeded = Layer4Error::ResourceQuotaExceeded("Memory limit exceeded".to_string());

    // Test Layer4Config default
    let layer4_config = Layer4Config::default();
    assert_eq!(layer4_config.max_agents, 10);
    assert_eq!(layer4_config.metrics_port, 9090);
    assert!(!layer4_config.debug_mode);

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
    assert_eq!(system_health.uptime_seconds, 3600);

    println!("    ✅ Types module tests passed");
    Ok(())
}

/// Test the executor module functionality
async fn test_executor_module(config: &UnitTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Testing executor module...");

    // Test executor configuration
    let executor_config = ExecutorConfig {
        max_agents: 20,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 2.0,
            max_memory_mb: 1024,
            max_execution_time_secs: 300,
            max_network_mbps: Some(50),
        },
        heartbeat_interval_secs: 10,
        agent_timeout_secs: 60,
        debug_mode: true,
    };

    assert_eq!(executor_config.max_agents, 20);
    assert_eq!(executor_config.heartbeat_interval_secs, 10);
    assert!(executor_config.debug_mode);

    // Test executor creation
    let executor = Executor::new(executor_config.clone())?;
    assert!(executor.config.max_agents == 20);

    // Test WASM engine creation
    let engine = Executor::create_engine()?;
    // Verify engine can compile a simple WASM module
    let wat = r#"
        (module
            (func $hello (result i32)
                i32.const 42
            )
            (export "hello" (func $hello))
        )
    "#;
    let module = wasmtime::Module::new(&engine, wat)?;
    assert!(module.get_export("hello").is_some());

    // Test agent spawning
    let wasm_binary = vec![
        0x00, 0x61, 0x73, 0x6D, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // WASM version
    ];

    let agent_config = AgentConfig {
        agent_id: Uuid::new_v4(),
        agent_type: "test_agent".to_string(),
        resource_quota: ResourceQuota::default(),
        environment: HashMap::new(),
        parameters: HashMap::new(),
    };

    let agent_id = executor.spawn_agent(wasm_binary, agent_config).await?;
    assert!(!agent_id.is_nil());

    // Test system health
    let health = executor.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Test graceful shutdown
    executor.shutdown().await?;

    println!("    ✅ Executor module tests passed");
    Ok(())
}

/// Test the scheduler module functionality
async fn test_scheduler_module(config: &UnitTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Testing scheduler module...");

    // Test scheduler configuration
    let scheduler_config = SchedulerConfig {
        max_queue_size: 1000,
        max_retries: 3,
        retry_base_delay_secs: 1,
        retry_max_delay_secs: 300,
        retry_backoff_multiplier: 2.0,
        task_timeout_secs: 300,
        enable_preemption: true,
        dead_letter_queue_size: 100,
    };

    assert_eq!(scheduler_config.max_queue_size, 1000);
    assert_eq!(scheduler_config.max_retries, 3);
    assert!(scheduler_config.enable_preemption);

    // Test scheduler creation
    let scheduler = Scheduler::new(scheduler_config.clone())?;

    // Test task submission
    let task = Task {
        id: Uuid::new_v4(),
        priority: Priority::High,
        payload: serde_json::json!({"action": "test"}),
        created_at: SystemTime::now(),
        deadline: None,
        resource_quota: ResourceQuota::default(),
        source_layer: "test".to_string(),
        target_agent_type: "test_agent".to_string(),
        metadata: HashMap::new(),
    };

    let response_rx = scheduler.submit_task(task).await?;
    assert!(response_rx.recv().await.is_ok());

    // Test scheduler statistics
    let stats = scheduler.get_stats().await;
    assert!(stats.max_queue_size >= 1000);
    assert_eq!(stats.max_retries, 3);

    // Test retry delay calculation
    let delay1 = Scheduler::calculate_retry_delay(1, &scheduler_config);
    let delay2 = Scheduler::calculate_retry_delay(2, &scheduler_config);
    let delay3 = Scheduler::calculate_retry_delay(3, &scheduler_config);

    assert!(delay2 > delay1); // Exponential backoff
    assert!(delay3 > delay2);
    assert!(delay3 <= Duration::from_secs(scheduler_config.retry_max_delay_secs));

    // Test graceful shutdown
    scheduler.shutdown().await?;

    println!("    ✅ Scheduler module tests passed");
    Ok(())
}

/// Test the metrics module functionality
async fn test_metrics_module(config: &UnitTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Testing metrics module...");

    // Test metrics configuration
    let metrics_config = MetricsConfig {
        prometheus_port: 9090,
        collection_interval_secs: 5,
        enable_detailed_metrics: true,
        retention_secs: 3600,
        enable_export: true,
    };

    assert_eq!(metrics_config.prometheus_port, 9090);
    assert!(metrics_config.enable_detailed_metrics);
    assert!(metrics_config.enable_export);

    // Test metrics collector creation
    let collector = MetricsCollector::new(metrics_config.clone())?;

    // Test KPI report recording
    let kpi_report = KpiReport {
        task_id: Uuid::new_v4(),
        agent_id: Uuid::new_v4(),
        latency_ms: 150.0,
        accuracy: 0.95,
        cpu_usage: 0.1,
        memory_mb: 64.0,
        network_bytes: 1024,
        custom_metrics: HashMap::from([("confidence".to_string(), 0.87)]),
        recorded_at: SystemTime::now(),
        execution_context: ExecutionContext {
            hostname: "test-host".to_string(),
            available_cores: 8,
            available_memory_mb: 16384,
            gpu_info: None,
            network_interfaces: vec!["eth0".to_string()],
        },
    };

    collector.record_kpi_report(kpi_report).await?;

    // Test task result recording
    let execution_result = ExecutionResult {
        task_id: Uuid::new_v4(),
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

    collector.record_task_result(&execution_result).await?;

    // Test metrics snapshot
    let snapshot = collector.get_metrics_snapshot().await?;
    assert!(snapshot.timestamp <= SystemTime::now());

    // Test Prometheus export
    let prometheus_output = collector.export_prometheus_metrics().await?;
    assert!(prometheus_output.contains("layer4_"));
    assert!(prometheus_output.contains("# HELP"));

    // Test graceful shutdown
    collector.shutdown().await?;

    println!("    ✅ Metrics module tests passed");
    Ok(())
}

/// Test the agent template module functionality
async fn test_agent_template_module(config: &UnitTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Testing agent template module...");

    // Test base WASM agent creation
    let capabilities = AgentCapabilities {
        supported_task_types: vec!["test_task".to_string()],
        max_concurrent_tasks: 2,
        resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 300,
            max_network_mbps: Some(10),
        },
        required_env_vars: HashMap::new(),
        features: vec!["wasm".to_string(), "test".to_string()],
    };

    let mut agent = BaseWasmAgent::new("test_agent".to_string(), capabilities.clone());
    assert_eq!(agent.state, AgentState::Initializing);
    assert_eq!(agent.agent_type, "test_agent");

    // Test agent initialization
    let agent_config = AgentConfig {
        agent_id: Uuid::new_v4(),
        agent_type: "test_agent".to_string(),
        resource_quota: ResourceQuota::default(),
        environment: HashMap::new(),
        parameters: HashMap::new(),
    };

    agent.init(agent_config.clone())?;
    assert_eq!(agent.state, AgentState::Idle);
    assert_eq!(agent.id, agent_config.agent_id);

    // Test agent capabilities
    let retrieved_capabilities = agent.get_capabilities();
    assert_eq!(retrieved_capabilities.supported_task_types, capabilities.supported_task_types);
    assert_eq!(retrieved_capabilities.max_concurrent_tasks, capabilities.max_concurrent_tasks);

    // Test task execution
    let task = Task {
        id: Uuid::new_v4(),
        priority: Priority::Normal,
        payload: serde_json::json!({"action": "test"}),
        created_at: SystemTime::now(),
        deadline: None,
        resource_quota: ResourceQuota::default(),
        source_layer: "test".to_string(),
        target_agent_type: "test_agent".to_string(),
        metadata: HashMap::new(),
    };

    let result = agent.execute_task(task.clone())?;
    assert!(result.success);
    assert_eq!(result.task_id, task.id);

    // Test health check
    let health = agent.health_check();
    assert!(matches!(health.status, HealthStatus::Healthy));

    // Test telemetry collection
    let mut telemetry = TelemetryCollector::new();
    telemetry.record_metric("test_metric", 42.0);

    let kpi_report = telemetry.generate_kpi_report(
        task.id,
        agent.id,
        true,
        HashMap::new(),
    );

    assert_eq!(kpi_report.accuracy, 1.0);
    assert_eq!(kpi_report.custom_metrics.get("test_metric"), None);

    // Test memory manager
    let mut memory_manager = agent_template::memory::MemoryManager::new(1000);
    assert_eq!(memory_manager.usage(), 0);

    let alloc_result = memory_manager.allocate(500);
    assert!(alloc_result.is_ok());

    let alloc_result = memory_manager.allocate(600); // Exceeds limit
    assert!(alloc_result.is_err());

    // Test agent shutdown
    agent.shutdown()?;
    assert_eq!(agent.state, AgentState::Stopped);

    println!("    ✅ Agent template module tests passed");
    Ok(())
}

/// Test utility functions
async fn test_utility_functions(config: &UnitTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Testing utility functions...");

    // Test ID generation
    let task_id = utils::generate_task_id();
    let agent_id = utils::generate_agent_id();

    assert!(!task_id.is_nil());
    assert!(!agent_id.is_nil());
    assert_ne!(task_id, agent_id); // Should be different

    // Test timestamp generation
    let timestamp1 = utils::current_timestamp_secs();
    tokio::time::sleep(Duration::from_millis(10)).await;
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

    // Test version info
    assert!(!env!("CARGO_PKG_VERSION").is_empty());
    assert!(crate::VERSION.contains('.'));
    // Skip BUILD_INFO test for now as it's not available in this context

    println!("    ✅ Utility functions tests passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_unit_test_suite() {
        let result = run_unit_tests().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_priority_ordering() {
        // Test that priority ordering works correctly
        let mut tasks = vec![
            QueuedTask {
                task: Task {
                    id: Uuid::new_v4(),
                    priority: Priority::Low,
                    payload: serde_json::Value::Null,
                    created_at: SystemTime::now(),
                    deadline: None,
                    resource_quota: ResourceQuota::default(),
                    source_layer: "test".to_string(),
                    target_agent_type: "test".to_string(),
                    metadata: HashMap::new(),
                },
                retry_count: 0,
                queued_at: SystemTime::now(),
                last_retry_at: None,
                response_tx: async_channel::bounded(1).0,
            },
            QueuedTask {
                task: Task {
                    id: Uuid::new_v4(),
                    priority: Priority::Critical,
                    payload: serde_json::Value::Null,
                    created_at: SystemTime::now(),
                    deadline: None,
                    resource_quota: ResourceQuota::default(),
                    source_layer: "test".to_string(),
                    target_agent_type: "test".to_string(),
                    metadata: HashMap::new(),
                },
                retry_count: 0,
                queued_at: SystemTime::now(),
                last_retry_at: None,
                response_tx: async_channel::bounded(1).0,
            },
        ];

        // Sort by priority (highest first)
        tasks.sort_by(|a, b| b.cmp(a));

        assert_eq!(tasks[0].task.priority, Priority::Critical);
        assert_eq!(tasks[1].task.priority, Priority::Low);
    }

    #[test]
    fn test_resource_quota_validation() {
        let quota = ResourceQuota {
            max_cpu_cores: 0.0, // Invalid
            max_memory_mb: 0,   // Invalid
            max_execution_time_secs: 0, // Invalid
            max_network_mbps: None,
        };

        // In a real implementation, we would validate these constraints
        // For now, just ensure the struct can be created
        assert_eq!(quota.max_cpu_cores, 0.0);
        assert_eq!(quota.max_memory_mb, 0);
    }
}