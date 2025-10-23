//! Test utilities and mock implementations for Layer 4 testing
//!
//! This module provides comprehensive testing utilities, mock implementations,
//! and helper functions for testing the Layer 4 execution fabric.

use chimera_layer4::types::*;
use chimera_layer4::executor::*;
use chimera_layer4::scheduler::*;
use chimera_layer4::metrics::*;
use chimera_layer4::agent_template::*;
use chimera_layer4::Layer4Fabric;
use chimera_layer4::utils;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration, Instant};
use tokio::time::{timeout, sleep};

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    /// Unique environment identifier
    pub id: String,
    /// Test configuration
    pub config: TestConfig,
    /// Temporary directory for test files
    pub temp_dir: String,
    /// Mock services
    pub mock_services: MockServices,
    /// Test metrics
    pub metrics: TestMetrics,
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Enable verbose logging
    pub verbose: bool,
    /// Test timeout in seconds
    pub timeout_secs: u64,
    /// Cleanup after tests
    pub cleanup: bool,
    /// Mock external dependencies
    pub mock_external: bool,
    /// Test data directory
    pub data_dir: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            timeout_secs: 300,
            cleanup: true,
            mock_external: true,
            data_dir: "./test_data".to_string(),
        }
    }
}

/// Mock services for testing
#[derive(Debug, Clone)]
pub struct MockServices {
    /// Mock Redis service
    pub redis: Option<MockRedis>,
    /// Mock file system
    pub filesystem: Option<MockFilesystem>,
    /// Mock network services
    pub network: Option<MockNetwork>,
    /// Mock external APIs
    pub external_apis: HashMap<String, MockApi>,
}

/// Mock Redis implementation for testing
#[derive(Debug, Clone)]
pub struct MockRedis {
    /// Mock data store
    pub data: Arc<Mutex<HashMap<String, String>>>,
    /// Connection status
    pub connected: bool,
}

impl MockRedis {
    /// Create new mock Redis instance
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            connected: true,
        }
    }

    /// Set mock Redis value
    pub fn set(&self, key: &str, value: &str) {
        if let Ok(mut data) = self.data.lock() {
            data.insert(key.to_string(), value.to_string());
        }
    }

    /// Get mock Redis value
    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(data) = self.data.lock() {
            data.get(key).cloned()
        } else {
            None
        }
    }
}

/// Mock filesystem for testing
#[derive(Debug, Clone)]
pub struct MockFilesystem {
    /// Mock files
    pub files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// Mock directories
    pub directories: Arc<Mutex<Vec<String>>>,
}

impl MockFilesystem {
    /// Create new mock filesystem
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            directories: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create mock file
    pub fn create_file(&self, path: &str, content: Vec<u8>) {
        if let Ok(mut files) = self.files.lock() {
            files.insert(path.to_string(), content);
        }
    }

    /// Read mock file
    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        if let Ok(files) = self.files.lock() {
            files.get(path).cloned()
        } else {
            None
        }
    }
}

/// Mock network services
#[derive(Debug, Clone)]
pub struct MockNetwork {
    /// Mock HTTP responses
    pub http_responses: Arc<Mutex<HashMap<String, String>>>,
    /// Network latency simulation in milliseconds
    pub latency_ms: u64,
}

impl MockNetwork {
    /// Create new mock network
    pub fn new() -> Self {
        Self {
            http_responses: Arc::new(Mutex::new(HashMap::new())),
            latency_ms: 10,
        }
    }

    /// Set mock HTTP response
    pub fn set_response(&self, url: &str, response: &str) {
        if let Ok(mut responses) = self.http_responses.lock() {
            responses.insert(url.to_string(), response.to_string());
        }
    }

    /// Get mock HTTP response
    pub fn get_response(&self, url: &str) -> Option<String> {
        if let Ok(responses) = self.http_responses.lock() {
            responses.get(url).cloned()
        } else {
            None
        }
    }
}

/// Mock API for external services
#[derive(Debug, Clone)]
pub struct MockApi {
    /// API name
    pub name: String,
    /// Mock responses
    pub responses: HashMap<String, serde_json::Value>,
    /// Response delay in milliseconds
    pub delay_ms: u64,
}

