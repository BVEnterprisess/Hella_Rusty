//! Integration tests for Layer 4 Execution Fabric
//!
//! This module provides comprehensive integration testing for Layer 4
//! component interactions, end-to-end workflows, and cross-module
//! functionality validation.

use chimera_layer4::types::*;
use chimera_layer4::executor::*;
use chimera_layer4::scheduler::*;
use chimera_layer4::metrics::*;
use chimera_layer4::agent_template::*;
use chimera_layer4::Layer4Fabric;
use chimera_layer4::utils;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tokio::time::{timeout, sleep};

/// Test configuration for integration tests
#[derive(Debug, Clone)]
struct IntegrationTestConfig {
    /// Overall test timeout
    pub test_timeout: Duration,
    /// Enable verbose logging
    pub verbose: bool,
    /// Number of test agents to spawn
    pub agent_count: usize,
    /// Number of test tasks to execute
    pub task_count: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            test_timeout: Duration::from_secs(300), // 5 minutes
            verbose: false,
            agent_count: 5,
            task_count: 20,
            enable_monitoring: true,
        }
    }
}

/// Run all integration tests for Layer 4
pub async fn run_integration_tests() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::default();

    println!("🔗 Starting Layer 4 integration tests...");

    // Test basic component integration
    test_basic_component_integration(&config).await?;

    // Test full execution pipeline
    test_full_execution_pipeline(&config).await?;

    // Test error handling and recovery
    test_error_handling_and_recovery(&config).await?;

    // Test metrics collection integration
    test_metrics_collection_integration(&config).await?;

    // Test resource management integration
    test_resource_management_integration(&config).await?;

    // Test concurrent execution
    test_concurrent_execution(&config).await?;

    // Test graceful shutdown
    test_graceful_shutdown(&config).await?;

    println!("✅ All integration tests passed!");
    Ok(())
}

/// Test basic component integration
async fn test_basic_component_integration(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing basic component integration...");

    // Create Layer 4 configuration
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 60,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 100,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9090,
        debug_mode: config.verbose,
    };

    // Create Layer 4 fabric
    let layer4 = Layer4Fabric::new(layer4_config).await?;

    // Verify fabric was created successfully
    assert!(layer4.get_config().max_agents == config.agent_count);

    // Test that all components are properly initialized
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Test scheduler statistics
    let scheduler_stats = layer4.get_scheduler_stats().await;
    assert!(scheduler_stats.max_queue_size >= 100);

    // Test metrics snapshot
    let metrics_snapshot = layer4.get_metrics_snapshot().await?;
    assert!(metrics_snapshot.timestamp <= SystemTime::now());

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Basic component integration test passed");
    Ok(())
}

/// Test full execution pipeline
async fn test_full_execution_pipeline(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing full execution pipeline...");

    // Create Layer 4 fabric
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 60,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 100,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9091, // Different port to avoid conflicts
        debug_mode: config.verbose,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;

    // Start the fabric
    layer4.start().await?;

    // Create test tasks
    let mut tasks = Vec::new();
    for i in 0..config.task_count {
        let task = Task {
            id: utils::generate_task_id(),
            priority: if i % 3 == 0 { Priority::High } else { Priority::Normal },
            payload: serde_json::json!({
                "action": "test_execution",
                "task_number": i,
                "data": format!("test_data_{}", i)
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(60)),
            resource_quota: ResourceQuota {
                max_cpu_cores: 0.5,
                max_memory_mb: 256,
                max_execution_time_secs: 30,
                max_network_mbps: Some(5),
            },
            source_layer: "integration_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("test_id".to_string(), format!("test_{}", i)),
                ("created_by".to_string(), "integration_test".to_string()),
            ]),
        };
        tasks.push(task);
    }

    // Execute tasks and collect results
    let mut results = Vec::new();
    for task in tasks {
        let result = timeout(
            Duration::from_secs(30),
            layer4.execute_task(task)
        ).await??;

        results.push(result);
    }

    // Verify results
    assert_eq!(results.len(), config.task_count);

    let successful_tasks = results.iter().filter(|r| r.success).count();
    let success_rate = successful_tasks as f32 / config.task_count as f32;

    if config.verbose {
        println!("    Task success rate: {:.2}% ({}/{})",
                success_rate * 100.0, successful_tasks, config.task_count);
    }

    // Should have at least 80% success rate for integration test
    assert!(success_rate >= 0.8, "Task success rate too low: {:.2}%", success_rate * 100.0);

    // Verify execution metrics
    for result in &results {
        assert!(result.execution_time_ms > 0);
        assert!(result.resource_usage.cpu_seconds >= 0.0);
        assert!(result.resource_usage.memory_peak_mb >= 0.0);
        assert!(result.completed_at <= SystemTime::now());
    }

    // Test system health after execution
    let health = layer4.get_health().await;
    if config.verbose {
        println!("    System health after execution: {:?}", health.status);
        println!("    Active agents: {}", health.active_agents);
        println!("    Pending tasks: {}", health.pending_tasks);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Full execution pipeline test passed");
    Ok(())
}

