//! # Capacity Planner
//!
//! Predictive capacity planning and resource forecasting for the
//! autonomous AI system based on historical usage patterns.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Capacity planning and forecasting engine
pub struct CapacityPlanner {
    /// Historical usage data
    usage_history: Arc<RwLock<Vec<UsageDataPoint>>>,
    /// Forecasting models
    forecasting_models: Arc<RwLock<HashMap<String, ForecastingModel>>>,
    /// Configuration
    config: ResourceConfig,
    /// Planning policies
    policies: Arc<RwLock<PlanningPolicies>>,
}

impl CapacityPlanner {
    /// Create a new capacity planner
    pub async fn new(config: ResourceConfig) -> Result<Self> {
        info!("Initializing capacity planner...");

        let planner = Self {
            usage_history: Arc::new(RwLock::new(Vec::new())),
            forecasting_models: Arc::new(RwLock::new(HashMap::new())),
            config,
            policies: Arc::new(RwLock::new(PlanningPolicies::default())),
        };

        // Initialize forecasting models
        planner.initialize_models().await?;

        info!("✅ Capacity planner initialized successfully");
        Ok(planner)
    }

    /// Start the capacity planner service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting capacity planner service...");

        // Start data collection
        self.start_data_collection().await?;

        info!("✅ Capacity planner service started successfully");
        Ok(())
    }

    /// Stop the capacity planner service
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping capacity planner service...");

        // Stop data collection
        self.stop_data_collection().await?;

        info!("✅ Capacity planner service stopped successfully");
        Ok(())
    }

    /// Record usage data point
    pub async fn record_usage(&self, usage: UsageDataPoint) -> Result<()> {
        debug!("Recording usage data point: {:?}", usage);

        let mut history = self.usage_history.write().await;
        history.push(usage);

        // Keep only last 30 days of data
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        history.retain(|point| point.timestamp > thirty_days_ago);

        // Update forecasting models
        self.update_models().await?;

        Ok(())
    }

    /// Get capacity recommendations
    pub async fn get_recommendations(&self) -> Result<CapacityRecommendations> {
        debug!("Generating capacity recommendations...");

        let mut recommendations = Vec::new();

        // Analyze current trends
        let trends = self.analyze_trends().await?;

        // Generate scaling recommendations
        if let Some(scaling) = self.generate_scaling_recommendations(&trends).await? {
            recommendations.push(scaling);
        }

        // Generate cost optimizations
        let cost_opts = self.generate_cost_optimizations(&trends).await?;
        recommendations.extend(cost_opts);

        // Generate performance improvements
        let perf_improvements = self.generate_performance_improvements(&trends).await?;
        recommendations.extend(perf_improvements);

        // Generate risk assessments
        let risks = self.assess_risks(&trends).await?;
        recommendations.extend(risks);

        Ok(CapacityRecommendations {
            scaling_recommendations: recommendations.iter()
                .filter(|r| matches!(r, ScalingRecommendation { .. }))
                .cloned()
                .collect(),
            cost_optimizations: recommendations.iter()
                .filter(|r| matches!(r, CostOptimization { .. }))
                .cloned()
                .collect(),
            performance_improvements: recommendations.iter()
                .filter(|r| matches!(r, PerformanceImprovement { .. }))
                .cloned()
                .collect(),
            risk_assessments: recommendations.iter()
                .filter(|r| matches!(r, RiskAssessment { .. }))
                .cloned()
                .collect(),
            generated_at: Utc::now(),
        })
    }

    /// Get usage forecast for a specific time period
    pub async fn get_usage_forecast(&self, hours_ahead: u32) -> Result<UsageForecast> {
        debug!("Generating usage forecast for {} hours ahead", hours_ahead);

        let models = self.forecasting_models.read().await;
        let history = self.usage_history.read().await;

        if history.len() < 24 {
            return Err(anyhow::anyhow!("Insufficient historical data for forecasting"));
        }

        // Generate forecast using linear regression model
        let forecast = self.generate_linear_forecast(&history, hours_ahead).await?;

        Ok(forecast)
    }

    /// Health check implementation
    pub async fn health_check(&self) -> Result<()> {
        // Check if we can access usage history
        let _history = self.usage_history.read().await;

        // Check if models are available
        let _models = self.forecasting_models.read().await;

        Ok(())
    }

    /// Readiness check
    pub async fn is_ready(&self) -> bool {
        self.health_check().await.is_ok() && !self.usage_history.read().await.is_empty()
    }

    // Private helper methods

    async fn initialize_models(&self) -> Result<()> {
        let mut models = self.forecasting_models.write().await;

        // Initialize GPU usage forecasting model
        models.insert("gpu_usage".to_string(), ForecastingModel {
            model_type: ModelType::LinearRegression,
            parameters: HashMap::new(),
            accuracy: 0.85,
            last_trained: Utc::now(),
        });

        // Initialize CPU usage forecasting model
        models.insert("cpu_usage".to_string(), ForecastingModel {
            model_type: ModelType::LinearRegression,
            parameters: HashMap::new(),
            accuracy: 0.82,
            last_trained: Utc::now(),
        });

        // Initialize memory usage forecasting model
        models.insert("memory_usage".to_string(), ForecastingModel {
            model_type: ModelType::LinearRegression,
            parameters: HashMap::new(),
            accuracy: 0.78,
            last_trained: Utc::now(),
        });

        info!("✅ Forecasting models initialized");
        Ok(())
    }

    async fn start_data_collection(&self) -> Result<()> {
        // In a real implementation, this would start background data collection
        info!("📊 Data collection started");
        Ok(())
    }

    async fn stop_data_collection(&self) -> Result<()> {
        info!("📊 Data collection stopped");
        Ok(())
    }

    async fn update_models(&self) -> Result<()> {
        let history = self.usage_history.read().await;

        if history.len() < 10 {
            return Ok(()); // Not enough data to update models
        }

        // Update each forecasting model
        let mut models = self.forecasting_models.write().await;

        for (model_name, model) in models.iter_mut() {
            match self.train_model(model_name, &history).await {
                Ok(new_accuracy) => {
                    model.accuracy = new_accuracy;
                    model.last_trained = Utc::now();
                }
                Err(e) => {
                    warn!("Failed to train model {}: {}", model_name, e);
                }
            }
        }

        Ok(())
    }

    async fn analyze_trends(&self) -> Result<UsageTrends> {
        let history = self.usage_history.read().await;

        if history.len() < 24 {
            return Ok(UsageTrends::default());
        }

        // Calculate trends over last 24 hours
        let recent_data: Vec<_> = history.iter().rev().take(24).collect();
        let older_data: Vec<_> = history.iter().rev().skip(24).take(24).collect();

        let gpu_trend = self.calculate_trend(&recent_data, &older_data, |d| d.gpu_utilization);
        let cpu_trend = self.calculate_trend(&recent_data, &older_data, |d| d.cpu_utilization);
        let memory_trend = self.calculate_trend(&recent_data, &older_data, |d| d.memory_utilization);

        Ok(UsageTrends {
            gpu_trend,
            cpu_trend,
            memory_trend,
            trend_strength: self.calculate_trend_strength(&[gpu_trend, cpu_trend, memory_trend]),
            confidence: 0.8,
        })
    }

    async fn generate_scaling_recommendations(&self, trends: &UsageTrends) -> Result<Option<ScalingRecommendation>> {
        let mut recommendations = Vec::new();

        // GPU scaling recommendation
        if trends.gpu_trend > 0.1 {
            recommendations.push(ScalingRecommendation {
                layer: "layer7".to_string(),
                action: ScalingAction::ScaleUp { factor: 1.2 },
                rationale: format!("GPU usage trending upward by {:.1}%", trends.gpu_trend * 100.0),
                expected_impact: Impact {
                    cost_impact: -15.0, // Negative means cost increase
                    performance_impact: 20.0,
                    reliability_impact: 5.0,
                },
                confidence: trends.confidence,
            });
        } else if trends.gpu_trend < -0.1 {
            recommendations.push(ScalingRecommendation {
                layer: "layer7".to_string(),
                action: ScalingAction::ScaleDown { factor: 0.8 },
                rationale: format!("GPU usage trending downward by {:.1}%", trends.gpu_trend.abs() * 100.0),
                expected_impact: Impact {
                    cost_impact: 10.0, // Positive means cost savings
                    performance_impact: -5.0,
                    reliability_impact: 0.0,
                },
                confidence: trends.confidence,
            });
        }

        Ok(recommendations.into_iter().next())
    }

    async fn generate_cost_optimizations(&self, trends: &UsageTrends) -> Result<Vec<CostOptimization>> {
        let mut optimizations = Vec::new();

        // Check for over-provisioning
        if trends.gpu_trend < -0.05 {
            optimizations.push(CostOptimization {
                optimization_type: OptimizationType::RightSizing,
                description: "GPU resources appear over-provisioned based on usage trends".to_string(),
                potential_savings: 25.0,
                effort: EffortLevel::Medium,
                priority: Priority::High,
            });
        }

        // Check for scheduling opportunities
        if self.detect_off_peak_usage().await? {
            optimizations.push(CostOptimization {
                optimization_type: OptimizationType::Scheduling,
                description: "Off-peak usage detected - consider scheduled scaling".to_string(),
                potential_savings: 15.0,
                effort: EffortLevel::Low,
                priority: Priority::Normal,
            });
        }

        Ok(optimizations)
    }

    async fn generate_performance_improvements(&self, trends: &UsageTrends) -> Result<Vec<PerformanceImprovement>> {
        let mut improvements = Vec::new();

        // Check for resource contention
        if trends.gpu_trend > 0.8 {
            improvements.push(PerformanceImprovement {
                area: "GPU Allocation".to_string(),
                description: "High GPU utilization detected - consider resource optimization".to_string(),
                expected_gain: 15.0,
                effort: EffortLevel::Medium,
            });
        }

        Ok(improvements)
    }

    async fn assess_risks(&self, trends: &UsageTrends) -> Result<Vec<RiskAssessment>> {
        let mut risks = Vec::new();

        // Resource exhaustion risk
        if trends.gpu_trend > 0.2 {
            risks.push(RiskAssessment {
                risk_type: RiskType::ResourceExhaustion,
                risk_level: RiskLevel::Medium,
                description: "GPU resources trending toward exhaustion".to_string(),
                mitigation: vec![
                    "Scale up GPU resources".to_string(),
                    "Implement resource quotas".to_string(),
                    "Optimize GPU utilization".to_string(),
                ],
            });
        }

        Ok(risks)
    }

    async fn generate_linear_forecast(&self, history: &[UsageDataPoint], hours_ahead: u32) -> Result<UsageForecast> {
        // Simple linear regression for forecasting
        let n = history.len() as f64;

        let gpu_sum: f64 = history.iter().map(|h| h.gpu_utilization).sum();
        let cpu_sum: f64 = history.iter().map(|h| h.cpu_utilization).sum();
        let memory_sum: f64 = history.iter().map(|h| h.memory_utilization).sum();

        let gpu_mean = gpu_sum / n;
        let cpu_mean = cpu_sum / n;
        let memory_mean = memory_sum / n;

        // Simple trend calculation (difference between recent and older averages)
        let recent_count = (history.len() / 2).max(1);
        let recent_gpu: f64 = history.iter().rev().take(recent_count).map(|h| h.gpu_utilization).sum::<f64>() / recent_count as f64;
        let older_gpu: f64 = history.iter().take(recent_count).map(|h| h.gpu_utilization).sum::<f64>() / recent_count as f64;

        let gpu_trend = recent_gpu - older_gpu;

        Ok(UsageForecast {
            forecast_points: (1..=hours_ahead).map(|hour| {
                let timestamp = Utc::now() + chrono::Duration::hours(hour as i64);
                UsageDataPoint {
                    timestamp,
                    gpu_utilization: (gpu_mean + gpu_trend * hour as f64).clamp(0.0, 1.0),
                    cpu_utilization: (cpu_mean + (cpu_mean * 0.1 * hour as f64)).clamp(0.0, 1.0),
                    memory_utilization: (memory_mean + (memory_mean * 0.05 * hour as f64)).clamp(0.0, 1.0),
                    cost_per_hour: 0.0, // Would be calculated based on resource usage
                }
            }).collect(),
            confidence: 0.7,
            model_used: "linear_regression".to_string(),
        })
    }

    async fn train_model(&self, model_name: &str, history: &[UsageDataPoint]) -> Result<f64> {
        // In a real implementation, this would train ML models
        // For now, return a simulated accuracy
        Ok(0.8 + (rand::random::<f64>() * 0.15)) // 0.8 to 0.95
    }

    fn calculate_trend<T, F>(&self, recent: &[T], older: &[T], extractor: F) -> f64
    where
        F: Fn(&T) -> f64,
    {
        if recent.is_empty() || older.is_empty() {
            return 0.0;
        }

        let recent_avg: f64 = recent.iter().map(&extractor).sum::<f64>() / recent.len() as f64;
        let older_avg: f64 = older.iter().map(&extractor).sum::<f64>() / older.len() as f64;

        recent_avg - older_avg
    }

    fn calculate_trend_strength(&self, trends: &[f64]) -> f64 {
        trends.iter().map(|t| t.abs()).sum::<f64>() / trends.len() as f64
    }

    async fn detect_off_peak_usage(&self) -> Result<bool> {
        let history = self.usage_history.read().await;

        if history.len() < 48 {
            return Ok(false); // Not enough data
        }

        // Check for usage patterns indicating off-peak periods
        let avg_usage: f64 = history.iter()
            .map(|h| (h.gpu_utilization + h.cpu_utilization + h.memory_utilization) / 3.0)
            .sum::<f64>() / history.len() as f64;

        let low_usage_count = history.iter()
            .filter(|h| (h.gpu_utilization + h.cpu_utilization + h.memory_utilization) / 3.0 < avg_usage * 0.5)
            .count();

        Ok(low_usage_count > history.len() / 4) // More than 25% low usage
    }
}

