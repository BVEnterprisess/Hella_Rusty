//! Core type definitions for Layer 4 Execution Fabric
//!
//! This module defines all the essential data structures used throughout
//! the WASM agent execution system, including tasks, KPIs, agent states,
//! and inter-layer communication protocols.
//!
//! All types implement proper serialization for cross-layer communication
//! and provide comprehensive error handling for production use.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Unique identifier for tasks in the execution system
pub type TaskId = Uuid;

/// Unique identifier for WASM agents
pub type AgentId = Uuid;

/// Priority levels for task scheduling
///
/// Higher values indicate higher priority. The scheduler uses this to
/// determine task execution order, with Critical tasks always executing
/// before Background tasks.
///
/// # Examples
/// ```
/// use chimera_layer4::Priority;
///
/// let critical_task = Priority::Critical; // Executes first
/// let background_task = Priority::Background; // Executes last
/// assert!(critical_task > background_task);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Mission-critical tasks that must execute immediately
    Critical = 100,
    /// High-priority tasks for important operations
    High = 75,
    /// Standard priority for regular tasks
    Normal = 50,
    /// Low-priority tasks that can wait
    Low = 25,
    /// Background tasks with minimal priority
    Background = 1,
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by discriminant value (Critical=100 > High=75 > ... > Background=1)
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Execution state of a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is queued and waiting for execution
    Pending,
    /// Task is currently being executed by an agent
    Running,
    /// Task completed successfully
    Completed,
    /// Task execution failed
    Failed,
    /// Task was cancelled before completion
    Cancelled,
    /// Task exceeded execution timeout
    Timeout,
}

/// Resource quotas for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Maximum CPU cores to allocate
    pub max_cpu_cores: f32,
    /// Maximum memory in MB
    pub max_memory_mb: u32,
    /// Maximum execution time
    pub max_execution_time_secs: u64,
    /// Maximum network bandwidth (optional)
    pub max_network_mbps: Option<u32>,
}

/// Comprehensive task definition
///
/// A Task represents a unit of work to be executed by the Layer 4 execution fabric.
/// Tasks flow through the system from discovery (Layer 2) to validation (Layer 3)
/// to execution (Layer 4) to refinement (Layer 5) and evolution (Layer 7).
///
/// # Examples
/// ```
/// use chimera_layer4::*;
/// use std::time::SystemTime;
///
/// let task = Task {
///     id: utils::generate_task_id(),
///     priority: Priority::High,
///     payload: serde_json::json!({"action": "analyze", "data": "file.json"}),
///     created_at: SystemTime::now(),
///     deadline: None,
///     resource_quota: ResourceQuota::default(),
///     source_layer: "layer2".to_string(),
///     target_agent_type: "data_analyzer".to_string(),
///     metadata: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    ///
    /// Generated using UUID v4 for global uniqueness across the entire
    /// Project Chimera ecosystem.
    pub id: TaskId,

    /// Task priority for scheduling
    ///
    /// Higher priority tasks are executed before lower priority ones.
    /// Critical tasks always execute before Background tasks.
    pub priority: Priority,

    /// Task payload containing execution parameters
    ///
    /// Uses JSON for maximum flexibility across different agent types.
    /// The payload structure depends on the target_agent_type.
    pub payload: serde_json::Value,

    /// Creation timestamp
    ///
    /// When the task was first created in the system.
    /// Used for scheduling and timeout calculations.
    pub created_at: SystemTime,

    /// Optional deadline for completion
    ///
    /// If specified, the task must complete by this time or be cancelled.
    /// Used for SLA enforcement and priority boosting.
    pub deadline: Option<SystemTime>,

    /// Resource requirements for task execution
    ///
    /// Specifies CPU, memory, time, and network constraints.
    /// Enforced by the WASM runtime for security and fairness.
    pub resource_quota: ResourceQuota,

    /// Source layer that created this task
    ///
    /// Identifies which layer (layer2, layer3, etc.) originated the task.
    /// Used for tracing and debugging execution flow.
    pub source_layer: String,

    /// Target agent type for execution
    ///
    /// Specifies which type of WASM agent should handle this task.
    /// Must match an available agent's supported_task_types.
    pub target_agent_type: String,

    /// Execution metadata
    ///
    /// Additional key-value pairs for task-specific information.
    /// Can include tracing IDs, user context, or custom parameters.
    pub metadata: HashMap<String, String>,
}

