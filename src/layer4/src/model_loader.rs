//! The AI model loader.
use anyhow::{Context, Result};
use candle_core::{Device, Tensor, DType};
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};
use super::model_types::*;

/// The AI model loader.
pub struct ModelLoader {
    /// The device the model is loaded on.
    pub device: Device,
    cache: HashMap<String, LoadedModel>,
    max_cache_size: usize,
}

impl ModelLoader {
    /// Creates a new model loader.
    pub fn new() -> Result<Self> {
        let device = if candle_core::utils::cuda_is_available() {
            info!("🎮 CUDA available, using GPU");
            Device::new_cuda(0)?
        } else {
            info!("💻 CUDA not available, using CPU");
            Device::Cpu
        };
        Ok(Self {
            device,
            cache: HashMap::new(),
            max_cache_size: 3,
        })
    }

    /// Creates a new model loader with a specific device.
    pub fn with_device(device: Device) -> Self {
        Self {
            device,
            cache: HashMap::new(),
            max_cache_size: 3,
        }
    }

    /// Loads a safetensors model.
    pub async fn load_safetensors(&mut self, path: &Path) -> Result<LoadedModel> {
        let path_str = path.to_string_lossy().to_string();
        // Check cache first
        if let Some(cached) = self.cache.get(&path_str) {
            info!("📦 Loading model from cache: {}", path_str);
            return Ok(cached.clone());
        }

        info!("🔄 Loading model from disk: {}", path_str);
        // Read safetensors file
        let buffer = fs::read(path).await
            .context(format!("Failed to read safetensors file: {}", path_str))?;

        // Parse safetensors
        let safetensors = SafeTensors::deserialize(&buffer)
            .context("Failed to parse safetensors format")?;

        // Load all tensors
        let mut weights = HashMap::new();
        let mut total_params = 0usize;
        for (name, tensor_view) in safetensors.tensors() {
            let shape: Vec<usize> = tensor_view.shape().to_vec();
            let dtype = self.convert_dtype(tensor_view.dtype());

            // Get raw data
            let data = tensor_view.data();

            // Create Candle tensor
            let tensor = match dtype {
                DType::F32 => {
                    let floats: Vec<f32> = data
                        .chunks_exact(4)
                        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect();
                    Tensor::from_vec(floats, shape.as_slice(), &self.device)?
                }
                DType::F16 => {
                    // Handle F16 if needed
                    warn!("F16 dtype detected, converting to F32");
                    let floats: Vec<f32> = data
                        .chunks_exact(2)
                        .map(|chunk| {
                            half::f16::from_le_bytes([chunk[0], chunk[1]]).to_f32()
                        })
                        .collect();
                    Tensor::from_vec(floats, shape.as_slice(), &self.device)?
                }
                _ => {
                    warn!("Unsupported dtype for tensor {}, skipping", name);
                    continue;
                }
            };
            let param_count: usize = shape.iter().product();
            total_params += param_count;
            weights.insert(name.to_string(), tensor);
        }

        // Detect architecture from tensor names
        let architecture = self.detect_architecture(&weights);

        // Extract metadata
        let metadata = self.extract_metadata(&weights, architecture.clone(), total_params)?;

        let model = LoadedModel {
            metadata,
            device: self.device.clone(),
            weights,
            path: path_str.clone(),
        };

        // Cache the model (with LRU eviction if needed)
        if self.cache.len() >= self.max_cache_size {
            // Simple FIFO eviction (you could make this LRU)
            if let Some(first_key) = self.cache.keys().next().cloned() {
                info!("🗑️ Evicting cached model: {}", first_key);
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(path_str.clone(), model.clone());
        info!("✅ Model loaded successfully: {} parameters", total_params);
        Ok(model)
    }

    fn convert_dtype(&self, dtype: safetensors::Dtype) -> DType {
        match dtype {
            safetensors::Dtype::F32 => DType::F32,
            safetensors::Dtype::F16 => DType::F16,
            safetensors::Dtype::BF16 => DType::BF16,
            _ => DType::F32, // Default fallback
        }
    }

    fn detect_architecture(&self, weights: &HashMap<String, Tensor>) -> ModelArchitecture {
        let keys: Vec<&String> = weights.keys().collect();

        // BERT detection: "bert.encoder", "bert.embeddings"
        if keys.iter().any(|k| k.contains("bert.encoder")) {
            return ModelArchitecture::Bert;
        }
        // GPT-2 detection: "transformer.h", "transformer.wte"
        if keys.iter().any(|k| k.contains("transformer.h")) {
            return ModelArchitecture::Gpt2;
        }
        // Llama detection: "model.layers", "lm_head"
        if keys.iter().any(|k| k.contains("model.layers")) {
            return ModelArchitecture::Llama;
        }
        ModelArchitecture::Generic
    }

    fn extract_metadata(
        &self,
        weights: &HashMap<String, Tensor>,
        architecture: ModelArchitecture,
        total_params: usize,
    ) -> Result<ModelMetadata> {
        // Count layers
        let num_layers = weights
            .keys()
            .filter(|k| k.contains(".layer") || k.contains(".h.") || k.contains(".layers."))
            .map(|k| {
                // Extract layer number from key
                k.split('.')
                    .find_map(|part| part.parse::<usize>().ok())
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
            + 1;

        // Detect hidden size from embedding layer
        let hidden_size = weights
            .iter()
            .find(|(k, _)| k.contains("embed") || k.contains("wte"))
            .and_then(|(_, tensor)| tensor.dims().last().copied())
            .unwrap_or(768); // Default to BERT-base size

        // Detect vocab size
        let vocab_size = weights
            .iter()
            .find(|(k, _)| k.contains("embed") || k.contains("wte"))
            .and_then(|(_, tensor)| tensor.dims().first().copied())
            .unwrap_or(30522); // Default to BERT-base vocab

        Ok(ModelMetadata {
            architecture,
            num_parameters: total_params,
            num_layers,
            hidden_size,
            vocab_size,
            config: HashMap::new(),
        })
    }

    /// Clears the model cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Returns a cached model.
    pub fn get_cached_model(&self, path: &str) -> Option<&LoadedModel> {
        self.cache.get(path)
    }
}

impl Default for ModelLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default ModelLoader")
    }
}
