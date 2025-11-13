use axum::response::IntoResponse;
use chimera_core::{Platform, PlatformConfig, PlatformContext};
use clap::Parser;
use dotenvy::dotenv;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "configs/platform.toml")]
    config: PathBuf,

    /// Port to bind to
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Agent name
    #[arg(short, long)]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();

    info!(agent = %args.name, "starting Chimera agent");

    let config = PlatformConfig::load_from_path(Some(args.config.clone()))?;
    let platform = Platform::new(config);
    let runtime = platform.start().await?;
    let context = runtime.context();

    let app = build_router(context.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!(%addr, "agent listening");
    axum::serve(listener, app).await?;

    runtime.shutdown().await?;
    Ok(())
}

fn build_router(context: PlatformContext) -> axum::Router {
    axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/predict", axum::routing::post(predict))
        .route("/status", axum::routing::get(agent_status))
        .with_state(context)
}

async fn health_check() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chimera_core::utils::timestamp_now(),
    }))
}

async fn predict(
    axum::extract::State(platform): axum::extract::State<PlatformContext>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> axum::response::Response {
    info!("received prediction request: {:?}", payload);

    let client_ip = std::net::IpAddr::from([127, 0, 0, 1]);
    if let Err(e) = platform
        .rate_limiter()
        .check_rate_limit(client_ip, "/predict")
    {
        error!("rate limit exceeded for {}: {:?}", client_ip, e);
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({ "error": "Rate limit exceeded" })),
        )
            .into_response();
    }

    if let Err(errors) = chimera_core::utils::validate_request_payload(&payload) {
        return (axum::http::StatusCode::BAD_REQUEST, axum::Json(errors)).into_response();
    }

    if let Err(err) = platform
        .audit_logger()
        .log_api_access(None, "/predict", "POST", 200, None)
    {
        error!(?err, "failed to record audit log for predict request");
    }

    let response = serde_json::json!({
        "result": "Prediction completed",
        "confidence": 0.95,
        "processing_time_ms": 150
    });

    axum::Json(response).into_response()
}

async fn agent_status(
    axum::extract::State(_platform): axum::extract::State<PlatformContext>,
) -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "name": "chimera-agent",
        "status": "active",
        "uptime_seconds": 3600,
        "requests_processed": 150,
        "average_response_time_ms": 145
    }))
}