impl MockApi {
    /// Create new mock API
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            responses: HashMap::new(),
            delay_ms: 50,
        }
    }

    /// Add mock response
    pub fn add_response(&mut self, endpoint: &str, response: serde_json::Value) {
        self.responses.insert(endpoint.to_string(), response);
    }
}

/// Test metrics collection
#[derive(Debug, Clone)]
pub struct TestMetrics {
    /// Test start time
    pub start_time: SystemTime,
    /// Test execution times
    pub execution_times: Vec<Duration>,
    /// Memory usage measurements
    pub memory_usage: Vec<f64>,
    /// CPU usage measurements
    pub cpu_usage: Vec<f32>,
    /// Error counts by type
    pub errors: HashMap<String, usize>,
}

impl TestMetrics {
    /// Create new test metrics collector
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            execution_times: Vec::new(),
            memory_usage: Vec::new(),
            cpu_usage: Vec::new(),
            errors: HashMap::new(),
        }
    }

    /// Record execution time
    pub fn record_execution_time(&mut self, duration: Duration) {
        self.execution_times.push(duration);
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, usage_mb: f64) {
        self.memory_usage.push(usage_mb);
    }

    /// Record CPU usage
    pub fn record_cpu_usage(&mut self, usage_percent: f32) {
        self.cpu_usage.push(usage_percent);
    }

    /// Record error
    pub fn record_error(&mut self, error_type: &str) {
        *self.errors.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Get average execution time
    pub fn average_execution_time(&self) -> Duration {
        if self.execution_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total_nanos: u128 = self.execution_times.iter()
                .map(|d| d.as_nanos())
                .sum();
            Duration::from_nanos((total_nanos / self.execution_times.len() as u128) as u64)
        }
    }

    /// Get average memory usage
    pub fn average_memory_usage(&self) -> f64 {
        if self.memory_usage.is_empty() {
            0.0
        } else {
            self.memory_usage.iter().sum::<f64>() / self.memory_usage.len() as f64
        }
    }

    /// Get average CPU usage
    pub fn average_cpu_usage(&self) -> f32 {
        if self.cpu_usage.is_empty() {
            0.0
        } else {
            self.cpu_usage.iter().sum::<f32>() / self.cpu_usage.len() as f32
        }
    }
}

/// Initialize test environment
pub async fn init_test_environment(config: TestConfig) -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    let env_id = format!("test_env_{}",
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());

    let temp_dir = format!("{}/{}", config.data_dir, env_id);

    // Create temporary directory
    std::fs::create_dir_all(&temp_dir)?;

    // Initialize mock services
    let mock_services = MockServices {
        redis: if config.mock_external { Some(MockRedis::new()) } else { None },
        filesystem: if config.mock_external { Some(MockFilesystem::new()) } else { None },
        network: if config.mock_external { Some(MockNetwork::new()) } else { None },
        external_apis: HashMap::new(),
    };

    let test_env = TestEnvironment {
        id: env_id,
        config,
        temp_dir,
        mock_services,
        metrics: TestMetrics::new(),
    };

    if test_env.config.verbose {
        println!("  🧪 Initialized test environment: {}", test_env.id);
    }

    Ok(test_env)
}

/// Cleanup test environment
pub async fn cleanup_test_environment(mut env: TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    if env.config.cleanup {
        // Cleanup temporary files
        if std::path::Path::new(&env.temp_dir).exists() {
            std::fs::remove_dir_all(&env.temp_dir)?;
        }

        if env.config.verbose {
            println!("  🧪 Cleaned up test environment: {}", env.id);
        }
    }

    Ok(())
}

/// Create test WASM binary
pub fn create_test_wasm_binary() -> Vec<u8> {
    // Create a minimal valid WASM binary for testing
    vec![
        0x00, 0x61, 0x73, 0x6D, // WASM magic number "\0asm"
        0x01, 0x00, 0x00, 0x00, // WASM version 1
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F, // Type section
        0x03, 0x02, 0x01, 0x00, // Function section
        0x07, 0x0A, 0x01, 0x06, 0x6D, 0x65, 0x6D, 0x6F, 0x72, 0x79, 0x02, 0x00, // Export section
        0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // Code section
    ]
}