/// Key Performance Indicators reported by agents
///
/// KpiReport contains comprehensive performance metrics from task execution.
/// These metrics feed into Layer 5 (Refinement) for continuous improvement
/// and Layer 7 (Evolution) for agent genome optimization.
///
/// # Examples
/// ```
/// use chimera_layer4::*;
/// use std::time::SystemTime;
///
/// let report = KpiReport {
///     task_id: Uuid::new_v4(),
///     agent_id: Uuid::new_v4(),
///     latency_ms: 150.0,
///     accuracy: 0.95,
///     cpu_usage: 0.1,
///     memory_mb: 64.0,
///     network_bytes: 1024,
///     custom_metrics: HashMap::from([
///         ("confidence_score".to_string(), 0.87),
///         ("processing_quality".to_string(), 0.92),
///     ]),
///     recorded_at: SystemTime::now(),
///     execution_context: ExecutionContext {
///         hostname: "agent-node-1".to_string(),
///         available_cores: 8,
///         available_memory_mb: 16384,
///         gpu_info: None,
///         network_interfaces: vec!["eth0".to_string()],
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiReport {
    /// Associated task ID
    ///
    /// Links this KPI report to the specific task that was executed.
    /// Used for correlating performance data with task outcomes.
    pub task_id: TaskId,

    /// Agent that generated this report
    ///
    /// Identifies which WASM agent instance produced these metrics.
    /// Used for agent-specific performance tracking and evolution.
    pub agent_id: AgentId,

    /// Execution latency in milliseconds
    ///
    /// Total time taken from task start to completion.
    /// Lower values indicate better performance.
    pub latency_ms: f64,

    /// Accuracy/success rate (0.0 to 1.0)
    ///
    /// Measure of task execution quality. 1.0 indicates perfect execution,
    /// 0.0 indicates complete failure. Used for fitness scoring.
    pub accuracy: f64,

    /// CPU usage during execution (0.0 to 1.0)
    ///
    /// Fraction of available CPU cores utilized during task execution.
    /// Used for resource optimization and capacity planning.
    pub cpu_usage: f32,

    /// Memory usage in MB
    ///
    /// Peak memory consumption during task execution.
    /// Used for memory quota optimization and leak detection.
    pub memory_mb: f32,

    /// Network I/O in bytes
    ///
    /// Total bytes transmitted and received during execution.
    /// Used for network quota enforcement and cost optimization.
    pub network_bytes: u64,

    /// Custom metrics specific to the task type
    ///
    /// Agent-specific performance indicators. Examples include
    /// confidence scores, quality metrics, or domain-specific KPIs.
    pub custom_metrics: HashMap<String, f64>,

    /// Timestamp when KPIs were recorded
    ///
    /// When the metrics were captured from the agent.
    /// Used for time-series analysis and trend detection.
    pub recorded_at: SystemTime,

    /// Execution environment details
    ///
    /// Information about the runtime environment where the task executed.
    /// Used for performance correlation and debugging.
    pub execution_context: ExecutionContext,
}

/// Execution context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Host system information
    pub hostname: String,
    /// Available CPU cores
    pub available_cores: usize,
    /// Available memory in MB
    pub available_memory_mb: u64,
    /// GPU information (if available)
    pub gpu_info: Option<GpuInfo>,
    /// Network interfaces
    pub network_interfaces: Vec<String>,
}

/// GPU information for execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU model name
    pub name: String,
    /// GPU memory in MB
    pub memory_mb: u32,
    /// CUDA compute capability (if applicable)
    pub compute_capability: Option<String>,
    /// Driver version
    pub driver_version: String,
}

