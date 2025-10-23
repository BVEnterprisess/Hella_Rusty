//! Stress tests for Layer 4 Execution Fabric
//!
//! This module provides comprehensive stress testing for the Layer 4 system,
//! validating behavior under extreme load, resource exhaustion, and
//! failure conditions.

use crate::types::*;
use crate::executor::*;
use crate::scheduler::*;
use crate::metrics::*;
use crate::agent_template::*;
use chimera_layer4::Layer4Fabric;
use chimera_layer4::utils;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, Duration, Instant};
use tokio::time::{timeout, sleep};

/// Stress test configuration
#[derive(Debug, Clone)]
struct StressTestConfig {
    /// Test duration in seconds
    pub duration_secs: u64,
    /// Maximum number of concurrent agents
    pub max_agents: usize,
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Task failure rate (0.0 to 1.0)
    pub task_failure_rate: f32,
    /// Resource exhaustion level (0.0 to 1.0)
    pub resource_exhaustion_level: f32,
    /// Enable chaos testing (random failures)
    pub enable_chaos: bool,
    /// Stress test thresholds
    pub thresholds: StressThresholds,
}

#[derive(Debug, Clone)]
struct StressThresholds {
    /// Minimum acceptable success rate under stress
    pub min_success_rate: f32,
    /// Maximum acceptable average latency under stress in milliseconds
    pub max_avg_latency_ms: f64,
    /// Maximum acceptable system recovery time in seconds
    pub max_recovery_time_secs: u64,
    /// Minimum acceptable throughput degradation (0.0 to 1.0)
    pub min_throughput_degradation: f32,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            duration_secs: 300, // 5 minutes
            max_agents: 50,
            max_concurrent_tasks: 1000,
            task_failure_rate: 0.1, // 10% failure rate
            resource_exhaustion_level: 0.8, // 80% resource usage
            enable_chaos: true,
            thresholds: StressThresholds {
                min_success_rate: 0.85, // 85% success rate minimum
                max_avg_latency_ms: 1000.0, // 1 second max latency
                max_recovery_time_secs: 30, // 30 seconds max recovery
                min_throughput_degradation: 0.5, // At least 50% of baseline throughput
            },
        }
    }
}

/// Stress test results
#[derive(Debug, Clone)]
struct StressTestResults {
    /// Test configuration used
    pub config: StressTestConfig,
    /// Total test duration
    pub total_duration: Duration,
    /// Task execution statistics under stress
    pub task_stats: StressTaskStats,
    /// System behavior under stress
    pub system_behavior: SystemStressBehavior,
    /// Recovery metrics
    pub recovery_metrics: RecoveryMetrics,
    /// Whether system met stress thresholds
    pub thresholds_met: bool,
}

#[derive(Debug, Clone)]
struct StressTaskStats {
    /// Total tasks attempted
    pub total_tasks: usize,
    /// Successfully completed tasks
    pub successful_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Timed out tasks
    pub timed_out_tasks: usize,
    /// Success rate percentage
    pub success_rate: f32,
    /// Average latency under stress in milliseconds
    pub avg_latency_ms: f64,
    /// Peak latency observed in milliseconds
    pub peak_latency_ms: f64,
    /// Throughput under stress in tasks per second
    pub throughput_tps: f64,
}

#[derive(Debug, Clone)]
struct SystemStressBehavior {
    /// Peak concurrent agents
    pub peak_agents: usize,
    /// Peak queue depth
    pub peak_queue_depth: usize,
    /// Number of agent failures
    pub agent_failures: usize,
    /// Number of system restarts
    pub system_restarts: usize,
    /// Resource exhaustion events
    pub resource_exhaustion_events: usize,
    /// Dead letter queue size at peak
    pub peak_dlq_size: usize,
}

#[derive(Debug, Clone)]
struct RecoveryMetrics {
    /// Time to recover from failures in seconds
    pub recovery_time_secs: f64,
    /// Number of automatic recoveries
    pub automatic_recoveries: usize,
    /// Number of manual interventions needed
    pub manual_interventions: usize,
    /// System availability percentage
    pub availability_percentage: f32,
}