/// Create test task with default values
pub fn create_test_task() -> Task {
    Task {
        id: utils::generate_task_id(),
        priority: Priority::Normal,
        payload: serde_json::json!({"action": "test", "data": "test_data"}),
        created_at: SystemTime::now(),
        deadline: Some(SystemTime::now() + Duration::from_secs(60)),
        resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        source_layer: "test".to_string(),
        target_agent_type: "test_agent".to_string(),
        metadata: HashMap::from([("test".to_string(), "true".to_string())]),
    }
}

/// Create test agent configuration
pub fn create_test_agent_config() -> AgentConfig {
    AgentConfig {
        agent_id: utils::generate_agent_id(),
        agent_type: "test_agent".to_string(),
        resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        environment: HashMap::from([
            ("TEST_MODE".to_string(), "true".to_string()),
            ("LOG_LEVEL".to_string(), "debug".to_string()),
        ]),
        parameters: HashMap::from([
            ("test_param".to_string(), serde_json::json!("test_value")),
        ]),
    }
}

/// Create test Layer 4 configuration
pub fn create_test_layer4_config() -> Layer4Config {
    Layer4Config {
        max_agents: 10,
        default_resource_quota: ResourceQuota {
            max_cpu_cores: 1.0,
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_network_mbps: Some(10),
        },
        task_queue_capacity: 1000,
        kpi_reporting_interval_secs: 5,
        heartbeat_interval_secs: 10,
        agent_timeout_secs: 60,
        redis_url: "redis://localhost:6379".to_string(),
        metrics_port: 9090,
        debug_mode: false,
    }
}

/// Create test KPI report
pub fn create_test_kpi_report() -> KpiReport {
    KpiReport {
        task_id: utils::generate_task_id(),
        agent_id: utils::generate_agent_id(),
        latency_ms: 150.0,
        accuracy: 0.95,
        cpu_usage: 0.1,
        memory_mb: 64.0,
        network_bytes: 1024,
        custom_metrics: HashMap::from([
            ("confidence_score".to_string(), 0.87),
            ("processing_quality".to_string(), 0.92),
        ]),
        recorded_at: SystemTime::now(),
        execution_context: ExecutionContext {
            hostname: "test-host".to_string(),
            available_cores: 8,
            available_memory_mb: 16384,
            gpu_info: None,
            network_interfaces: vec!["eth0".to_string()],
        },
    }
}

/// Create test execution result
pub fn create_test_execution_result(success: bool) -> ExecutionResult {
    ExecutionResult {
        task_id: utils::generate_task_id(),
        success,
        output: if success {
            serde_json::json!({"result": "success", "data": "test_output"})
        } else {
            serde_json::json!({"error": "test_error", "details": "Test failure"})
        },
        execution_time_ms: 150,
        resource_usage: ResourceUsage {
            cpu_seconds: 0.1,
            memory_peak_mb: 64.0,
            network_tx_bytes: 512,
            network_rx_bytes: 256,
            disk_io_ops: 10,
            gpu_utilization: None,
        },
        error: if success { None } else { Some("Test error occurred".to_string()) },
        completed_at: SystemTime::now(),
    }
}

/// Wait for condition with timeout
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout_secs: u64,
    check_interval_ms: u64,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let timeout_duration = Duration::from_secs(timeout_secs);
    let start_time = Instant::now();

    while start_time.elapsed() < timeout_duration {
        if condition().await {
            return Ok(());
        }
        sleep(Duration::from_millis(check_interval_ms)).await;
    }

    Err("Condition not met within timeout".into())
}

/// Assert with timeout
pub async fn assert_with_timeout<F, Fut>(
    condition: F,
    timeout_secs: u64,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    match wait_for_condition(condition, timeout_secs, 100).await {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Assertion failed: {}", message).into()),
    }
}

/// Generate test data of specified size
pub fn generate_test_data(size_mb: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size_mb * 1024 * 1024);
    for i in 0..data.capacity() {
        data.push((i % 256) as u8);
    }
    data
}

