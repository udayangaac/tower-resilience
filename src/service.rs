use crate::{error::ResilienceError, policy::Policy};
use rand::Rng;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use tokio::{sync::Semaphore, time};
use tower::Service;

#[derive(Clone)]
pub struct ResilienceService<S> {
    inner: S,
    policy: Policy,
    sem: Arc<Semaphore>,
}

impl<S> ResilienceService<S> {
    pub(crate) fn new(inner: S, policy: Policy, sem: Arc<Semaphore>) -> Self {
        Self { inner, policy, sem }
    }
}

impl<S, Req> Service<Req> for ResilienceService<S>
where
    S: Service<Req> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
    Req: Clone + Send + 'static,
{
    type Response = S::Response;
    type Error = ResilienceError<S::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(ResilienceError::Inner)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let mut inner = self.inner.clone();
        let policy = self.policy.clone();
        let sem = self.sem.clone();

        Box::pin(async move {
            let _permit = sem.acquire_owned().await.expect("semaphore closed");
            let mut attempt: usize = 0;

            loop {
                let fut = inner.call(req.clone());
                let timed = time::timeout(policy.timeout, fut).await;

                match timed {
                    Ok(Ok(resp)) => return Ok(resp),

                    Ok(Err(e)) => {
                        if attempt >= policy.max_retries {
                            return Err(ResilienceError::Inner(e));
                        }
                    }

                    Err(_) => {
                        if attempt >= policy.max_retries {
                            return Err(ResilienceError::Timeout);
                        }
                    }
                }

                attempt += 1;
                sleep_backoff(attempt, policy.base_delay, policy.max_delay).await;
            }
        })
    }
}

async fn sleep_backoff(attempt: usize, base: Duration, max: Duration) {
    let base_ms = base.as_millis().max(1) as u64;

    let exp = (attempt as u32).min(16);
    let candidate_ms = base_ms.saturating_mul(1u64 << exp);

    let cap_ms = max.as_millis().max(1) as u64;
    let delay_ms = candidate_ms.min(cap_ms);

    let jitter = rand::thread_rng().gen_range(0..=(delay_ms / 4));
    time::sleep(Duration::from_millis(delay_ms + jitter)).await;
}