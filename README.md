# Async Throttle

[<img alt="github" src="https://img.shields.io/badge/github-wcygan/async--throttle-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/wcygan/async-throttle)
[<img alt="crates.io" src="https://img.shields.io/crates/v/async-throttle.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/async-throttle)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-async--throttle-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/async-throttle)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/wcygan/async-throttle/test.yml?branch=main&style=for-the-badge" height="20">](https://github.com/wcygan/async-throttle/actions?query=branch%3Amain)
[![codecov](https://codecov.io/github/wcygan/async-throttle/branch/main/graph/badge.svg?token=FW4Z2BUX1J)](https://codecov.io/github/wcygan/async-throttle)

Asynchronous Rate Limiting

This crate provides coarse and fine-grained ways to throttle the rate at which asynchronous tasks are executed.

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
async-throttle = "0.3.2"
```

You can use the fine-grained rate limiter like so:

```rust
#[tokio::main]
async fn main() {
    let period = std::time::Duration::from_secs(5);
    let rate_limiter = MultiRateLimiter::new(period);

    // This completes instantly
    rate_limiter.throttle("foo", || computation()).await;

    // This completes instantly
    rate_limiter.throttle("bar", || computation()).await;

    // This takes 5 seconds to complete because the key "foo" is rate limited
    rate_limiter.throttle("foo", || computation()).await;
}
```

