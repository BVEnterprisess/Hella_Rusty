//! # Layer 1 Types and Data Structures
//!
//! This module defines all the core types and data structures used throughout Layer 1 (Discovery).
//! These types provide the foundation for environmental awareness, system monitoring, and data collection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Unique identifier for discovered systems
pub type SystemId = String;

/// Unique identifier for data collection sources
pub type SourceId = String;

/// Unique identifier for monitoring checks
pub type CheckId = String;

/// Discovery service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Environmental scanner configuration
    pub scanner: ScannerConfig,
    /// System monitor configuration
    pub monitor: MonitorConfig,
    /// Data collector configuration
    pub collector: CollectorConfig,
    /// Integration hub configuration
    pub integration: IntegrationConfig,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            scanner: ScannerConfig::default(),
            monitor: MonitorConfig::default(),
            collector: CollectorConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

/// Environmental scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Scan interval in seconds
    pub scan_interval_seconds: u64,
    /// Maximum scan duration in seconds
    pub max_scan_duration_seconds: u64,
    /// Network timeout in seconds
    pub network_timeout_seconds: u64,
    /// Discovery cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable deep scanning (more thorough but slower)
    pub deep_scan_enabled: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            scan_interval_seconds: 300, // 5 minutes
            max_scan_duration_seconds: 60, // 1 minute
            network_timeout_seconds: 10,
            cache_ttl_seconds: 3600, // 1 hour
            deep_scan_enabled: false,
        }
    }
}

/// System monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Health check interval in seconds
    pub check_interval_seconds: u64,
    /// Alert threshold for CPU usage (percentage)
    pub cpu_alert_threshold: f64,
    /// Alert threshold for memory usage (percentage)
    pub memory_alert_threshold: f64,
    /// Alert threshold for disk usage (percentage)
    pub disk_alert_threshold: f64,
    /// Enable real-time monitoring
    pub real_time_enabled: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60, // 1 minute
            cpu_alert_threshold: 80.0,
            memory_alert_threshold: 85.0,
            disk_alert_threshold: 90.0,
            real_time_enabled: true,
        }
    }
}

/// Data collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Collection interval in seconds
    pub collection_interval_seconds: u64,
    /// Maximum batch size for data collection
    pub max_batch_size: usize,
    /// Data retention period in hours
    pub retention_hours: u64,
    /// Enable data compression
    pub compression_enabled: bool,
    /// External API timeout in seconds
    pub api_timeout_seconds: u64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 30, // 30 seconds
            max_batch_size: 1000,
            retention_hours: 168, // 7 days
            compression_enabled: true,
            api_timeout_seconds: 30,
        }
    }
}

/// Integration hub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Layer communication timeout in seconds
    pub layer_timeout_seconds: u64,
    /// Maximum retry attempts for failed communications
    pub max_retry_attempts: u32,
    /// Enable data encryption for inter-layer communication
    pub encryption_enabled: bool,
    /// Queue size for outgoing messages
    pub queue_size: usize,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            layer_timeout_seconds: 10,
            max_retry_attempts: 3,
            encryption_enabled: true,
            queue_size: 10000,
        }
    }
}

/// Represents a discovered system or service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSystem {
    /// Unique system identifier
    pub id: SystemId,
    /// System name or hostname
    pub name: String,
    /// System type (server, container, service, etc.)
    pub system_type: SystemType,
    /// Network address
    pub address: String,
    /// Port number
    pub port: Option<u16>,
    /// System status
    pub status: SystemStatus,
    /// System capabilities and features
    pub capabilities: Vec<String>,
    /// Resource information
    pub resources: SystemResources,
    /// Metadata and tags
    pub metadata: HashMap<String, String>,
    /// Discovery timestamp
    pub discovered_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Types of systems that can be discovered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemType {
    Server,
    Container,
    Service,
    Database,
    NetworkDevice,
    Storage,
    LoadBalancer,
    APIEndpoint,
    Custom(String),
}

