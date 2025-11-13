use std::sync::Arc;

use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::agents::AgentRegistry;
use crate::audit_logging::AuditLogger;
use crate::orchestration::orchestration_service;
use crate::platform::config::{AuditSettings, PlatformConfig, RateLimitingSettings};
use crate::platform::context::PlatformContext;
use crate::platform::service::ServiceRegistration;
use crate::platform::telemetry::telemetry_service;
use crate::rate_limiting::RateLimiter;

pub struct Platform {
    config: PlatformConfig,
    services: Vec<ServiceRegistration>,
}

impl Platform {
    pub fn new(config: PlatformConfig) -> Self {
        let mut services = Vec::new();
        services.push(telemetry_service(config.observability.clone()));

        Self { config, services }
    }

    pub fn register_service(&mut self, service: ServiceRegistration) {
        self.services.push(service);
    }

    pub async fn start(self) -> Result<PlatformRuntime> {
        initialize_logging(&self.config);

        let audit_logger = Arc::new(init_audit_logger(&self.config.audit)?);
        let rate_limiter = Arc::new(RateLimiter::from_settings(&self.config.rate_limiting));
        let agent_registry = AgentRegistry::from_catalog(self.config.agent_catalog());

        let root_token = CancellationToken::new();
        let context = PlatformContext::new(
            self.config.clone(),
            audit_logger,
            rate_limiter,
            agent_registry,
            root_token.child_token(),
        );

        let mut tasks = Vec::new();
        let mut services = self.services;
        services.push(orchestration_service(context.agents()));

        for service in services {
            let handle = service.spawn(context.clone(), root_token.child_token());
            tasks.push((service.name().to_string(), handle));
        }

        info!("platform boot completed");

        Ok(PlatformRuntime {
            context,
            cancel_token: root_token,
            tasks,
        })
    }
}

pub struct PlatformRuntime {
    context: PlatformContext,
    cancel_token: CancellationToken,
    tasks: Vec<(String, JoinHandle<Result<()>>)>,
}

impl PlatformRuntime {
    pub fn context(&self) -> PlatformContext {
        self.context.clone()
    }

    pub async fn shutdown(self) -> Result<()> {
        self.cancel_token.cancel();

        for (name, handle) in self.tasks {
            match handle.await {
                Ok(Ok(())) => info!(service = %name, "service shutdown cleanly"),
                Ok(Err(err)) => {
                    return Err(err);
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        Ok(())
    }
}

fn initialize_logging(config: &PlatformConfig) {
    let filter = &config.observability.log_level;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter.as_str())
        .try_init();
}

fn init_audit_logger(settings: &AuditSettings) -> Result<AuditLogger> {
    AuditLogger::new(&settings.log_path, settings.retention_days)
        .map_err(|err| anyhow::anyhow!(err.to_string()))
}

impl RateLimiter {
    pub fn from_settings(settings: &RateLimitingSettings) -> Self {
        use crate::rate_limiting::{RateLimit, RateLimitConfig};
        use std::time::Duration;

        let default = RateLimit {
            requests: settings.default.requests,
            window: Duration::from_secs(settings.default.window_seconds),
        };

        let endpoints = settings
            .endpoints
            .iter()
            .map(|(endpoint, rule)| {
                (
                    endpoint.clone(),
                    RateLimit {
                        requests: rule.requests,
                        window: Duration::from_secs(rule.window_seconds),
                    },
                )
            })
            .collect();

        RateLimiter::new(RateLimitConfig {
            default,
            endpoints,
            burst_limit: settings.default.burst,
        })
    }
}
