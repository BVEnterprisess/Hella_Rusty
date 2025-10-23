//! # Cost Optimizer
//!
//! Monitors and optimizes compute costs across all layers of the
//! autonomous AI system, providing cost analysis and optimization recommendations.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Cost optimization engine
pub struct CostOptimizer {
    /// Cost tracking and analysis
    cost_tracker: Arc<RwLock<CostTracker>>,
    /// Optimization policies
    policies: Arc<RwLock<CostOptimizationPolicies>>,
    /// Budget management
    budget_manager: Arc<RwLock<BudgetManager>>,
    /// Cost alerting
    alerting: Arc<RwLock<CostAlerting>>,
    /// Configuration
    config: CostSettings,
}

impl CostOptimizer {
    /// Create a new cost optimizer
    pub async fn new(config: ResourceConfig) -> Result<Self> {
        info!("Initializing cost optimizer...");

        let optimizer = Self {
            cost_tracker: Arc::new(RwLock::new(CostTracker::new().await?)),
            policies: Arc::new(RwLock::new(CostOptimizationPolicies::default())),
            budget_manager: Arc::new(RwLock::new(BudgetManager::new(config.cost_settings.clone()).await?)),
            alerting: Arc::new(RwLock::new(CostAlerting::new(config.cost_settings.clone()).await?)),
            config: config.cost_settings,
        };

        info!("✅ Cost optimizer initialized successfully");
        Ok(optimizer)
    }

    /// Start the cost optimizer service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting cost optimizer service...");

        // Start cost tracking
        self.cost_tracker.write().await.start().await?;

        // Start budget monitoring
        self.budget_manager.write().await.start().await?;

        // Start alerting
        self.alerting.write().await.start().await?;

