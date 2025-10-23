//! # Chimera Layer 5 - Refinement
//!
//! Layer 5 serves as the optimization and continuous improvement engine for Project Chimera.
//! It consumes KPI data from Layer 4 to drive autonomous system enhancement through
//! machine learning and pattern recognition.

pub mod types;
pub mod kpi_ingestion;
pub mod optimization;
pub mod pattern_recognition;
pub mod feedback_loop;
pub mod ab_testing;
pub mod integration;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use types::*;
pub use kpi_ingestion::*;
pub use optimization::*;
pub use pattern_recognition::*;
pub use feedback_loop::*;
pub use ab_testing::*;
pub use integration::*;

/// Initialize the Layer 5 system with configuration
pub async fn init_layer5(config: Layer5Config) -> Result<Layer5System, Layer5Error> {
    // Initialize logging
    tracing_subscriber::init();

    // Initialize KPI ingestion
    let ingestion_service = KpiIngestionService::new(config.ingestion_config).await?;

    // Initialize optimization framework
    let optimizer = OptimizationFramework::new(config.optimization_config).await?;

    // Initialize pattern recognition
    let pattern_analyzer = PatternRecognitionEngine::new(config.pattern_config).await?;

    // Initialize feedback loop
    let feedback_loop = FeedbackLoopSystem::new(config.feedback_config).await?;

    // Initialize A/B testing
    let ab_testing = ABTestingFramework::new(config.ab_config).await?;

    // Initialize integrations
    let integrations = IntegrationManager::new(config.integration_config).await?;

    Ok(Layer5System {
        ingestion_service,
        optimizer,
        pattern_analyzer,
        feedback_loop,
        ab_testing,
        integrations,
    })
}