use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::platform::config::ObservabilitySettings;
use crate::platform::service::ServiceRegistration;

pub fn telemetry_service(settings: ObservabilitySettings) -> ServiceRegistration {
    ServiceRegistration::new(
        "telemetry",
        Arc::new(move |_ctx, token: CancellationToken| {
            let settings = settings.clone();
            tokio::spawn(async move {
                let outcome: Result<()> = async {
                    if !settings.enable_metrics {
                        return Ok(());
                    }

                    let app = Router::new().route("/metrics", get(metrics_handler));
                    let addr = SocketAddr::from(([0, 0, 0, 0], settings.metrics_port));
                    let shutdown = token.clone();

                    let listener = tokio::net::TcpListener::bind(addr).await?;
                    info!(port = settings.metrics_port, "telemetry server started");

                    axum::serve(listener, app)
                        .with_graceful_shutdown(async move {
                            shutdown.cancelled().await;
                            info!("shutting down telemetry server");
                        })
                        .await?;

                    Ok(())
                }
                .await;

                outcome
            })
        }),
    )
}

async fn metrics_handler() -> &'static str {
    "# HELP chimera_agents_active Number of active agents\n# TYPE chimera_agents_active gauge\nchimera_agents_active 0\n# HELP chimera_requests_total Total number of requests processed\n# TYPE chimera_requests_total counter\nchimera_requests_total 0\n"
}
