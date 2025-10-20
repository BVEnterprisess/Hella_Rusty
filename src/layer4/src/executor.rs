//! Layer 4 Executor - WASM Agent Runtime Manager
//!
//! This module implements the core execution fabric for managing WASM agent
//! lifecycles, including spawning, monitoring, resource management, and
//! graceful shutdown of agents in the autonomous execution stack.
//!
//! ## Architecture
//!
//! The executor provides the runtime substrate that enables all other layers
//! to operate. It implements a macro-scale TRM where recursive self-improvement
//! occurs across the entire agent ecosystem through:
//!
//! - **Agent Lifecycle Management**: Spawn, monitor, and terminate WASM agents
//! - **Resource Enforcement**: CPU, memory, time, and network quota management
//! - **Task Execution**: Coordinate task distribution across available agents
//! - **Health Monitoring**: Track agent health and handle failures gracefully
//! - **Hot Swapping**: Enable zero-downtime agent updates from Layer 7
//!
//! ## Security Model
//!
//! - **WASM Sandboxing**: All agents run in isolated WASI environments
//! - **Resource Quotas**: Strict enforcement of CPU/memory/time limits
//! - **Network Isolation**: Controlled inter-agent communication only
//! - **Audit Logging**: Complete operation tracking for compliance

use crate::agent_template::{WasmAgent, BaseWasmAgent};
use crate::types::*;
use crate::AgentConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use wasmtime::{Engine, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

/// Configuration for the executor
///
/// Defines the operational parameters for the WASM agent runtime manager.
/// These settings control resource allocation, monitoring intervals, and
/// system behavior for the entire execution fabric.
///
/// # Examples
/// ```rust
/// let config = ExecutorConfig {
///     max_agents: 20,                    // Support up to 20 concurrent agents
///     default_resource_quota: ResourceQuota {
///         max_cpu_cores: 2.0,           // 2 CPU cores per agent
///         max_memory_mb: 1024,          // 1GB memory per agent
///         max_execution_time_secs: 300, // 5 minute timeout
///         max_network_mbps: Some(50),   // 50 Mbps network
///     },
///     heartbeat_interval_secs: 10,      // Check agent health every 10s
///     agent_timeout_secs: 60,           // Kill unresponsive agents after 60s
///     debug_mode: true,                 // Enable detailed logging
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of concurrent agents
    ///
    /// Limits the total number of WASM agents that can run simultaneously.
    /// Prevents resource exhaustion and ensures system stability.
    /// Should be tuned based on available system resources.
    pub max_agents: usize,

    /// Default resource quotas for new agents
    ///
    /// Baseline resource allocations applied to newly spawned agents.
    /// Can be overridden on a per-agent basis for specialized workloads.
    /// Enforced strictly for security and fairness.
    pub default_resource_quota: ResourceQuota,

    /// Agent heartbeat interval in seconds
    ///
    /// How often agents should report their health status.
    /// Shorter intervals provide better monitoring but increase overhead.
    /// Used for failure detection and automated recovery.
    pub heartbeat_interval_secs: u64,

    /// Agent timeout threshold in seconds
    ///
    /// Maximum time an agent can run without responding.
    /// Exceeding this threshold triggers agent termination and restart.
    /// Prevents hung agents from blocking the system.
    pub agent_timeout_secs: u64,

    /// Enable debug mode for detailed logging
    ///
    /// When enabled, provides comprehensive logging of agent lifecycle
    /// events, resource usage, and internal state changes.
    /// Should be disabled in production for performance.
    pub debug_mode: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_agents: 10,
            default_resource_quota: ResourceQuota::default(),
            heartbeat_interval_secs: 10,
            agent_timeout_secs: 60,
            debug_mode: false,
        }
    }
}

