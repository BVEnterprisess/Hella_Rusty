use chimera_layer4::model_loader::ModelLoader;
use std::path::PathBuf;

#[tokio::test]
async fn test_load_tiny_bert() {
    // Download a tiny BERT model for testing
    // You can use: https://huggingface.co/google/bert_uncased_L-2_H-128_A-2
    // Or create a minimal safetensors file for testing
    let model_path = PathBuf::from("tests/fixtures/tiny_bert.safetensors");
    if !model_path.exists() {
        eprintln!("⚠️ Test model not found at {:?}", model_path);
        eprintln!(" Download from: https://huggingface.co/google/bert_uncased_L-2_H-128_A-2");
        return; // Skip test if model not available
    }
    let mut loader = ModelLoader::new().unwrap();
    let model = loader.load_safetensors(&model_path).await;
    assert!(model.is_ok(), "Model loading failed: {:?}", model.err());
    let model = model.unwrap();

    // Verify metadata
    assert!(model.num_parameters() > 1000, "Model should have parameters");
    assert!(model.metadata.num_layers > 0, "Model should have layers");
    assert!(model.metadata.hidden_size > 0, "Model should have hidden size");
}

#[tokio::test]
async fn test_model_caching() {
    let model_path = PathBuf::from("tests/fixtures/tiny_bert.safetensors");
    if !model_path.exists() {
        return; // Skip if model not available
    }
    let mut loader = ModelLoader::new().unwrap();
    // Load model first time
    let start = std::time::Instant::now();
    loader.load_safetensors(&model_path).await.unwrap();
    let first_load_time = start.elapsed();

    // Load same model again (should be cached)
    let start = std::time::Instant::now();
    loader.load_safetensors(&model_path).await.unwrap();
    let cached_load_time = start.elapsed();

    // Cached load should be much faster
    assert!(
        cached_load_time < first_load_time / 2,
        "Cached load should be faster: {:?} vs {:?}",
        cached_load_time,
        first_load_time
    );
}

#[tokio::test]
async fn test_invalid_file() {
    let mut loader = ModelLoader::new().unwrap();
    let result = loader
        .load_safetensors(&PathBuf::from("/nonexistent/model.safetensors"))
        .await;
    assert!(result.is_err(), "Should fail on nonexistent file");
}

#[tokio::test]
async fn test_device_selection() {
    let loader = ModelLoader::new().unwrap();
    // Should successfully create loader regardless of CUDA availability
    assert!(loader.device.is_cpu() || loader.device.is_cuda());
}
