use std::sync::Arc;

use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::platform::context::PlatformContext;

pub type ServiceSpawner = Arc<
    dyn Fn(PlatformContext, CancellationToken) -> JoinHandle<Result<()>> + Send + Sync + 'static,
>;

pub struct ServiceRegistration {
    name: String,
    spawner: ServiceSpawner,
}

impl ServiceRegistration {
    pub fn new<N: Into<String>>(name: N, spawner: ServiceSpawner) -> Self {
        Self {
            name: name.into(),
            spawner,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn spawn(
        &self,
        context: PlatformContext,
        token: CancellationToken,
    ) -> JoinHandle<Result<()>> {
        (self.spawner)(context, token)
    }
}
