//! # Data Collector
//!
//! The Data Collector is responsible for gathering data from multiple sources including
//! system metrics, application logs, network traffic, external APIs, and databases.
//! It provides a unified interface for data ingestion and preprocessing.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Data collector for multi-source data ingestion
pub struct DataCollector {
    config: CollectorConfig,
    data_sources: Arc<Mutex<HashMap<SourceId, Box<dyn DataSource>>>>,
    data_batches: Arc<Mutex<Vec<DataBatch>>>,
    statistics: Arc<Mutex<CollectionStatistics>>,
    is_running: Arc<Mutex<bool>>,
}

impl DataCollector {
    /// Create a new data collector
    pub async fn new(config: CollectorConfig) -> Result<Self, DiscoveryError> {
        let data_sources = Arc::new(Mutex::new(HashMap::new()));
        let data_batches = Arc::new(Mutex::new(Vec::new()));
        let statistics = Arc::new(Mutex::new(CollectionStatistics {
            total_data_points: 0,
            data_points_per_second: 0.0,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            quality_score: 1.0,
        }));

        let mut collector = Self {
            config,
            data_sources,
            data_batches,
            statistics,
            is_running: Arc::new(Mutex::new(false)),
        };

        // Initialize default data sources
        collector.initialize_default_sources().await?;

        Ok(collector)
    }