/// Agent lifecycle states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is being initialized
    Initializing,
    /// Agent is ready to accept tasks
    Idle,
    /// Agent is currently executing a task
    Busy,
    /// Agent has failed and needs attention
    Failed,
    /// Agent is being terminated
    Terminating,
    /// Agent is permanently stopped
    Stopped,
}

/// WASM agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmAgent {
    /// Unique agent identifier
    pub id: AgentId,
    /// Agent type/class name
    pub agent_type: String,
    /// Current state
    pub state: AgentState,
    /// WASM binary data
    pub wasm_binary: Vec<u8>,
    /// Agent capabilities and metadata
    pub capabilities: AgentCapabilities,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last heartbeat timestamp
    pub last_heartbeat: SystemTime,
    /// Execution statistics
    pub stats: AgentStats,
}

/// Agent capabilities and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Supported task types
    pub supported_task_types: Vec<String>,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Resource requirements
    pub resource_quota: ResourceQuota,
    /// Required environment variables
    pub required_env_vars: HashMap<String, String>,
    /// Optional features
    pub features: Vec<String>,
}

/// Agent execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    /// Total tasks completed
    pub tasks_completed: u64,
    /// Total tasks failed
    pub tasks_failed: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Total CPU time consumed (seconds)
    pub total_cpu_seconds: f64,
    /// Total memory peak in MB
    pub memory_peak_mb: f32,
    /// Last successful execution
    pub last_success_at: Option<SystemTime>,
    /// Last failure timestamp
    pub last_failure_at: Option<SystemTime>,
}

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Unique request identifier
    pub id: Option<serde_json::Value>,
    /// Method name to invoke
    pub method: String,
    /// Method parameters
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request identifier (matches request)
    pub id: Option<serde_json::Value>,
    /// Successful result (mutually exclusive with error)
    pub result: Option<serde_json::Value>,
    /// Error details (mutually exclusive with result)
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code (-32000 to -32099 for implementation-defined errors)
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional error data
    pub data: Option<serde_json::Value>,
}

/// Execution result from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Associated task ID
    pub task_id: TaskId,
    /// Success or failure status
    pub success: bool,
    /// Execution output data
    pub output: serde_json::Value,
    /// Execution duration
    pub execution_time_ms: u64,
    /// Resource usage during execution
    pub resource_usage: ResourceUsage,
    /// Error details if execution failed
    pub error: Option<String>,
    /// Timestamp of completion
    pub completed_at: SystemTime,
}

/// Detailed resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU time used in seconds
    pub cpu_seconds: f64,
    /// Peak memory usage in MB
    pub memory_peak_mb: f32,
    /// Network bytes transmitted
    pub network_tx_bytes: u64,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Disk I/O operations
    pub disk_io_ops: u64,
    /// GPU utilization (0.0 to 1.0, if applicable)
    pub gpu_utilization: Option<f32>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_seconds: 0.0,
            memory_peak_mb: 0.0,
            network_tx_bytes: 0,
            network_rx_bytes: 0,
            disk_io_ops: 0,
            gpu_utilization: None,
        }
    }
}

/// Configuration for the Layer 4 execution system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer4Config {
    /// Maximum number of concurrent agents
    pub max_agents: usize,
    /// Default resource quotas for agents
    pub default_resource_quota: ResourceQuota,
    /// Task queue capacity
    pub task_queue_capacity: usize,
    /// KPI reporting interval
    pub kpi_reporting_interval_secs: u64,
    /// Agent heartbeat interval
    pub heartbeat_interval_secs: u64,
    /// Agent timeout threshold
    pub agent_timeout_secs: u64,
    /// Redis connection string for event bus
    pub redis_url: String,
    /// Prometheus metrics port
    pub metrics_port: u16,
    /// Enable debug logging
    pub debug_mode: bool,
}

