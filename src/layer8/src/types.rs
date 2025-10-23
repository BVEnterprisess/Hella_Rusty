//! # Layer 8 Types and Data Structures
//!
//! Core types and data structures for the resource management layer.
//! Defines resource allocation, cost optimization, and GPU management types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Resource configuration for the resource manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// GPU resource limits
    pub gpu_limits: GpuLimits,
    /// Cost optimization settings
    pub cost_settings: CostSettings,
    /// Integration settings for other layers
    pub integration: IntegrationSettings,
    /// Monitoring and alerting configuration
    pub monitoring: MonitoringSettings,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            gpu_limits: GpuLimits::default(),
            cost_settings: CostSettings::default(),
            integration: IntegrationSettings::default(),
            monitoring: MonitoringSettings::default(),
        }
    }
}

/// GPU resource limits and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuLimits {
    /// Maximum number of GPUs per allocation
    pub max_gpus_per_allocation: u32,
    /// Maximum memory per GPU in GB
    pub max_memory_per_gpu_gb: u64,
    /// Maximum allocation time in minutes
    pub max_allocation_time_minutes: u64,
    /// GPU utilization threshold for optimization
    pub utilization_threshold: f64,
}

impl Default for GpuLimits {
    fn default() -> Self {
        Self {
            max_gpus_per_allocation: 4,
            max_memory_per_gpu_gb: 24,
            max_allocation_time_minutes: 480, // 8 hours
            utilization_threshold: 0.8,
        }
    }
}

/// Cost optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSettings {
    /// Cost per GPU hour in USD
    pub cost_per_gpu_hour: f64,
    /// Budget limits per layer
    pub budget_limits: HashMap<String, f64>,
    /// Cost optimization targets
    pub optimization_targets: OptimizationTargets,
    /// Cost alerting thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for CostSettings {
    fn default() -> Self {
        let mut budget_limits = HashMap::new();
        budget_limits.insert("layer4".to_string(), 100.0);
        budget_limits.insert("layer5".to_string(), 200.0);
        budget_limits.insert("layer7".to_string(), 150.0);

        Self {
            cost_per_gpu_hour: 0.5,
            budget_limits,
            optimization_targets: OptimizationTargets::default(),
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Cost optimization targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTargets {
    /// Target cost reduction percentage
    pub cost_reduction_target: f64,
    /// Target resource utilization
    pub utilization_target: f64,
    /// Target efficiency improvement
    pub efficiency_target: f64,
}

impl Default for OptimizationTargets {
    fn default() -> Self {
        Self {
            cost_reduction_target: 0.2, // 20% reduction
            utilization_target: 0.85,   // 85% utilization
            efficiency_target: 0.15,    // 15% efficiency improvement
        }
    }
}

/// Cost alerting thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Warning threshold for daily costs
    pub daily_cost_warning: f64,
    /// Critical threshold for daily costs
    pub daily_cost_critical: f64,
    /// Budget overrun threshold
    pub budget_overrun_threshold: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            daily_cost_warning: 50.0,
            daily_cost_critical: 100.0,
            budget_overrun_threshold: 0.9, // 90% of budget
        }
    }
}

/// Integration settings for other layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSettings {
    /// Layer 4 integration endpoints
    pub layer4_endpoints: LayerEndpoints,
    /// Layer 5 integration endpoints
    pub layer5_endpoints: LayerEndpoints,
    /// Layer 7 integration endpoints
    pub layer7_endpoints: LayerEndpoints,
    /// Request timeout settings
    pub timeouts: TimeoutSettings,
}

impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            layer4_endpoints: LayerEndpoints::default(),
            layer5_endpoints: LayerEndpoints::default(),
            layer7_endpoints: LayerEndpoints::default(),
            timeouts: TimeoutSettings::default(),
        }
    }
}

