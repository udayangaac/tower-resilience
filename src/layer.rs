use crate::{policy::Policy, service::ResilienceService};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower::Layer;

#[derive(Clone, Debug)]
pub struct ResilienceLayer {
    policy: Policy,
}

impl ResilienceLayer {
    pub fn new(policy: Policy) -> Self {
        Self { policy }
    }
}

impl<S> Layer<S> for ResilienceLayer {
    type Service = ResilienceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let sem = Arc::new(Semaphore::new(self.policy.max_inflight));
        ResilienceService::new(inner, self.policy.clone(), sem)
    }
}