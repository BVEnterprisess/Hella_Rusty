//! Simplified model training utilities.
//!
//! The original project described full LoRA training built on top of
//! Candle.  Maintaining those heavy dependencies without a working
//! implementation introduced a large amount of dead code and build
//! failures.  This module keeps a lightweight, testable façade that
//! exercises the surrounding orchestration while clearly signalling
//! that the behaviour is simulated.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub base_model: String,
    pub dataset_path: PathBuf,
    pub output_dir: PathBuf,
    pub learning_rate: f32,
    pub num_epochs: usize,
    pub batch_size: usize,
    pub save_steps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSummary {
    pub total_epochs: usize,
    pub total_steps: usize,
    pub output_adapter: PathBuf,
    pub metrics: Vec<EpochMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochMetrics {
    pub epoch: usize,
    pub average_loss: f32,
    pub samples_processed: usize,
}

pub struct LoRATrainer {
    config: TrainingConfig,
}

impl LoRATrainer {
    pub fn new(config: TrainingConfig) -> Self {
        Self { config }
    }

    pub async fn train(&self) -> Result<TrainingSummary, Box<dyn std::error::Error>> {
        validate_paths(&self.config)?;

        let mut metrics = Vec::new();
        let mut total_steps = 0usize;

        for epoch in 0..self.config.num_epochs {
            let epoch_start = Instant::now();
            // Simulate a handful of gradient steps.
            for step in 0..self.config.save_steps.max(1) {
                sleep(Duration::from_millis(25)).await;
                total_steps += 1;

                if step > 0 && step % self.config.save_steps == 0 {
                    save_checkpoint(&self.config.output_dir, epoch, step).await?;
                }
            }

            let elapsed = epoch_start.elapsed().as_secs_f32();
            let average_loss = (1.0 / (epoch as f32 + 1.0)).clamp(0.01, 1.0);
            metrics.push(EpochMetrics {
                epoch,
                average_loss,
                samples_processed: self.config.batch_size * self.config.save_steps.max(1),
            });

            tracing::info!(
                "Completed epoch {} in {:.2}s (avg loss {:.3})",
                epoch + 1,
                elapsed,
                average_loss
            );
        }

        let output_adapter = self.config.output_dir.join("chimera_adapter.safetensors");
        tokio::fs::create_dir_all(&self.config.output_dir).await?;
        tokio::fs::write(&output_adapter, b"simulated adapter").await?;

        Ok(TrainingSummary {
            total_epochs: self.config.num_epochs,
            total_steps,
            output_adapter,
            metrics,
        })
    }
}

fn validate_paths(config: &TrainingConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.base_model.trim().is_empty() {
        return Err("base model must be provided".into());
    }

    if config.dataset_path.as_os_str().is_empty() {
        return Err("dataset path must be provided".into());
    }

    if config.output_dir.as_os_str().is_empty() {
        return Err("output directory must be provided".into());
    }

    Ok(())
}

async fn save_checkpoint(
    output_dir: &PathBuf,
    epoch: usize,
    step: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint_path = output_dir.join(format!("checkpoint_epoch{}_step{}.ckpt", epoch, step));
    tokio::fs::write(checkpoint_path, b"checkpoint").await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trainer_produces_summary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TrainingConfig {
            base_model: "mistral".into(),
            dataset_path: temp_dir.path().join("dataset.jsonl"),
            output_dir: temp_dir.path().join("output"),
            learning_rate: 1e-4,
            num_epochs: 2,
            batch_size: 8,
            save_steps: 4,
        };

        let trainer = LoRATrainer::new(config.clone());
        let summary = trainer.train().await.unwrap();

        assert_eq!(summary.total_epochs, config.num_epochs);
        assert!(!summary.metrics.is_empty());
        assert!(summary.output_adapter.exists());
    }
}
