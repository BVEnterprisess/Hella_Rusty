use chimera_core::*;
use clap::Parser;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tracing::{info, error};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "configs/chimera.toml")]
    config: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Agent name
    #[arg(short, long)]
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting Chimera Agent: {}", args.name);

    // Load configuration
    let config_content = tokio::fs::read_to_string(&args.config).await?;
    let config: ChimeraConfig = toml::from_str(&config_content)?;

    // Initialize platform
    let platform = init_platform(config).await?;

    // Start HTTP server
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/predict", axum::routing::post(predict))
        .route("/status", axum::routing::get(agent_status))
        .layer(tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn(rate_limit_middleware))
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Agent {} listening on {}", args.name, addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn predict(
    axum::extract::State(state): axum::extract::State<Platform>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    info!("Received prediction request: {:?}", payload);

    // TODO: Implement actual inference logic
    let response = serde_json::json!({
        "result": "Prediction completed",
        "confidence": 0.95,
        "processing_time_ms": 150
    });

    axum::Json(response)
}

async fn agent_status(
    axum::extract::State(state): axum::extract::State<Platform>,
) -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "name": "chimera-agent",
        "status": "active",
        "uptime_seconds": 3600,
        "requests_processed": 150,
        "average_response_time_ms": 145
    }))
}

async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<Platform>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl axum::response::IntoResponse {
    // Extract client IP (simplified for example)
    let client_ip = std::net::IpAddr::from([127, 0, 0, 1]);

    // Check rate limit
    if let Err(e) = state.rate_limiter.check_rate_limit(client_ip, request.uri().path()) {
        error!("Rate limit exceeded for {}: {:?}", client_ip, e);
        return axum::response::Response::builder()
            .status(429)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error": "Rate limit exceeded"}"#))
            .unwrap()
            .into_response();
    }

    next.run(request).await
}