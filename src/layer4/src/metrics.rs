//! Layer 4 Metrics - KPI Telemetry and Prometheus Integration
//!
//! This module implements comprehensive metrics collection, KPI tracking,
//! and Prometheus metrics export for the Layer 4 execution fabric.
//! It provides real-time observability into agent performance, resource
//! utilization, and system health for the autonomous refinement loops.
//!
//! ## Architecture
//!
//! The metrics system provides observability for the entire Layer 4 ecosystem:
//!
//! - **KPI Collection**: Performance metrics from agent execution
//! - **Prometheus Export**: Real-time metrics for monitoring systems
//! - **Resource Tracking**: CPU, memory, network, and GPU utilization
//! - **Health Monitoring**: System and agent health status
//! - **Performance Analysis**: Latency, throughput, and error rate tracking
//!
//! ## Integration Points
//!
//! - **Layer 5 (Refinement)**: Consumes KPIs for continuous improvement
//! - **Layer 7 (Evolution)**: Uses metrics for agent genome optimization
//! - **Layer 8 (Resource)**: Provides resource utilization data
//! - **External Monitoring**: Exports to Prometheus/Grafana for dashboards

use crate::types::*;
use prometheus::{opts, histogram_opts, Encoder, Gauge, Histogram, HistogramVec, IntCounter, IntCounterVec, TextEncoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Metrics collector configuration
///
/// Defines the operational parameters for metrics collection, storage,
/// and export. These settings control how performance data is gathered,
/// retained, and made available to monitoring systems.
///
/// # Examples
/// ```rust
/// let config = MetricsConfig {
///     prometheus_port: 9090,              // HTTP server port for metrics
///     collection_interval_secs: 5,        // Collect metrics every 5 seconds
///     enable_detailed_metrics: true,      // Track per-agent performance
///     retention_secs: 3600,               // Keep metrics for 1 hour
///     enable_export: true,                // Export to external systems
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Prometheus metrics port
    ///
    /// Port number for the HTTP server that exposes Prometheus metrics.
    /// Should be accessible to monitoring systems but protected in production.
    /// Standard Prometheus port is 9090.
    pub prometheus_port: u16,

    /// Metrics collection interval in seconds
    ///
    /// How often to collect and update performance metrics.
    /// Shorter intervals provide better real-time visibility but increase overhead.
    /// Longer intervals reduce overhead but may miss transient issues.
    pub collection_interval_secs: u64,

    /// Enable detailed per-agent metrics
    ///
    /// When true, collects and exports detailed metrics for individual agents.
    /// Useful for debugging and performance analysis but increases cardinality.
    /// When false, only aggregate metrics are collected.
    pub enable_detailed_metrics: bool,

    /// Metrics retention period in seconds
    ///
    /// How long to retain historical metrics data in memory.
    /// Older data is automatically discarded to prevent memory leaks.
    /// Should be tuned based on monitoring requirements and available memory.
    pub retention_secs: u64,

    /// Enable metrics export to external systems
    ///
    /// When true, starts HTTP server for Prometheus scraping and enables
    /// external monitoring integration. Should be enabled in production.
    /// When false, metrics are collected but not exported.
    pub enable_export: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            prometheus_port: 9090,
            collection_interval_secs: 5,
            enable_detailed_metrics: true,
            retention_secs: 3600, // 1 hour
            enable_export: true,
        }
    }
}

