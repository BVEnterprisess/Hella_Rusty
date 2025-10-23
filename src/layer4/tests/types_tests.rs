//! Comprehensive tests for Layer 4 type definitions
//!
//! Tests cover:
//! - Serialization/deserialization
//! - Type safety and validation
//! - Default implementations
//! - Error handling
//! - Type conversions

mod common;

use chimera_layer4::*;
use common::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

// ============================================================================
// Priority Tests
// ============================================================================

#[test]
fn test_priority_ordering() {
    assert!(Priority::Critical > Priority::High);
    assert!(Priority::High > Priority::Normal);
    assert!(Priority::Normal > Priority::Low);
    assert!(Priority::Low > Priority::Background);
}

#[test]
fn test_priority_serialization() {
    let priority = Priority::Critical;
    let json = serde_json::to_string(&priority).unwrap();
    let deserialized: Priority = serde_json::from_str(&json).unwrap();
    assert_eq!(priority, deserialized);
}

#[test]
fn test_all_priorities_serialize() {
    let priorities = vec![
        Priority::Critical,
        Priority::High,
        Priority::Normal,
        Priority::Low,
        Priority::Background,
    ];
    
    for priority in priorities {
        let json = serde_json::to_string(&priority).unwrap();
        let deserialized: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(priority, deserialized);
    }
}

// ============================================================================
// Task State Tests
// ============================================================================

#[test]
fn test_task_state_serialization() {
    let states = vec![
        TaskState::Pending,
        TaskState::Running,
        TaskState::Completed,
        TaskState::Failed,
        TaskState::Cancelled,
        TaskState::Timeout,
    ];
    
    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: TaskState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

#[test]
fn test_task_state_equality() {
    assert_eq!(TaskState::Pending, TaskState::Pending);
    assert_ne!(TaskState::Pending, TaskState::Running);
    assert_ne!(TaskState::Completed, TaskState::Failed);
}

// ============================================================================
// Health Status Tests
// ============================================================================

#[test]
fn test_health_status_serialization() {
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Critical,
    ];
    
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
}

// ============================================================================
// ResourceQuota Tests
// ============================================================================

#[test]
fn test_resource_quota_default() {
    let quota = ResourceQuota::default();
    
    assert_eq!(quota.max_cpu_cores, 1.0);
    assert_eq!(quota.max_memory_mb, 512);
    assert_eq!(quota.max_execution_time_secs, 300);
    assert_eq!(quota.max_network_mbps, Some(10));
}

#[test]
fn test_resource_quota_serialization() {
    let quota = ResourceQuota {
        max_cpu_cores: 2.0,
        max_memory_mb: 1024,
        max_execution_time_secs: 600,
        max_network_mbps: Some(50),
    };
    
    let json = serde_json::to_string(&quota).unwrap();
    let deserialized: ResourceQuota = serde_json::from_str(&json).unwrap();
    
    assert_eq!(quota.max_cpu_cores, deserialized.max_cpu_cores);
    assert_eq!(quota.max_memory_mb, deserialized.max_memory_mb);
    assert_eq!(quota.max_execution_time_secs, deserialized.max_execution_time_secs);
    assert_eq!(quota.max_network_mbps, deserialized.max_network_mbps);
}

#[test]
fn test_resource_quota_no_network_limit() {
    let quota = ResourceQuota {
        max_cpu_cores: 1.0,
        max_memory_mb: 512,
        max_execution_time_secs: 300,
        max_network_mbps: None,
    };
    
    let json = serde_json::to_string(&quota).unwrap();
    let deserialized: ResourceQuota = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.max_network_mbps, None);
}

// ============================================================================
// Task Tests
// ============================================================================

#[test]
fn test_task_creation() {
    let task = test_task_with_priority(Priority::High);
    
    assert!(task.id != Uuid::nil());
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.target_agent_type, "test_agent");
}

#[test]
fn test_task_serialization() {
    let task = test_task_with_priority(Priority::Normal);
    
    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    
    assert_eq!(task.id, deserialized.id);
    assert_eq!(task.priority, deserialized.priority);
    assert_eq!(task.target_agent_type, deserialized.target_agent_type);
}

