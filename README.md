# Tower Resilience

[![Rust](https://github.com/udayangaac/tower-resilience/actions/workflows/rust.yml/badge.svg)](https://github.com/udayangaac/tower-resilience/actions/workflows/rust.yml)


A composable Tower middleware library that adds production grade resilience primitives such as retries, timeouts, and circuit breakers.


```rust
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
```