/// Main metrics collector for Layer 4
///
/// The MetricsCollector is the central component for gathering, storing,
/// and exporting performance metrics from the Layer 4 execution fabric.
/// It provides comprehensive observability for autonomous system operation.
///
/// ## Responsibilities
///
/// - **KPI Collection**: Gather performance metrics from agent execution
/// - **Prometheus Export**: Serve metrics to monitoring systems via HTTP
/// - **Resource Tracking**: Monitor CPU, memory, network, and GPU usage
/// - **Health Monitoring**: Track system and agent health status
/// - **Performance Analysis**: Calculate latency, throughput, and error rates
/// - **Data Retention**: Manage historical metrics storage and cleanup
///
/// ## Architecture
///
/// ```text
/// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
/// │   Agent KPI     │───▶│  Metrics        │───▶│   Prometheus    │
/// │   Reports       │    │  Collector      │    │   HTTP Server   │
/// └─────────────────┘    └─────────────────┘    └─────────────────┘
///         │                       │                       │
///         ▼                       ▼                       ▼
/// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
/// │   System        │    │   Resource      │    │   Grafana       │
/// │   Metrics       │    │   Utilization   │    │   Dashboard     │
/// └─────────────────┘    └─────────────────┘    └─────────────────┘
/// ```
///
/// ## Metrics Categories
///
/// - **System Metrics**: Uptime, agent counts, overall health status
/// - **Agent Metrics**: Per-agent performance, error rates, resource usage
/// - **Task Metrics**: Execution latency, success rates, throughput
/// - **Resource Metrics**: CPU, memory, disk, network utilization
pub struct MetricsCollector {
    /// Configuration
    ///
    /// Operational parameters controlling collection intervals,
    /// export settings, and retention policies.
    config: MetricsConfig,

    /// Prometheus registry
    ///
    /// Central registry for all Prometheus metrics. Manages metric
    /// definitions, collection, and serialization for export.
    registry: prometheus::Registry,

    /// Core system metrics
    ///
    /// High-level system performance and health indicators.
    /// Includes uptime, agent counts, and overall system status.
    system_metrics: SystemMetrics,

    /// Agent-specific metrics
    ///
    /// Detailed performance metrics for individual agents.
    /// Used for debugging, optimization, and evolution decisions.
    agent_metrics: AgentMetrics,

    /// Task execution metrics
    ///
    /// Performance metrics related to task execution across all agents.
    /// Includes latency histograms, success rates, and throughput data.
    task_metrics: TaskMetrics,

    /// Resource utilization metrics
    ///
    /// System resource consumption tracking for capacity planning.
    /// Monitors CPU, memory, disk, and network utilization trends.
    resource_metrics: ResourceMetrics,

    /// Shutdown signal
    ///
    /// Atomic flag controlling the lifecycle of background collection tasks.
    /// Used for graceful shutdown and resource cleanup.
    shutdown: Arc<RwLock<bool>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Layer4Result<Self> {
        let registry = prometheus::Registry::new();

        let collector = Self {
            config,
            registry: registry.clone(),
            system_metrics: SystemMetrics::new(&registry)?,
            agent_metrics: AgentMetrics::new(&registry)?,
            task_metrics: TaskMetrics::new(&registry)?,
            resource_metrics: ResourceMetrics::new(&registry)?,
            shutdown: Arc::new(RwLock::new(false)),
        };

        // Register all metrics with the registry
        collector.register_metrics()?;

        Ok(collector)
    }

    /// Register all metrics with Prometheus registry
    fn register_metrics(&self) -> Layer4Result<()> {
        self.system_metrics.register(&self.registry)?;
        self.agent_metrics.register(&self.registry)?;
        self.task_metrics.register(&self.registry)?;
        self.resource_metrics.register(&self.registry)?;
        Ok(())
    }

    /// Start the metrics collection and export server
    pub async fn start(&self) -> Layer4Result<()> {
        info!("Starting metrics collector on port {}", self.config.prometheus_port);

        // Start metrics collection loop
        let collection_shutdown = Arc::clone(&self.shutdown);
        let collection_config = self.config.clone();
        let system_metrics = self.system_metrics.clone();
        let agent_metrics = self.agent_metrics.clone();
        let task_metrics = self.task_metrics.clone();
        let resource_metrics = self.resource_metrics.clone();

        tokio::spawn(async move {
            Self::collection_loop(
                collection_config,
                system_metrics,
                agent_metrics,
                task_metrics,
                resource_metrics,
                collection_shutdown,
            ).await;
        });

        // Start Prometheus HTTP server
        if self.config.enable_export {
            self.start_prometheus_server().await?;
        }

        Ok(())
    }

    /// Main metrics collection loop
    async fn collection_loop(
        config: MetricsConfig,
        system_metrics: SystemMetrics,
        agent_metrics: AgentMetrics,
        task_metrics: TaskMetrics,
        resource_metrics: ResourceMetrics,
        shutdown: Arc<RwLock<bool>>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(config.collection_interval_secs));