/// Run all stress tests for Layer 4
pub async fn run_stress_tests() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = StressTestConfig::default();

    println!("🔥 Starting Layer 4 stress tests...");

    // Test high concurrency stress
    test_high_concurrency_stress(&config).await?;

    // Test resource exhaustion stress
    test_resource_exhaustion_stress(&config).await?;

    // Test failure injection stress
    test_failure_injection_stress(&config).await?;

    // Test long-duration stress
    test_long_duration_stress(&config).await?;

    // Test recovery under stress
    test_recovery_under_stress(&config).await?;

    // Test chaos engineering scenarios
    if config.enable_chaos {
        test_chaos_engineering(&config).await?;
    }

    println!("✅ All stress tests passed!");
    Ok(())
}

/// Test high concurrency stress
async fn test_high_concurrency_stress(config: &StressTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔥 Testing high concurrency stress...");

    // Create Layer 4 fabric for high concurrency
    let layer4_config = Layer4Config {
        max_agents: config.max_agents,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.5, // Smaller quotas for more agents
            max_memory_mb: 128,
            max_execution_time_secs: 30,
            max_network_mbps: Some(5),
        },
        task_queue_capacity: 5000, // Large queue for stress testing
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 3,
        agent_timeout_secs: 20,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9200,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create high-concurrency stress tasks
    let mut tasks = Vec::new();
    let tasks_per_batch = 100;

    for batch in 0..(config.max_concurrent_tasks / tasks_per_batch) {
        for i in 0..tasks_per_batch {
            let global_task_id = batch * tasks_per_batch + i;
            let task = Task {
                id: utils::generate_task_id(),
                priority: if i % 10 == 0 { Priority::High } else { Priority::Normal },
                payload: serde_json::json!({
                    "action": "concurrency_stress_test",
                    "batch_id": batch,
                    "task_id": global_task_id,
                    "stress_level": "high"
                }),
                created_at: SystemTime::now(),
                deadline: Some(SystemTime::now() + Duration::from_secs(60)),
                resource_quota: ResourceQuota {
                    max_cpu_cores: 0.3,
                    max_memory_mb: 64,
                    max_execution_time_secs: 15,
                    max_network_mbps: Some(2),
                },
                source_layer: "concurrency_stress_test".to_string(),
                target_agent_type: "test_agent".to_string(),
                metadata: HashMap::from([
                    ("stress_test".to_string(), "concurrency".to_string()),
                    ("batch".to_string(), batch.to_string()),
                ]),
            };
            tasks.push(task);
        }
    }

    // Execute tasks with high concurrency
    let start_time = Instant::now();
    let mut handles = Vec::new();
    let mut execution_times = Vec::new();
    let mut success_count = Arc::new(AtomicUsize::new(0));
    let mut failure_count = Arc::new(AtomicUsize::new(0));
    let mut timeout_count = Arc::new(AtomicUsize::new(0));

    for task in tasks {
        let layer4_clone = &layer4;
        let success_counter = Arc::clone(&success_count);
        let failure_counter = Arc::clone(&failure_count);
        let timeout_counter = Arc::clone(&timeout_count);

        let handle = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = timeout(Duration::from_secs(20), layer4_clone.execute_task(task)).await;

            let execution_time = task_start.elapsed().as_millis() as f64;

            match result {
                Ok(Ok(execution_result)) => {
                    success_counter.fetch_add(1, Ordering::Relaxed);
                    if execution_result.success {
                        execution_times.push(execution_time);
                    }
                }
                Ok(Err(_)) => {
                    failure_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    timeout_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            execution_time
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _execution_time = handle.await?;
    }

    let total_duration = start_time.elapsed();

    // Collect final statistics
    let successful_tasks = success_count.load(Ordering::Relaxed);
    let failed_tasks = failure_count.load(Ordering::Relaxed);
    let timed_out_tasks = timeout_count.load(Ordering::Relaxed);
    let total_completed = successful_tasks + failed_tasks + timed_out_tasks;

    let success_rate = if total_completed > 0 {
        successful_tasks as f32 / total_completed as f32
    } else {
        0.0
    };

    let avg_latency = if execution_times.is_empty() {
        0.0
    } else {
        execution_times.iter().sum::<f64>() / execution_times.len() as f64
    };

    let throughput_tps = total_completed as f64 / total_duration.as_secs_f64();

    println!("    High Concurrency Stress Results:");
    println!("      Tasks attempted: {}", config.max_concurrent_tasks);
    println!("      Tasks completed: {}", total_completed);
    println!("      Successful tasks: {}", successful_tasks);
    println!("      Failed tasks: {}", failed_tasks);
    println!("      Timed out tasks: {}", timed_out_tasks);
    println!("      Success rate: {:.2}%", success_rate * 100.0);
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());
    println!("      Average latency: {:.2}ms", avg_latency);
    println!("      Throughput: {:.2} tasks/second", throughput_tps);

    // Validate stress test results
    assert!(success_rate >= config.thresholds.min_success_rate,
            "Success rate under stress too low: {:.2}% < {:.2}%",
            success_rate * 100.0, config.thresholds.min_success_rate * 100.0);

    assert!(avg_latency <= config.thresholds.max_avg_latency_ms,
            "Average latency under stress too high: {:.2}ms > {:.2}ms",
            avg_latency, config.thresholds.max_avg_latency_ms);

    // Check system health after stress test
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    let scheduler_stats = layer4.get_scheduler_stats().await;
    if scheduler_stats.queued_tasks > 1000 {
        println!("      Warning: High queue depth after stress test: {}", scheduler_stats.queued_tasks);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ High concurrency stress test passed");
    Ok(())
}

/// Test resource exhaustion stress
async fn test_resource_exhaustion_stress(config: &StressTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔥 Testing resource exhaustion stress...");

    // Create Layer 4 fabric with limited resources
    let layer4_config = Layer4Config {
        max_agents: 5, // Very limited agents
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 0.2, // Very limited CPU
            max_memory_mb: 32,  // Very limited memory
            max_execution_time_secs: 10,
            max_network_mbps: Some(1),
        },
        task_queue_capacity: 100, // Small queue
        kpi_reporting_interval_secs: 1,
        heartbeat_interval_secs: 2,
        agent_timeout_secs: 5,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9201,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create resource-intensive tasks
    let mut tasks = Vec::new();
    for i in 0..50 { // More tasks than can be handled
        let task = Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({
                "action": "resource_exhaustion_test",
                "task_id": i,
                "memory_mb": 100, // Request more memory than available
                "cpu_cores": 2.0, // Request more CPU than available
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(20)),
            resource_quota: ResourceQuota {
                max_cpu_cores: 1.0, // More than available per agent
                max_memory_mb: 256, // More than available per agent
                max_execution_time_secs: 15,
                max_network_mbps: Some(10),
            },
            source_layer: "resource_exhaustion_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("stress_test".to_string(), "resource_exhaustion".to_string()),
            ]),
        };
        tasks.push(task);
    }

    // Execute resource exhaustion tasks
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut resource_exhaustion_count = 0;

    for task in tasks {
        let result = timeout(Duration::from_secs(25), layer4.execute_task(task)).await;

        match result {
            Ok(Ok(execution_result)) => {
                results.push(execution_result);
            }
            Ok(Err(e)) => {
                if e.to_string().contains("quota") || e.to_string().contains("resource") {
                    resource_exhaustion_count += 1;
                }
                if config.enable_chaos {
                    println!("      Resource exhaustion detected: {}", e);
                }
            }
            Err(_) => {
                // Task timed out due to resource constraints
                if config.enable_chaos {
                    println!("      Task timed out due to resource constraints");
                }
            }
        }
    }

    let total_duration = start_time.elapsed();

    println!("    Resource Exhaustion Stress Results:");
    println!("      Tasks attempted: {}", tasks.len());
    println!("      Tasks completed: {}", results.len());
    println!("      Resource exhaustion events: {}", resource_exhaustion_count);
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());

    // Validate resource exhaustion handling
    assert!(resource_exhaustion_count > 0,
            "Expected resource exhaustion events, but none occurred");

    // System should still be functional after resource exhaustion
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Check that some tasks still succeeded despite resource constraints
    if !results.is_empty() {
        let success_rate = results.iter().filter(|r| r.success).count() as f32 / results.len() as f32;
        println!("      Success rate despite resource constraints: {:.2}%", success_rate * 100.0);
    }

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Resource exhaustion stress test passed");
    Ok(())
}

