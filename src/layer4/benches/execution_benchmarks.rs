//! Layer 4 Execution Benchmarks
//!
//! Comprehensive performance benchmarks for agent spawning, task execution,
//! and resource management. These benchmarks establish baseline performance
//! and detect regressions.

use chimera_layer4::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::SystemTime;
use tokio::runtime::Runtime;

/// Benchmark agent spawning performance
fn bench_agent_spawn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("agent_spawn");
    
    // Target: <50ms per agent spawn
    group.bench_function("spawn_single_agent", |b| {
        b.to_async(&rt).iter(|| async {
            let config = Layer4Config::default();
            let layer4 = Layer4Fabric::new(config).await.unwrap();
            
            let wasm_binary = vec![0u8; 100]; // Mock WASM binary
            let agent_config = AgentConfig {
                agent_id: utils::generate_agent_id(),
                agent_type: "test_agent".to_string(),
                resource_quota: utils::default_resource_quota(),
                environment: std::collections::HashMap::new(),
                parameters: std::collections::HashMap::new(),
            };
            
            black_box(layer4.spawn_agent(wasm_binary, agent_config).await.unwrap())
        });
    });
    
    group.finish();
}

/// Benchmark task execution latency
fn bench_task_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("task_execution");
    group.throughput(Throughput::Elements(1));
    
    // Target: <100ms average latency
    group.bench_function("execute_simple_task", |b| {
        b.to_async(&rt).iter(|| async {
            let config = Layer4Config::default();
            let layer4 = Layer4Fabric::new(config).await.unwrap();
            layer4.start().await.unwrap();
            
            let task = utils::default_task();
            
            black_box(layer4.execute_task(task).await.unwrap())
        });
    });
    
    group.finish();
}

/// Benchmark task scheduling throughput
fn bench_task_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("task_throughput");
    
    for task_count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*task_count as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(task_count),
            task_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let config = Layer4Config {
                        task_queue_capacity: 10000,
                        ..Default::default()
                    };
                    let layer4 = Layer4Fabric::new(config).await.unwrap();
                    layer4.start().await.unwrap();
                    
                    let tasks: Vec<_> = (0..count)
                        .map(|_| utils::default_task())
                        .collect();
                    
                    for task in tasks {
                        black_box(layer4.execute_task(task).await.unwrap());
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent agent performance
fn bench_concurrent_agents(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_agents");
    
    // Target: 10+ concurrent agents
    for agent_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(agent_count),
            agent_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let config = Layer4Config {
                        max_agents: count,
                        ..Default::default()
                    };
                    let layer4 = Layer4Fabric::new(config).await.unwrap();
                    layer4.start().await.unwrap();
                    
                    // Spawn multiple agents concurrently
                    let mut handles = vec![];
                    for _ in 0..count {
                        let wasm_binary = vec![0u8; 100];
                        let agent_config = AgentConfig {
                            agent_id: utils::generate_agent_id(),
                            agent_type: "test_agent".to_string(),
                            resource_quota: utils::default_resource_quota(),
                            environment: std::collections::HashMap::new(),
                            parameters: std::collections::HashMap::new(),
                        };
                        
                        handles.push(layer4.spawn_agent(wasm_binary, agent_config));
                    }
                    
                    // Wait for all agents to spawn
                    for handle in handles {
                        black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_agent_spawn,
    bench_task_execution,
    bench_task_throughput,
    bench_concurrent_agents
);
criterion_main!(benches);