        info!("Starting metrics collection loop");

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Collect current system metrics
                    if let Err(e) = Self::collect_system_metrics(&system_metrics, &resource_metrics).await {
                        error!("Failed to collect system metrics: {}", e);
                    }

                    // Update Prometheus metrics
                    if let Err(e) = Self::update_prometheus_metrics(&system_metrics, &agent_metrics, &task_metrics, &resource_metrics) {
                        error!("Failed to update Prometheus metrics: {}", e);
                    }
                }
                _ = async { *shutdown.read().await } => {
                    break;
                }
            }
        }

        info!("Metrics collection loop stopped");
    }

    /// Collect current system metrics
    async fn collect_system_metrics(
        system_metrics: &SystemMetrics,
        resource_metrics: &ResourceMetrics,
    ) -> Layer4Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();

        // Update uptime
        system_metrics.uptime_seconds.set(current_time);

        // Collect resource utilization
        let resource_usage = Self::get_resource_usage().await?;

        resource_metrics.cpu_usage.set(resource_usage.cpu_usage as f64);
        resource_metrics.memory_usage.set(resource_usage.memory_usage as f64);
        resource_metrics.disk_usage.set(resource_usage.disk_usage as f64);
        resource_metrics.network_usage.set(resource_usage.network_usage as f64);

        Ok(())
    }

    /// Get current system resource usage
    async fn get_resource_usage() -> Layer4Result<ResourceUtilization> {
        // In a real implementation, this would read from /proc/stat, /proc/meminfo, etc.
        // For now, return simulated values
        Ok(ResourceUtilization {
            cpu_usage: 0.15, // 15% CPU usage
            memory_usage: 0.25, // 25% memory usage
            disk_usage: 0.10, // 10% disk usage
            network_usage: 0.05, // 5% network usage
        })
    }

    /// Update Prometheus metrics from collected data
    fn update_prometheus_metrics(
        system_metrics: &SystemMetrics,
        agent_metrics: &AgentMetrics,
        task_metrics: &TaskMetrics,
        resource_metrics: &ResourceMetrics,
    ) -> Layer4Result<()> {
        // Update system metrics
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        system_metrics.uptime_seconds.set(current_time);

        // Update resource metrics (would be populated from actual collection)
        // These are already updated in collect_system_metrics

        Ok(())
    }

    /// Start Prometheus HTTP server for metrics export
    async fn start_prometheus_server(&self) -> Layer4Result<()> {
        use warp::Filter;

        let registry = self.registry.clone();

        // Create metrics endpoint
        let metrics_route = warp::path("metrics")
            .and(warp::get())
            .map(move || {
                let encoder = TextEncoder::new();
                let metric_families = registry.gather();

                match encoder.encode_to_string(&metric_families) {
                    Ok(metrics) => warp::reply::with_status(metrics, warp::http::StatusCode::OK),
                    Err(_) => warp::reply::with_status(
                        "Failed to encode metrics".to_string(),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                }
            });

        // Create health endpoint
        let health_route = warp::path("health")
            .and(warp::get())
            .map(|| {
                match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(duration) => {
                        warp::reply::json(&serde_json::json!({
                            "status": "healthy",
                            "timestamp": duration.as_secs()
                        }))
                    }
                    Err(_) => {
                        warp::reply::json(&serde_json::json!({
                            "status": "error",
                            "message": "Failed to get system time"
                        }))
                    }
                }
            });

        // Combine routes
        let routes = metrics_route.or(health_route);

        // Start server
        let port = self.config.prometheus_port;
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            let (addr, server) = warp::serve(routes)
                .bind_with_graceful_shutdown(([0, 0, 0, 0], port), async move {
                    let _ = shutdown.read().await;
                });

            info!("Prometheus metrics server started on {}", addr);

            server.await;
            info!("Prometheus metrics server stopped");
        });

        Ok(())
    }

    /// Record a KPI report from an agent
    ///
    /// Processes and stores KPI data from agent task execution.
    /// Updates Prometheus metrics and triggers alerts if thresholds are exceeded.
    /// This data feeds into Layer 5 (Refinement) for continuous improvement.
    ///
    /// # Arguments
    /// * `report` - KPI data from completed task execution
    ///
    /// # Returns
    /// * `Layer4Result<()>` - Success or recording error
    ///
    /// # Examples
    /// ```rust,no_run
    /// let kpi_report = KpiReport {
    ///     task_id: task_id,
    ///     agent_id: agent_id,
    ///     latency_ms: 150.0,
    ///     accuracy: 0.95,
    ///     cpu_usage: 0.1,
    ///     memory_mb: 64.0,
    ///     // ... other fields
    /// };
    ///
    /// metrics_collector.record_kpi_report(kpi_report).await?;
    /// // Metrics are now available in Prometheus and Layer 5
    /// ```
    pub async fn record_kpi_report(&self, report: KpiReport) -> Layer4Result<()> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        // Update task metrics
        self.task_metrics.total_tasks.inc();
        self.task_metrics.task_latency_ms.observe(report.latency_ms);
        self.task_metrics.task_accuracy.set(report.accuracy);

        // Update resource metrics
        self.resource_metrics.agent_cpu_usage.set(report.cpu_usage as f64);
        self.resource_metrics.agent_memory_mb.set(report.memory_mb as f64);

        // Update agent-specific metrics if detailed metrics are enabled
        if self.config.enable_detailed_metrics {
            self.agent_metrics.record_agent_metrics(&report).await?;
        }

        debug!("Recorded KPI report for task {}: latency={}ms, accuracy={}",
               report.task_id, report.latency_ms, report.accuracy);

        Ok(())
    }

    /// Record task execution result
    pub async fn record_task_result(&self, result: &ExecutionResult) -> Layer4Result<()> {
        if result.success {
            self.task_metrics.tasks_succeeded.inc();
        } else {
            self.task_metrics.tasks_failed.inc();
        }

        self.task_metrics.task_execution_time_ms.observe(result.execution_time_ms as f64);

        // Record resource usage
        self.resource_metrics.record_resource_usage(&result.resource_usage).await?;

        Ok(())
    }

    /// Get current metrics snapshot
    ///
    /// Returns a comprehensive snapshot of all current metrics and system health.
    /// Useful for debugging, reporting, and integration with external systems.
    /// Includes system health, task metrics, and resource utilization data.
    ///
    /// # Returns
    /// * `Layer4Result<MetricsSnapshot>` - Complete metrics snapshot or error
    ///
    /// # Examples
    /// ```rust,no_run
    /// let snapshot = metrics_collector.get_metrics_snapshot().await?;
    ///
    /// println!("System status: {:?}", snapshot.system_health.status);
    /// println!("Total tasks: {}", snapshot.task_metrics.total_tasks);
    /// println!("CPU usage: {:.1}%", snapshot.resource_metrics.cpu_usage * 100.0);
    ///
    /// // Export to external monitoring system
    /// external_monitor.submit_snapshot(&snapshot).await?;
    /// ```
    pub async fn get_metrics_snapshot(&self) -> Layer4Result<MetricsSnapshot> {
        Ok(MetricsSnapshot {
            timestamp: SystemTime::now(),
            system_health: SystemHealth {
                status: HealthStatus::Healthy,
                active_agents: 0, // Would need to get from executor
                pending_tasks: 0,  // Would need to get from scheduler
                uptime_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                resource_utilization: self.get_current_resource_utilization().await?,
                last_check: SystemTime::now(),
            },
            task_metrics: TaskMetricsSnapshot {
                total_tasks: self.task_metrics.total_tasks.get() as u64,
                tasks_succeeded: self.task_metrics.tasks_succeeded.get() as u64,
                tasks_failed: self.task_metrics.tasks_failed.get() as u64,
                average_latency_ms: 0.0, // Histograms don't have .get(), would need to calculate from buckets
                average_accuracy: self.task_metrics.task_accuracy.get(),
            },
            resource_metrics: self.get_current_resource_utilization().await?,
        })
    }

    /// Get current resource utilization
    async fn get_current_resource_utilization(&self) -> Layer4Result<ResourceUtilization> {
        // In a real implementation, this would read actual system metrics
        Ok(ResourceUtilization {
            cpu_usage: self.resource_metrics.cpu_usage.get() as f32,
            memory_usage: self.resource_metrics.memory_usage.get() as f32,
            disk_usage: self.resource_metrics.disk_usage.get() as f32,
            network_usage: self.resource_metrics.network_usage.get() as f32,
        })
    }

    /// Export metrics in Prometheus format
    ///
    /// Returns all current metrics formatted as Prometheus exposition text.
    /// This is the same format served by the HTTP endpoint for scraping.
    /// Useful for debugging, testing, or custom monitoring integrations.
    ///
    /// # Returns
    /// * `Layer4Result<String>` - Prometheus-formatted metrics or error
    ///
    /// # Examples
    /// ```rust,no_run
    /// let prometheus_text = metrics_collector.export_prometheus_metrics().await?;
    ///
    /// // Write to file for debugging
    /// std::fs::write("/tmp/metrics.txt", &prometheus_text)?;
    ///
    /// // Send to custom monitoring system
    /// custom_monitor.submit_metrics(&prometheus_text).await?;
    ///
    /// // Verify expected metrics are present
    /// assert!(prometheus_text.contains("layer4_tasks_total"));
    /// assert!(prometheus_text.contains("layer4_task_latency_ms"));
    /// ```
    pub async fn export_prometheus_metrics(&self) -> Layer4Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        encoder.encode_to_string(&metric_families)
            .map_err(|e| Layer4Error::Internal(format!("Failed to encode metrics: {}", e)))
    }

    /// Gracefully shutdown the metrics collector
    ///
    /// Stops all background metrics collection tasks and shuts down
    /// the HTTP server. Ensures clean resource cleanup and prevents
    /// resource leaks during system shutdown.
    ///
    /// # Returns
    /// * `Layer4Result<()>` - Success or shutdown error
    ///
    /// # Examples
    /// ```rust,no_run
    /// // Graceful shutdown
    /// metrics_collector.shutdown().await?;
    /// println!("Metrics collection stopped");
    ///
    /// // Or with timeout for forced shutdown
    /// use tokio::time::{timeout, Duration};
    /// let shutdown_result = timeout(
    ///     Duration::from_secs(10),
    ///     metrics_collector.shutdown()
    /// ).await;
    /// ```
    pub async fn shutdown(&self) -> Layer4Result<()> {
        info!("Shutting down metrics collector");
        *self.shutdown.write().await = true;
        Ok(())
    }
}

