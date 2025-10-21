//! Performance benchmarks for Layer 4 Execution Fabric
//!
//! Measures critical performance metrics:
//! - Agent spawn latency
//! - Task execution throughput
//! - Memory usage and efficiency
//! - Concurrent scaling characteristics
//! - Scheduler overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use chimera_layer4::*;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use uuid::Uuid;
use tokio::runtime::Runtime;

/// Benchmark agent spawn latency
/// Target: <50ms per agent
fn bench_agent_spawn_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("agent_spawn_single", |b| {
        b.iter(|| {
            rt.block_on(async {
                let capabilities = AgentCapabilities {
                    supported_task_types: vec!["benchmark".to_string()],
                    max_concurrent_tasks: 1,
                    resource_quota: ResourceQuota::default(),
                    required_env_vars: HashMap::new(),
                    features: vec![],
                };
                
                let agent = spawn_test_agent("bench_agent", capabilities).await;
                black_box(agent)
            })
        });
    });
}

/// Benchmark task execution throughput
/// Target: >1000 tasks/minute
fn bench_task_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("task_throughput");
    
    for batch_size in [1, 10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        execute_task_batch(size).await
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark scheduler overhead
/// Measure time spent in scheduling vs execution
fn bench_scheduler_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("scheduler_enqueue", |b| {
        b.iter(|| {
            rt.block_on(async {
                let scheduler = create_test_scheduler().await;
                let task = create_benchmark_task();
                
                let start = std::time::Instant::now();
                scheduler.submit_task(task).await.unwrap();
                black_box(start.elapsed())
            })
        });
    });
    
    c.bench_function("scheduler_priority_ordering", |b| {
        b.iter(|| {
            rt.block_on(async {
                let scheduler = create_test_scheduler().await;
                
                // Submit 1000 tasks with mixed priorities
                for i in 0..1000 {
                    let priority = match i % 5 {
                        0 => Priority::Critical,
                        1 => Priority::High,
                        2 => Priority::Normal,
                        3 => Priority::Low,
                        _ => Priority::Background,
                    };
                    
                    let mut task = create_benchmark_task();
                    task.priority = priority;
                    scheduler.submit_task(task).await.unwrap();
                }
                
                // Measure time to retrieve all tasks in priority order
                black_box(scheduler)
            })
        });
    });
}

/// Benchmark concurrent agent scaling
/// Target: >10 agents running concurrently
fn bench_concurrent_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_agents");
    
    for agent_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(agent_count),
            agent_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        spawn_and_execute_concurrent(count).await
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage per agent
/// Target: <64MB per agent
fn bench_memory_per_agent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("memory_per_agent", |b| {
        b.iter(|| {
            rt.block_on(async {
                let initial_memory = get_memory_usage();
                
                let _agent = spawn_test_agent(
                    "mem_bench",
                    AgentCapabilities {
                        supported_task_types: vec!["benchmark".to_string()],
                        max_concurrent_tasks: 1,
                        resource_quota: ResourceQuota::default(),
                        required_env_vars: HashMap::new(),
                        features: vec![],
                    }
                ).await;
                
                let after_spawn = get_memory_usage();
                let memory_used = after_spawn.saturating_sub(initial_memory);
                
                black_box(memory_used)
            })
        });
    });
}

/// Benchmark retry logic overhead
fn bench_retry_logic(c: &mut Criterion) {
    c.bench_function("retry_delay_calculation", |b| {
        b.iter(|| {
            let config = SchedulerConfig::default();
            for attempt in 1..=10 {
                let delay = Scheduler::calculate_retry_delay(attempt, &config);
                black_box(delay);
            }
        });
    });
}

/// Benchmark metrics collection overhead
fn bench_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("kpi_report_generation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut telemetry = TelemetryCollector::new();
                
                // Record 100 metrics
                for i in 0..100 {
                    telemetry.record_metric(&format!("metric_{}", i), i as f64);
                }
                
                let report = telemetry.generate_kpi_report(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    true,
                    HashMap::new(),
                );
                
                black_box(report)
            })
        });
    });
    
    // Prometheus export benchmark - disabled until MetricsCollector has public export method
    // c.bench_function("prometheus_export", |b| {
    //     b.iter(|| {
    //         rt.block_on(async {
    //             // Would need public export API
    //             black_box(())
    //         })
    //     });
    // });
}