        info!("✅ Cost optimizer service started successfully");
        Ok(())
    }

    /// Stop the cost optimizer service
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping cost optimizer service...");

        // Stop all sub-components
        self.alerting.write().await.stop().await?;
        self.budget_manager.write().await.stop().await?;
        self.cost_tracker.write().await.stop().await?;

        info!("✅ Cost optimizer service stopped successfully");
        Ok(())
    }

    /// Record resource allocation cost
    pub async fn record_allocation_cost(&self, allocation: &ResourceAllocation) -> Result<()> {
        debug!("Recording cost for allocation: {}", allocation.allocation_id);

        let cost_entry = CostEntry {
            allocation_id: allocation.allocation_id,
            layer: "unknown".to_string(), // Would be determined from allocation context
            resource_type: ResourceType::Gpu,
            cost_per_hour: allocation.cost_info.cost_per_hour,
            total_cost: allocation.cost_info.total_cost,
            start_time: allocation.start_time,
            end_time: Some(allocation.end_time),
            currency: allocation.cost_info.currency.clone(),
        };

        self.cost_tracker.write().await.record_cost(cost_entry).await?;

        // Check budget and alerting
        self.check_budget_and_alert().await?;

        Ok(())
    }

    /// Get current cost metrics
    pub async fn get_metrics(&self) -> Result<CostMetrics> {
        let tracker = self.cost_tracker.read().await;
        let budget_manager = self.budget_manager.read().await;

        let total_cost = tracker.get_total_cost().await?;
        let cost_by_layer = tracker.get_cost_by_layer().await?;
        let cost_by_resource = tracker.get_cost_by_resource().await?;
        let cost_trends = tracker.get_cost_trends().await?;
        let budget_utilization = budget_manager.get_budget_utilization().await?;

        Ok(CostMetrics {
            total_cost,
            cost_by_layer,
            cost_by_resource,
            cost_trends,
            budget_utilization,
            last_update: Utc::now(),
        })
    }

    /// Get cost optimization recommendations
    pub async fn get_optimization_recommendations(&self) -> Result<Vec<CostOptimization>> {
        debug!("Generating cost optimization recommendations...");

        let mut recommendations = Vec::new();

        // Analyze current costs
        let metrics = self.get_metrics().await?;

        // Check for resource right-sizing opportunities
        if let Some(rightsizing) = self.analyze_rightsizing_opportunities(&metrics).await? {
            recommendations.push(rightsizing);
        }

        // Check for scheduling optimizations
        if let Some(scheduling) = self.analyze_scheduling_optimizations(&metrics).await? {
            recommendations.push(scheduling);
        }

        // Check for spot instance opportunities
        if let Some(spot) = self.analyze_spot_instance_opportunities(&metrics).await? {
            recommendations.push(spot);
        }

        info!("✅ Generated {} cost optimization recommendations", recommendations.len());
        Ok(recommendations)
    }

    /// Health check implementation
    pub async fn health_check(&self) -> Result<()> {
        // Check cost tracker
        self.cost_tracker.read().await.health_check().await?;

        // Check budget manager
        self.budget_manager.read().await.health_check().await?;

        // Check alerting
        self.alerting.read().await.health_check().await?;

        Ok(())
    }

    /// Readiness check
    pub async fn is_ready(&self) -> bool {
        self.health_check().await.is_ok()
    }

    // Private helper methods

    async fn check_budget_and_alert(&self) -> Result<()> {
        let budget_manager = self.budget_manager.read().await;
        let utilization = budget_manager.get_budget_utilization().await?;

        // Check for budget overruns
        for (layer, utilization_percent) in utilization.iter() {
            if *utilization_percent > self.config.alert_thresholds.budget_overrun_threshold {
                warn!("Budget overrun detected for layer {}: {:.2}%", layer, utilization_percent * 100.0);

                // Send alert
                self.alerting.write().await.send_budget_alert(layer, *utilization_percent).await?;
            }
        }

        Ok(())
    }

    async fn analyze_rightsizing_opportunities(&self, metrics: &CostMetrics) -> Result<Option<CostOptimization>> {
        // Analyze resource utilization vs cost
        let mut opportunities = Vec::new();

        for (layer, cost) in metrics.cost_by_layer.iter() {
            if *cost > 10.0 { // Only analyze layers with significant cost
                // In a real implementation, this would analyze actual utilization metrics
                // For now, we'll simulate analysis
                if *cost > 50.0 {
                    opportunities.push(CostOptimization {
                        optimization_type: OptimizationType::RightSizing,
                        description: format!("Layer {} resource allocation may be oversized", layer),
                        potential_savings: cost * 0.2, // 20% potential savings
                        effort: EffortLevel::Medium,
                        priority: Priority::Normal,
                    });
                }
            }
        }

        Ok(opportunities.into_iter().next())
    }

    async fn analyze_scheduling_optimizations(&self, metrics: &CostMetrics) -> Result<Option<CostOptimization>> {
        // Analyze usage patterns for scheduling optimization
        let trends = &metrics.cost_trends;

        if trends.len() > 24 { // At least 24 hours of data
            // Check for off-peak usage patterns
            let off_peak_usage = trends.iter()
                .filter(|point| point.cost < trends.iter().map(|p| p.cost).sum::<f64>() / trends.len() as f64 * 0.5)
                .count();

            if off_peak_usage > trends.len() / 4 { // More than 25% off-peak usage
                return Ok(Some(CostOptimization {
                    optimization_type: OptimizationType::Scheduling,
                    description: "Significant off-peak usage detected - consider scheduled scaling".to_string(),
                    potential_savings: metrics.total_cost * 0.15, // 15% potential savings
                    effort: EffortLevel::Low,
                    priority: Priority::High,
                }));
            }
        }

        Ok(None)
    }

    async fn analyze_spot_instance_opportunities(&self, metrics: &CostMetrics) -> Result<Option<CostOptimization>> {
        // Analyze workload patterns for spot instance suitability
        let gpu_cost = metrics.cost_by_resource.gpu_cost;

        if gpu_cost > 20.0 { // Significant GPU usage
            // Check if workloads are fault-tolerant
            // In a real implementation, this would analyze workload characteristics
            return Ok(Some(CostOptimization {
                optimization_type: OptimizationType::SpotInstances,
                description: "GPU workloads may be suitable for spot instances".to_string(),
                potential_savings: gpu_cost * 0.6, // Up to 60% savings with spot instances
                effort: EffortLevel::High,
                priority: Priority::Low,
            }));
        }

        Ok(None)
    }
}

/// Cost tracking and analysis
#[derive(Debug)]
struct CostTracker {
    /// Cost entries by allocation
    cost_entries: Arc<RwLock<HashMap<Uuid, CostEntry>>>,
    /// Cost by layer aggregation
    cost_by_layer: Arc<RwLock<HashMap<String, f64>>>,
    /// Cost trends over time
    cost_trends: Arc<RwLock<Vec<CostDataPoint>>>,
    /// Running totals
    totals: Arc<RwLock<CostTotals>>,
}