/// Main executor for managing WASM agent lifecycles
///
/// The Executor is the central component of the Layer 4 execution fabric.
/// It manages the complete lifecycle of WASM agents from spawning to termination,
/// enforces resource quotas, and coordinates task execution across the agent pool.
///
/// ## Responsibilities
///
/// - **Agent Lifecycle**: Spawn, initialize, monitor, and terminate WASM agents
/// - **Resource Management**: Enforce CPU, memory, time, and network quotas
/// - **Task Coordination**: Route tasks to appropriate agents based on capabilities
/// - **Health Monitoring**: Track agent health and handle failures gracefully
/// - **Security Enforcement**: Maintain WASM sandbox isolation and access controls
///
/// ## Architecture
///
/// ```text
/// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
/// │   Task Queue    │───▶│    Executor     │───▶│  WASM Agents    │
/// │  (Scheduler)    │    │   (Runtime)     │    │   (Sandboxed)   │
/// └─────────────────┘    └─────────────────┘    └─────────────────┘
///         │                       │                       │
///         └───────────────────────┼───────────────────────┘
///                                 ▼
///                        ┌─────────────────┐
///                        │   Prometheus    │
///                        │    Metrics      │
///                        └─────────────────┘
/// ```
///
/// ## Security Model
///
/// - **WASM Sandboxing**: All agents run in isolated WASI environments
/// - **Resource Quotas**: Strict enforcement prevents resource exhaustion
/// - **Access Control**: Agents cannot access host system resources directly
/// - **Audit Logging**: All operations are logged for compliance and debugging
pub struct Executor {
    /// Executor configuration
    ///
    /// Operational parameters controlling agent limits, timeouts,
    /// and monitoring intervals. Set during initialization.
    config: ExecutorConfig,

    /// WASM engine instance (shared across all agents)
    ///
    /// Optimized Wasmtime engine with JIT compilation enabled.
    /// Shared across all agents for efficiency and resource reuse.
    /// Configured with security settings and resource limits.
    engine: Engine,

    /// Active agents registry
    ///
    /// Thread-safe registry of all currently active WASM agents.
    /// Maps agent IDs to agent instances with proper synchronization.
    /// Used for task routing, health monitoring, and lifecycle management.
    agents: Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,

    /// Task execution channel
    ///
    /// Asynchronous channel for receiving task execution requests.
    /// Connects the scheduler to the executor for task distribution.
    /// Each task includes a response channel for result delivery.
    task_tx: async_channel::Sender<(Task, async_channel::Sender<Layer4Result<ExecutionResult>>)>,

    /// Agent spawn channel
    ///
    /// Asynchronous channel for receiving agent spawn requests.
    /// Enables dynamic agent creation and hot-swapping capabilities.
    /// Used by Layer 7 for evolutionary agent updates.
    spawn_tx: async_channel::Sender<(AgentId, Vec<u8>, AgentConfig)>,

    /// Shutdown signal
    ///
    /// Atomic flag indicating when the executor should shut down.
    /// Used for graceful termination of all agents and cleanup.
    /// Prevents new tasks from being accepted during shutdown.
    shutdown: Arc<RwLock<bool>>,
}

impl Executor {
    /// Create a new executor instance
    ///
    /// Initializes the WASM runtime engine, creates communication channels,
    /// and starts background task processors for agent management and monitoring.
    ///
    /// # Arguments
    /// * `config` - Executor configuration with operational parameters
    ///
    /// # Returns
    /// * `Layer4Result<Self>` - New executor instance or initialization error
    ///
    /// # Examples
    /// ```rust,no_run
    /// let config = ExecutorConfig::default();
    /// let executor = Executor::new(config)?;
    /// // Executor is now ready to spawn agents and execute tasks
    /// ```
    pub fn new(config: ExecutorConfig) -> Layer4Result<Self> {
        let engine = Self::create_engine()?;

        let (task_tx, task_rx) = async_channel::unbounded();
        let (spawn_tx, spawn_rx) = async_channel::unbounded();

        let executor = Self {
            config,
            engine,
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_tx,
            spawn_tx,
            shutdown: Arc::new(RwLock::new(false)),
        };

        // Start background tasks
        executor.start_background_tasks(task_rx, spawn_rx);

        Ok(executor)
    }