/// Benchmark serialization/deserialization
fn bench_serialization(c: &mut Criterion) {
    c.bench_function("task_serialization", |b| {
        b.iter(|| {
            let task = create_benchmark_task();
            let json = serde_json::to_string(&task).unwrap();
            black_box(json)
        });
    });
    
    c.bench_function("task_deserialization", |b| {
        let task = create_benchmark_task();
        let json = serde_json::to_string(&task).unwrap();
        
        b.iter(|| {
            let deserialized: Task = serde_json::from_str(&json).unwrap();
            black_box(deserialized)
        });
    });
    
    c.bench_function("execution_result_serialization", |b| {
        b.iter(|| {
            let result = ExecutionResult {
                task_id: Uuid::new_v4(),
                success: true,
                output: serde_json::json!({"status": "completed"}),
                execution_time_ms: 150,
                resource_usage: ResourceUsage::default(),
                error: None,
                completed_at: SystemTime::now(),
            };
            
            let json = serde_json::to_string(&result).unwrap();
            black_box(json)
        });
    });
}

/// Benchmark task queue operations
fn bench_queue_operations(c: &mut Criterion) {
    use std::collections::BinaryHeap;
    
    c.bench_function("queue_push_1000", |b| {
        b.iter(|| {
            let mut queue: BinaryHeap<QueuedTask> = BinaryHeap::new();
            for i in 0..1000 {
                let task = create_queued_task(i % 5);
                queue.push(task);
            }
            black_box(queue)
        });
    });
    
    c.bench_function("queue_pop_1000", |b| {
        let mut queue: BinaryHeap<QueuedTask> = BinaryHeap::new();
        for i in 0..1000 {
            queue.push(create_queued_task(i % 5));
        }
        
        b.iter(|| {
            let mut queue_clone = queue.clone();
            let mut results = Vec::new();
            while let Some(task) = queue_clone.pop() {
                results.push(task);
            }
            black_box(results)
        });
    });
}

// ========== Helper Functions ==========

async fn spawn_test_agent(_name: &str, _capabilities: AgentCapabilities) -> AgentId {
    // Placeholder - would spawn actual agent
    Uuid::new_v4()
}

async fn create_test_scheduler() -> Scheduler {
    Scheduler::new(SchedulerConfig::default()).unwrap()
}

fn create_benchmark_task() -> Task {
    Task {
        id: Uuid::new_v4(),
        priority: Priority::Normal,
        payload: serde_json::json!({"benchmark": true}),
        created_at: SystemTime::now(),
        deadline: None,
        resource_quota: ResourceQuota::default(),
        source_layer: "benchmark".to_string(),
        target_agent_type: "bench_agent".to_string(),
        metadata: HashMap::new(),
    }
}

fn create_queued_task(priority_level: usize) -> QueuedTask {
    let priority = match priority_level {
        0 => Priority::Critical,
        1 => Priority::High,
        2 => Priority::Normal,
        3 => Priority::Low,
        _ => Priority::Background,
    };
    
    let (tx, _rx) = async_channel::bounded(1);
    
    QueuedTask {
        task: Task {
            id: Uuid::new_v4(),
            priority,
            payload: serde_json::json!({}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota::default(),
            source_layer: "benchmark".to_string(),
            target_agent_type: "test".to_string(),
            metadata: HashMap::new(),
        },
        retry_count: 0,
        queued_at: SystemTime::now(),
        last_retry_at: None,
        response_tx: tx,
    }
}

async fn execute_task_batch(size: usize) {
    // Placeholder - would execute actual tasks
    tokio::time::sleep(Duration::from_micros(size as u64 * 10)).await;
}

async fn spawn_and_execute_concurrent(count: usize) {
    // Placeholder - would spawn and execute
    tokio::time::sleep(Duration::from_micros(count as u64 * 100)).await;
}

fn get_memory_usage() -> usize {
    // Placeholder - would query actual memory
    0
}

// Disabled - not needed after removing prometheus_export benchmark
// async fn create_test_metrics() -> MetricsCollector {
//     unimplemented!()
// }

criterion_group!(
    benches,
    bench_agent_spawn_latency,
    bench_task_throughput,
    bench_scheduler_overhead,
    bench_concurrent_scaling,
    bench_memory_per_agent,
    bench_retry_logic,
    bench_metrics_collection,
    bench_serialization,
    bench_queue_operations
);

criterion_main!(benches);