/// Layer endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerEndpoints {
    /// Health check endpoint
    pub health_endpoint: String,
    /// Resource request endpoint
    pub resource_endpoint: String,
    /// Metrics endpoint
    pub metrics_endpoint: String,
}

impl Default for LayerEndpoints {
    fn default() -> Self {
        Self {
            health_endpoint: "http://localhost:8000/health".to_string(),
            resource_endpoint: "http://localhost:8000/api/resources".to_string(),
            metrics_endpoint: "http://localhost:8000/metrics".to_string(),
        }
    }
}

/// Timeout settings for layer communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutSettings {
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Retry attempts
    pub retry_attempts: u32,
}

impl Default for TimeoutSettings {
    fn default() -> Self {
        Self {
            request_timeout_seconds: 30,
            connection_timeout_seconds: 10,
            retry_attempts: 3,
        }
    }
}

/// Monitoring and alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
    /// Alert check interval in seconds
    pub alert_interval_seconds: u64,
    /// Metrics retention period in days
    pub retention_days: u32,
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            metrics_interval_seconds: 60,
            alert_interval_seconds: 30,
            retention_days: 30,
        }
    }
}

/// Resource allocation request from other layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// Unique request ID
    pub request_id: Uuid,
    /// Requesting layer (layer4, layer5, layer7)
    pub requesting_layer: String,
    /// Resource requirements
    pub requirements: ResourceRequirements,
    /// Priority level
    pub priority: Priority,
    /// Maximum cost willing to pay
    pub max_cost_per_hour: Option<f64>,
    /// Requested duration in minutes
    pub duration_minutes: u64,
    /// Timestamp of request
    pub timestamp: DateTime<Utc>,
}

impl ResourceRequest {
    /// Create a new resource request
    pub fn new(
        requesting_layer: String,
        requirements: ResourceRequirements,
        priority: Priority,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            requesting_layer,
            requirements,
            priority,
            max_cost_per_hour: None,
            duration_minutes: 60,
            timestamp: Utc::now(),
        }
    }
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Number of GPUs required
    pub gpu_count: u32,
    /// GPU memory per GPU in GB
    pub gpu_memory_gb: u64,
    /// CPU cores required
    pub cpu_cores: u32,
    /// RAM required in GB
    pub ram_gb: u64,
    /// Storage required in GB
    pub storage_gb: u64,
    /// Special requirements
    pub special_requirements: Vec<String>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            gpu_count: 1,
            gpu_memory_gb: 8,
            cpu_cores: 4,
            ram_gb: 16,
            storage_gb: 50,
            special_requirements: Vec::new(),
        }
    }
}

/// Priority levels for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Critical priority - immediate allocation required
    Critical,
    /// High priority - allocate within minutes
    High,
    /// Normal priority - allocate within hours
    Normal,
    /// Low priority - allocate when resources available
    Low,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Resource allocation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Allocation ID
    pub allocation_id: Uuid,
    /// Original request ID
    pub request_id: Uuid,
    /// Allocated resources
    pub allocated_resources: AllocatedResources,
    /// Cost information
    pub cost_info: CostInfo,
    /// Allocation status
    pub status: AllocationStatus,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
}

impl ResourceAllocation {
    /// Create a new allocation
    pub fn new(
        request_id: Uuid,
        allocated_resources: AllocatedResources,
        cost_info: CostInfo,
    ) -> Self {
        let now = Utc::now();
        Self {
            allocation_id: Uuid::new_v4(),
            request_id,
            allocated_resources,
            cost_info,
            status: AllocationStatus::Active,
            start_time: now,
            end_time: now + chrono::Duration::minutes(60),
        }
    }
}

/// Actually allocated resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    /// Allocated GPU IDs
    pub gpu_ids: Vec<String>,
    /// Allocated CPU cores
    pub cpu_cores: u32,
    /// Allocated RAM in GB
    pub ram_gb: u64,
    /// Allocated storage in GB
    pub storage_gb: u64,
    /// Kubernetes pod/node information
    pub kubernetes_info: KubernetesInfo,
}