/// Current status of a discovered system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemStatus {
    Online,
    Offline,
    Degraded,
    Unknown,
    Maintenance,
}

/// Resource information for a system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// CPU cores available
    pub cpu_cores: Option<u32>,
    /// Memory in MB
    pub memory_mb: Option<u64>,
    /// Disk space in GB
    pub disk_gb: Option<u64>,
    /// Network bandwidth in Mbps
    pub network_mbps: Option<u32>,
    /// GPU information
    pub gpu_info: Option<String>,
}

/// System health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check identifier
    pub check_id: CheckId,
    /// System being checked
    pub system_id: SystemId,
    /// Check type
    pub check_type: HealthCheckType,
    /// Check status
    pub status: HealthStatus,
    /// Check duration in milliseconds
    pub duration_ms: u64,
    /// Error message if check failed
    pub error_message: Option<String>,
    /// Additional metrics from the check
    pub metrics: HashMap<String, f64>,
    /// Check timestamp
    pub timestamp: DateTime<Utc>,
}

/// Types of health checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthCheckType {
    Connectivity,
    Performance,
    ResourceUsage,
    ServiceAvailability,
    Security,
    Custom(String),
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Collected data batch from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBatch {
    /// Source identifier
    pub source_id: SourceId,
    /// Data collection timestamp
    pub timestamp: DateTime<Utc>,
    /// Raw data points
    pub data_points: Vec<DataPoint>,
    /// Data quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Metadata about the collection
    pub metadata: HashMap<String, String>,
}

/// Individual data point in a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Metric name
    pub metric_name: String,
    /// Metric value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Tags for categorization
    pub tags: HashMap<String, String>,
    /// Data point timestamp
    pub timestamp: DateTime<Utc>,
}

/// Data source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataSourceType {
    SystemMetrics,
    ApplicationLogs,
    NetworkTraffic,
    ExternalAPI,
    Database,
    FileSystem,
    Custom(String),
}

/// Complete system state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// Environmental state
    pub environmental: EnvironmentalState,
    /// Monitoring state
    pub monitoring: MonitoringState,
    /// Collection state
    pub collection: CollectionState,
    /// State timestamp
    pub timestamp: DateTime<Utc>,
}

/// Environmental scanning state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalState {
    /// Discovered systems
    pub systems: HashMap<SystemId, DiscoveredSystem>,
    /// Network topology
    pub network_topology: NetworkTopology,
    /// Resource inventory
    pub resource_inventory: ResourceInventory,
    /// Last scan timestamp
    pub last_scan: DateTime<Utc>,
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Network segments
    pub segments: Vec<NetworkSegment>,
    /// Connection graph
    pub connections: HashMap<String, Vec<String>>,
    /// Network health status
    pub health_status: NetworkHealth,
}

/// Network segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSegment {
    /// Segment identifier
    pub id: String,
    /// Network range (CIDR notation)
    pub range: String,
    /// Systems in this segment
    pub systems: Vec<SystemId>,
    /// Segment status
    pub status: NetworkStatus,
}

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkStatus {
    Operational,
    Degraded,
    Outage,
    Maintenance,
}

/// Network health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    /// Overall health score (0.0 to 1.0)
    pub score: f64,
    /// Latency statistics
    pub latency_ms: Option<f64>,
    /// Packet loss percentage
    pub packet_loss_percent: Option<f64>,
    /// Bandwidth utilization
    pub bandwidth_utilization: Option<f64>,
}

/// Resource inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInventory {
    /// Total CPU cores
    pub total_cpu_cores: u32,
    /// Total memory in MB
    pub total_memory_mb: u64,
    /// Total disk space in GB
    pub total_disk_gb: u64,
    /// Available resources
    pub available: ResourceAvailability,
}

/// Available resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailability {
    /// Available CPU cores
    pub cpu_cores: u32,
    /// Available memory in MB
    pub memory_mb: u64,
    /// Available disk space in GB
    pub disk_gb: u64,
    /// Available network bandwidth in Mbps
    pub network_mbps: u32,
}

