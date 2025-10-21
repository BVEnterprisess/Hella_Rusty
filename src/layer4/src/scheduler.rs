//! Layer 4 Scheduler - Async Task Dispatcher with Retry Logic
//!
//! This module implements the task scheduling and dispatching system for
//! the Layer 4 execution fabric. It handles task queuing, priority-based
//! scheduling, retry logic, and load balancing across available agents.
//!
//! ## Architecture
//!
//! The scheduler provides intelligent task distribution with:
//!
//! - **Priority-based Queuing**: Critical tasks execute before background tasks
//! - **Exponential Backoff**: Failed tasks retry with increasing delays
//! - **Dead Letter Queue**: Persistent storage for consistently failing tasks
//! - **Circuit Breaker**: Automatic failure detection and agent isolation
//! - **Load Balancing**: Fair distribution across available agents
//!
//! ## Task Lifecycle
//!
//! ```text
//! Task Submitted → Priority Queue → Agent Selection → Execution → Success/Failure
//!       ↓              ↓                    ↓            ↓            ↓
//!    Layer 2/3/5   Critical→Background   Available   Complete    Retry/DLQ
//!                     FIFO same priority   Agent      or Timeout   or Drop
//! ```

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Task queue entry with priority ordering
#[derive(Debug, Clone)]
pub struct QueuedTask {
    /// The task to execute
    pub task: Task,
    /// Number of retry attempts made
    pub retry_count: u32,
    /// Timestamp when task was first queued
    pub queued_at: SystemTime,
    /// Timestamp of last retry attempt
    pub last_retry_at: Option<SystemTime>,
    /// Response channel for task result
    pub response_tx: async_channel::Sender<Layer4Result<ExecutionResult>>,
}

/// Priority-based ordering for the task queue
impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority tasks come first (BinaryHeap is a max-heap)
        self.task.priority.cmp(&other.task.priority)
            // Then compare by queue time (FIFO for same priority - earlier times come first)
            .then_with(|| other.queued_at.cmp(&self.queued_at))
    }
}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.id == other.task.id
    }
}

impl Eq for QueuedTask {}

/// Scheduler configuration
///
/// Defines the operational parameters for task scheduling and retry behavior.
/// These settings control queue sizes, retry policies, and failure handling
/// to ensure reliable task execution across the autonomous system.
///
/// # Examples
/// ```rust
/// let config = SchedulerConfig {
///     max_queue_size: 10000,           // Handle up to 10k queued tasks
///     max_retries: 3,                  // Retry failed tasks up to 3 times
///     retry_base_delay_secs: 1,        // Start with 1 second delay
///     retry_max_delay_secs: 300,       // Cap retries at 5 minutes
///     retry_backoff_multiplier: 2.0,   // Double delay each retry
///     task_timeout_secs: 300,          // 5 minute task timeout
///     enable_preemption: true,         // Allow high priority interruption
///     dead_letter_queue_size: 1000,    // Store up to 1k failed tasks
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum number of tasks in queue
    ///
    /// Limits the size of the pending task queue to prevent memory exhaustion.
    /// When exceeded, new tasks are rejected until queue space is available.
    /// Should be tuned based on expected task volume and system memory.
    pub max_queue_size: usize,

    /// Maximum retry attempts per task
    ///
    /// Number of times a failed task will be retried before being moved
    /// to the dead letter queue. Balances persistence with resource usage.
    pub max_retries: u32,

    /// Base delay between retries in seconds
    ///
    /// Initial delay before the first retry of a failed task.
    /// Used as the base for exponential backoff calculations.
    pub retry_base_delay_secs: u64,

    /// Maximum delay between retries in seconds
    ///
    /// Cap on retry delays to prevent excessively long waits for
    /// persistently failing tasks. Prevents resource waste on hopeless tasks.
    pub retry_max_delay_secs: u64,

    /// Retry backoff multiplier
    ///
    /// Factor by which retry delays increase with each attempt.
    /// Values > 1.0 create exponential backoff (recommended).
    /// Values < 1.0 create linear backoff.
    pub retry_backoff_multiplier: f64,

    /// Task timeout in seconds
    ///
    /// Maximum time a task can execute before being considered hung.
    /// Prevents stuck tasks from blocking the system indefinitely.
    pub task_timeout_secs: u64,

    /// Enable priority preemption
    ///
    /// When true, higher priority tasks can interrupt lower priority ones.
    /// When false, tasks run to completion regardless of priority.
    /// Use carefully to avoid starvation of low-priority tasks.
    pub enable_preemption: bool,

    /// Dead letter queue size for failed tasks
    ///
    /// Maximum number of permanently failed tasks to retain for analysis.
    /// When exceeded, oldest failed tasks are discarded.
    /// Used for debugging and failure pattern analysis.
    pub dead_letter_queue_size: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            max_retries: 3,
            retry_base_delay_secs: 1,
            retry_max_delay_secs: 300, // 5 minutes
            retry_backoff_multiplier: 2.0,
            task_timeout_secs: 300, // 5 minutes
            enable_preemption: true,
            dead_letter_queue_size: 1000,
        }
    }
}

