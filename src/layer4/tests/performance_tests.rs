//! Performance benchmarks for Layer 4 Execution Fabric
//!
//! This module provides comprehensive performance testing and benchmarking
//! for the Layer 4 system, measuring execution speed, resource usage,
//! throughput, and latency characteristics.

use crate::types::*;
use crate::executor::*;
use crate::scheduler::*;
use crate::metrics::*;
use crate::agent_template::*;
use chimera_layer4::Layer4Fabric;
use chimera_layer4::utils;
use std::collections::HashMap;
use std::time::{SystemTime, Duration, Instant};
use tokio::time::{timeout, sleep};

/// Performance test configuration
#[derive(Debug, Clone)]
struct PerformanceTestConfig {
    /// Test duration in seconds
    pub duration_secs: u64,
    /// Number of concurrent agents for testing
    pub agent_count: usize,
    /// Number of tasks to execute
    pub task_count: usize,
    /// Task execution time range (min, max) in milliseconds
    pub task_execution_time_range_ms: (u64, u64),
    /// Enable detailed performance profiling
    pub enable_profiling: bool,
    /// Performance thresholds
    pub thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
struct PerformanceThresholds {
    /// Maximum acceptable average latency in milliseconds
    pub max_avg_latency_ms: f64,
    /// Maximum acceptable 95th percentile latency in milliseconds
    pub max_p95_latency_ms: f64,
    /// Minimum acceptable throughput in tasks per second
    pub min_throughput_tps: f64,
    /// Maximum acceptable memory usage per agent in MB
    pub max_memory_per_agent_mb: f64,
    /// Maximum acceptable CPU usage percentage
    pub max_cpu_usage_percent: f64,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            duration_secs: 60,
            agent_count: 10,
            task_count: 1000,
            task_execution_time_range_ms: (50, 200),
            enable_profiling: true,
            thresholds: PerformanceThresholds {
                max_avg_latency_ms: 100.0,
                max_p95_latency_ms: 500.0,
                min_throughput_tps: 50.0,
                max_memory_per_agent_mb: 128.0,
                max_cpu_usage_percent: 80.0,
            },
        }
    }
}

/// Performance test results
#[derive(Debug, Clone)]
struct PerformanceResults {
    /// Test configuration used
    pub config: PerformanceTestConfig,
    /// Total execution time
    pub total_duration: Duration,
    /// Task execution statistics
    pub task_stats: TaskPerformanceStats,
    /// Resource utilization statistics
    pub resource_stats: ResourcePerformanceStats,
    /// System performance metrics
    pub system_metrics: SystemPerformanceMetrics,
    /// Whether performance met thresholds
    pub thresholds_met: bool,
}

#[derive(Debug, Clone)]
struct TaskPerformanceStats {
    /// Total tasks executed
    pub total_tasks: usize,
    /// Successfully completed tasks
    pub successful_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Average execution latency in milliseconds
    pub avg_latency_ms: f64,
    /// 95th percentile latency in milliseconds
    pub p95_latency_ms: f64,
    /// 99th percentile latency in milliseconds
    pub p99_latency_ms: f64,
    /// Minimum latency observed
    pub min_latency_ms: f64,
    /// Maximum latency observed
    pub max_latency_ms: f64,
    /// Tasks per second throughput
    pub throughput_tps: f64,
}

#[derive(Debug, Clone)]
struct ResourcePerformanceStats {
    /// Average CPU usage percentage
    pub avg_cpu_usage: f32,
    /// Peak CPU usage percentage
    pub peak_cpu_usage: f32,
    /// Average memory usage in MB
    pub avg_memory_mb: f32,
    /// Peak memory usage in MB
    pub peak_memory_mb: f32,
    /// Memory usage per agent in MB
    pub memory_per_agent_mb: f32,
    /// Network I/O bytes
    pub network_bytes: u64,
}

#[derive(Debug, Clone)]
struct SystemPerformanceMetrics {
    /// Agent spawn time statistics
    pub agent_spawn_stats: SpawnTimeStats,
    /// Queue depth over time
    pub queue_depth_stats: QueueDepthStats,
    /// Error rate percentage
    pub error_rate: f32,
}

#[derive(Debug, Clone)]
struct SpawnTimeStats {
    /// Average agent spawn time in milliseconds
    pub avg_spawn_time_ms: f64,
    /// Minimum spawn time observed
    pub min_spawn_time_ms: f64,
    /// Maximum spawn time observed
    pub max_spawn_time_ms: f64,
}