/// Create mock agent for testing
pub struct MockAgent {
    /// Agent ID
    pub id: AgentId,
    /// Agent type
    pub agent_type: String,
    /// Supported task types
    pub supported_tasks: Vec<String>,
    /// Execution delay in milliseconds
    pub execution_delay_ms: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    /// Resource usage simulation
    pub resource_usage: ResourceUsage,
}

impl MockAgent {
    /// Create new mock agent
    pub fn new(agent_type: &str, supported_tasks: Vec<&str>) -> Self {
        Self {
            id: utils::generate_agent_id(),
            agent_type: agent_type.to_string(),
            supported_tasks: supported_tasks.iter().map(|s| s.to_string()).collect(),
            execution_delay_ms: 100,
            success_rate: 0.9,
            resource_usage: ResourceUsage {
                cpu_seconds: 0.1,
                memory_peak_mb: 64.0,
                network_tx_bytes: 1024,
                network_rx_bytes: 512,
                disk_io_ops: 10,
                gpu_utilization: None,
            },
        }
    }

    /// Execute mock task
    pub async fn execute_mock_task(&self, task: &Task) -> Layer4Result<ExecutionResult> {
        // Simulate execution delay
        sleep(Duration::from_millis(self.execution_delay_ms)).await;

        // Simulate success/failure based on success rate
        let success = rand::random::<f32>() < self.success_rate;

        Ok(ExecutionResult {
            task_id: task.id,
            success,
            output: if success {
                serde_json::json!({
                    "result": "mock_success",
                    "agent_id": self.id,
                    "agent_type": self.agent_type,
                    "execution_delay_ms": self.execution_delay_ms
                })
            } else {
                serde_json::json!({
                    "error": "mock_failure",
                    "agent_id": self.id,
                    "agent_type": self.agent_type
                })
            },
            execution_time_ms: self.execution_delay_ms,
            resource_usage: self.resource_usage.clone(),
            error: if success { None } else { Some("Mock agent failure".to_string()) },
            completed_at: SystemTime::now(),
        })
    }
}

impl WasmAgent for MockAgent {
    fn init(&mut self, _config: AgentConfig) -> Layer4Result<()> {
        Ok(())
    }

    fn execute_task(&mut self, task: Task) -> Layer4Result<ExecutionResult> {
        // Use async execution through tokio runtime
        let agent = self;
        let task_clone = task;

        // This is a simplified synchronous implementation
        // In a real implementation, this would be async
        Ok(ExecutionResult {
            task_id: task_clone.id,
            success: true,
            output: serde_json::json!({"mock": "result"}),
            execution_time_ms: 100,
            resource_usage: ResourceUsage {
                cpu_seconds: 0.1,
                memory_peak_mb: 64.0,
                network_tx_bytes: 0,
                network_rx_bytes: 0,
                disk_io_ops: 0,
                gpu_utilization: None,
            },
            error: None,
            completed_at: SystemTime::now(),
        })
    }

    fn get_capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            supported_task_types: self.supported_tasks.clone(),
            max_concurrent_tasks: 5,
            resource_quota: ResourceQuota {
                max_cpu_cores: 1.0,
                max_memory_mb: 256,
                max_execution_time_secs: 30,
                max_network_mbps: Some(10),
            },
            required_env_vars: HashMap::new(),
            features: vec!["mock".to_string(), "test".to_string()],
        }
    }

    fn shutdown(&mut self) -> Layer4Result<()> {
        Ok(())
    }

    fn health_check(&self) -> AgentHealth {
        AgentHealth {
            status: HealthStatus::Healthy,
            resource_usage: self.resource_usage.clone(),
            last_success: Some(SystemTime::now()),
            error_count: 0,
            metrics: HashMap::new(),
        }
    }
}

/// Test data generator
pub struct TestDataGenerator {
    /// Random seed for reproducible tests
    pub seed: u64,
}