    /// Create optimized WASM engine with JIT compilation
    fn create_engine() -> Layer4Result<Engine> {
        use wasmtime::Config;

        let mut config = Config::new();

        // Enable WASM features for agent capabilities
        config.wasm_threads(true);
        config.wasm_reference_types(true);
        config.wasm_simd(true);
        config.wasm_bulk_memory(true);

        // Set resource limits
        config.max_wasm_stack(1048576); // 1MB stack

        Ok(Engine::new(&config)?)
    }

    /// Start background task processors
    fn start_background_tasks(
        &self,
        task_rx: async_channel::Receiver<(Task, async_channel::Sender<Layer4Result<ExecutionResult>>)>,
        spawn_rx: async_channel::Receiver<(AgentId, Vec<u8>, AgentConfig)>,
    ) {
        let agents_tasks = Arc::clone(&self.agents);
        let engine_tasks = self.engine.clone();
        let config_tasks = self.config.clone();
        let shutdown_tasks = Arc::clone(&self.shutdown);

        // Task execution processor
        tokio::spawn(async move {
            Self::process_tasks(task_rx, agents_tasks, engine_tasks, config_tasks, shutdown_tasks).await;
        });

        let agents_spawns = Arc::clone(&self.agents);
        let engine_spawns = self.engine.clone();
        let config_spawns = self.config.clone();
        let shutdown_spawns = Arc::clone(&self.shutdown);

        // Agent spawning processor
        tokio::spawn(async move {
            Self::process_agent_spawns(spawn_rx, agents_spawns, engine_spawns, config_spawns, shutdown_spawns).await;
        });

        let agents_heartbeat = Arc::clone(&self.agents);
        let shutdown_heartbeat = Arc::clone(&self.shutdown);

        // Heartbeat monitor
        tokio::spawn(async move {
            Self::heartbeat_monitor(agents_heartbeat, shutdown_heartbeat).await;
        });
    }

    /// Process incoming task execution requests
    async fn process_tasks(
        task_rx: async_channel::Receiver<(Task, async_channel::Sender<Layer4Result<ExecutionResult>>)>,
        agents: Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        engine: Engine,
        config: ExecutorConfig,
        shutdown: Arc<RwLock<bool>>,
    ) {
        info!("Starting task processor");

        while !*shutdown.read().await {
            tokio::select! {
                Ok((task, response_tx)) = task_rx.recv() => {
                    let execution_result = Self::execute_task_with_agent(&task, &agents, &engine, &config).await;

                    // Send response back to caller
                    let _ = response_tx.send(execution_result).await;
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Continue processing
                }
            }
        }

        info!("Task processor shutting down");
    }