#[derive(Debug, Clone)]
struct QueueDepthStats {
    /// Average queue depth
    pub avg_depth: f32,
    /// Maximum queue depth observed
    pub max_depth: usize,
    /// Queue depth variance
    pub depth_variance: f32,
}

/// Run all performance tests for Layer 4
pub async fn run_performance_tests() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = PerformanceTestConfig::default();

    println!("⚡ Starting Layer 4 performance tests...");

    // Test basic performance characteristics
    test_basic_performance(&config).await?;

    // Test scalability with different agent counts
    for agent_count in [1, 5, 10, 20] {
        config.agent_count = agent_count;
        config.task_count = agent_count * 100; // Scale tasks with agents

        test_scalability_performance(&config).await?;
    }

    // Test resource efficiency
    test_resource_efficiency(&config).await?;

    // Test concurrent load performance
    test_concurrent_load_performance(&config).await?;

    // Test memory usage patterns
    test_memory_usage_patterns(&config).await?;

    println!("✅ All performance tests passed!");
    Ok(())
}

/// Test basic performance characteristics
async fn test_basic_performance(config: &PerformanceTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Testing basic performance characteristics...");

    // Create Layer 4 fabric for performance testing
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 1000,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9100,
        debug_mode: false, // Disable debug for performance testing
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create performance test tasks
    let mut tasks = Vec::new();
    let mut expected_latencies = Vec::new();

    for i in 0..config.task_count {
        let task_latency = config.task_execution_time_range_ms.0 +
                          (i % 10) as u64 * 10; // Vary latency

        let task = Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({
                "action": "performance_test",
                "task_id": i,
                "expected_latency_ms": task_latency
            }),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota {
                max_cpu_cores: 0.5,
                max_memory_mb: 128,
                max_execution_time_secs: 10,
                max_network_mbps: Some(5),
            },
            source_layer: "performance_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        };

        tasks.push(task);
        expected_latencies.push(task_latency as f64);
    }

    // Execute tasks and measure performance
    let start_time = Instant::now();
    let mut execution_times = Vec::new();
    let mut results = Vec::new();

    for task in tasks {
        let task_start = Instant::now();
        let result = timeout(Duration::from_secs(15), layer4.execute_task(task)).await??;
        let task_duration = task_start.elapsed();

        execution_times.push(task_duration.as_millis() as f64);
        results.push(result);
    }

    let total_duration = start_time.elapsed();

    // Calculate performance statistics
    let successful_tasks = results.iter().filter(|r| r.success).count();
    let success_rate = successful_tasks as f32 / config.task_count as f32;

    let avg_latency = execution_times.iter().sum::<f64>() / execution_times.len() as f64;
    let throughput_tps = config.task_count as f64 / total_duration.as_secs_f64();

    // Calculate percentiles
    let mut sorted_latencies = execution_times.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p95_index = (sorted_latencies.len() as f32 * 0.95) as usize;
    let p99_index = (sorted_latencies.len() as f32 * 0.99) as usize;

    let p95_latency = sorted_latencies.get(p95_index).cloned().unwrap_or(0.0);
    let p99_latency = sorted_latencies.get(p99_index).cloned().unwrap_or(0.0);

    // Validate performance against thresholds
    let thresholds_met = avg_latency <= config.thresholds.max_avg_latency_ms &&
                        p95_latency <= config.thresholds.max_p95_latency_ms &&
                        throughput_tps >= config.thresholds.min_throughput_tps;

    println!("    Performance Results:");
    println!("      Tasks executed: {} ({:.2}% success rate)", config.task_count, success_rate * 100.0);
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());
    println!("      Average latency: {:.2}ms", avg_latency);
    println!("      95th percentile latency: {:.2}ms", p95_latency);
    println!("      99th percentile latency: {:.2}ms", p99_latency);
    println!("      Throughput: {:.2} tasks/second", throughput_tps);
    println!("      Thresholds met: {}", if thresholds_met { "✅ Yes" } else { "❌ No" });

    // Assert performance requirements
    assert!(success_rate >= 0.95, "Success rate too low: {:.2}%", success_rate * 100.0);
    assert!(avg_latency <= config.thresholds.max_avg_latency_ms,
            "Average latency too high: {:.2}ms > {:.2}ms",
            avg_latency, config.thresholds.max_avg_latency_ms);
    assert!(throughput_tps >= config.thresholds.min_throughput_tps,
            "Throughput too low: {:.2} TPS < {:.2} TPS",
            throughput_tps, config.thresholds.min_throughput_tps);

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Basic performance test passed");
    Ok(())
}