impl TestDataGenerator {
    /// Create new test data generator
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate test tasks
    pub fn generate_tasks(&self, count: usize, priority_distribution: &[f32]) -> Vec<Task> {
        let mut tasks = Vec::new();

        for i in 0..count {
            let priority_roll = (i as f32 / count as f32) * priority_distribution.iter().sum::<f32>();
            let priority = if priority_roll < priority_distribution[0] {
                Priority::Critical
            } else if priority_roll < priority_distribution[0] + priority_distribution[1] {
                Priority::High
            } else if priority_roll < priority_distribution[0] + priority_distribution[1] + priority_distribution[2] {
                Priority::Normal
            } else if priority_roll < priority_distribution[0] + priority_distribution[1] + priority_distribution[2] + priority_distribution[3] {
                Priority::Low
            } else {
                Priority::Background
            };

            let task = Task {
                id: utils::generate_task_id(),
                priority,
                payload: serde_json::json!({
                    "task_id": i,
                    "data_size": 100 + (i % 1000),
                    "complexity": i % 10,
                }),
                created_at: SystemTime::now(),
                deadline: Some(SystemTime::now() + Duration::from_secs(60)),
                resource_quota: ResourceQuota {
                    max_cpu_cores: 0.5 + (i as f32 % 2.0),
                    max_memory_mb: 128 + (i % 512),
                    max_execution_time_secs: 30,
                    max_network_mbps: Some(5 + (i % 20)),
                },
                source_layer: format!("test_layer_{}", i % 5),
                target_agent_type: format!("test_agent_{}", i % 3),
                metadata: HashMap::from([
                    ("batch_id".to_string(), (i / 100).to_string()),
                    ("test_generator".to_string(), "true".to_string()),
                ]),
            };

            tasks.push(task);
        }

        tasks
    }

    /// Generate test WASM binaries
    pub fn generate_wasm_binaries(&self, count: usize) -> Vec<Vec<u8>> {
        let mut binaries = Vec::new();

        for i in 0..count {
            // Generate slightly different WASM binaries for testing
            let mut binary = create_test_wasm_binary();

            // Modify binary slightly based on index for variety
            if i < binary.len() {
                binary[i % binary.len()] = binary[i % binary.len()].wrapping_add(i as u8);
            }

            binaries.push(binary);
        }

        binaries
    }
}

/// Performance measurement utilities
pub struct PerformanceMeasurer {
    /// Measurement start time
    pub start_time: Option<Instant>,
    /// Measurements collected
    pub measurements: Vec<Duration>,
}

impl PerformanceMeasurer {
    /// Create new performance measurer
    pub fn new() -> Self {
        Self {
            start_time: None,
            measurements: Vec::new(),
        }
    }

    /// Start measurement
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// End measurement and record duration
    pub fn end(&mut self) {
        if let Some(start) = self.start_time {
            let duration = start.elapsed();
            self.measurements.push(duration);
            self.start_time = None;
        }
    }

    /// Get average measurement
    pub fn average(&self) -> Duration {
        if self.measurements.is_empty() {
            Duration::from_millis(0)
        } else {
            let total_nanos: u128 = self.measurements.iter()
                .map(|d| d.as_nanos())
                .sum();
            Duration::from_nanos((total_nanos / self.measurements.len() as u128) as u64)
        }
    }

    /// Get minimum measurement
    pub fn min(&self) -> Duration {
        self.measurements.iter().min().cloned().unwrap_or(Duration::from_millis(0))
    }

    /// Get maximum measurement
    pub fn max(&self) -> Duration {
        self.measurements.iter().max().cloned().unwrap_or(Duration::from_millis(0))
    }

    /// Get measurement count
    pub fn count(&self) -> usize {
        self.measurements.len()
    }
}

/// Load testing utilities
pub struct LoadTester {
    /// Number of concurrent workers
    pub workers: usize,
    /// Tasks per worker
    pub tasks_per_worker: usize,
    /// Worker handles
    pub handles: Vec<tokio::task::JoinHandle<()>>,
}

impl LoadTester {
    /// Create new load tester
    pub fn new(workers: usize, tasks_per_worker: usize) -> Self {
        Self {
            workers,
            tasks_per_worker,
            handles: Vec::new(),
        }
    }