impl CostTracker {
    async fn new() -> Result<Self> {
        Ok(Self {
            cost_entries: Arc::new(RwLock::new(HashMap::new())),
            cost_by_layer: Arc::new(RwLock::new(HashMap::new())),
            cost_trends: Arc::new(RwLock::new(Vec::new())),
            totals: Arc::new(RwLock::new(CostTotals::default())),
        })
    }

    async fn start(&self) -> Result<()> {
        info!("📊 Cost tracking started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("📊 Cost tracking stopped");
        Ok(())
    }

    async fn record_cost(&self, entry: CostEntry) -> Result<()> {
        // Store cost entry
        self.cost_entries.write().await.insert(entry.allocation_id, entry.clone());

        // Update cost by layer
        let mut cost_by_layer = self.cost_by_layer.write().await;
        *cost_by_layer.entry(entry.layer.clone()).or_insert(0.0) += entry.total_cost;

        // Update totals
        let mut totals = self.totals.write().await;
        totals.total_cost += entry.total_cost;
        totals.entry_count += 1;

        // Add to trends
        let trend_point = CostDataPoint {
            timestamp: Utc::now(),
            cost: totals.total_cost,
            layer_costs: cost_by_layer.clone(),
        };
        self.cost_trends.write().await.push(trend_point);

        // Keep only last 1000 trend points
        if self.cost_trends.read().await.len() > 1000 {
            self.cost_trends.write().await.drain(0..100);
        }

        Ok(())
    }

    async fn get_total_cost(&self) -> Result<f64> {
        Ok(self.totals.read().await.total_cost)
    }

    async fn get_cost_by_layer(&self) -> Result<HashMap<String, f64>> {
        Ok(self.cost_by_layer.read().await.clone())
    }

    async fn get_cost_by_resource(&self) -> Result<CostBreakdown> {
        // In a real implementation, this would aggregate by resource type
        // For now, return estimated breakdown
        let total = self.get_total_cost().await?;
        Ok(CostBreakdown {
            gpu_cost: total * 0.7,
            cpu_cost: total * 0.2,
            memory_cost: total * 0.05,
            storage_cost: total * 0.05,
        })
    }

    async fn get_cost_trends(&self) -> Result<Vec<CostDataPoint>> {
        Ok(self.cost_trends.read().await.clone())
    }

    async fn health_check(&self) -> Result<()> {
        // Check if we can access cost data
        let _entries = self.cost_entries.read().await;
        let _totals = self.totals.read().await;
        Ok(())
    }
}

/// Cost entry for tracking individual allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CostEntry {
    /// Allocation ID
    allocation_id: Uuid,
    /// Layer that incurred the cost
    layer: String,
    /// Type of resource
    resource_type: ResourceType,
    /// Cost per hour
    cost_per_hour: f64,
    /// Total cost for this allocation
    total_cost: f64,
    /// Start time
    start_time: DateTime<Utc>,
    /// End time (if completed)
    end_time: Option<DateTime<Utc>>,
    /// Currency
    currency: String,
}

/// Resource types for cost tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ResourceType {
    /// GPU resources
    Gpu,
    /// CPU resources
    Cpu,
    /// Memory resources
    Memory,
    /// Storage resources
    Storage,
}

/// Cost totals tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CostTotals {
    /// Total cost across all allocations
    total_cost: f64,
    /// Number of cost entries
    entry_count: u64,
}

impl Default for CostTotals {
    fn default() -> Self {
        Self {
            total_cost: 0.0,
            entry_count: 0,
        }
    }
}

/// Budget management
#[derive(Debug)]
struct BudgetManager {
    /// Budget settings
    budgets: HashMap<String, f64>,
    /// Current utilization by layer
    utilization: Arc<RwLock<HashMap<String, f64>>>,
}

