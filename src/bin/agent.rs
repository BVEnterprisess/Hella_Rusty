use axum::response::IntoResponse;
use chimera_core::*;
use clap::Parser;
use dotenvy::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

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
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Chimera Agent: {}", args.name);

    // Load configuration
    let config_content = tokio::fs::read_to_string(&args.config).await?;
    let config: ChimeraConfig = toml::from_str(&config_content)?;

    // Initialize platform
    let platform = Arc::new(init_platform(config).await?);

    // Start HTTP server
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/predict", axum::routing::post(predict))
        .route("/status", axum::routing::get(agent_status))
        .with_state(platform.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Agent {} listening on {}", args.name, addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": utils::timestamp_now(),
    }))
}

async fn predict(
    axum::extract::State(platform): axum::extract::State<Arc<Platform>>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> axum::response::Response {
    info!("Received prediction request: {:?}", payload);

    let client_ip = std::net::IpAddr::from([127, 0, 0, 1]);
    if let Err(e) = platform
        .rate_limiter
        .check_rate_limit(client_ip, "/predict")
    {
        error!("Rate limit exceeded for {}: {:?}", client_ip, e);
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({ "error": "Rate limit exceeded" })),
        )
            .into_response();
    }

    if let Err(errors) = utils::validate_request_payload(&payload) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(errors),
        )
            .into_response();
    }

    platform
        .audit_logger
        .log_api_access(None, "/predict", "POST", 200, None)
        .ok();

    let response = serde_json::json!({
        "result": "Prediction completed",
        "confidence": 0.95,
        "processing_time_ms": 150
    });

    axum::Json(response).into_response()
}

async fn agent_status(
    axum::extract::State(_platform): axum::extract::State<Arc<Platform>>,
) -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "name": "chimera-agent",
        "status": "active",
        "uptime_seconds": 3600,
        "requests_processed": 150,
        "average_response_time_ms": 145
    }))
}