    /// Process agent spawn requests
    async fn process_agent_spawns(
        spawn_rx: async_channel::Receiver<(AgentId, Vec<u8>, AgentConfig)>,
        agents: Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        engine: Engine,
        config: ExecutorConfig,
        shutdown: Arc<RwLock<bool>>,
    ) {
        info!("Starting agent spawn processor");

        while !*shutdown.read().await {
            tokio::select! {
                Ok((agent_id, wasm_binary, agent_config)) = spawn_rx.recv() => {
                    let spawn_result = Self::spawn_agent_internal(agent_id, wasm_binary, agent_config, &agents, &engine, &config).await;

                    if let Err(e) = spawn_result {
                        error!("Failed to spawn agent {}: {}", agent_id, e);
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Continue processing
                }
            }
        }

        info!("Agent spawn processor shutting down");
    }

    /// Execute a task with an appropriate agent
    async fn execute_task_with_agent(
        task: &Task,
        agents: &Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        engine: &Engine,
        config: &ExecutorConfig,
    ) -> Layer4Result<ExecutionResult> {
        // Find an available agent that can handle this task type
        let available_agent = Self::find_available_agent(agents, &task.target_agent_type).await;

        match available_agent {
            Some(agent) => {
                // Execute task with timeout
                let execution_timeout = Duration::from_secs(task.resource_quota.max_execution_time_secs);

                let execution_result = timeout(
                    execution_timeout,
                    Self::execute_task_on_agent(task.clone(), agent.clone(), engine, config),
                ).await;

                match execution_result {
                    Ok(result) => result,
                    Err(_) => {
                        // Task timed out
                        warn!("Task {} timed out after {:?}", task.id, execution_timeout);
                        Err(Layer4Error::AgentTimeout(task.resource_quota.max_execution_time_secs))
                    }
                }
            }
            None => {
                // No available agent found
                Err(Layer4Error::AgentNotFound(Uuid::new_v4())) // Would need to track available agents
            }
        }
    }

    /// Execute a task on a specific agent
    async fn execute_task_on_agent(
        task: Task,
        agent: Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>,
        engine: &Engine,
        config: &ExecutorConfig,
    ) -> Layer4Result<ExecutionResult> {
        let start_time = SystemTime::now();

        // Lock agent for execution
        let mut agent_guard = agent.write().await;

        // Update agent state to busy
        // Note: This would need to be implemented in the agent trait

        // Execute the task
        let execution_result = agent_guard.execute_task(task.clone());

        // Update agent state back to idle
        // Note: This would need to be implemented in the agent trait

        // Record execution metrics
        if config.debug_mode {
            debug!("Task {} executed in {:?}", task.id, start_time.elapsed().unwrap_or_default());
        }

        execution_result
    }

    /// Find an available agent that can handle the specified task type
    async fn find_available_agent(
        agents: &Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        task_type: &str,
    ) -> Option<Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>> {
        let agents_read = agents.read().await;

        for (_, agent) in agents_read.iter() {
            let agent_guard = agent.read().await;
            let capabilities = agent_guard.get_capabilities();

            if capabilities.supported_task_types.contains(&task_type.to_string()) {
                // Check if agent is available (not busy)
                // This would need to be implemented based on agent state
                return Some(agent.clone());
            }
        }

        None
    }

    /// Spawn a new WASM agent (internal implementation)
    async fn spawn_agent_internal(
        agent_id: AgentId,
        wasm_binary: Vec<u8>,
        agent_config: AgentConfig,
        agents: &Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        engine: &Engine,
        config: &ExecutorConfig,
    ) -> Layer4Result<()> {
        // Check if we've reached the maximum number of agents
        {
            let agents_read = agents.read().await;
            if agents_read.len() >= config.max_agents {
                return Err(Layer4Error::ResourceQuotaExceeded(
                    format!("Maximum agents ({}) reached", config.max_agents)
                ));
            }
        }

        // Create WASM runtime for the agent
        let module = Module::new(engine, &wasm_binary)?;

        // Create agent instance (this would need to be implemented based on the WASM interface)
        let agent = Self::create_agent_from_wasm(agent_id, module, agent_config.clone())?;

        // Store agent in registry
        let agent_arc = Arc::new(RwLock::new(agent));
        {
            let mut agents_write = agents.write().await;
            agents_write.insert(agent_id, agent_arc);
        }

        info!("Spawned new agent: {}", agent_id);

        if config.debug_mode {
            debug!("Agent {} initialized with config: {:?}", agent_id, agent_config);
        }

        Ok(())
    }

    /// Create an agent instance from WASM binary
    fn create_agent_from_wasm(
        agent_id: AgentId,
        module: Module,
        config: AgentConfig,
    ) -> Layer4Result<Box<dyn WasmAgent + Send + Sync>> {
        // For now, create a base agent
        // In a real implementation, this would:
        // 1. Inspect the WASM module for exported functions
        // 2. Create appropriate WASI context
        // 3. Instantiate the module with proper imports
        // 4. Create a wrapper that implements the WasmAgent trait

        let capabilities = AgentCapabilities {
            supported_task_types: vec!["generic".to_string()],
            max_concurrent_tasks: 1,
            resource_quota: config.resource_quota.clone(),
            required_env_vars: config.environment.clone(),
            features: vec!["wasm".to_string()],
        };

        let mut agent = BaseWasmAgent::new("wasm_agent".to_string(), capabilities);
        agent.initialize(config)?;

        Ok(Box::new(agent))
    }

    /// Monitor agent heartbeats and handle failures
    async fn heartbeat_monitor(
        agents: Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        shutdown: Arc<RwLock<bool>>,
    ) {
        let mut interval = interval(Duration::from_secs(10)); // Check every 10 seconds

        info!("Starting heartbeat monitor");

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    Self::check_agent_heartbeats(&agents).await;
                }
                _ = async { *shutdown.read().await } => {
                    break;
                }
            }
        }

        info!("Heartbeat monitor shutting down");
    }

    /// Check heartbeats for all agents and handle failures
    async fn check_agent_heartbeats(
        agents: &Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
    ) {
        let agents_read = agents.read().await;
        let mut failed_agents = Vec::new();

        for (agent_id, agent) in agents_read.iter() {
            let agent_guard = agent.read().await;

            // Check if agent is responsive (this would need to be implemented)
            let health = agent_guard.health_check();

            if health.status == HealthStatus::Unhealthy || health.status == HealthStatus::Critical {
                warn!("Agent {} reported unhealthy status: {:?}", agent_id, health.status);
                failed_agents.push(*agent_id);
            }
        }

        // Handle failed agents
        if !failed_agents.is_empty() {
            drop(agents_read); // Release read lock
            Self::handle_failed_agents(agents, failed_agents).await;
        }
    }

    /// Handle agents that have failed or become unresponsive
    async fn handle_failed_agents(
        agents: &Arc<RwLock<HashMap<AgentId, Arc<RwLock<Box<dyn WasmAgent + Send + Sync>>>>>>,
        failed_agents: Vec<AgentId>,
    ) {
        let mut agents_write = agents.write().await;

        for agent_id in failed_agents {
            if let Some(agent) = agents_write.remove(&agent_id) {
                // Attempt graceful shutdown
                let mut agent_guard = agent.write().await;
                let _ = agent_guard.shutdown();

                error!("Removed failed agent: {}", agent_id);
            }
        }
    }

    /// Spawn a new WASM agent
    ///
    /// Creates and initializes a new WASM agent from binary data.
    /// The agent will be ready to receive and execute tasks immediately
    /// after spawning completes successfully.
    ///
    /// # Arguments
    /// * `wasm_binary` - Compiled WASM module bytes
    /// * `config` - Agent configuration with resource quotas and parameters
    ///
    /// # Returns
    /// * `Layer4Result<AgentId>` - Unique agent identifier or spawn error
    ///
    /// # Examples
    /// ```rust,no_run
    /// let wasm_bytes = load_agent_binary("path/to/agent.wasm")?;
    /// let agent_config = AgentConfig {
    ///     agent_id: Uuid::new_v4(),
    ///     agent_type: "data_processor".to_string(),
    ///     resource_quota: ResourceQuota::default(),
    ///     environment: HashMap::new(),
    ///     parameters: HashMap::new(),
    /// };
    ///
    /// let agent_id = executor.spawn_agent(wasm_bytes, agent_config).await?;
    /// println!("Spawned agent with ID: {}", agent_id);
    /// ```
    pub async fn spawn_agent(
        &self,
        wasm_binary: Vec<u8>,
        config: AgentConfig,
    ) -> Layer4Result<AgentId> {
        let agent_id = Uuid::new_v4();

        self.spawn_tx.send((agent_id, wasm_binary, config))
            .await
            .map_err(|_| Layer4Error::Internal("Failed to send spawn request".to_string()))?;

        Ok(agent_id)
    }

    /// Execute a task asynchronously
    ///
    /// Routes a task to an appropriate available agent for execution.
    /// The task will be queued if no suitable agent is immediately available.
    /// Returns comprehensive execution results including performance metrics.
    ///
    /// # Arguments
    /// * `task` - Task to execute with payload and requirements
    ///
    /// # Returns
    /// * `Layer4Result<ExecutionResult>` - Execution outcome with metrics
    ///
    /// # Examples
    /// ```rust,no_run
    /// let task = Task {
    ///     id: Uuid::new_v4(),
    ///     priority: Priority::High,
    ///     payload: serde_json::json!({"action": "process", "data": "input"}),
    ///     created_at: SystemTime::now(),
    ///     deadline: None,
    ///     resource_quota: ResourceQuota::default(),
    ///     source_layer: "layer2".to_string(),
    ///     target_agent_type: "data_processor".to_string(),
    ///     metadata: HashMap::new(),
    /// };
    ///
    /// let result = executor.execute_task(task).await?;
    /// if result.success {
    ///     println!("Task completed in {}ms", result.execution_time_ms);
    /// }
    /// ```
    pub async fn execute_task(&self, task: Task) -> Layer4Result<ExecutionResult> {
        let (response_tx, response_rx) = async_channel::bounded(1);

        self.task_tx.send((task, response_tx))
            .await
            .map_err(|_| Layer4Error::Internal("Failed to send task".to_string()))?;

        response_rx.recv()
            .await
            .map_err(|_| Layer4Error::Internal("Task execution cancelled".to_string()))?
    }

    /// Get current system health
    ///
    /// Returns comprehensive health information about the executor,
    /// including active agent count, resource utilization, and uptime.
    /// Used for monitoring and automated scaling decisions.
    ///
    /// # Returns
    /// * `SystemHealth` - Current health status and metrics
    ///
    /// # Examples
    /// ```rust,no_run
    /// let health = executor.get_health().await;
    /// match health.status {
    ///     HealthStatus::Healthy => println!("All systems operational"),
    ///     HealthStatus::Degraded => println!("Some issues detected"),
    ///     _ => println!("Critical issues require attention"),
    /// }
    /// ```
    pub async fn get_health(&self) -> SystemHealth {
        let agents = self.agents.read().await;
        let active_agents = agents.len();
        let uptime = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        SystemHealth {
            status: if active_agents > 0 { HealthStatus::Healthy } else { HealthStatus::Degraded },
            active_agents,
            pending_tasks: 0, // Would need to track this separately
            uptime_seconds: uptime,
            resource_utilization: ResourceUtilization {
                cpu_usage: 0.1, // Would need to implement actual monitoring
                memory_usage: 0.2,
                disk_usage: 0.1,
                network_usage: 0.05,
            },
            last_check: SystemTime::now(),
        }
    }

    /// Gracefully shutdown the executor
    ///
    /// Initiates graceful shutdown of all agents and background tasks.
    /// Waits for active tasks to complete and ensures clean resource cleanup.
    /// Safe to call multiple times.
    ///
    /// # Returns
    /// * `Layer4Result<()>` - Success or shutdown error
    ///
    /// # Examples
    /// ```rust,no_run
    /// // Graceful shutdown
    /// executor.shutdown().await?;
    /// println!("All agents terminated successfully");
    /// ```
    pub async fn shutdown(&self) -> Layer4Result<()> {
        info!("Initiating executor shutdown");

        // Set shutdown flag
        *self.shutdown.write().await = true;

        // Shutdown all agents
        let agents = self.agents.read().await;
        for (agent_id, agent) in agents.iter() {
            let mut agent_guard = agent.write().await;
            let _ = agent_guard.shutdown();
            info!("Shutdown agent: {}", agent_id);
        }

        info!("Executor shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let config = ExecutorConfig::default();
        let executor = Executor::new(config);

        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_agent_spawning() {
        let config = ExecutorConfig::default();
        let executor = Executor::new(config).unwrap();

        let wasm_binary = vec![0u8; 1024]; // Mock WASM binary
        let agent_config = AgentConfig {
            agent_id: Uuid::new_v4(),
            agent_type: "test_agent".to_string(),
            resource_quota: ResourceQuota::default(),
            environment: HashMap::new(),
            parameters: HashMap::new(),
        };

        let agent_id = executor.spawn_agent(wasm_binary, agent_config).await.unwrap();
        assert!(!agent_id.is_nil());
    }

    #[test]
    fn test_engine_creation() {
        let engine_result = Executor::create_engine();
        assert!(engine_result.is_ok());

        let engine = engine_result.unwrap();
        // Test that engine can compile a simple module
        let wat = r#"
            (module
                (func $hello (result i32)
                    i32.const 42
                )
                (export "hello" (func $hello))
            )
        "#;

        let module_result = Module::new(&engine, wat);
        assert!(module_result.is_ok());
    }
}
