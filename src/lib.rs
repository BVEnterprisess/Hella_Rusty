//! Project Chimera - Enterprise AI Agent Platform
//!
//! Modernized runtime primitives for orchestrating AI agents with
//! production-friendly DevOps ergonomics.

pub mod agents;
pub mod audit_logging;
pub mod inference;
pub mod orchestration;
pub mod platform;
pub mod rate_limiting;
pub mod training;
pub mod utils;

pub use platform::{Platform, PlatformConfig, PlatformContext, PlatformRuntime};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn platform_bootstraps_with_defaults() {
        let config = PlatformConfig::default();
        let platform = Platform::new(config);
        let runtime = platform.start().await.expect("platform should start");
        assert_eq!(runtime.context().config().metadata.name, "project-chimera");
        runtime.shutdown().await.expect("runtime shutdown cleanly");
    }
}