/// Monitoring state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringState {
    /// Active health checks
    pub health_checks: HashMap<CheckId, HealthCheck>,
    /// System performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Active alerts
    pub alerts: Vec<Alert>,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network I/O metrics
    pub network_io: NetworkIOMetrics,
    /// Process information
    pub processes: Vec<ProcessInfo>,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIOMetrics {
    /// Bytes received per second
    pub bytes_received_per_sec: u64,
    /// Bytes transmitted per second
    pub bytes_transmitted_per_sec: u64,
    /// Active connections
    pub active_connections: u32,
    /// Connection errors
    pub connection_errors: u32,
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Process status
    pub status: ProcessStatus,
}

/// Process status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert identifier
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Affected system
    pub system_id: Option<SystemId>,
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Acknowledged status
    pub acknowledged: bool,
    /// Acknowledged by
    pub acknowledged_by: Option<String>,
    /// Acknowledged timestamp
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Data collection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionState {
    /// Active data sources
    pub data_sources: HashMap<SourceId, DataSourceInfo>,
    /// Recent data batches
    pub recent_batches: Vec<DataBatch>,
    /// Collection statistics
    pub statistics: CollectionStatistics,
    /// Last collection timestamp
    pub last_collection: DateTime<Utc>,
}

/// Data source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceInfo {
    /// Source identifier
    pub id: SourceId,
    /// Source type
    pub source_type: DataSourceType,
    /// Source status
    pub status: SourceStatus,
    /// Collection interval in seconds
    pub collection_interval_seconds: u64,
    /// Last successful collection
    pub last_success: Option<DateTime<Utc>>,
    /// Last error
    pub last_error: Option<String>,
    /// Configuration
    pub config: HashMap<String, String>,
}

/// Data source status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceStatus {
    Active,
    Inactive,
    Error,
    Disabled,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStatistics {
    /// Total data points collected
    pub total_data_points: u64,
    /// Data points per second
    pub data_points_per_second: f64,
    /// Collection success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average collection latency in milliseconds
    pub avg_latency_ms: f64,
    /// Data quality score (0.0 to 1.0)
    pub quality_score: f64,
}

/// Discovery data for inter-layer communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryData {
    /// Full system scan results
    FullScanResult(FullScanResult),
    /// System state update
    SystemStateUpdate(SystemState),
    /// Health check results
    HealthCheckResults(Vec<HealthCheck>),
    /// Data collection batch
    DataBatch(DataBatch),
    /// Alert notification
    Alert(Alert),
    /// Performance metrics update
    PerformanceUpdate(PerformanceMetrics),
}

/// Complete scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullScanResult {
    /// Environmental scan result
    pub scan: ScanResult,
    /// Monitoring check result
    pub monitoring: MonitoringResult,
    /// Data collection result
    pub collection: CollectionResult,
    /// Scan timestamp
    pub timestamp: DateTime<Utc>,
}

/// Environmental scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Number of systems discovered
    pub systems_discovered: u32,
    /// Amount of data collected (in bytes)
    pub data_collected: u64,
    /// Issues found during scan
    pub issues_found: Vec<ScanIssue>,
    /// Scan timestamp
    pub timestamp: DateTime<Utc>,
}

/// Scan issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue title
    pub title: String,
    /// Issue description
    pub description: String,
    /// Affected system
    pub system_id: Option<SystemId>,
    /// Issue category
    pub category: IssueCategory,
    /// Resolution suggestions
    pub suggestions: Vec<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Issue categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueCategory {
    Security,
    Performance,
    Configuration,
    Network,
    Resource,
    Custom(String),
}

/// Monitoring check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringResult {
    /// Total checks performed
    pub checks_performed: u32,
    /// Healthy checks
    pub healthy_checks: u32,
    /// Warning checks
    pub warning_checks: u32,
    /// Critical checks
    pub critical_checks: u32,
    /// Average check duration in milliseconds
    pub avg_check_duration_ms: f64,
    /// Issues found
    pub issues: Vec<MonitoringIssue>,
}