    /// Execute load test
    pub async fn execute_load_test<F, Fut>(
        &mut self,
        test_fn: F,
    ) -> Result<Vec<Duration>, Box<dyn std::error::Error>>
    where
        F: Fn(usize, usize) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Duration> + Send,
    {
        let mut execution_times = Vec::new();

        for worker_id in 0..self.workers {
            let test_fn_clone = test_fn.clone();
            let tasks_per_worker = self.tasks_per_worker;

            let handle = tokio::spawn(async move {
                for task_id in 0..tasks_per_worker {
                    let execution_time = test_fn_clone(worker_id, task_id).await;
                    // In a real implementation, would collect execution times
                    let _ = execution_time;
                }
            });

            self.handles.push(handle);
        }

        // Wait for all workers to complete
        for handle in self.handles.drain(..) {
            handle.await?;
        }

        Ok(execution_times)
    }
}

/// Test result validator
pub struct TestResultValidator {
    /// Expected success rate
    pub expected_success_rate: f32,
    /// Maximum allowed latency in milliseconds
    pub max_latency_ms: f64,
    /// Minimum throughput in tasks per second
    pub min_throughput_tps: f64,
}

impl TestResultValidator {
    /// Create new test result validator
    pub fn new() -> Self {
        Self {
            expected_success_rate: 0.95,
            max_latency_ms: 1000.0,
            min_throughput_tps: 10.0,
        }
    }

    /// Validate test results
    pub fn validate_results(&self, results: &[ExecutionResult], duration: Duration) -> Result<(), String> {
        let total_tasks = results.len();
        let successful_tasks = results.iter().filter(|r| r.success).count();
        let success_rate = successful_tasks as f32 / total_tasks as f32;

        let avg_latency = results.iter()
            .map(|r| r.execution_time_ms as f64)
            .sum::<f64>() / total_tasks as f64;

        let throughput_tps = total_tasks as f64 / duration.as_secs_f64();

        if success_rate < self.expected_success_rate {
            return Err(format!(
                "Success rate too low: {:.2}% < {:.2}%",
                success_rate * 100.0, self.expected_success_rate * 100.0
            ));
        }

        if avg_latency > self.max_latency_ms {
            return Err(format!(
                "Average latency too high: {:.2}ms > {:.2}ms",
                avg_latency, self.max_latency_ms
            ));
        }

        if throughput_tps < self.min_throughput_tps {
            return Err(format!(
                "Throughput too low: {:.2} TPS < {:.2} TPS",
                throughput_tps, self.min_throughput_tps
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_redis() {
        let redis = MockRedis::new();
        assert!(redis.connected);
        assert!(redis.get("nonexistent").is_none());

        redis.set("test_key", "test_value");
        assert_eq!(redis.get("test_key"), Some("test_value".to_string()));
    }

    #[test]
    fn test_mock_filesystem() {
        let fs = MockFilesystem::new();
        assert!(fs.files.lock().unwrap().is_empty());

        let test_data = vec![1, 2, 3, 4, 5];
        fs.create_file("/test/file.txt", test_data.clone());
        assert_eq!(fs.read_file("/test/file.txt"), Some(test_data));
    }

    #[test]
    fn test_test_data_generator() {
        let generator = TestDataGenerator::new(42);

        let tasks = generator.generate_tasks(10, &[0.1, 0.2, 0.4, 0.2, 0.1]);
        assert_eq!(tasks.len(), 10);

        let binaries = generator.generate_wasm_binaries(5);
        assert_eq!(binaries.len(), 5);
        assert!(!binaries[0].is_empty());
    }

    #[test]
    fn test_performance_measurer() {
        let mut measurer = PerformanceMeasurer::new();

        measurer.start();
        std::thread::sleep(Duration::from_millis(10));
        measurer.end();

        assert_eq!(measurer.count(), 1);
        assert!(measurer.average() >= Duration::from_millis(10));
    }

    #[test]
    fn test_test_result_validator() {
        let validator = TestResultValidator::new();

        let results = vec![
            create_test_execution_result(true),
            create_test_execution_result(true),
            create_test_execution_result(false),
        ];

        let validation = validator.validate_results(&results, Duration::from_secs(1));
        assert!(validation.is_ok());
    }
}