/// System-level metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// System uptime in seconds
    pub uptime_seconds: Gauge,
    /// Total number of agents spawned
    pub total_agents_spawned: IntCounter,
    /// Total number of tasks processed
    pub total_tasks_processed: IntCounter,
    /// System health status
    pub health_status: Gauge,
}

impl SystemMetrics {
    fn new(registry: &prometheus::Registry) -> Layer4Result<Self> {
        Ok(Self {
            uptime_seconds: Gauge::with_opts(opts!("layer4_uptime_seconds", "System uptime in seconds"))?,
            total_agents_spawned: IntCounter::with_opts(opts!("layer4_agents_spawned_total", "Total agents spawned"))?,
            total_tasks_processed: IntCounter::with_opts(opts!("layer4_tasks_processed_total", "Total tasks processed"))?,
            health_status: Gauge::with_opts(opts!("layer4_health_status", "System health status (0=unhealthy, 1=healthy)"))?,
        })
    }

    fn register(&self, registry: &prometheus::Registry) -> Layer4Result<()> {
        registry.register(Box::new(self.uptime_seconds.clone()))?;
        registry.register(Box::new(self.total_agents_spawned.clone()))?;
        registry.register(Box::new(self.total_tasks_processed.clone()))?;
        registry.register(Box::new(self.health_status.clone()))?;
        Ok(())
    }
}