/// Error types for the Layer 4 system
///
/// Comprehensive error handling for all Layer 4 operations.
/// Each error type includes context for debugging and monitoring.
/// All errors implement Display and are serializable for distributed tracing.
///
/// # Examples
/// ```
/// use chimera_layer4::*;
///
/// async fn example() -> Layer4Result<()> {
///     let task = utils::default_task();
///     // Task execution that might fail...
///     Ok(())
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum Layer4Error {
    /// WASM runtime errors from wasmtime execution
    ///
    /// Occurs when WASM agent execution fails due to compilation,
    /// instantiation, or runtime errors. Usually indicates a problem
    /// with the agent binary or resource constraints.
    #[error("WASM runtime error: {0}")]
    WasmRuntime(#[from] wasmtime::Error),

    /// Serialization/deserialization errors
    ///
    /// Occurs when converting between Rust types and JSON/network formats.
    /// Usually indicates a data structure mismatch or corrupted data.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Task not found in the system
    ///
    /// Occurs when attempting to operate on a task that doesn't exist
    /// in the scheduler or executor. Usually indicates a race condition
    /// or incorrect task lifecycle management.
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),

    /// Agent not found in the system
    ///
    /// Occurs when attempting to operate on an agent that doesn't exist
    /// or has been terminated. Usually indicates agent lifecycle issues.
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),

    /// Resource quota exceeded
    ///
    /// Occurs when an agent or task attempts to use more resources
    /// than allocated. This is a security feature to prevent resource
    /// exhaustion attacks and ensure fair resource distribution.
    #[error("Resource quota exceeded: {0}")]
    ResourceQuotaExceeded(String),

    /// Agent execution timeout
    ///
    /// Occurs when an agent fails to complete within the allocated time.
    /// This prevents hung agents from blocking the system and enables
    /// automatic recovery through retry mechanisms.
    #[error("Agent timeout after {0} seconds")]
    AgentTimeout(u64),

    /// Inter-process communication errors
    ///
    /// Occurs when communication between system components fails.
    /// Usually indicates network issues, serialization problems,
    /// or component lifecycle mismatches.
    #[error("Communication error: {0}")]
    Communication(#[from] std::io::Error),

    /// Redis backend errors
    ///
    /// Occurs when the Redis event bus or cache is unavailable.
    /// This affects task queuing, metrics collection, and inter-layer
    /// communication. Usually indicates infrastructure issues.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Configuration validation errors
    ///
    /// Occurs when system configuration is invalid or incomplete.
    /// This is usually caught during startup and indicates deployment
    /// or configuration management issues.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal system errors
    ///
    /// Catch-all for unexpected errors in the Layer 4 system.
    /// These should be rare in production and usually indicate
    /// bugs or unexpected runtime conditions.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Prometheus metrics errors
    #[error("Prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),

    /// System time errors
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
}

/// Result type alias for Layer 4 operations
pub type Layer4Result<T> = Result<T, Layer4Error>;

/// Health status of the Layer 4 system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall system status
    pub status: HealthStatus,
    /// Number of active agents
    pub active_agents: usize,
    /// Number of pending tasks
    pub pending_tasks: usize,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
    /// Last health check timestamp
    pub last_check: SystemTime,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is operating normally
    Healthy,
    /// System is operational but performance is degraded
    Degraded,
    /// System is not functioning correctly
    Unhealthy,
    /// System is in a critical state and requires immediate attention
    Critical,
}

/// System resource utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization (0.0 to 1.0)
    pub cpu_usage: f32,
    /// Memory utilization (0.0 to 1.0)
    pub memory_usage: f32,
    /// Disk utilization (0.0 to 1.0)
    pub disk_usage: f32,
    /// Network utilization (0.0 to 1.0)
    pub network_usage: f32,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_cpu_cores: 1.0,
            max_memory_mb: 512,
            max_execution_time_secs: 300, // 5 minutes
            max_network_mbps: Some(10),
        }
    }
}

impl Default for Layer4Config {
    fn default() -> Self {
        Self {
            max_agents: 10,
            default_resource_quota: ResourceQuota::default(),
            task_queue_capacity: 1000,
            kpi_reporting_interval_secs: 5,
            heartbeat_interval_secs: 10,
            agent_timeout_secs: 60,
            redis_url: "redis://localhost:6379".to_string(),
            metrics_port: 9090,
            debug_mode: false,
        }
    }
}
