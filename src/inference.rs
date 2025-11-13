//! Inference engine for AI model processing
//!
//! Provides a lightweight mock inference implementation that keeps
//! the rest of the platform exercising request/response flows without
//! requiring heavyweight ML dependencies.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Default)]
pub struct InferenceEngine {
    model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub repetition_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_used: usize,
    pub processing_time_ms: u64,
    pub confidence: f32,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self { model_name: None }
    }

    pub fn load_model<P: AsRef<Path>>(
        &mut self,
        model_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let model_name = model_path
            .as_ref()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| model_path.as_ref().to_string_lossy().into_owned());

        self.model_name = Some(model_name);
        Ok(())
    }

    pub async fn generate(
        &self,
        request: InferenceRequest,
    ) -> Result<InferenceResponse, Box<dyn std::error::Error>> {
        let model_name = self
            .model_name
            .as_ref()
            .ok_or_else(|| "model not loaded".to_string())?;

        let start = Instant::now();
        let prompt_token_count = request.prompt.split_whitespace().count().max(1);
        let simulated_output_tokens = request.max_tokens.min(128);

        let response_text = format!(
            "[{model}] Responding to: {prompt}",
            model = model_name,
            prompt = request.prompt
        );

        let processing_time_ms = start.elapsed().as_millis() as u64;
        let confidence = estimate_confidence(&request);

        Ok(InferenceResponse {
            text: response_text,
            tokens_used: prompt_token_count + simulated_output_tokens,
            processing_time_ms,
            confidence,
        })
    }
}

fn estimate_confidence(request: &InferenceRequest) -> f32 {
    let temperature_component = (1.0 - request.temperature.clamp(0.0, 1.5) / 1.5).max(0.2);
    let diversity_component = request.top_p.clamp(0.1, 1.0);
    ((temperature_component + diversity_component) / 2.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inference_engine_generates_response() {
        let mut engine = InferenceEngine::new();
        engine.load_model("models/test-model").unwrap();

        let request = InferenceRequest {
            prompt: "Hello, world!".to_string(),
            max_tokens: 32,
            temperature: 0.7,
            top_p: 0.9,
            repetition_penalty: 1.0,
        };

        let response = engine.generate(request).await.unwrap();

        assert!(response.text.contains("Hello, world"));
        assert!(response.tokens_used > 0);
        assert!(response.confidence >= 0.0 && response.confidence <= 1.0);
    }
}