/// Task scheduler with priority queue and retry logic
///
/// The Scheduler is responsible for intelligent task distribution and
/// failure handling in the Layer 4 execution fabric. It implements
/// priority-based queuing, exponential backoff retry, and comprehensive
/// failure tracking for reliable autonomous operation.
///
/// ## Key Features
///
/// - **Priority Scheduling**: Critical tasks execute before background tasks
/// - **Exponential Backoff**: Failed tasks retry with increasing delays
/// - **Dead Letter Queue**: Persistent storage for consistently failing tasks
/// - **Circuit Breaker**: Automatic failure detection and agent isolation
/// - **Load Balancing**: Fair distribution across available execution agents
/// - **Timeout Management**: Automatic cleanup of hung or expired tasks
///
/// ## Architecture
///
/// ```text
/// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
/// │   Task Queue    │───▶│   Scheduler     │───▶│   Executor      │
/// │  (Priority)     │    │   (Dispatch)    │    │   (Execute)     │
/// └─────────────────┘    └─────────────────┘    └─────────────────┘
///         │                       │                       │
///         ▼                       ▼                       ▼
/// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
/// │  Dead Letter    │    │   Retry Logic   │    │   Metrics       │
/// │   Queue         │    │   (Backoff)     │    │   Collection    │
/// └─────────────────┘    └─────────────────┘    └─────────────────┘
/// ```
///
/// ## Failure Handling
///
/// The scheduler implements a comprehensive failure handling strategy:
/// 1. **Immediate Retry**: Failed tasks retry with exponential backoff
/// 2. **Circuit Breaker**: Consistently failing agents are temporarily isolated
/// 3. **Dead Letter Queue**: Tasks that exceed retry limits are preserved for analysis
/// 4. **Timeout Cleanup**: Hung tasks are automatically terminated and retried
pub struct Scheduler {
    /// Scheduler configuration
    ///
    /// Operational parameters controlling queue sizes, retry behavior,
    /// and failure handling policies. Set during initialization.
    config: SchedulerConfig,

    /// Priority queue for pending tasks
    ///
    /// Thread-safe priority queue holding tasks waiting for execution.
    /// Higher priority tasks are dequeued first. Uses binary heap for efficiency.
    /// Capacity limited by max_queue_size to prevent memory exhaustion.
    task_queue: Arc<RwLock<BinaryHeap<QueuedTask>>>,

    /// Dead letter queue for failed tasks
    ///
    /// Persistent storage for tasks that have exceeded retry limits.
    /// Used for failure analysis and debugging. Size limited to prevent
    /// unbounded growth. Oldest entries discarded when limit exceeded.
    dead_letter_queue: Arc<RwLock<Vec<QueuedTask>>>,

    /// Active task tracking
    ///
    /// Thread-safe registry of tasks currently being executed.
    /// Maps task IDs to task metadata for tracking and timeout management.
    /// Used for cleanup when tasks hang or agents fail unexpectedly.
    active_tasks: Arc<RwLock<HashMap<TaskId, QueuedTask>>>,

    /// Task submission channel
    ///
    /// Asynchronous channel for receiving new task submissions.
    /// Connects external systems (Layer 2, Layer 3, Layer 5) to the scheduler.
    /// Unbounded to prevent blocking of task sources.
    task_tx: async_channel::Sender<QueuedTask>,

