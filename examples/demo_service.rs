use std::{convert::Infallible, sync::{Arc, Mutex}, task::{Context, Poll}, time::Duration};

use tower::{Service, ServiceBuilder};
use tower_resilience::{Policy, ResilienceLayer};

#[derive(Clone)]
struct FlakyService {
    remaining_failures: Arc<Mutex<i32>>,
}

impl Service<()> for FlakyService {
    type Response = &'static str;
    type Error = &'static str;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        let remaining = self.remaining_failures.clone();
        Box::pin(async move {
            let mut n = remaining.lock().unwrap();
            if *n > 0 {
                *n -= 1;
                Err("temporary error")
            } else {
                Ok("success")
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Infallible> {
    let policy = Policy {
        timeout: Duration::from_millis(500),
        max_retries: 3,
        base_delay: Duration::from_millis(30),
        max_delay: Duration::from_millis(200),
        max_inflight: 10,
    };

    let svc = FlakyService {
        remaining_failures: Arc::new(Mutex::new(2)),
    };

    let mut resilient = ServiceBuilder::new()
        .layer(ResilienceLayer::new(policy))
        .service(svc);

    let res = resilient.call(()).await;
    println!("Result: {:?}", res);

    Ok(())
}