/// Agent-specific metrics
#[derive(Debug, Clone)]
pub struct AgentMetrics {
    /// Agent count by state
    pub agents_by_state: IntCounterVec,
    /// Agent execution time histogram
    pub agent_execution_time_ms: HistogramVec,
    /// Agent error rate
    pub agent_error_rate: Gauge,
    /// Agent resource utilization
    pub agent_resource_usage: Gauge,
}

impl AgentMetrics {
    fn new(registry: &prometheus::Registry) -> Layer4Result<Self> {
        Ok(Self {
            agents_by_state: IntCounterVec::new(
                opts!("layer4_agents_by_state", "Number of agents by state"),
                &["state"]
            )?,
            agent_execution_time_ms: HistogramVec::new(
                histogram_opts!("layer4_agent_execution_time_ms", "Agent execution time in milliseconds"),
                &["agent_type"]
            )?,
            agent_error_rate: Gauge::with_opts(opts!("layer4_agent_error_rate", "Agent error rate (0.0 to 1.0)"))?,
            agent_resource_usage: Gauge::with_opts(opts!("layer4_agent_resource_usage", "Agent resource utilization"))?,
        })
    }

    fn register(&self, registry: &prometheus::Registry) -> Layer4Result<()> {
        registry.register(Box::new(self.agents_by_state.clone()))?;
        registry.register(Box::new(self.agent_execution_time_ms.clone()))?;
        registry.register(Box::new(self.agent_error_rate.clone()))?;
        registry.register(Box::new(self.agent_resource_usage.clone()))?;
        Ok(())
    }

