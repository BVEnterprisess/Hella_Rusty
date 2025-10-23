//! Pattern Recognition & Trend Analysis for Layer 5

use crate::types::*;
use ndarray::{Array1, Array2};
use ndarray_stats::CorrelationExt;
use statrs::distribution::{ContinuousCDF, Normal};
use std::collections::HashMap;
use tracing::{info, warn};

/// Pattern Recognition Engine
pub struct PatternRecognitionEngine {
    correlation_analyzer: CorrelationAnalyzer,
    behavior_clusterer: BehaviorClusterer,
    trend_analyzer: TrendAnalyzer,
}

impl PatternRecognitionEngine {
    pub async fn new(config: PatternConfig) -> Result<Self, PatternError> {
        Ok(Self {
            correlation_analyzer: CorrelationAnalyzer::new(config.correlation_threshold),
            behavior_clusterer: BehaviorClusterer::new(),
            trend_analyzer: TrendAnalyzer::new(config.trend_threshold, config.anomaly_threshold),
        })
    }

    /// Analyze patterns in KPI data
    pub async fn analyze_patterns(&self, kpis: Vec<KpiBatch>) -> Result<Vec<PatternResult>, PatternError> {
        let mut results = Vec::new();

        // Correlation analysis
        if let Ok(correlation_result) = self.correlation_analyzer.analyze(&kpis).await {
            results.push(correlation_result);
        }

        // Clustering analysis
        if let Ok(cluster_result) = self.behavior_clusterer.cluster(&kpis).await {
            results.push(cluster_result);
        }

        // Trend analysis
        if let Ok(trend_result) = self.trend_analyzer.analyze_trends(&kpis).await {
            results.push(trend_result);
        }

        Ok(results)
    }
}

/// Correlation Analyzer
pub struct CorrelationAnalyzer {
    correlation_matrix: HashMap<(String, String), f64>,
    significance_threshold: f64,
}

impl CorrelationAnalyzer {
    pub fn new(threshold: f64) -> Self {
        Self {
            correlation_matrix: HashMap::new(),
            significance_threshold: threshold,
        }
    }

    pub async fn analyze(&self, kpis: &[KpiBatch]) -> Result<PatternResult, PatternError> {
        if kpis.len() < 2 {
            return Err(PatternError::InsufficientData);
        }

        // Extract metrics
        let mut metrics: HashMap<String, Vec<f64>> = HashMap::new();
        for kpi in kpis {
            for (key, value) in &kpi.metrics {
                metrics.entry(key.clone()).or_insert_with(Vec::new).push(*value);
            }
        }

        // Calculate correlations
        let mut correlations = HashMap::new();
        let keys: Vec<String> = metrics.keys().cloned().collect();
        for i in 0..keys.len() {
            for j in i+1..keys.len() {
                let key1 = &keys[i];
                let key2 = &keys[j];
                let data1 = metrics.get(key1).unwrap();
                let data2 = metrics.get(key2).unwrap();

                if let Some(corr) = self.calculate_correlation(data1, data2) {
                    if corr.abs() > self.significance_threshold {
                        correlations.insert(format!("{} vs {}", key1, key2), corr);
                    }
                }
            }
        }

        Ok(PatternResult {
            pattern_type: PatternType::Correlation,
            confidence: 0.9,
            metrics: correlations,
            timestamp: chrono::Utc::now(),
        })
    }

    fn calculate_correlation(&self, data1: &[f64], data2: &[f64]) -> Option<f64> {
        if data1.len() != data2.len() || data1.len() < 3 {
            return None;
        }

        let arr1 = Array1::from_vec(data1.to_vec());
        let arr2 = Array1::from_vec(data2.to_vec());
        arr1.correlation(&arr2).ok()
    }
}

/// Behavior Clusterer
pub struct BehaviorClusterer {
    clusters: Vec<AgentCluster>,
    distance_metric: DistanceMetric,
}

struct AgentCluster {
    center: HashMap<String, f64>,
    agents: Vec<AgentId>,
}

#[derive(Clone)]
enum DistanceMetric {
    Euclidean,
    Manhattan,
}

