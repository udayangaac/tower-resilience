use std::{sync::{Arc, Mutex}, task::{Context, Poll}, time::Duration};
use tower::{Service, ServiceBuilder};
use tower_resilience::{Policy, ResilienceLayer};

#[derive(Clone)]
struct Flaky {
    remaining: Arc<Mutex<i32>>,
}

impl Service<()> for Flaky {
    type Response = &'static str;
    type Error = &'static str;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        let remaining = self.remaining.clone();
        Box::pin(async move {
            let mut n = remaining.lock().unwrap();
            if *n > 0 {
                *n -= 1;
                Err("fail")
            } else {
                Ok("ok")
            }
        })
    }
}

#[tokio::test]
async fn retries_until_success() {
    let policy = Policy {
        timeout: Duration::from_millis(200),
        max_retries: 5,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(5),
        max_inflight: 5,
    };

    let svc = Flaky {
        remaining: Arc::new(Mutex::new(2)),
    };

    let mut resilient = ServiceBuilder::new()
        .layer(ResilienceLayer::new(policy))
        .service(svc);

    let out = resilient.call(()).await;
    assert_eq!(out.unwrap(), "ok");
}