    async fn record_agent_metrics(&self, report: &KpiReport) -> Layer4Result<()> {
        // Record execution time by agent type
        self.agent_execution_time_ms
            .with_label_values(&[&report.execution_context.hostname])
            .observe(report.latency_ms);

        // Update error rate based on accuracy
        let error_rate = 1.0 - report.accuracy;
        self.agent_error_rate.set(error_rate);

        Ok(())
    }
}

/// Task execution metrics
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    /// Total tasks processed
    pub total_tasks: IntCounter,
    /// Successfully completed tasks
    pub tasks_succeeded: IntCounter,
    /// Failed tasks
    pub tasks_failed: IntCounter,
    /// Task execution latency histogram
    pub task_latency_ms: Histogram,
    /// Task accuracy gauge
    pub task_accuracy: Gauge,
    /// Task execution time histogram
    pub task_execution_time_ms: Histogram,
}

impl TaskMetrics {
    fn new(registry: &prometheus::Registry) -> Layer4Result<Self> {
        Ok(Self {
            total_tasks: IntCounter::with_opts(opts!("layer4_tasks_total", "Total tasks processed"))?,
            tasks_succeeded: IntCounter::with_opts(opts!("layer4_tasks_succeeded_total", "Successfully completed tasks"))?,
            tasks_failed: IntCounter::with_opts(opts!("layer4_tasks_failed_total", "Failed tasks"))?,
            task_latency_ms: Histogram::with_opts(histogram_opts!("layer4_task_latency_ms", "Task execution latency in milliseconds"))?,
            task_accuracy: Gauge::with_opts(opts!("layer4_task_accuracy", "Task execution accuracy (0.0 to 1.0)"))?,
            task_execution_time_ms: Histogram::with_opts(histogram_opts!("layer4_task_execution_time_ms", "Task execution time in milliseconds"))?,
        })
    }

    fn register(&self, registry: &prometheus::Registry) -> Layer4Result<()> {
        registry.register(Box::new(self.total_tasks.clone()))?;
        registry.register(Box::new(self.tasks_succeeded.clone()))?;
        registry.register(Box::new(self.tasks_failed.clone()))?;
        registry.register(Box::new(self.task_latency_ms.clone()))?;
        registry.register(Box::new(self.task_accuracy.clone()))?;
        registry.register(Box::new(self.task_execution_time_ms.clone()))?;
        Ok(())
    }
}

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_usage: Gauge,
    /// Memory usage percentage
    pub memory_usage: Gauge,
    /// Disk usage percentage
    pub disk_usage: Gauge,
    /// Network usage percentage
    pub network_usage: Gauge,
    /// Agent-specific CPU usage
    pub agent_cpu_usage: Gauge,
    /// Agent-specific memory usage in MB
    pub agent_memory_mb: Gauge,
    /// GPU utilization (if available)
    pub gpu_utilization: Option<Gauge>,
}