/// Usage data point for capacity planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageDataPoint {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// GPU utilization (0.0 to 1.0)
    pub gpu_utilization: f64,
    /// CPU utilization (0.0 to 1.0)
    pub cpu_utilization: f64,
    /// Memory utilization (0.0 to 1.0)
    pub memory_utilization: f64,
    /// Cost per hour at this point
    pub cost_per_hour: f64,
}

/// Usage trends analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrends {
    /// GPU usage trend
    pub gpu_trend: f64,
    /// CPU usage trend
    pub cpu_trend: f64,
    /// Memory usage trend
    pub memory_trend: f64,
    /// Overall trend strength
    pub trend_strength: f64,
    /// Confidence in trend analysis
    pub confidence: f64,
}

impl Default for UsageTrends {
    fn default() -> Self {
        Self {
            gpu_trend: 0.0,
            cpu_trend: 0.0,
            memory_trend: 0.0,
            trend_strength: 0.0,
            confidence: 0.0,
        }
    }
}

/// Usage forecast for future capacity planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageForecast {
    /// Forecasted data points
    pub forecast_points: Vec<UsageDataPoint>,
    /// Confidence in forecast (0.0 to 1.0)
    pub confidence: f64,
    /// Model used for forecasting
    pub model_used: String,
}

