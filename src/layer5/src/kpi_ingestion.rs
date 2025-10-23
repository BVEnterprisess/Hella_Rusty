//! KPI Ingestion & Processing Engine for Layer 5

use crate::types::*;
use async_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{info, error, warn};

/// KPI Ingestion Service
pub struct KpiIngestionService {
    buffer: Arc<Mutex<Vec<KpiBatch>>>,
    processor: Arc<KpiProcessor>,
    config: IngestionConfig,
    sender: Sender<KpiBatch>,
    receiver: Receiver<KpiBatch>,
}

impl KpiIngestionService {
    /// Create a new KPI ingestion service
    pub async fn new(config: IngestionConfig) -> Result<Self, IngestionError> {
        let (sender, receiver) = async_channel::unbounded();

        let processor = Arc::new(KpiProcessor::new().await?);

        Ok(Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            processor,
            config,
            sender,
            receiver,
        })
    }

    /// Ingest a KPI batch
    pub async fn ingest_kpi(&self, kpi: KpiBatch) -> Result<(), IngestionError> {
        // Validate KPI data
        self.validate_kpi(&kpi)?;

        // Add to buffer
        self.add_to_buffer(kpi).await?;

        // Trigger processing if buffer is full or timeout reached
        if self.should_process().await {
            self.process_buffer().await?;
        }

        Ok(())
    }

    /// Validate KPI data
    fn validate_kpi(&self, kpi: &KpiBatch) -> Result<(), IngestionError> {
        if kpi.metrics.is_empty() {
            return Err(IngestionError::Validation("Metrics cannot be empty".to_string()));
        }

        for (key, value) in &kpi.metrics {
            if !value.is_finite() {
                return Err(IngestionError::Validation(format!("Invalid metric value for {}: {}", key, value)));
            }
        }

        Ok(())
    }

    /// Add KPI to buffer
    async fn add_to_buffer(&self, kpi: KpiBatch) -> Result<(), IngestionError> {
        let mut buffer = self.buffer.lock().await;
        if buffer.len() >= self.config.buffer_size {
            return Err(IngestionError::BufferOverflow);
        }
        buffer.push(kpi);
        Ok(())
    }

    /// Check if buffer should be processed
    async fn should_process(&self) -> bool {
        let buffer = self.buffer.lock().await;
        buffer.len() >= self.config.buffer_size
    }

    /// Process the buffer
    async fn process_buffer(&self) -> Result<(), IngestionError> {
        let mut buffer = self.buffer.lock().await;
        if buffer.is_empty() {
            return Ok(());
        }

        let batch = std::mem::take(&mut *buffer);
        drop(buffer);

        // Process the batch
        match timeout(Duration::from_millis(self.config.batch_timeout_ms), self.processor.process_batch(batch)).await {
            Ok(Ok(())) => {
                info!("Successfully processed KPI batch");
            }
            Ok(Err(e)) => {
                error!("Failed to process KPI batch: {:?}", e);
                return Err(e);
            }
            Err(_) => {
                warn!("KPI batch processing timed out");
                return Err(IngestionError::Timeout);
            }
        }

        Ok(())
    }

    /// Start the ingestion service
    pub async fn start(&self) -> Result<(), IngestionError> {
        let receiver = self.receiver.clone();
        let processor = self.processor.clone();

        tokio::spawn(async move {
            while let Ok(kpi) = receiver.recv().await {
                if let Err(e) = processor.process_single(kpi).await {
                    error!("Failed to process single KPI: {:?}", e);
                }
            }
        });

        Ok(())
    }
}

/// KPI Processor
pub struct KpiProcessor {
    // In a real implementation, this would connect to databases, etc.
}

impl KpiProcessor {
    pub async fn new() -> Result<Self, IngestionError> {
        // Initialize connections, etc.
        Ok(Self {})
    }

    /// Process a batch of KPIs
    pub async fn process_batch(&self, batch: Vec<KpiBatch>) -> Result<(), IngestionError> {
        for kpi in batch {
            self.process_single(kpi).await?;
        }
        Ok(())
    }

    /// Process a single KPI
    pub async fn process_single(&self, kpi: KpiBatch) -> Result<(), IngestionError> {
        // In a real implementation:
        // - Store in time-series database
        // - Trigger pattern recognition
        // - Update metrics

        info!("Processing KPI for agent {}: {:?}", kpi.agent_id, kpi.metrics);

        // Simulate processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(())
    }
}