    /// Start the data collector
    pub async fn start(&mut self) -> Result<(), DiscoveryError> {
        info!("Starting Data Collector");
        *self.is_running.lock().await = true;

        // Start collection loop
        let config = self.config.clone();
        let data_sources = self.data_sources.clone();
        let data_batches = self.data_batches.clone();
        let statistics = self.statistics.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let collection_interval = Duration::from_secs(config.collection_interval_seconds);
            let mut interval = interval(collection_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !*is_running.lock().await {
                            break;
                        }

                        if let Err(e) = Self::perform_collection_cycle(
                            &config,
                            &data_sources,
                            &data_batches,
                            &statistics,
                        ).await {
                            error!("Collection cycle failed: {}", e);
                        }
                    }
                }
            }
        });

        info!("Data Collector started successfully");
        Ok(())
    }

    /// Stop the data collector
    pub async fn stop(&mut self) -> Result<(), DiscoveryError> {
        info!("Stopping Data Collector");
        *self.is_running.lock().await = false;
        info!("Data Collector stopped successfully");
        Ok(())
    }

    /// Get current collection state
    pub async fn get_state(&self) -> Result<CollectionState, DiscoveryError> {
        let data_sources = self.data_sources.lock().await;
        let data_batches = self.data_batches.lock().await;
        let statistics = self.statistics.lock().await;

        let mut sources_map = HashMap::new();
        for (id, source) in data_sources.iter() {
            sources_map.insert(id.clone(), DataSourceInfo {
                id: id.clone(),
                source_type: source.get_source_type(),
                status: SourceStatus::Active, // Would be determined by actual status
                collection_interval_seconds: self.config.collection_interval_seconds,
                last_success: Some(Utc::now()), // Would be tracked properly
                last_error: None,
                config: source.get_config(),
            });
        }

        Ok(CollectionState {
            data_sources: sources_map,
            recent_batches: data_batches.clone(),
            statistics: statistics.clone(),
            last_collection: Utc::now(),
        })
    }

    /// Collect data from all sources
    pub async fn collect_all(&mut self) -> Result<CollectionResult, DiscoveryError> {
        info!("Performing full data collection");

        let mut sources_processed = 0;
        let mut data_points_collected = 0;
        let mut errors = Vec::new();
        let start_time = std::time::Instant::now();

        let data_sources = self.data_sources.lock().await;
        for (source_id, source) in data_sources.iter() {
            match source.collect_data().await {
                Ok(data_batch) => {
                    sources_processed += 1;
                    data_points_collected += data_batch.data_points.len() as u64;

                    // Store the batch
                    self.data_batches.lock().await.push(data_batch);

                    // Update statistics
                    let mut stats = self.statistics.lock().await;
                    stats.total_data_points += data_points_collected;
                }
                Err(e) => {
                    error!("Data source {} failed: {}", source_id, e);
                    errors.push(CollectionError {
                        source_id: source_id.clone(),
                        error_type: CollectionErrorType::ConnectionError,
                        message: e.to_string(),
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        let success_rate = if sources_processed + errors.len() > 0 {
            sources_processed as f64 / (sources_processed + errors.len()) as f64
        } else {
            1.0
        };

        Ok(CollectionResult {
            sources_processed,
            data_points_collected,
            duration_ms: duration,
            success_rate,
            errors,
        })
    }

    /// Get collector health status
    pub async fn health_check(&self) -> Result<ComponentHealth, DiscoveryError> {
        let is_running = *self.is_running.lock().await;
        let sources_count = self.data_sources.lock().await.len();
        let statistics = self.statistics.lock().await;

        let status = if is_running && statistics.success_rate > 0.9 {
            ServiceStatus::Healthy
        } else if is_running {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "data-collector".to_string(),
            status,
            check_duration_ms: 0,
            error_message: None,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("sources_count".to_string(), sources_count as f64);
                metrics.insert("total_data_points".to_string(), statistics.total_data_points as f64);
                metrics.insert("success_rate".to_string(), statistics.success_rate);
                metrics.insert("avg_latency_ms".to_string(), statistics.avg_latency_ms);
                metrics.insert("quality_score".to_string(), statistics.quality_score);
                metrics
            },
        })
    }

    /// Add a new data source
    pub async fn add_data_source(&self, source: Box<dyn DataSource>) -> Result<(), DiscoveryError> {
        let source_id = source.get_source_id();
        self.data_sources.lock().await.insert(source_id, source);
        info!("Added data source: {}", source_id);
        Ok(())
    }

    /// Remove a data source
    pub async fn remove_data_source(&self, source_id: &SourceId) -> Result<(), DiscoveryError> {
        self.data_sources.lock().await.remove(source_id);
        info!("Removed data source: {}", source_id);
        Ok(())
    }

    /// Initialize default data sources
    async fn initialize_default_sources(&mut self) -> Result<(), DiscoveryError> {
        // System metrics source
        self.add_data_source(Box::new(SystemMetricsSource::new())).await?;

        // Application logs source
        self.add_data_source(Box::new(ApplicationLogsSource::new())).await?;

        // Network traffic source
        self.add_data_source(Box::new(NetworkTrafficSource::new())).await?;

        info!("Initialized {} default data sources", 3);
        Ok(())
    }

    /// Perform data collection cycle
    async fn perform_collection_cycle(
        config: &CollectorConfig,
        data_sources: &Arc<Mutex<HashMap<SourceId, Box<dyn DataSource>>>>,
        data_batches: &Arc<Mutex<Vec<DataBatch>>>,
        statistics: &Arc<Mutex<CollectionStatistics>>,
    ) -> Result<(), DiscoveryError> {
        debug!("Starting data collection cycle");

        let mut total_data_points = 0;
        let mut successful_collections = 0;
        let mut failed_collections = 0;
        let mut total_latency = 0.0;

        let sources = data_sources.lock().await;
        for (source_id, source) in sources.iter() {
            let start_time = std::time::Instant::now();

            match source.collect_data().await {
                Ok(data_batch) => {
                    successful_collections += 1;
                    total_data_points += data_batch.data_points.len() as u64;
                    total_latency += start_time.elapsed().as_secs_f64() * 1000.0;

                    // Store the batch
                    data_batches.lock().await.push(data_batch);
                }
                Err(e) => {
                    failed_collections += 1;
                    error!("Data source {} failed: {}", source_id, e);
                }
            }
        }

        // Update statistics
        let mut stats = statistics.lock().await;
        stats.total_data_points += total_data_points;

        if successful_collections + failed_collections > 0 {
            stats.success_rate = successful_collections as f64 / (successful_collections + failed_collections) as f64;
        }

        if successful_collections > 0 {
            stats.avg_latency_ms = total_latency / successful_collections as f64;
            stats.data_points_per_second = total_data_points as f64 / config.collection_interval_seconds as f64;
        }

        debug!("Data collection cycle completed: {} points collected", total_data_points);
        Ok(())
    }
}

/// Trait for data sources
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Collect data from this source
    async fn collect_data(&self) -> Result<DataBatch, CollectionError>;

    /// Get the source identifier
    fn get_source_id(&self) -> SourceId;

    /// Get the source type
    fn get_source_type(&self) -> DataSourceType;

    /// Check if source is available
    fn is_available(&self) -> bool {
        true
    }

    /// Get source configuration
    fn get_config(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// System metrics data source
struct SystemMetricsSource {
    source_id: SourceId,
    config: HashMap<String, String>,
}

impl SystemMetricsSource {
    fn new() -> Self {
        Self {
            source_id: "system-metrics".to_string(),
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl DataSource for SystemMetricsSource {
    async fn collect_data(&self) -> Result<DataBatch, CollectionError> {
        let sys = sysinfo::System::new_all();
        let timestamp = Utc::now();

        let mut data_points = Vec::new();

        // CPU metrics
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        data_points.push(DataPoint {
            metric_name: "cpu_usage_percent".to_string(),
            value: cpu_usage as f64,
            unit: "percent".to_string(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("component".to_string(), "cpu".to_string());
                tags
            },
            timestamp,
        });

        // Memory metrics
        let total_memory = sys.total_memory() as f64;
        let used_memory = sys.used_memory() as f64;
        let memory_usage = (used_memory / total_memory) * 100.0;

        data_points.push(DataPoint {
            metric_name: "memory_usage_percent".to_string(),
            value: memory_usage,
            unit: "percent".to_string(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("component".to_string(), "memory".to_string());
                tags
            },
            timestamp,
        });

        // Disk metrics (placeholder)
        data_points.push(DataPoint {
            metric_name: "disk_usage_percent".to_string(),
            value: 50.0, // Placeholder
            unit: "percent".to_string(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("component".to_string(), "disk".to_string());
                tags
            },
            timestamp,
        });

        Ok(DataBatch {
            source_id: self.source_id.clone(),
            timestamp,
            data_points,
            quality_score: 0.95, // High quality for system metrics
            metadata: HashMap::new(),
        })
    }

    fn get_source_id(&self) -> SourceId {
        self.source_id.clone()
    }

    fn get_source_type(&self) -> DataSourceType {
        DataSourceType::SystemMetrics
    }
}

/// Application logs data source
struct ApplicationLogsSource {
    source_id: SourceId,
    config: HashMap<String, String>,
}

impl ApplicationLogsSource {
    fn new() -> Self {
        Self {
            source_id: "application-logs".to_string(),
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl DataSource for ApplicationLogsSource {
    async fn collect_data(&self) -> Result<DataBatch, CollectionError> {
        // This would read from actual log files or log aggregation systems
        // For now, return placeholder data
        let timestamp = Utc::now();

        let data_points = vec![
            DataPoint {
                metric_name: "log_entries_count".to_string(),
                value: 100.0,
                unit: "count".to_string(),
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("level".to_string(), "info".to_string());
                    tags
                },
                timestamp,
            },
            DataPoint {
                metric_name: "error_rate".to_string(),
                value: 0.05,
                unit: "ratio".to_string(),
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("level".to_string(), "error".to_string());
                    tags
                },
                timestamp,
            },
        ];

        Ok(DataBatch {
            source_id: self.source_id.clone(),
            timestamp,
            data_points,
            quality_score: 0.90,
            metadata: HashMap::new(),
        })
    }

    fn get_source_id(&self) -> SourceId {
        self.source_id.clone()
    }

    fn get_source_type(&self) -> DataSourceType {
        DataSourceType::ApplicationLogs
    }
}

/// Network traffic data source
struct NetworkTrafficSource {
    source_id: SourceId,
    config: HashMap<String, String>,
}

impl NetworkTrafficSource {
    fn new() -> Self {
        Self {
            source_id: "network-traffic".to_string(),
            config: HashMap::new(),
        }
    }
}

#[async_trait]
impl DataSource for NetworkTrafficSource {
    async fn collect_data(&self) -> Result<DataBatch, CollectionError> {
        // This would collect actual network traffic data
        // For now, return placeholder data
        let timestamp = Utc::now();

        let data_points = vec![
            DataPoint {
                metric_name: "bytes_received_per_sec".to_string(),
                value: 1024.0 * 100.0, // 100 KB/s
                unit: "bytes_per_sec".to_string(),
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("direction".to_string(), "inbound".to_string());
                    tags
                },
                timestamp,
            },
            DataPoint {
                metric_name: "bytes_transmitted_per_sec".to_string(),
                value: 1024.0 * 50.0, // 50 KB/s
                unit: "bytes_per_sec".to_string(),
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("direction".to_string(), "outbound".to_string());
                    tags
                },
                timestamp,
            },
            DataPoint {
                metric_name: "active_connections".to_string(),
                value: 25.0,
                unit: "count".to_string(),
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("type".to_string(), "tcp".to_string());
                    tags
                },
                timestamp,
            },
        ];

        Ok(DataBatch {
            source_id: self.source_id.clone(),
            timestamp,
            data_points,
            quality_score: 0.85,
            metadata: HashMap::new(),
        })
    }

    fn get_source_id(&self) -> SourceId {
        self.source_id.clone()
    }

    fn get_source_type(&self) -> DataSourceType {
        DataSourceType::NetworkTraffic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_collector_creation() {
        let config = CollectorConfig::default();
        let collector = DataCollector::new(config).await;
        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_system_metrics_source() {
        let source = SystemMetricsSource::new();
        let result = source.collect_data().await;
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.source_id, "system-metrics");
        assert_eq!(batch.source_type, DataSourceType::SystemMetrics);
        assert!(!batch.data_points.is_empty());
        assert!(batch.quality_score > 0.9);
    }

    #[tokio::test]
    async fn test_application_logs_source() {
        let source = ApplicationLogsSource::new();
        let result = source.collect_data().await;
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.source_id, "application-logs");
        assert_eq!(batch.source_type, DataSourceType::ApplicationLogs);
        assert!(!batch.data_points.is_empty());
    }

    #[tokio::test]
    async fn test_network_traffic_source() {
        let source = NetworkTrafficSource::new();
        let result = source.collect_data().await;
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.source_id, "network-traffic");
        assert_eq!(batch.source_type, DataSourceType::NetworkTraffic);
        assert!(!batch.data_points.is_empty());
    }

    #[test]
    fn test_data_source_types() {
        assert_eq!(DataSourceType::SystemMetrics, DataSourceType::SystemMetrics);
        assert_eq!(DataSourceType::ApplicationLogs, DataSourceType::ApplicationLogs);
        assert_eq!(DataSourceType::NetworkTraffic, DataSourceType::NetworkTraffic);
        assert_eq!(DataSourceType::ExternalAPI, DataSourceType::ExternalAPI);
        assert_eq!(DataSourceType::Database, DataSourceType::Database);
    }
}