/// Forecasting model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastingModel {
    /// Model type
    pub model_type: ModelType,
    /// Model parameters
    pub parameters: HashMap<String, f64>,
    /// Model accuracy
    pub accuracy: f64,
    /// Last training timestamp
    pub last_trained: DateTime<Utc>,
}

/// Model types for forecasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    /// Linear regression model
    LinearRegression,
    /// ARIMA model
    Arima,
    /// LSTM neural network
    Lstm,
    /// Prophet model
    Prophet,
}

/// Planning policies for capacity management
#[derive(Debug, Clone)]
struct PlanningPolicies {
    /// Forecast horizon in hours
    forecast_horizon_hours: u32,
    /// Scaling sensitivity
    scaling_sensitivity: f64,
    /// Risk tolerance for recommendations
    risk_tolerance: f64,
}

impl Default for PlanningPolicies {
    fn default() -> Self {
        Self {
            forecast_horizon_hours: 168, // 1 week
            scaling_sensitivity: 0.1,    // 10% change threshold
            risk_tolerance: 0.2,         // 20% risk tolerance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capacity_planner_initialization() {
        let config = ResourceConfig::default();
        let planner = CapacityPlanner::new(config).await;

        assert!(planner.is_ok());
    }

    #[tokio::test]
    async fn test_usage_recording() {
        let config = ResourceConfig::default();
        let planner = CapacityPlanner::new(config).await.unwrap();

        let usage = UsageDataPoint {
            timestamp: Utc::now(),
            gpu_utilization: 0.7,
            cpu_utilization: 0.5,
            memory_utilization: 0.6,
            cost_per_hour: 2.5,
        };

        let result = planner.record_usage(usage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_recommendations_generation() {
        let config = ResourceConfig::default();
        let planner = CapacityPlanner::new(config).await.unwrap();

        // Add some usage data
        for i in 0..25 {
            let usage = UsageDataPoint {
                timestamp: Utc::now() - chrono::Duration::hours(i),
                gpu_utilization: 0.5 + (i as f64 * 0.01),
                cpu_utilization: 0.4,
                memory_utilization: 0.6,
                cost_per_hour: 2.0,
            };
            planner.record_usage(usage).await.unwrap();
        }

        let recommendations = planner.get_recommendations().await;
        assert!(recommendations.is_ok());
    }
}