/// Test error handling and recovery
async fn test_error_handling_and_recovery(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing error handling and recovery...");

    // Create Layer 4 fabric with short timeouts for faster error testing
    let layer4_config = Layer4Config {
        max_agents: 3,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 10, // Short timeout
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 50,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 2,
        agent_timeout_secs: 5, // Short timeout for faster testing
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9092,
        debug_mode: config.verbose,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create tasks that will fail (invalid agent type)
    let failing_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "failing_action"}),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(10)),
            resource_quota: ResourceQuota::default(),
            source_layer: "test".to_string(),
            target_agent_type: "nonexistent_agent".to_string(), // This will cause failure
            metadata: HashMap::new(),
        },
        Task {
            id: utils::generate_task_id(),
            priority: Priority::High,
            payload: serde_json::json!({"action": "another_failing_action"}),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(10)),
            resource_quota: ResourceQuota::default(),
            source_layer: "test".to_string(),
            target_agent_type: "nonexistent_agent".to_string(), // This will cause failure
            metadata: HashMap::new(),
        },
    ];

    // Execute failing tasks
    for task in failing_tasks {
        let result = timeout(
            Duration::from_secs(15),
            layer4.execute_task(task)
        ).await;

        // Should either timeout or return an error
        match result {
            Ok(Err(e)) => {
                // Expected error result
                if config.verbose {
                    println!("    Task failed as expected: {}", e);
                }
            }
            Ok(Ok(result)) => {
                // If task somehow succeeded, it should have error information
                assert!(result.error.is_some() || !result.success);
            }
            Err(_) => {
                // Task timed out - also acceptable for error testing
                if config.verbose {
                    println!("    Task timed out as expected");
                }
            }
        }
    }

    // Test system recovery - system should still be healthy
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Test scheduler stats after failures
    let scheduler_stats = layer4.get_scheduler_stats().await;
    if config.verbose {
        println!("    Scheduler stats after failures: {:?}", scheduler_stats);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Error handling and recovery test passed");
    Ok(())
}

