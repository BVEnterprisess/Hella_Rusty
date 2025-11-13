use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::agents::AgentRegistry;
use crate::audit_logging::AuditLogger;
use crate::platform::config::PlatformConfig;
use crate::rate_limiting::RateLimiter;

#[derive(Clone)]
pub struct PlatformContext {
    shared: Arc<SharedState>,
    shutdown: CancellationToken,
}

struct SharedState {
    config: PlatformConfig,
    audit_logger: Arc<AuditLogger>,
    rate_limiter: Arc<RateLimiter>,
    agent_registry: AgentRegistry,
}

impl PlatformContext {
    pub(crate) fn new(
        config: PlatformConfig,
        audit_logger: Arc<AuditLogger>,
        rate_limiter: Arc<RateLimiter>,
        agent_registry: AgentRegistry,
        shutdown: CancellationToken,
    ) -> Self {
        let shared = SharedState {
            config,
            audit_logger,
            rate_limiter,
            agent_registry,
        };

        Self {
            shared: Arc::new(shared),
            shutdown,
        }
    }

    pub fn config(&self) -> &PlatformConfig {
        &self.shared.config
    }

    pub fn audit_logger(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.shared.audit_logger)
    }

    pub fn rate_limiter(&self) -> Arc<RateLimiter> {
        Arc::clone(&self.shared.rate_limiter)
    }

    pub fn agents(&self) -> AgentRegistry {
        self.shared.agent_registry.clone()
    }

    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }
}