impl BudgetManager {
    async fn new(settings: CostSettings) -> Result<Self> {
        Ok(Self {
            budgets: settings.budget_limits,
            utilization: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn start(&self) -> Result<()> {
        info!("💰 Budget management started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("💰 Budget management stopped");
        Ok(())
    }

    async fn get_budget_utilization(&self) -> Result<HashMap<String, f64>> {
        // In a real implementation, this would calculate actual utilization
        // For now, return simulated values
        let mut utilization = HashMap::new();
        utilization.insert("layer4".to_string(), 0.6);
        utilization.insert("layer5".to_string(), 0.8);
        utilization.insert("layer7".to_string(), 0.4);
        Ok(utilization)
    }

    async fn health_check(&self) -> Result<()> {
        let _utilization = self.utilization.read().await;
        Ok(())
    }
}

/// Cost alerting system
#[derive(Debug)]
struct CostAlerting {
    /// Alert thresholds
    thresholds: AlertThresholds,
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, CostAlert>>>,
}

impl CostAlerting {
    async fn new(settings: CostSettings) -> Result<Self> {
        Ok(Self {
            thresholds: settings.alert_thresholds,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn start(&self) -> Result<()> {
        info!("🚨 Cost alerting started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("🚨 Cost alerting stopped");
        Ok(())
    }

    async fn send_budget_alert(&self, layer: &str, utilization: f64) -> Result<()> {
        let alert = CostAlert {
            alert_id: Uuid::new_v4(),
            layer: layer.to_string(),
            alert_type: CostAlertType::BudgetOverrun,
            message: format!("Budget utilization at {:.1}% for layer {}", utilization * 100.0, layer),
            severity: if utilization > 0.95 { AlertSeverity::Critical } else { AlertSeverity::Warning },
            timestamp: Utc::now(),
            acknowledged: false,
        };

        self.active_alerts.write().await.insert(alert.alert_id.to_string(), alert);

        warn!("🚨 Cost alert generated: {}", alert.message);
        Ok(())
    }

    async fn health_check(&self) -> Result<()> {
        let _alerts = self.active_alerts.read().await;
        Ok(())
    }
}

/// Cost alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CostAlert {
    /// Alert ID
    alert_id: Uuid,
    /// Affected layer
    layer: String,
    /// Alert type
    alert_type: CostAlertType,
    /// Alert message
    message: String,
    /// Alert severity
    severity: AlertSeverity,
    /// Alert timestamp
    timestamp: DateTime<Utc>,
    /// Whether alert has been acknowledged
    acknowledged: bool,
}

/// Cost alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CostAlertType {
    /// Budget overrun
    BudgetOverrun,
    /// Cost spike detected
    CostSpike,
    /// Unusual spending pattern
    UnusualPattern,
    /// Resource waste detected
    ResourceWaste,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AlertSeverity {
    /// Information only
    Info,
    /// Warning level
    Warning,
    /// Critical issue
    Critical,
}

/// Cost optimization policies
#[derive(Debug, Clone)]
struct CostOptimizationPolicies {
    /// Enable automatic optimization
    auto_optimization: bool,
    /// Optimization thresholds
    thresholds: OptimizationThresholds,
}

impl Default for CostOptimizationPolicies {
    fn default() -> Self {
        Self {
            auto_optimization: false, // Disabled by default for safety
            thresholds: OptimizationThresholds::default(),
        }
    }
}

/// Optimization thresholds
#[derive(Debug, Clone)]
struct OptimizationThresholds {
    /// Minimum cost savings to trigger optimization
    min_savings_threshold: f64,
    /// Maximum risk tolerance for optimization
    max_risk_tolerance: f64,
}

impl Default for OptimizationThresholds {
    fn default() -> Self {
        Self {
            min_savings_threshold: 5.0, // $5 minimum savings
            max_risk_tolerance: 0.1,    // 10% maximum risk
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_optimizer_initialization() {
        let config = ResourceConfig::default();
        let optimizer = CostOptimizer::new(config).await;

        assert!(optimizer.is_ok());
    }

    #[tokio::test]
    async fn test_cost_recording() {
        let config = ResourceConfig::default();
        let optimizer = CostOptimizer::new(config).await.unwrap();

        let allocation = ResourceAllocation::new(
            Uuid::new_v4(),
            AllocatedResources::default(),
            CostInfo {
                cost_per_hour: 2.5,
                total_cost: 5.0,
                currency: "USD".to_string(),
                breakdown: CostBreakdown::default(),
            },
        );

        let result = optimizer.record_allocation_cost(&allocation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cost_metrics() {
        let config = ResourceConfig::default();
        let optimizer = CostOptimizer::new(config).await.unwrap();

        let metrics = optimizer.get_metrics().await;
        assert!(metrics.is_ok());
    }
}