/// Test failure injection stress
async fn test_failure_injection_stress(config: &StressTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔥 Testing failure injection stress...");

    // Create Layer 4 fabric for failure testing
    let layer4_config = Layer4Config {
        max_agents: config.max_agents,
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
        metrics_port: 9202,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create tasks with controlled failure rate
    let mut tasks = Vec::new();
    for i in 0..200 {
        let should_fail = (i as f32 / 200.0) < config.task_failure_rate;
        let task = Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({
                "action": "failure_injection_test",
                "task_id": i,
                "should_fail": should_fail,
                "failure_type": if should_fail { "simulated_error" } else { "none" }
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(30)),
            resource_quota: ResourceQuota::default(),
            source_layer: "failure_injection_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("stress_test".to_string(), "failure_injection".to_string()),
            ]),
        };
        tasks.push(task);
    }

    // Execute failure injection tasks
    let start_time = Instant::now();
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut execution_times = Vec::new();

    for task in tasks {
        let task_start = Instant::now();
        let result = timeout(Duration::from_secs(35), layer4.execute_task(task)).await;

        let execution_time = task_start.elapsed().as_millis() as f64;

        match result {
            Ok(Ok(execution_result)) => {
                if execution_result.success {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                execution_times.push(execution_time);
            }
            Ok(Err(_)) => {
                failure_count += 1;
            }
            Err(_) => {
                failure_count += 1;
            }
        }
    }

    let total_duration = start_time.elapsed();

    // Calculate failure injection results
    let total_completed = success_count + failure_count;
    let success_rate = if total_completed > 0 {
        success_count as f32 / total_completed as f32
    } else {
        0.0
    };

    let expected_failures = (tasks.len() as f32 * config.task_failure_rate) as usize;
    let actual_failure_rate = if total_completed > 0 {
        failure_count as f32 / total_completed as f32
    } else {
        0.0
    };

    let avg_latency = if execution_times.is_empty() {
        0.0
    } else {
        execution_times.iter().sum::<f64>() / execution_times.len() as f64
    };

    println!("    Failure Injection Stress Results:");
    println!("      Tasks attempted: {}", tasks.len());
    println!("      Tasks completed: {}", total_completed);
    println!("      Successful tasks: {}", success_count);
    println!("      Failed tasks: {}", failure_count);
    println!("      Success rate: {:.2}%", success_rate * 100.0);
    println!("      Expected failure rate: {:.2}%", config.task_failure_rate * 100.0);
    println!("      Actual failure rate: {:.2}%", actual_failure_rate * 100.0);
    println!("      Average latency: {:.2}ms", avg_latency);
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());

    // Validate failure injection results
    assert!(success_rate >= config.thresholds.min_success_rate,
            "Success rate under failure injection too low: {:.2}%", success_rate * 100.0);

    // Failure rate should be close to expected (within 20% tolerance)
    let failure_rate_tolerance = 0.2;
    let failure_rate_diff = (actual_failure_rate - config.task_failure_rate).abs();
    assert!(failure_rate_diff <= failure_rate_tolerance,
            "Failure rate deviation too high: expected {:.2}%, got {:.2}%",
            config.task_failure_rate * 100.0, actual_failure_rate * 100.0);

    // System should remain stable despite failures
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Failure injection stress test passed");
    Ok(())
}