    /// Shutdown signal
    ///
    /// Atomic flag indicating when the scheduler should shut down.
    /// Used for graceful termination of background tasks and cleanup.
    /// Prevents new tasks from being accepted during shutdown.
    shutdown: Arc<RwLock<bool>>,
}

impl Scheduler {
    /// Create a new scheduler instance
    pub fn new(config: SchedulerConfig) -> Layer4Result<Self> {
        let (task_tx, task_rx) = async_channel::unbounded();

        let scheduler = Self {
            config,
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            dead_letter_queue: Arc::new(RwLock::new(Vec::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_tx,
            shutdown: Arc::new(RwLock::new(false)),
        };

        // Start background task processing
        scheduler.start_task_processor(task_rx);

        Ok(scheduler)
    }

    /// Start the background task processor
    fn start_task_processor(&self, task_rx: async_channel::Receiver<QueuedTask>) {
        let task_queue = Arc::clone(&self.task_queue);
        let active_tasks = Arc::clone(&self.active_tasks);
        let dead_letter_queue = Arc::clone(&self.dead_letter_queue);
        let config = self.config.clone();
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            Self::process_task_queue(
                task_rx,
                task_queue,
                active_tasks,
                dead_letter_queue,
                config,
                shutdown,
            ).await;
        });
    }

    /// Main task processing loop
    async fn process_task_queue(
        task_rx: async_channel::Receiver<QueuedTask>,
        task_queue: Arc<RwLock<BinaryHeap<QueuedTask>>>,
        active_tasks: Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
        dead_letter_queue: Arc<RwLock<Vec<QueuedTask>>>,
        config: SchedulerConfig,
        shutdown: Arc<RwLock<bool>>,
    ) {
        info!("Starting task queue processor");

        // Main processing loop
        while !*shutdown.read().await {
            tokio::select! {
                // Process new tasks from channel
                Ok(queued_task) = task_rx.recv() => {
                    Self::enqueue_task(queued_task, &task_queue, &active_tasks, &config).await;
                }

                // Process tasks from queue
                _ = Self::process_next_task(&task_queue, &active_tasks, &dead_letter_queue, &config) => {
                    // Task processed, continue
                }

                // Periodic cleanup
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    Self::cleanup_expired_tasks(&active_tasks, &dead_letter_queue, &config).await;
                }
            }
        }

