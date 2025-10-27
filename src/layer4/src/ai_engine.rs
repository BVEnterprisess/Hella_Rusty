//! The AI engine for loading and managing models.
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use crate::model_loader::ModelLoader;
use crate::model_types::LoadedModel;

/// The AI engine for loading and managing models.
pub struct AIEngine {
    model_loader: ModelLoader,
    loaded_models: HashMap<String, LoadedModel>,
}

impl AIEngine {
    /// Creates a new AI engine.
    pub fn new() -> Result<Self> {
        Ok(Self {
            model_loader: ModelLoader::new()?,
            loaded_models: HashMap::new(),
        })
    }

    /// Loads a model.
    pub async fn load_model(&mut self, model_path: &Path) -> Result<String> {
        let model = self.model_loader.load_safetensors(model_path).await?;
        let model_id = uuid::Uuid::new_v4().to_string();
        tracing::info!(
            "Loaded model {} with {} parameters",
            model_id,
            model.num_parameters()
        );
        self.loaded_models.insert(model_id.clone(), model);
        Ok(model_id)
    }

    /// Returns a loaded model.
    pub fn get_model(&self, model_id: &str) -> Option<&LoadedModel> {
        self.loaded_models.get(model_id)
    }
}