impl Default for AllocatedResources {
    fn default() -> Self {
        Self {
            gpu_ids: Vec::new(),
            cpu_cores: 0,
            ram_gb: 0,
            storage_gb: 0,
            kubernetes_info: KubernetesInfo::default(),
        }
    }
}

/// Kubernetes resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesInfo {
    /// Pod name
    pub pod_name: String,
    /// Node name
    pub node_name: String,
    /// Namespace
    pub namespace: String,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Resource requests
    pub requests: ResourceLimits,
}

impl Default for KubernetesInfo {
    fn default() -> Self {
        Self {
            pod_name: String::new(),
            node_name: String::new(),
            namespace: "default".to_string(),
            limits: ResourceLimits::default(),
            requests: ResourceLimits::default(),
        }
    }
}

/// Kubernetes resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit
    pub cpu: String,
    /// Memory limit
    pub memory: String,
    /// GPU limit
    pub gpu: Option<String>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu: "4000m".to_string(),
            memory: "16Gi".to_string(),
            gpu: Some("1".to_string()),
        }
    }
}

/// Cost information for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    /// Cost per hour
    pub cost_per_hour: f64,
    /// Total estimated cost
    pub total_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost breakdown by resource type
    pub breakdown: CostBreakdown,
}

impl Default for CostInfo {
    fn default() -> Self {
        Self {
            cost_per_hour: 0.0,
            total_cost: 0.0,
            currency: "USD".to_string(),
            breakdown: CostBreakdown::default(),
        }
    }
}

/// Cost breakdown by resource type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// GPU costs
    pub gpu_cost: f64,
    /// CPU costs
    pub cpu_cost: f64,
    /// Memory costs
    pub memory_cost: f64,
    /// Storage costs
    pub storage_cost: f64,
}

impl Default for CostBreakdown {
    fn default() -> Self {
        Self {
            gpu_cost: 0.0,
            cpu_cost: 0.0,
            memory_cost: 0.0,
            storage_cost: 0.0,
        }
    }
}

/// Allocation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllocationStatus {
    /// Allocation is active and running
    Active,
    /// Allocation is pending approval
    Pending,
    /// Allocation is being provisioned
    Provisioning,
    /// Allocation completed successfully
    Completed,
    /// Allocation failed
    Failed,
    /// Allocation was cancelled
    Cancelled,
    /// Allocation expired
    Expired,
}

impl Default for AllocationStatus {
    fn default() -> Self {
        AllocationStatus::Pending
    }
}

/// Health status for the resource manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub healthy: bool,
    /// Component-specific health issues
    pub issues: HashMap<String, String>,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

impl HealthStatus {
    /// Create a healthy status
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            issues: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Add an issue to the health status
    pub fn add_issue(&mut self, component: &str, issue: String) {
        self.healthy = false;
        self.issues.insert(component.to_string(), issue);
    }
}

/// Readiness status for Kubernetes probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessStatus {
    /// Whether the service is ready to accept requests
    pub ready: bool,
    /// List of readiness issues
    pub issues: Vec<String>,
}

/// Liveness status for Kubernetes probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessStatus {
    /// Whether the service is alive
    pub alive: bool,
}

/// GPU status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    /// Available GPU count
    pub available_gpus: u32,
    /// Total GPU count
    pub total_gpus: u32,
    /// GPU utilization percentages
    pub utilization: Vec<f64>,
    /// GPU memory usage in GB
    pub memory_usage_gb: Vec<u64>,
    /// GPU temperature in Celsius
    pub temperatures: Vec<f64>,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// Cost metrics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Total cost for current period
    pub total_cost: f64,
    /// Cost by layer
    pub cost_by_layer: HashMap<String, f64>,
    /// Cost by resource type
    pub cost_by_resource: CostBreakdown,
    /// Cost trends over time
    pub cost_trends: Vec<CostDataPoint>,
    /// Budget utilization
    pub budget_utilization: HashMap<String, f64>,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// Cost data point for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Cost at this point
    pub cost: f64,
    /// Layer breakdown
    pub layer_costs: HashMap<String, f64>,
}