/// Test metrics collection integration
async fn test_metrics_collection_integration(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing metrics collection integration...");

    // Create Layer 4 fabric with metrics enabled
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 60,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 100,
        kpi_reporting_interval_secs: 1, // Fast metrics collection
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9093,
        debug_mode: config.verbose,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Execute some tasks to generate metrics
    let tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::High,
            payload: serde_json::json!({"action": "metrics_test_1"}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota::default(),
            source_layer: "metrics_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "metrics_test_2"}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota::default(),
            source_layer: "metrics_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    for task in tasks {
        let _result = layer4.execute_task(task).await?;
    }

    // Wait for metrics to be collected
    sleep(Duration::from_secs(3)).await;

    // Test metrics snapshot
    let metrics_snapshot = layer4.get_metrics_snapshot().await?;
    assert!(metrics_snapshot.timestamp <= SystemTime::now());

    // Test Prometheus metrics export
    let prometheus_metrics = layer4.export_prometheus_metrics().await?;
    assert!(prometheus_metrics.contains("layer4_"));
    assert!(prometheus_metrics.contains("# HELP"));
    assert!(prometheus_metrics.contains("# TYPE"));

    // Verify specific metrics are present
    assert!(prometheus_metrics.contains("layer4_uptime_seconds") ||
            prometheus_metrics.contains("layer4_tasks_total") ||
            prometheus_metrics.contains("layer4_cpu_usage"));

    if config.verbose {
        println!("    Exported metrics length: {} characters", prometheus_metrics.len());
        println!("    Contains uptime metric: {}",
                prometheus_metrics.contains("layer4_uptime_seconds"));
        println!("    Contains task metrics: {}",
                prometheus_metrics.contains("layer4_tasks_total"));
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Metrics collection integration test passed");
    Ok(())
}

/// Test resource management integration
async fn test_resource_management_integration(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing resource management integration...");

    // Create Layer 4 fabric with limited resources
    let layer4_config = Layer4Config {
        max_agents: 2, // Limited agents
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.5, // Limited CPU
            max_memory_mb: 256, // Limited memory
            max_execution_time_secs: 30,
            max_network_mbps: Some(5),
        },
        task_queue_capacity: 10, // Small queue
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 15,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9094,
        debug_mode: config.verbose,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create resource-intensive tasks
    let resource_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "resource_test", "size": "large"}),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(30)),
            resource_quota: ResourceQuota {
                max_cpu_cores: 2.0, // More than available
                max_memory_mb: 1024, // More than available
                max_execution_time_secs: 60, // Longer than limit
                max_network_mbps: Some(20),
            },
            source_layer: "resource_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    // Execute resource-intensive task
    let result = timeout(
        Duration::from_secs(45),
        layer4.execute_task(resource_tasks[0].clone())
    ).await;

    // Should either complete, fail due to resource limits, or timeout
    match result {
        Ok(Ok(execution_result)) => {
            if config.verbose {
                println!("    Task completed: success={}, time={}ms",
                        execution_result.success, execution_result.execution_time_ms);
            }
            // If successful, verify resource usage is tracked
            assert!(execution_result.resource_usage.cpu_seconds >= 0.0);
            assert!(execution_result.resource_usage.memory_peak_mb >= 0.0);
        }
        Ok(Err(e)) => {
            if config.verbose {
                println!("    Task failed due to resource limits: {}", e);
            }
            // Expected due to resource constraints
        }
        Err(_) => {
            if config.verbose {
                println!("    Task timed out due to resource constraints");
            }
            // Expected due to resource constraints
        }
    }

    // Test system health after resource-intensive operations
    let health = layer4.get_health().await;
    if config.verbose {
        println!("    System health after resource test: {:?}", health.status);
        println!("    Resource utilization: CPU={:.2}%, Memory={:.2}%",
                health.resource_utilization.cpu_usage * 100.0,
                health.resource_utilization.memory_usage * 100.0);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Resource management integration test passed");
    Ok(())
}