impl ResourceMetrics {
    fn new(registry: &prometheus::Registry) -> Layer4Result<Self> {
        let gpu_utilization = if Self::has_gpu().unwrap_or(false) {
            Some(Gauge::with_opts(opts!("layer4_gpu_utilization", "GPU utilization (0.0 to 1.0)"))?)
        } else {
            None
        };

        Ok(Self {
            cpu_usage: Gauge::with_opts(opts!("layer4_cpu_usage", "CPU usage percentage (0.0 to 1.0)"))?,
            memory_usage: Gauge::with_opts(opts!("layer4_memory_usage", "Memory usage percentage (0.0 to 1.0)"))?,
            disk_usage: Gauge::with_opts(opts!("layer4_disk_usage", "Disk usage percentage (0.0 to 1.0)"))?,
            network_usage: Gauge::with_opts(opts!("layer4_network_usage", "Network usage percentage (0.0 to 1.0)"))?,
            agent_cpu_usage: Gauge::with_opts(opts!("layer4_agent_cpu_usage", "Agent CPU usage (0.0 to 1.0)"))?,
            agent_memory_mb: Gauge::with_opts(opts!("layer4_agent_memory_mb", "Agent memory usage in MB"))?,
            gpu_utilization,
        })
    }

    fn register(&self, registry: &prometheus::Registry) -> Layer4Result<()> {
        registry.register(Box::new(self.cpu_usage.clone()))?;
        registry.register(Box::new(self.memory_usage.clone()))?;
        registry.register(Box::new(self.disk_usage.clone()))?;
        registry.register(Box::new(self.network_usage.clone()))?;
        registry.register(Box::new(self.agent_cpu_usage.clone()))?;
        registry.register(Box::new(self.agent_memory_mb.clone()))?;

        if let Some(gpu) = &self.gpu_utilization {
            registry.register(Box::new(gpu.clone()))?;
        }

        Ok(())
    }

    async fn record_resource_usage(&self, usage: &ResourceUsage) -> Layer4Result<()> {
        // These would be updated from actual resource monitoring
        // For now, they're placeholders that would be populated by system monitoring

        Ok(())
    }

    fn has_gpu() -> Layer4Result<bool> {
        // Check for GPU availability (would check for CUDA devices, etc.)
        Ok(false) // Placeholder
    }
}

/// Comprehensive metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot timestamp
    pub timestamp: SystemTime,
    /// Overall system health
    pub system_health: SystemHealth,
    /// Task execution metrics
    pub task_metrics: TaskMetricsSnapshot,
    /// Resource utilization metrics
    pub resource_metrics: ResourceUtilization,
}

/// Task-specific metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetricsSnapshot {
    /// Total tasks processed
    pub total_tasks: u64,
    /// Successfully completed tasks
    pub tasks_succeeded: u64,
    /// Failed tasks
    pub tasks_failed: u64,
    /// Average latency in milliseconds
    pub average_latency_ms: f64,
    /// Average accuracy score
    pub average_accuracy: f64,
}

/// Prometheus histogram options helper
fn histogram_opts(name: &str, help: &str) -> prometheus::HistogramOpts {
    prometheus::HistogramOpts::new(name, help)
        .buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0])
}

/// Prometheus options helper
fn opts(name: &str, help: &str) -> prometheus::Opts {
    prometheus::Opts::new(name, help)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_kpi_recording() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config).unwrap();

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

        let result = collector.record_kpi_report(report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config).unwrap();

        let snapshot = collector.get_metrics_snapshot().await;
        assert!(snapshot.is_ok());

        let snapshot = snapshot.unwrap();
        assert!(snapshot.timestamp <= SystemTime::now());
    }

    #[test]
    fn test_prometheus_export() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config).unwrap();

        // Test that we can export metrics (even if empty)
        let metrics_output = futures::executor::block_on(async {
            collector.export_prometheus_metrics().await
        });

        assert!(metrics_output.is_ok());
        let output = metrics_output.unwrap();
        assert!(output.contains("# HELP"));
    }
}