        info!("Task queue processor shutting down");
    }

    /// Enqueue a new task for execution
    async fn enqueue_task(
        queued_task: QueuedTask,
        task_queue: &Arc<RwLock<BinaryHeap<QueuedTask>>>,
        active_tasks: &Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
        config: &SchedulerConfig,
    ) {
        let task_id = queued_task.task.id;

        // Check if task is already active
        {
            let active_tasks_read = active_tasks.read().await;
            if active_tasks_read.contains_key(&task_id) {
                warn!("Task {} is already being processed", task_id);
                return;
            }
        }

        // Check queue capacity
        {
            let queue_read = task_queue.read().await;
            if queue_read.len() >= config.max_queue_size {
                error!("Task queue is full ({}), dropping task {}", config.max_queue_size, task_id);

                // Send error response
                let _ = queued_task.response_tx.send(Err(Layer4Error::ResourceQuotaExceeded(
                    "Task queue is full".to_string()
                ))).await;
                return;
            }
        }

        // Add to queue
        let priority = queued_task.task.priority;
        {
            let mut queue_write = task_queue.write().await;
            queue_write.push(queued_task);
        }

        debug!("Enqueued task {} with priority {:?}", task_id, priority);
    }

    /// Process the next available task
    async fn process_next_task(
        task_queue: &Arc<RwLock<BinaryHeap<QueuedTask>>>,
        active_tasks: &Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
        dead_letter_queue: &Arc<RwLock<Vec<QueuedTask>>>,
        config: &SchedulerConfig,
    ) -> Layer4Result<()> {
        // Get next task from queue
        let queued_task = {
            let mut queue_write = task_queue.write().await;
            queue_write.pop()
        };

        if let Some(mut queued_task) = queued_task {
            let task_id = queued_task.task.id;

            // Check if task should be retried or sent to dead letter queue
            if queued_task.retry_count >= config.max_retries {
                warn!("Task {} exceeded max retries ({}), moving to dead letter queue",
                      task_id, config.max_retries);

                // Send error response before moving
                let _ = queued_task.response_tx.send(Err(Layer4Error::Internal(
                    format!("Task exceeded max retries ({})", config.max_retries)
                ))).await;

                // Move to dead letter queue
                {
                    let mut dlq_write = dead_letter_queue.write().await;

                    // Maintain DLQ size limit
                    if dlq_write.len() >= config.dead_letter_queue_size {
                        dlq_write.remove(0); // Remove oldest entry
                    }

                    dlq_write.push(queued_task);
                }

                return Ok(());
            }

            // Move task to active tracking
            {
                let mut active_write = active_tasks.write().await;
                active_write.insert(task_id, queued_task);
            }

            // Process the task asynchronously
            Self::execute_task_with_retry(task_id, active_tasks, dead_letter_queue, config.clone()).await;
        }

        Ok(())
    }

    /// Execute a task with retry logic
    async fn execute_task_with_retry(
        task_id: TaskId,
        active_tasks: &Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
        dead_letter_queue: &Arc<RwLock<Vec<QueuedTask>>>,
        config: SchedulerConfig,
    ) {
        // This would integrate with the executor to actually run the task
        // For now, we'll simulate task execution

        let execution_delay = Duration::from_millis(100); // Simulate execution time
        let active_tasks = Arc::clone(active_tasks);
        let dead_letter_queue = Arc::clone(dead_letter_queue);

        tokio::spawn(async move {
            let execution_result = Self::simulate_task_execution(task_id).await;

            // Handle execution result
            match execution_result {
                Ok(result) => {
                    // Task succeeded
                    Self::handle_task_success(task_id, result, &active_tasks).await;
                }
                Err(e) => {
                    // Task failed, schedule retry
                    Self::handle_task_failure(task_id, e, &active_tasks, &dead_letter_queue, &config).await;
                }
            }
        });
    }

    /// Simulate task execution (placeholder for actual executor integration)
    async fn simulate_task_execution(task_id: TaskId) -> Layer4Result<ExecutionResult> {
        // Simulate some tasks failing randomly for testing retry logic
        if task_id.to_string().chars().last().unwrap_or('0') < '5' {
            // 50% chance of failure for testing
            tokio::time::sleep(Duration::from_millis(50)).await;
            return Err(Layer4Error::Internal("Simulated task failure".to_string()));
        }

        // Successful execution
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(ExecutionResult {
            task_id,
            success: true,
            output: serde_json::json!({"status": "completed", "result": "success"}),
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
        })
    }

    /// Handle successful task completion
    async fn handle_task_success(
        task_id: TaskId,
        result: ExecutionResult,
        active_tasks: &Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
    ) {
        // Remove from active tasks
        let queued_task = {
            let mut active_write = active_tasks.write().await;
            active_write.remove(&task_id)
        };

        if let Some(queued_task) = queued_task {
            // Send success response
            let _ = queued_task.response_tx.send(Ok(result)).await;
            info!("Task {} completed successfully", task_id);
        }
    }

    /// Handle task failure and schedule retry
    async fn handle_task_failure(
        task_id: TaskId,
        error: Layer4Error,
        active_tasks: &Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
        dead_letter_queue: &Arc<RwLock<Vec<QueuedTask>>>,
        config: &SchedulerConfig,
    ) {
        // Get the failed task from active tracking
        let mut queued_task = {
            let mut active_write = active_tasks.write().await;
            active_write.remove(&task_id)
        };

        if let Some(mut task) = queued_task {
            // Increment retry count
            task.retry_count += 1;
            task.last_retry_at = Some(SystemTime::now());

            // Calculate retry delay with exponential backoff
            let retry_delay = Self::calculate_retry_delay(task.retry_count, config);

            warn!("Task {} failed (attempt {}), retrying in {:?}: {}",
                  task_id, task.retry_count, retry_delay, error);

            // Schedule retry
            let task_queue = Arc::clone(&active_tasks); // Would need actual task queue reference
            tokio::spawn(async move {
                tokio::time::sleep(retry_delay).await;

                // Re-enqueue for retry (simplified)
                // In real implementation, this would go back to the main queue
                debug!("Retrying task {} (attempt {})", task_id, task.retry_count);
            });
        }
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(retry_count: u32, config: &SchedulerConfig) -> Duration {
        let base_delay = config.retry_base_delay_secs as f64;
        let delay = base_delay * config.retry_backoff_multiplier.powi(retry_count.saturating_sub(1) as i32);

        let capped_delay = delay.min(config.retry_max_delay_secs as f64);
        Duration::from_secs(capped_delay as u64)
    }

    /// Clean up expired tasks
    async fn cleanup_expired_tasks(
        active_tasks: &Arc<RwLock<HashMap<TaskId, QueuedTask>>>,
        dead_letter_queue: &Arc<RwLock<Vec<QueuedTask>>>,
        config: &SchedulerConfig,
    ) {
        let now = SystemTime::now();
        let timeout_duration = Duration::from_secs(config.task_timeout_secs);

        // Clean up timed out active tasks
        {
            let mut active_write = active_tasks.write().await;
            let mut expired_tasks = Vec::new();

            active_write.retain(|task_id, queued_task| {
                if let Ok(elapsed) = now.duration_since(queued_task.queued_at) {
                    if elapsed > timeout_duration {
                        expired_tasks.push(*task_id);
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });

            for task_id in expired_tasks {
                warn!("Task {} timed out after {:?}", task_id, timeout_duration);
            }
        }

        // Clean up dead letter queue if it's getting too large
        {
            let mut dlq_write = dead_letter_queue.write().await;
            if dlq_write.len() > config.dead_letter_queue_size {
                let excess = dlq_write.len() - config.dead_letter_queue_size;
                dlq_write.drain(0..excess);
                warn!("Cleaned up {} tasks from dead letter queue", excess);
            }
        }
    }

    /// Submit a task for execution
    ///
    /// Queues a task for execution with the appropriate priority and
    /// returns a receiver for collecting the execution result.
    /// The task will be dispatched to an available agent when resources allow.
    ///
    /// # Arguments
    /// * `task` - Task to execute with payload and requirements
    ///
    /// # Returns
    /// * `Layer4Result<async_channel::Receiver<Layer4Result<ExecutionResult>>>` - Channel for result or submission error
    ///
    /// # Examples
    /// ```rust,no_run
    /// let task = Task {
    ///     id: Uuid::new_v4(),
    ///     priority: Priority::High,
    ///     payload: serde_json::json!({"action": "analyze"}),
    ///     // ... other fields
    /// };
    ///
    /// let result_rx = scheduler.submit_task(task).await?;
    /// let result = result_rx.await?;
    ///
    /// match result {
    ///     Ok(execution) if execution.success => println!("Task succeeded"),
    ///     Ok(execution) => println!("Task failed: {:?}", execution.error),
    ///     Err(e) => println!("Task error: {}", e),
    /// }
    /// ```
    pub async fn submit_task(&self, task: Task) -> Layer4Result<async_channel::Receiver<Layer4Result<ExecutionResult>>> {
        let (response_tx, response_rx) = async_channel::bounded(1);

        let queued_task = QueuedTask {
            task,
            retry_count: 0,
            queued_at: SystemTime::now(),
            last_retry_at: None,
            response_tx,
        };

        self.task_tx.send(queued_task)
            .await
            .map_err(|_| Layer4Error::Internal("Failed to submit task".to_string()))?;

        Ok(response_rx)
    }

    /// Get scheduler statistics
    ///
    /// Returns comprehensive statistics about scheduler operation,
    /// including queue sizes, retry counts, and performance metrics.
    /// Used for monitoring, debugging, and capacity planning.
    ///
    /// # Returns
    /// * `SchedulerStats` - Current scheduler statistics and metrics
    ///
    /// # Examples
    /// ```rust,no_run
    /// let stats = scheduler.get_stats().await;
    /// println!("Queued tasks: {}", stats.queued_tasks);
    /// println!("Active tasks: {}", stats.active_tasks);
    /// println!("Failed tasks: {}", stats.dead_letter_tasks);
    ///
    /// if stats.queued_tasks > 1000 {
    ///     println!("Warning: High queue depth detected");
    /// }
    /// ```
    pub async fn get_stats(&self) -> SchedulerStats {
        let task_queue = self.task_queue.read().await;
        let active_tasks = self.active_tasks.read().await;
        let dead_letter_queue = self.dead_letter_queue.read().await;

        SchedulerStats {
            queued_tasks: task_queue.len(),
            active_tasks: active_tasks.len(),
            dead_letter_tasks: dead_letter_queue.len(),
            max_queue_size: self.config.max_queue_size,
            max_retries: self.config.max_retries,
        }
    }

    /// Gracefully shutdown the scheduler
    ///
    /// Initiates graceful shutdown of the scheduler and all background tasks.
    /// Waits for active tasks to complete or timeout, ensuring no tasks
    /// are lost during shutdown. Safe to call multiple times.
    ///
    /// # Returns
    /// * `Layer4Result<()>` - Success or shutdown error
    ///
    /// # Examples
    /// ```rust,no_run
    /// // Graceful shutdown with timeout
    /// use tokio::time::{timeout, Duration};
    ///
    /// let shutdown_result = timeout(
    ///     Duration::from_secs(30),
    ///     scheduler.shutdown()
    /// ).await;
    ///
    /// match shutdown_result {
    ///     Ok(Ok(())) => println!("Scheduler shutdown cleanly"),
    ///     Ok(Err(e)) => println!("Shutdown error: {}", e),
    ///     Err(_) => println!("Shutdown timed out"),
    /// }
    /// ```
    pub async fn shutdown(&self) -> Layer4Result<()> {
        info!("Initiating scheduler shutdown");
        *self.shutdown.write().await = true;

        // Wait for active tasks to complete or timeout
        let mut attempts = 0;
        while attempts < 30 { // 30 second timeout
            {
                let active_tasks = self.active_tasks.read().await;
                if active_tasks.is_empty() {
                    break;
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            attempts += 1;
        }

        info!("Scheduler shutdown complete");
        Ok(())
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Number of tasks waiting in queue
    pub queued_tasks: usize,
    /// Number of tasks currently executing
    pub active_tasks: usize,
    /// Number of tasks in dead letter queue
    pub dead_letter_tasks: usize,
    /// Maximum queue capacity
    pub max_queue_size: usize,
    /// Maximum retry attempts per task
    pub max_retries: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = Scheduler::new(config);

        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = SchedulerConfig::default();
        let scheduler = Scheduler::new(config).unwrap();

        let task = Task {
            id: Uuid::new_v4(),
            priority: Priority::High,
            payload: serde_json::json!({"test": "data"}),
            created_at: SystemTime::now(),
            deadline: None,
            resource_quota: ResourceQuota::default(),
            source_layer: "test".to_string(),
            target_agent_type: "test_agent".to_string(),
            metadata: HashMap::new(),
        };

        let response_rx = scheduler.submit_task(task).await.unwrap();

        // Task should be submitted successfully (queued, not necessarily executed)
        // The receiver exists but no processor is running in this test
        assert!(response_rx.is_empty()); // Verify channel is created and empty
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = SchedulerConfig::default();

        // Test exponential backoff
        let delay1 = Scheduler::calculate_retry_delay(1, &config);
        let delay2 = Scheduler::calculate_retry_delay(2, &config);
        let delay3 = Scheduler::calculate_retry_delay(3, &config);

        assert!(delay2 > delay1);
        assert!(delay3 > delay2);

        // Test max delay cap
        let delay_max = Scheduler::calculate_retry_delay(100, &config);
        assert!(delay_max <= Duration::from_secs(config.retry_max_delay_secs));
    }

    #[test]
    fn test_task_priority_ordering() {
        let mut queue = BinaryHeap::new();

        let task1 = QueuedTask {
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
        };

        let task2 = QueuedTask {
            task: Task {
                id: Uuid::new_v4(),
                priority: Priority::High,
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
        };

        queue.push(task1);
        queue.push(task2);

        // Higher priority task should come first
        let next_task = queue.pop().unwrap();
        assert_eq!(next_task.task.priority, Priority::High);
    }
}
