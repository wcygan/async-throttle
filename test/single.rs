#[cfg(test)]
mod tests {
    use anyhow::Result;
    use async_throttle::RateLimiter;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_throttle_empty() -> Result<()> {
        let rate_limiter = RateLimiter::new(Duration::from_millis(10));
        let start = Instant::now();

        for _ in 0..10 {
            rate_limiter.throttle(|| async {}).await;
        }

        let end = start.elapsed().as_millis();
        assert!(end >= 89);
        Ok(())
    }

    #[tokio::test]
    async fn test_throttle_fn() -> Result<()> {
        let rate_limiter = RateLimiter::new(Duration::from_millis(10));
        async fn hello() {
            println!("Hello, world!")
        }

        let start = Instant::now();
        for _ in 0..10 {
            rate_limiter.throttle(hello).await;
        }
        let end = start.elapsed().as_millis();

        assert!(end >= 89);
        Ok(())
    }

    #[tokio::test]
    async fn test_throttle_with_mutable_data() -> Result<()> {
        let rate_limiter = Arc::new(RateLimiter::new(Duration::from_millis(10)));
        let data = Arc::new(Mutex::new(0));

        async fn hello(data: Arc<Mutex<i32>>) {
            let mut data = data.lock().await;
            *data += 1;
        }

        let start = Instant::now();
        let futs = (0..10).map(|_| {
            let data = data.clone();
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move {
                rate_limiter.throttle(|| hello(data.clone())).await;
                Ok::<(), anyhow::Error>(())
            })
        });

        for fut in futs {
            fut.await??;
        }

        let end = start.elapsed().as_millis();
        let data = data.lock().await;
        assert!(end >= 89);
        assert_eq!(*data, 10);
        Ok(())
    }

    #[tokio::test]
    async fn test_throttle_fn_mut_with_mutable_data_2() -> Result<()> {
        let data = Arc::new(Mutex::new(Data { data: 0 }));
        let rate_limiter = Arc::new(RateLimiter::new(Duration::from_millis(10)));

        struct Data {
            data: i32,
        }

        impl Data {
            async fn increment(&mut self) {
                self.data += 1;
            }
        }

        async fn hello(data: Arc<Mutex<Data>>) {
            let mut data = data.lock().await;
            data.increment().await;
        }

        let start = Instant::now();
        let futs = (0..10).map(|_| {
            let data = data.clone();
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move {
                rate_limiter.throttle(|| hello(data.clone())).await;
                Ok::<(), anyhow::Error>(())
            })
        });

        for fut in futs {
            fut.await??;
        }

        let end = start.elapsed().as_millis();
        let data = data.lock().await;
        assert!(end >= 89);
        assert_eq!(data.data, 10);
        Ok(())
    }
}