/// Capacity planning recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityRecommendations {
    /// Recommended resource scaling
    pub scaling_recommendations: Vec<ScalingRecommendation>,
    /// Cost optimization suggestions
    pub cost_optimizations: Vec<CostOptimization>,
    /// Performance improvements
    pub performance_improvements: Vec<PerformanceImprovement>,
    /// Risk assessments
    pub risk_assessments: Vec<RiskAssessment>,
    /// Generated timestamp
    pub generated_at: DateTime<Utc>,
}

/// Scaling recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    /// Target layer
    pub layer: String,
    /// Recommended scaling action
    pub action: ScalingAction,
    /// Rationale
    pub rationale: String,
    /// Expected impact
    pub expected_impact: Impact,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

/// Scaling actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    /// Scale up resources
    ScaleUp { factor: f64 },
    /// Scale down resources
    ScaleDown { factor: f64 },
    /// No change needed
    NoChange,
}

/// Cost optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    /// Optimization type
    pub optimization_type: OptimizationType,
    /// Description
    pub description: String,
    /// Potential savings
    pub potential_savings: f64,
    /// Implementation effort
    pub effort: EffortLevel,
    /// Priority
    pub priority: Priority,
}

/// Optimization types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Resource right-sizing
    RightSizing,
    /// Schedule optimization
    Scheduling,
    /// Spot instance usage
    SpotInstances,
    /// Reserved instance optimization
    ReservedInstances,
}

/// Effort levels for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    /// Low effort (automated)
    Low,
    /// Medium effort (configuration changes)
    Medium,
    /// High effort (code changes required)
    High,
}

/// Performance improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Improvement area
    pub area: String,
    /// Description
    pub description: String,
    /// Expected performance gain
    pub expected_gain: f64,
    /// Implementation effort
    pub effort: EffortLevel,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Risk type
    pub risk_type: RiskType,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Description
    pub description: String,
    /// Mitigation strategies
    pub mitigation: Vec<String>,
}

/// Risk types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    /// Resource exhaustion
    ResourceExhaustion,
    /// Cost overrun
    CostOverrun,
    /// Performance degradation
    PerformanceDegradation,
    /// Security vulnerability
    SecurityRisk,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

/// Expected impact of a recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    /// Cost impact (positive = savings, negative = cost)
    pub cost_impact: f64,
    /// Performance impact (positive = improvement, negative = degradation)
    pub performance_impact: f64,
    /// Reliability impact (positive = improvement, negative = degradation)
    pub reliability_impact: f64,
}

impl Default for Impact {
    fn default() -> Self {
        Self {
            cost_impact: 0.0,
            performance_impact: 0.0,
            reliability_impact: 0.0,
        }
    }
}

/// Error types for the resource management layer
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    /// Resource allocation failed
    #[error("Resource allocation failed: {message}")]
    AllocationFailed { message: String },

    /// Insufficient resources available
    #[error("Insufficient resources: requested {requested}, available {available}")]
    InsufficientResources {
        requested: ResourceRequirements,
        available: ResourceRequirements,
    },

    /// Cost limit exceeded
    #[error("Cost limit exceeded: budget {budget}, current {current}")]
    CostLimitExceeded { budget: f64, current: f64 },

    /// Integration error with other layers
    #[error("Integration error with {layer}: {message}")]
    IntegrationError { layer: String, message: String },

    /// GPU management error
    #[error("GPU management error: {message}")]
    GpuError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Database error
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },
}

/// Result type for resource management operations
pub type ResourceResult<T> = Result<T, ResourceError>;