#[test]
fn test_task_with_deadline() {
    let deadline = SystemTime::now() + Duration::from_secs(300);
    let task = Task {
        id: Uuid::new_v4(),
        priority: Priority::High,
        payload: serde_json::json!({"test": "data"}),
        created_at: SystemTime::now(),
        deadline: Some(deadline),
        resource_quota: ResourceQuota::default(),
        source_layer: "test".to_string(),
        target_agent_type: "test".to_string(),
        metadata: HashMap::new(),
    };
    
    assert!(task.deadline.is_some());
}

#[test]
fn test_task_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());
    
    let task = Task {
        id: Uuid::new_v4(),
        priority: Priority::Normal,
        payload: serde_json::json!({}),
        created_at: SystemTime::now(),
        deadline: None,
        resource_quota: ResourceQuota::default(),
        source_layer: "test".to_string(),
        target_agent_type: "test".to_string(),
        metadata: metadata.clone(),
    };
    
    assert_eq!(task.metadata.len(), 2);
    assert_eq!(task.metadata.get("key1"), Some(&"value1".to_string()));
}

// ============================================================================
// ExecutionResult Tests
// ============================================================================

#[test]
fn test_execution_result_success() {
    let result = ExecutionResult {
        task_id: Uuid::new_v4(),
        success: true,
        output: serde_json::json!({"result": "completed"}),
        execution_time_ms: 150,
        resource_usage: ResourceUsage {
            cpu_seconds: 0.1,
            memory_peak_mb: 64.0,
            network_tx_bytes: 1024,
            network_rx_bytes: 512,
            disk_io_ops: 10,
            gpu_utilization: None,
        },
        error: None,
        completed_at: SystemTime::now(),
    };
    
    assert_execution_success(&result);
}

#[test]
fn test_execution_result_failure() {
    let result = ExecutionResult {
        task_id: Uuid::new_v4(),
        success: false,
        output: serde_json::json!({}),
        execution_time_ms: 50,
        resource_usage: ResourceUsage::default(),
        error: Some("Task failed".to_string()),
        completed_at: SystemTime::now(),
    };
    
    assert_execution_failure(&result);
}

#[test]
fn test_execution_result_serialization() {
    let result = ExecutionResult {
        task_id: Uuid::new_v4(),
        success: true,
        output: serde_json::json!({"data": "test"}),
        execution_time_ms: 100,
        resource_usage: ResourceUsage::default(),
        error: None,
        completed_at: SystemTime::now(),
    };
    
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ExecutionResult = serde_json::from_str(&json).unwrap();
    
    assert_eq!(result.task_id, deserialized.task_id);
    assert_eq!(result.success, deserialized.success);
}

// ============================================================================
// ResourceUsage Tests
// ============================================================================

#[test]
fn test_resource_usage_default() {
    let usage = ResourceUsage::default();
    
    assert_eq!(usage.cpu_seconds, 0.0);
    assert_eq!(usage.memory_peak_mb, 0.0);
    assert_eq!(usage.network_tx_bytes, 0);
    assert_eq!(usage.network_rx_bytes, 0);
    assert_eq!(usage.disk_io_ops, 0);
    assert_eq!(usage.gpu_utilization, None);
}

#[test]
fn test_resource_usage_with_gpu() {
    let usage = ResourceUsage {
        cpu_seconds: 1.5,
        memory_peak_mb: 128.0,
        network_tx_bytes: 2048,
        network_rx_bytes: 1024,
        disk_io_ops: 50,
        gpu_utilization: Some(0.75),
    };
    
    assert_eq!(usage.gpu_utilization, Some(0.75));
}

#[test]
fn test_resource_usage_serialization() {
    let usage = ResourceUsage {
        cpu_seconds: 0.5,
        memory_peak_mb: 64.0,
        network_tx_bytes: 512,
        network_rx_bytes: 256,
        disk_io_ops: 20,
        gpu_utilization: None,
    };
    
    let json = serde_json::to_string(&usage).unwrap();
    let deserialized: ResourceUsage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(usage.cpu_seconds, deserialized.cpu_seconds);
    assert_eq!(usage.memory_peak_mb, deserialized.memory_peak_mb);
}

// ============================================================================
// AgentConfig Tests
// ============================================================================