/// Test long-duration stress
async fn test_long_duration_stress(config: &StressTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔥 Testing long-duration stress ({}s)...", config.duration_secs);

    // Create Layer 4 fabric for long-duration testing
    let layer4_config = Layer4Config {
        max_agents: config.max_agents,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 60,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 2000,
        kpi_reporting_interval_secs: 5, // Less frequent for long tests
        heartbeat_interval_secs: 10,
        agent_timeout_secs: 45,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9203,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create continuous task stream for long-duration test
    let start_time = Instant::now();
    let mut task_count = 0;
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut recent_latencies = Vec::new();

    // Run stress test for specified duration
    let test_end = start_time + Duration::from_secs(config.duration_secs);

    while Instant::now() < test_end {
        // Create batch of tasks
        let batch_size = 10;
        let mut batch_tasks = Vec::new();

        for i in 0..batch_size {
            let task = Task {
                id: utils::generate_task_id(),
                priority: Priority::Normal,
                payload: serde_json::json!({
                    "action": "long_duration_test",
                    "batch_id": task_count / batch_size,
                    "task_id": i,
                    "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
                }),
                created_at: SystemTime::now(),
                deadline: Some(SystemTime::now() + Duration::from_secs(90)),
                resource_quota: ResourceQuota::default(),
                source_layer: "long_duration_test".to_string(),
                target_agent_type: "test_agent".to_string(),
                metadata: HashMap::from([
                    ("stress_test".to_string(), "long_duration".to_string()),
                ]),
            };
            batch_tasks.push(task);
            task_count += 1;
        }

        // Execute batch
        for task in batch_tasks {
            let task_start = Instant::now();
            let result = timeout(Duration::from_secs(120), layer4.execute_task(task)).await;

            let latency = task_start.elapsed().as_millis() as f64;
            recent_latencies.push(latency);

            // Keep only recent latencies for trend analysis
            if recent_latencies.len() > 100 {
                recent_latencies.remove(0);
            }

            match result {
                Ok(Ok(execution_result)) => {
                    if execution_result.success {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                }
                Ok(Err(_)) => {
                    failure_count += 1;
                }
                Err(_) => {
                    failure_count += 1;
                }
            }
        }

        // Brief pause between batches
        sleep(Duration::from_millis(100)).await;

        // Periodic health check
        if task_count % 50 == 0 {
            let health = layer4.get_health().await;
            if config.enable_chaos {
                println!("      Health check at {} tasks: {:?}", task_count, health.status);
            }
        }
    }

    let total_duration = start_time.elapsed();

    // Calculate long-duration stress results
    let total_completed = success_count + failure_count;
    let success_rate = if total_completed > 0 {
        success_count as f32 / total_completed as f32
    } else {
        0.0
    };

    let avg_latency = if recent_latencies.is_empty() {
        0.0
    } else {
        recent_latencies.iter().sum::<f64>() / recent_latencies.len() as f64
    };

    let throughput_tps = total_completed as f64 / total_duration.as_secs_f64();

    println!("    Long-Duration Stress Results:");
    println!("      Test duration: {:.2}s", total_duration.as_secs_f32());
    println!("      Tasks completed: {}", total_completed);
    println!("      Success rate: {:.2}%", success_rate * 100.0);
    println!("      Average latency: {:.2}ms", avg_latency);
    println!("      Throughput: {:.2} tasks/second", throughput_tps);

    // Validate long-duration stress results
    assert!(success_rate >= config.thresholds.min_success_rate,
            "Long-duration success rate too low: {:.2}%", success_rate * 100.0);

    assert!(avg_latency <= config.thresholds.max_avg_latency_ms,
            "Long-duration latency too high: {:.2}ms", avg_latency);

    // System should still be healthy after long-duration stress
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Long-duration stress test passed");
    Ok(())
}

/// Test recovery under stress
async fn test_recovery_under_stress(config: &StressTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔥 Testing recovery under stress...");

    // Create Layer 4 fabric for recovery testing
    let layer4_config = Layer4Config {
        max_agents: config.max_agents,
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
        metrics_port: 9204,
        debug_mode: false,
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Phase 1: Establish baseline performance
    let baseline_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "baseline_test"}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota::default(),
            source_layer: "recovery_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    let baseline_start = Instant::now();
    let baseline_result = layer4.execute_task(baseline_tasks[0].clone()).await?;
    let baseline_duration = baseline_start.elapsed();

    // Phase 2: Inject stress and failures
    let stress_tasks = (0..20).map(|i| {
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({
                "action": "stress_recovery_test",
                "task_id": i,
                "stress_phase": "injection"
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(30)),
            resource_quota: ResourceQuota::default(),
            source_layer: "recovery_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("stress_test".to_string(), "recovery".to_string()),
            ]),
        }
    }).collect::<Vec<_>>();

    let stress_start = Instant::now();
    let mut stress_failures = 0;

    for task in stress_tasks {
        let result = timeout(Duration::from_secs(35), layer4.execute_task(task)).await;

        if result.is_err() || result.unwrap().is_err() {
            stress_failures += 1;
        }
    }

    let stress_duration = stress_start.elapsed();

    // Phase 3: Test recovery
    let recovery_start = Instant::now();
    let recovery_tasks = vec![
        Task {
            id: utils::generate_task_id(),
            priority: Priority::Normal,
            payload: serde_json::json!({"action": "recovery_test"}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota::default(),
            source_layer: "recovery_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        },
    ];

    let recovery_result = layer4.execute_task(recovery_tasks[0].clone()).await?;
    let recovery_duration = recovery_start.elapsed();

    println!("    Recovery Under Stress Results:");
    println!("      Baseline task duration: {:.2}ms", baseline_duration.as_millis());
    println!("      Stress phase duration: {:.2}s", stress_duration.as_secs_f32());
    println!("      Stress failures: {}", stress_failures);
    println!("      Recovery task duration: {:.2}ms", recovery_duration.as_millis());
    println!("      Recovery time: {:.2}s", recovery_duration.as_secs_f32());

    // Validate recovery
    assert!(baseline_result.success, "Baseline task should succeed");
    assert!(recovery_result.success, "Recovery task should succeed");

    // Recovery should be reasonably fast (within 50% of baseline)
    let recovery_ratio = recovery_duration.as_millis() as f32 / baseline_duration.as_millis() as f32;
    assert!(recovery_ratio <= 1.5,
            "Recovery too slow: {:.2}x baseline", recovery_ratio);

    // System should be healthy after recovery
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Recovery under stress test passed");
    Ok(())
}

/// Test chaos engineering scenarios
async fn test_chaos_engineering(config: &StressTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔥 Testing chaos engineering scenarios...");

    // Create Layer 4 fabric for chaos testing
    let layer4_config = Layer4Config {
        max_agents: config.max_agents,
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
        metrics_port: 9205,
        debug_mode: true, // Enable debug for chaos testing
    };

    let layer4 = Layer4Fabric::new(layer4_config).await?;
    layer4.start().await?;

    // Create chaos test tasks with random failures and delays
    let mut tasks = Vec::new();
    for i in 0..100 {
        let chaos_factor = (i * 7 + 42) % 100; // Pseudo-random chaos factor

        let task = Task {
            id: utils::generate_task_id(),
            priority: match chaos_factor % 3 {
                0 => Priority::Critical,
                1 => Priority::High,
                _ => Priority::Normal,
            },
            payload: serde_json::json!({
                "action": "chaos_test",
                "task_id": i,
                "chaos_factor": chaos_factor,
                "chaos_type": match chaos_factor % 4 {
                    0 => "delay",
                    1 => "failure",
                    2 => "resource_spike",
                    _ => "normal",
                }
            }),
            created_at: SystemTime::now(),
            deadline: Some(SystemTime::now() + Duration::from_secs(45)),
            resource_quota: ResourceQuota {
                max_cpu_cores: if chaos_factor % 3 == 0 { 2.0 } else { 0.5 },
                max_memory_mb: if chaos_factor % 4 == 0 { 512 } else { 128 },
                max_execution_time_secs: if chaos_factor % 5 == 0 { 60 } else { 15 },
                max_network_mbps: Some(if chaos_factor % 2 == 0 { 20 } else { 5 }),
            },
            source_layer: "chaos_test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::from([
                ("chaos_test".to_string(), "true".to_string()),
                ("chaos_factor".to_string(), chaos_factor.to_string()),
            ]),
        };
        tasks.push(task);
    }

    // Execute chaos tasks
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut chaos_events = 0;

    for task in tasks {
        let result = timeout(Duration::from_secs(50), layer4.execute_task(task)).await;

        match result {
            Ok(Ok(execution_result)) => {
                results.push(execution_result);
            }
            Ok(Err(e)) => {
                chaos_events += 1;
                if config.enable_chaos {
                    println!("      Chaos event detected: {}", e);
                }
            }
            Err(_) => {
                chaos_events += 1;
                if config.enable_chaos {
                    println!("      Chaos timeout detected");
                }
            }
        }
    }

    let total_duration = start_time.elapsed();

    // Calculate chaos engineering results
    let successful_tasks = results.iter().filter(|r| r.success).count();
    let success_rate = successful_tasks as f32 / tasks.len() as f32;

    println!("    Chaos Engineering Results:");
    println!("      Tasks executed: {}", tasks.len());
    println!("      Successful tasks: {}", successful_tasks);
    println!("      Chaos events: {}", chaos_events);
    println!("      Success rate under chaos: {:.2}%", success_rate * 100.0);
    println!("      Total duration: {:.2}s", total_duration.as_secs_f32());

    // Validate chaos engineering results
    // System should maintain reasonable success rate even under chaos
    assert!(success_rate >= 0.6,
            "Success rate under chaos too low: {:.2}%", success_rate * 100.0);

    // System should remain functional despite chaos
    let health = layer4.get_health().await;
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded));

    // Cleanup
    layer4.shutdown().await?;

    println!("    ✅ Chaos engineering test passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_small_concurrency_stress() {
        let config = StressTestConfig {
            max_agents: 5,
            max_concurrent_tasks: 20,
            duration_secs: 30,
            task_failure_rate: 0.05,
            ..Default::default()
        };

        let result = test_high_concurrency_stress(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_small_resource_exhaustion() {
        let config = StressTestConfig {
            max_agents: 2,
            max_concurrent_tasks: 10,
            duration_secs: 30,
            ..Default::default()
        };

        let result = test_resource_exhaustion_stress(&config).await;
        assert!(result.is_ok());
    }
}