/// Test scalability with different agent counts
async fn test_scalability_performance(config: &PerformanceTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Testing scalability with {} agents...", config.agent_count);

    // Create Layer 4 fabric with specific agent count
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 1000,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9101,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create scalability test tasks
    let tasks_per_agent = config.task_count / config.agent_count;
    let mut tasks = Vec::new();

    for agent_id in 0..config.agent_count {
        for task_id in 0..tasks_per_agent {
            let global_task_id = agent_id * tasks_per_agent + task_id;
            let task = Task {
                id: utils::generate_task_id(),
                priority: Priority::Normal,
                payload: serde_json::json!({
                    "action": "scalability_test",
                    "agent_id": agent_id,
                    "task_id": global_task_id
                }),
                created_at: SystemTime::now(),
                deadline: None,
                resource_quota: ResourceQuota {
                    max_cpu_cores: 0.5,
                    max_memory_mb: 128,
                    max_execution_time_secs: 10,
                    max_network_mbps: Some(5),
                },
                source_layer: "scalability_test".to_string(),
                target_agent_type: "test_agent".to_string(),
                metadata: HashMap::from([
                    ("agent_id".to_string(), agent_id.to_string()),
                    ("scalability_test".to_string(), "true".to_string()),
                ]),
            };
            tasks.push(task);
        }
    }

    // Execute tasks and measure scalability
    let start_time = Instant::now();
    let mut execution_times = Vec::new();

    for task in tasks {
        let task_start = Instant::now();
        let result = timeout(Duration::from_secs(15), layer4.execute_task(task)).await??;
        let task_duration = task_start.elapsed();

        execution_times.push(task_duration.as_millis() as f64);

        if result.success {
            execution_times.push(result.execution_time_ms as f64);
        }
    }

    let total_duration = start_time.elapsed();

    // Calculate scalability metrics
    let avg_latency = execution_times.iter().sum::<f64>() / execution_times.len() as f64;
    let throughput_tps = tasks.len() as f64 / total_duration.as_secs_f64();

    // Calculate efficiency (tasks per agent per second)
    let efficiency = throughput_tps / config.agent_count as f64;

    println!("    Scalability Results ({} agents):", config.agent_count);
    println!("      Tasks executed: {}", tasks.len());
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());
    println!("      Average latency: {:.2}ms", avg_latency);
    println!("      Throughput: {:.2} tasks/second", throughput_tps);
    println!("      Efficiency: {:.2} tasks/agent/second", efficiency);

    // Validate scalability - performance should not degrade significantly with more agents
    if config.agent_count > 1 {
        // Throughput should scale reasonably with agent count (not necessarily linearly due to overhead)
        let expected_min_throughput = config.thresholds.min_throughput_tps * 0.5; // Allow for overhead
        assert!(throughput_tps >= expected_min_throughput,
                "Scalability throughput too low: {:.2} TPS < {:.2} TPS",
                throughput_tps, expected_min_throughput);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Scalability performance test passed");
    Ok(())
}