/// Monitoring issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringIssue {
    /// Check that failed
    pub check_id: CheckId,
    /// System affected
    pub system_id: SystemId,
    /// Issue description
    pub description: String,
    /// Severity level
    pub severity: IssueSeverity,
    /// Threshold value
    pub threshold: Option<f64>,
    /// Actual value
    pub actual_value: Option<f64>,
}

/// Data collection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResult {
    /// Sources processed
    pub sources_processed: u32,
    /// Data points collected
    pub data_points_collected: u64,
    /// Collection duration in milliseconds
    pub duration_ms: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Errors encountered
    pub errors: Vec<CollectionError>,
}

/// Collection error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionError {
    /// Source that failed
    pub source_id: SourceId,
    /// Error type
    pub error_type: CollectionErrorType,
    /// Error message
    pub message: String,
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
}

/// Collection error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollectionErrorType {
    ConnectionError,
    AuthenticationError,
    TimeoutError,
    DataFormatError,
    PermissionError,
    Custom(String),
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service name
    pub service: String,
    /// Overall service status
    pub status: ServiceStatus,
    /// Component health details
    pub components: Vec<ComponentHealth>,
    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Starting,
    Stopping,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Component status
    pub status: ServiceStatus,
    /// Health check duration in milliseconds
    pub check_duration_ms: u64,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Component metrics
    pub metrics: HashMap<String, f64>,
}

/// Discovery service errors
#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Scanner error: {0}")]
    ScannerError(String),

    #[error("Monitor error: {0}")]
    MonitorError(String),

    #[error("Collector error: {0}")]
    CollectorError(String),

    #[error("Integration error: {0}")]
    IntegrationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Scan error types
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Network timeout: {0}")]
    NetworkTimeout(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Scan cancelled")]
    Cancelled,
}

/// Health check error types
#[derive(Error, Debug)]
pub enum HealthError {
    #[error("Check timeout: {0}")]
    CheckTimeout(String),

    #[error("Check failed: {0}")]
    CheckFailed(String),

    #[error("Invalid check configuration: {0}")]
    InvalidConfiguration(String),

    #[error("System unreachable: {0}")]
    SystemUnreachable(String),
}

/// Collection error types
#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("Source unavailable: {0}")]
    SourceUnavailable(String),

    #[error("Data format invalid: {0}")]
    DataFormatInvalid(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Storage full: {0}")]
    StorageFull(String),

    #[error("Processing timeout: {0}")]
    ProcessingTimeout(String),
}

/// Integration error types
#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Layer communication failed: {0}")]
    LayerCommunicationFailed(String),

    #[error("Data serialization error: {0}")]
    DataSerializationError(String),

    #[error("Protocol mismatch: {0}")]
    ProtocolMismatch(String),

    #[error("Queue full: {0}")]
    QueueFull(String),

    #[error("Connection lost: {0}")]
    ConnectionLost(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.scanner.scan_interval_seconds, 300);
        assert_eq!(config.monitor.check_interval_seconds, 60);
        assert_eq!(config.collector.collection_interval_seconds, 30);
    }

    #[test]
    fn test_system_types() {
        assert_eq!(SystemType::Server, SystemType::Server);
        assert_eq!(SystemType::Container, SystemType::Container);
        assert_eq!(SystemType::Service, SystemType::Service);
    }

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Warning, HealthStatus::Warning);
        assert_eq!(HealthStatus::Critical, HealthStatus::Critical);
    }

    #[test]
    fn test_alert_severity() {
        assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
        assert_eq!(AlertSeverity::Warning, AlertSeverity::Warning);
        assert_eq!(AlertSeverity::Error, AlertSeverity::Error);
        assert_eq!(AlertSeverity::Critical, AlertSeverity::Critical);
    }
}