/// Test concurrent execution
async fn test_concurrent_execution(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing concurrent execution...");

    // Create Layer 4 fabric optimized for concurrency
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.5, // Smaller quotas for more agents
            max_memory_mb: 128,
            max_execution_time_secs: 30,
            max_network_mbps: Some(5),
        },
        task_queue_capacity: 200,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 3,
        agent_timeout_secs: 20,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9095,
        debug_mode: config.verbose,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create concurrent tasks
    let mut tasks = Vec::new();
    for i in 0..config.task_count {
        let task = Task {
            id: utils::generate_task_id(),
            priority: if i % 4 == 0 { Priority::High } else { Priority::Normal },
            payload: serde_json::json!({
                "action": "concurrent_test",
                "task_id": i,
                "delay_ms": 100 + (i % 5) * 50 // Vary execution time
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(60)),
            resource_quota: ResourceQuota {
                max_cpu_cores: 0.3,
                max_memory_mb: 64,
                max_execution_time_secs: 20,
                max_network_mbps: Some(2),
            },
            source_layer: "concurrent_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("concurrent_id".to_string(), i.to_string()),
            ]),
        };
        tasks.push(task);
    }

    // Execute tasks concurrently
    let start_time = SystemTime::now();
    let mut handles = Vec::new();

    for task in tasks {
        let layer4_clone = &layer4;
        let handle = tokio::spawn(async move {
            timeout(
                Duration::from_secs(25),
                layer4_clone.execute_task(task)
            ).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        match handle.await? {
            Ok(execution_result) => results.push(execution_result),
            Err(_) => {
                // Task timed out - count as incomplete
                if config.verbose {
                    println!("    Task timed out during concurrent execution");
                }
            }
        }
    }

    let elapsed = start_time.elapsed()?;
    let elapsed_seconds = elapsed.as_secs_f32();

    if config.verbose {
        println!("    Executed {} tasks in {:.2} seconds",
                results.len(), elapsed_seconds);
        println!("    Average tasks per second: {:.2}",
                results.len() as f32 / elapsed_seconds);
    }

    // Should complete most tasks (allow for some timeouts)
    let completion_rate = results.len() as f32 / config.task_count as f32;
    assert!(completion_rate >= 0.7, "Concurrent execution completion rate too low: {:.2}%",
            completion_rate * 100.0);

    // Verify all completed tasks have valid metrics
    for result in &results {
        assert!(result.execution_time_ms > 0);
        assert!(result.resource_usage.cpu_seconds >= 0.0);
        assert!(result.resource_usage.memory_peak_mb >= 0.0);
    }

    // Test final system health
    let health = layer4.get_health().await;
    if config.verbose {
        println!("    Final system health: {:?}", health.status);
        println!("    Final active agents: {}", health.active_agents);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Concurrent execution test passed");
    Ok(())
}

/// Test graceful shutdown
async fn test_graceful_shutdown(config: &IntegrationTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Testing graceful shutdown...");

    // Create Layer 4 fabric
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 60,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 100,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9096,
        debug_mode: config.verbose,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Submit some long-running tasks
    let long_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "long_running_test"}),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(30)),
            resource_quota: ResourceQuota::default(),
            source_layer: "shutdown_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    // Start task execution
    let task_handle = tokio::spawn(async move {
        for task in long_tasks {
            let _result = layer4.execute_task(task).await;
        }
    });

    // Wait a bit for task to start
    sleep(Duration::from_secs(2)).await;

    // Initiate graceful shutdown
    let shutdown_start = SystemTime::now();
    let shutdown_handle = tokio::spawn(async move {
        layer4.shutdown().await
    });

    // Wait for shutdown to complete
    let shutdown_result = timeout(Duration::from_secs(30), shutdown_handle).await??;
    let shutdown_elapsed = shutdown_start.elapsed()?;

    if config.verbose {
        println!("    Graceful shutdown completed in {:.2} seconds", shutdown_elapsed.as_secs_f32());
    }

    // Cancel the long-running task
    task_handle.abort();

    // Verify shutdown completed successfully
    assert!(shutdown_result.is_ok());

    // Shutdown should complete in reasonable time (less than 30 seconds)
    assert!(shutdown_elapsed < Duration::from_secs(30));

    println!("    ✅ Graceful shutdown test passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_integration_scenario() {
        let config = IntegrationTestConfig {
            agent_count: 2,
            task_count: 5,
            ..Default::default()
        };

        let result = test_basic_component_integration(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_small_execution_pipeline() {
        let config = IntegrationTestConfig {
            agent_count: 2,
            task_count: 3,
            ..Default::default()
        };

        let result = test_full_execution_pipeline(&config).await;
        assert!(result.is_ok());
    }
}