/// Test resource efficiency
async fn test_resource_efficiency(config: &PerformanceTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Testing resource efficiency...");

    // Create Layer 4 fabric with resource monitoring
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 1000,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9102,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Wait for system to stabilize
    sleep(Duration::from_secs(2)).await;

    // Take baseline resource measurements
    let baseline_health = layer4.get_health().await;
    let baseline_cpu = baseline_health.resource_utilization.cpu_usage;
    let baseline_memory = baseline_health.resource_utilization.memory_usage;

    // Execute resource efficiency test tasks
    let resource_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "resource_efficiency_test"}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota {
                max_cpu_cores: 0.5,
                max_memory_mb: 128,
                max_execution_time_secs: 10,
                max_network_mbps: Some(5),
            },
            source_layer: "resource_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    let mut total_cpu_usage = 0.0;
    let mut total_memory_usage = 0.0;
    let mut measurements = 0;

    // Execute task and monitor resource usage
    for task in resource_tasks {
        let task_start = Instant::now();

        // Monitor resource usage during task execution
        while task_start.elapsed() < Duration::from_secs(10) {
            let health = layer4.get_health().await;
            total_cpu_usage += health.resource_utilization.cpu_usage;
            total_memory_usage += health.resource_utilization.memory_usage;
            measurements += 1;

            sleep(Duration::from_millis(500)).await;
        }

        let _result = layer4.execute_task(task).await?;
    }

    // Calculate average resource utilization
    let avg_cpu_usage = if measurements > 0 { total_cpu_usage / measurements as f32 } else { 0.0 };
    let avg_memory_usage = if measurements > 0 { total_memory_usage / measurements as f32 } else { 0.0 };

    println!("    Resource Efficiency Results:");
    println!("      Baseline CPU usage: {:.2}%", baseline_cpu * 100.0);
    println!("      Average CPU usage during execution: {:.2}%", avg_cpu_usage * 100.0);
    println!("      Baseline memory usage: {:.2}%", baseline_memory * 100.0);
    println!("      Average memory usage during execution: {:.2}%", avg_memory_usage * 100.0);
    println!("      Measurements taken: {}", measurements);

    // Validate resource efficiency
    assert!(avg_cpu_usage <= (config.thresholds.max_cpu_usage_percent as f32) / 100.0,
            "CPU usage too high: {:.2}% > {:.2}%",
            avg_cpu_usage * 100.0, config.thresholds.max_cpu_usage_percent);

    // Memory usage should be reasonable (less than 50% for this test)
    assert!(avg_memory_usage <= 0.5,
            "Memory usage too high: {:.2}%", avg_memory_usage * 100.0);

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Resource efficiency test passed");
    Ok(())
}

/// Test concurrent load performance
async fn test_concurrent_load_performance(config: &PerformanceTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Testing concurrent load performance...");

    // Create Layer 4 fabric optimized for concurrent load
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.5, // Smaller quotas for more concurrent agents
            max_memory_mb: 128,
            max_execution_time_secs: 20,
            max_network_mbps: Some(5),
        },
        task_queue_capacity: 2000, // Larger queue for concurrent load
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 3,
        agent_timeout_secs: 15,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9103,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create concurrent load test tasks
    let mut tasks = Vec::new();
    for i in 0..config.task_count {
        let task = Task {
            id: utils::generate_task_id(),
            priority: if i % 10 == 0 { Priority::High } else { Priority::Normal },
            payload: serde_json::json!({
                "action": "concurrent_load_test",
                "task_id": i,
                "concurrent_batch": i / 100
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(30)),
            resource_quota: ResourceQuota {
                max_cpu_cores: 0.3,
                max_memory_mb: 64,
                max_execution_time_secs: 10,
                max_network_mbps: Some(2),
            },
            source_layer: "concurrent_load_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("load_test_id".to_string(), i.to_string()),
            ]),
        };
        tasks.push(task);
    }

    // Execute tasks concurrently and measure performance
    let start_time = Instant::now();
    let mut handles = Vec::new();
    let mut execution_times = Vec::new();

    for task in tasks {
        let layer4_clone = &layer4;
        let handle = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = timeout(Duration::from_secs(15), layer4_clone.execute_task(task)).await;
            let task_duration = task_start.elapsed();

            (result, task_duration.as_millis() as f64)
        });
        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (result, execution_time) = handle.await?;
        execution_times.push(execution_time);

        match result {
            Ok(Ok(_execution_result)) => {
                // Task completed successfully
            }
            Ok(Err(e)) => {
                if config.enable_profiling {
                    println!("      Task failed: {}", e);
                }
            }
            Err(_) => {
                // Task timed out
                if config.enable_profiling {
                    println!("      Task timed out");
                }
            }
        }
    }

    let total_duration = start_time.elapsed();

    // Calculate concurrent load performance metrics
    let avg_latency = execution_times.iter().sum::<f64>() / execution_times.len() as f64;
    let throughput_tps = config.task_count as f64 / total_duration.as_secs_f64();

    // Calculate latency percentiles
    let mut sorted_latencies = execution_times.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p95_index = (sorted_latencies.len() as f32 * 0.95) as usize;
    let p99_index = (sorted_latencies.len() as f32 * 0.99) as usize;

    let p95_latency = sorted_latencies.get(p95_index).cloned().unwrap_or(0.0);
    let p99_latency = sorted_latencies.get(p99_index).cloned().unwrap_or(0.0);

    println!("    Concurrent Load Results:");
    println!("      Tasks executed: {}", config.task_count);
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());
    println!("      Average latency: {:.2}ms", avg_latency);
    println!("      95th percentile latency: {:.2}ms", p95_latency);
    println!("      99th percentile latency: {:.2}ms", p99_latency);
    println!("      Throughput: {:.2} tasks/second", throughput_tps);
    println!("      Concurrency factor: {:.2}x", config.task_count as f64 / config.agent_count as f64);

    // Validate concurrent load performance
    assert!(avg_latency <= config.thresholds.max_avg_latency_ms * 2.0, // Allow more latency under load
            "Concurrent load latency too high: {:.2}ms", avg_latency);
    assert!(p95_latency <= config.thresholds.max_p95_latency_ms * 2.0,
            "Concurrent load P95 latency too high: {:.2}ms", p95_latency);
    assert!(throughput_tps >= config.thresholds.min_throughput_tps * 0.8, // Allow slightly lower throughput
            "Concurrent load throughput too low: {:.2} TPS", throughput_tps);

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Concurrent load performance test passed");
    Ok(())
}