impl BehaviorClusterer {
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            distance_metric: DistanceMetric::Euclidean,
        }
    }

    pub async fn cluster(&mut self, kpis: &[KpiBatch]) -> Result<PatternResult, PatternError> {
        if kpis.is_empty() {
            return Err(PatternError::InsufficientData);
        }

        // Simple k-means clustering (simplified)
        let k = 3; // Number of clusters
        self.initialize_clusters(kpis, k);
        self.perform_clustering(kpis, 10); // 10 iterations

        let mut metrics = HashMap::new();
        metrics.insert("num_clusters".to_string(), self.clusters.len() as f64);
        metrics.insert("total_agents".to_string(), kpis.len() as f64);

        Ok(PatternResult {
            pattern_type: PatternType::Seasonality, // Placeholder
            confidence: 0.8,
            metrics,
            timestamp: chrono::Utc::now(),
        })
    }

    fn initialize_clusters(&mut self, kpis: &[KpiBatch], k: usize) {
        self.clusters.clear();
        // Random initialization (simplified)
        for i in 0..k {
            let center = if let Some(kpi) = kpis.get(i) {
                kpi.metrics.clone()
            } else {
                HashMap::new()
            };
            self.clusters.push(AgentCluster {
                center,
                agents: Vec::new(),
            });
        }
    }

    fn perform_clustering(&mut self, kpis: &[KpiBatch], iterations: usize) {
        for _ in 0..iterations {
            // Assign agents to clusters
            for kpi in kpis {
                let closest_cluster = self.find_closest_cluster(&kpi.metrics);
                if let Some(cluster) = self.clusters.get_mut(closest_cluster) {
                    cluster.agents.push(kpi.agent_id);
                }
            }

            // Update cluster centers
            self.update_centers(kpis);
        }
    }

    fn find_closest_cluster(&self, metrics: &HashMap<String, f64>) -> usize {
        let mut min_distance = f64::INFINITY;
        let mut closest = 0;

        for (i, cluster) in self.clusters.iter().enumerate() {
            let distance = self.calculate_distance(metrics, &cluster.center);
            if distance < min_distance {
                min_distance = distance;
                closest = i;
            }
        }

        closest
    }

    fn calculate_distance(&self, a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
        match self.distance_metric {
            DistanceMetric::Euclidean => {
                a.values().zip(b.values()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
            }
            DistanceMetric::Manhattan => {
                a.values().zip(b.values()).map(|(x, y)| (x - y).abs()).sum()
            }
        }
    }

    fn update_centers(&mut self, kpis: &[KpiBatch]) {
        for cluster in &mut self.clusters {
            let mut new_center = HashMap::new();
            let num_agents = cluster.agents.len() as f64;

            if num_agents > 0.0 {
                for agent_id in &cluster.agents {
                    if let Some(kpi) = kpis.iter().find(|k| k.agent_id == *agent_id) {
                        for (key, value) in &kpi.metrics {
                            *new_center.entry(key.clone()).or_insert(0.0) += value / num_agents;
                        }
                    }
                }
            }

            cluster.center = new_center;
            cluster.agents.clear();
        }
    }
}

/// Trend Analyzer
pub struct TrendAnalyzer {
    trend_models: HashMap<String, TrendModel>,
    prediction_horizon: chrono::Duration,
    trend_threshold: f64,
    anomaly_threshold: f64,
}

struct TrendModel {
    slope: f64,
    intercept: f64,
    r_squared: f64,
}

impl TrendAnalyzer {
    pub fn new(trend_threshold: f64, anomaly_threshold: f64) -> Self {
        Self {
            trend_models: HashMap::new(),
            prediction_horizon: chrono::Duration::days(7),
            trend_threshold,
            anomaly_threshold,
        }
    }

    pub async fn analyze_trends(&mut self, kpis: &[KpiBatch]) -> Result<PatternResult, PatternError> {
        if kpis.len() < 5 {
            return Err(PatternError::InsufficientData);
        }

        let mut trend_metrics = HashMap::new();

        // Group by metric
        let mut metric_data: HashMap<String, Vec<(f64, f64)>> = HashMap::new(); // (timestamp, value)
        for kpi in kpis {
            let timestamp = kpi.timestamp.timestamp() as f64;
            for (key, value) in &kpi.metrics {
                metric_data.entry(key.clone()).or_insert_with(Vec::new).push((timestamp, *value));
            }
        }

        for (metric, data) in metric_data {
            if let Some(trend) = self.calculate_trend(&data) {
                if trend.r_squared.abs() > self.trend_threshold {
                    trend_metrics.insert(format!("{}_slope", metric), trend.slope);
                    trend_metrics.insert(format!("{}_r2", metric), trend.r_squared);
                }
            }

            // Anomaly detection
            if let Some(anomalies) = self.detect_anomalies(&data) {
                trend_metrics.insert(format!("{}_anomalies", metric), anomalies as f64);
            }
        }

        Ok(PatternResult {
            pattern_type: PatternType::Trend,
            confidence: 0.85,
            metrics: trend_metrics,
            timestamp: chrono::Utc::now(),
        })
    }

    fn calculate_trend(&self, data: &[(f64, f64)]) -> Option<TrendModel> {
        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let y_mean = sum_y / n;
        let ss_tot: f64 = data.iter().map(|(_, y)| (y - y_mean).powi(2)).sum();
        let ss_res: f64 = data.iter().map(|(x, y)| (y - (slope * x + intercept)).powi(2)).sum();
        let r_squared = if ss_tot != 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

        Some(TrendModel { slope, intercept, r_squared })
    }

    fn detect_anomalies(&self, data: &[(f64, f64)]) -> Option<usize> {
        if data.len() < 10 {
            return None;
        }

        // Simple Z-score based anomaly detection
        let values: Vec<f64> = data.iter().map(|(_, y)| *y).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|y| (y - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let anomaly_count = values.iter().filter(|&&y| (y - mean).abs() / std_dev > self.anomaly_threshold).count();

        if anomaly_count > 0 {
            Some(anomaly_count)
        } else {
            None
        }
    }
}