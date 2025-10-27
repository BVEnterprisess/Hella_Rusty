//! AI model loading and management types.
use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The architecture of a loaded model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelArchitecture {
    /// BERT model.
    Bert,
    /// GPT-2 model.
    Gpt2,
    /// Llama model.
    Llama,
    /// A generic model.
    Generic,
}

/// Metadata for a loaded model.
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// The architecture of the model.
    pub architecture: ModelArchitecture,
    /// The number of parameters in the model.
    pub num_parameters: usize,
    /// The number of layers in the model.
    pub num_layers: usize,
    /// The hidden size of the model.
    pub hidden_size: usize,
    /// The vocabulary size of the model.
    pub vocab_size: usize,
    /// The model's configuration.
    pub config: HashMap<String, serde_json::Value>,
}

/// A loaded model.
#[derive(Clone, Debug)]
pub struct LoadedModel {
    /// The model's metadata.
    pub metadata: ModelMetadata,
    /// The device the model is loaded on.
    pub device: Device,
    /// The model's weights.
    pub weights: HashMap<String, Tensor>,
    /// The path to the model file.
    pub path: String,
}

impl LoadedModel {
    /// Returns the number of parameters in the model.
    pub fn num_parameters(&self) -> usize {
        self.metadata.num_parameters
    }

    /// Returns the architecture of the model.
    pub fn architecture(&self) -> &ModelArchitecture {
        &self.metadata.architecture
    }
}