#[test]
fn test_agent_config_creation() {
    let config = test_agent_config();
    
    assert!(config.agent_id != Uuid::nil());
    assert_eq!(config.agent_type, "test_agent");
    assert_eq!(config.resource_quota.max_cpu_cores, 0.5);
}

#[test]
fn test_agent_config_serialization() {
    let config = test_agent_config();
    
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.agent_id, deserialized.agent_id);
    assert_eq!(config.agent_type, deserialized.agent_type);
}

#[test]
fn test_agent_config_with_environment() {
    let mut env = HashMap::new();
    env.insert("KEY1".to_string(), "value1".to_string());
    env.insert("KEY2".to_string(), "value2".to_string());
    
    let config = AgentConfig {
        agent_id: Uuid::new_v4(),
        agent_type: "test".to_string(),
        resource_quota: ResourceQuota::default(),
        environment: env.clone(),
        parameters: HashMap::new(),
    };
    
    assert_eq!(config.environment.len(), 2);
    assert_eq!(config.environment.get("KEY1"), Some(&"value1".to_string()));
}

// ============================================================================
// Layer4Error Tests
// ============================================================================

#[test]
fn test_error_display() {
    let error = Layer4Error::Internal("Test error".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Test error"));
}

#[test]
fn test_error_task_not_found() {
    let task_id = Uuid::new_v4();
    let error = Layer4Error::TaskNotFound(task_id);
    let display = format!("{}", error);
    assert!(display.contains(&task_id.to_string()));
}

#[test]
fn test_error_resource_quota_exceeded() {
    let error = Layer4Error::ResourceQuotaExceeded("Memory limit exceeded".to_string());
    assert!(matches!(error, Layer4Error::ResourceQuotaExceeded(_)));
}

// ============================================================================
// KpiReport Tests
// ============================================================================

#[test]
fn test_kpi_report_creation() {
    let report = KpiReport {
        task_id: Uuid::new_v4(),
        agent_id: Uuid::new_v4(),
        latency_ms: 150.0,
        accuracy: 0.95,
        cpu_usage: 0.1,
        memory_mb: 64.0,
        network_bytes: 1024,
        custom_metrics: HashMap::new(),
        recorded_at: SystemTime::now(),
        execution_context: ExecutionContext {
            hostname: "test-host".to_string(),
            available_cores: 4,
            available_memory_mb: 8192,
            gpu_info: None,
            network_interfaces: vec!["eth0".to_string()],
        },
    };
    
    assert!(report.latency_ms > 0.0);
    assert!(report.accuracy >= 0.0 && report.accuracy <= 1.0);
}

#[test]
fn test_kpi_report_serialization() {
    let report = KpiReport {
        task_id: Uuid::new_v4(),
        agent_id: Uuid::new_v4(),
        latency_ms: 200.0,
        accuracy: 0.92,
        cpu_usage: 0.15,
        memory_mb: 128.0,
        network_bytes: 2048,
        custom_metrics: HashMap::new(),
        recorded_at: SystemTime::now(),
        execution_context: ExecutionContext {
            hostname: "test".to_string(),
            available_cores: 8,
            available_memory_mb: 16384,
            gpu_info: None,
            network_interfaces: vec![],
        },
    };
    
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: KpiReport = serde_json::from_str(&json).unwrap();
    
    assert_eq!(report.task_id, deserialized.task_id);
    assert_eq!(report.latency_ms, deserialized.latency_ms);
}

// ============================================================================
// SystemHealth Tests
// ============================================================================

#[test]
fn test_system_health_healthy() {
    let health = SystemHealth {
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
    
    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.active_agents, 5);
}

#[test]
fn test_system_health_serialization() {
    let health = SystemHealth {
        status: HealthStatus::Degraded,
        active_agents: 3,
        pending_tasks: 50,
        uptime_seconds: 1800,
        resource_utilization: ResourceUtilization {
            cpu_usage: 0.75,
            memory_usage: 0.85,
            disk_usage: 0.60,
            network_usage: 0.40,
        },
        last_check: SystemTime::now(),
    };
    
    let json = serde_json::to_string(&health).unwrap();
    let deserialized: SystemHealth = serde_json::from_str(&json).unwrap();
    
    assert_eq!(health.status, deserialized.status);
    assert_eq!(health.active_agents, deserialized.active_agents);
}