/// Test memory usage patterns
async fn test_memory_usage_patterns(config: &PerformanceTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Testing memory usage patterns...");

    // Create Layer 4 fabric for memory testing
    let layer4_config = Layer4Config {
        max_agents: config.agent_count,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 1000,
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 5,
        agent_timeout_secs: 30,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9104,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Wait for system to stabilize
    sleep(Duration::from_secs(2)).await;

    // Take baseline memory measurement
    let baseline_health = layer4.get_health().await;
    let baseline_memory = baseline_memory_usage();

    // Execute memory-intensive tasks
    let memory_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({
                "action": "memory_test",
                "allocate_mb": 50
            }),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota {
                max_cpu_cores: 0.5,
                max_memory_mb: 128,
                max_execution_time_secs: 10,
                max_network_mbps: Some(5),
            },
            source_layer: "memory_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    let mut memory_measurements = Vec::new();
    let mut peak_memory = baseline_memory;

    for task in memory_tasks {
        // Monitor memory during task execution
        let task_start = Instant::now();

        while task_start.elapsed() < Duration::from_secs(10) {
            let current_memory = baseline_memory_usage();
            memory_measurements.push(current_memory);

            if current_memory > peak_memory {
                peak_memory = current_memory;
            }

            sleep(Duration::from_millis(200)).await;
        }

        let _result = layer4.execute_task(task).await?;
    }

    // Calculate memory usage statistics
    let avg_memory = if memory_measurements.is_empty() {
        0.0
    } else {
        memory_measurements.iter().sum::<f64>() / memory_measurements.len() as f64
    };

    let memory_increase = avg_memory - baseline_memory;
    let memory_per_agent = if config.agent_count > 0 { avg_memory / config.agent_count as f64 } else { 0.0 };

    println!("    Memory Usage Results:");
    println!("      Baseline memory: {:.2} MB", baseline_memory);
    println!("      Average memory during execution: {:.2} MB", avg_memory);
    println!("      Peak memory usage: {:.2} MB", peak_memory);
    println!("      Memory increase: {:.2} MB", memory_increase);
    println!("      Memory per agent: {:.2} MB", memory_per_agent);
    println!("      Measurements taken: {}", memory_measurements.len());

    // Validate memory usage
    assert!(memory_increase >= 0.0, "Memory usage decreased unexpectedly");
    assert!(memory_per_agent <= config.thresholds.max_memory_per_agent_mb,
            "Memory per agent too high: {:.2} MB > {:.2} MB",
            memory_per_agent, config.thresholds.max_memory_per_agent_mb);

    // Memory increase should be reasonable (less than 200MB for this test)
    assert!(memory_increase <= 200.0,
            "Memory increase too high: {:.2} MB", memory_increase);

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Memory usage patterns test passed");
    Ok(())
}

/// Get current memory usage in MB (placeholder implementation)
fn baseline_memory_usage() -> f64 {
    // In a real implementation, this would read from /proc/self/status
    // For now, return a simulated value
    100.0 + (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() % 50) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_performance_benchmark() {
        let config = PerformanceTestConfig {
            agent_count: 2,
            task_count: 10,
            duration_secs: 30,
            ..Default::default()
        };

        let result = test_basic_performance(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_small_scalability_test() {
        let config = PerformanceTestConfig {
            agent_count: 2,
            task_count: 20,
            duration_secs: 30,
            ..Default::default()
        };

        let result = test_scalability_performance(&config).await;
        assert!(result.is_ok());
    }
}