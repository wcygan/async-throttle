#[cfg(test)]
mod tests {
    use anyhow::Result;
    use async_throttle::MultiRateLimiter;
    use futures::future::join_all;
    use std::ops::Mul;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn sync_throttle_same_key() -> Result<()> {
        let rate_limiter = MultiRateLimiter::new(Duration::from_millis(10));
        let start = Instant::now();
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        for _ in 0..10 {
            rate_limiter
                .throttle("key", || async {
                    COUNT.fetch_add(1, SeqCst);
                })
                .await;
        }

        assert_eq!(COUNT.load(SeqCst), 10);
        assert!(start.elapsed().as_millis() > 89);
        Ok(())
    }

    #[tokio::test]
    async fn sync_throttle_multi_key() -> Result<()> {
        static ONE_THOUSAND_SECONDS: Duration = Duration::from_secs(1000);
        let rate_limiter = Arc::new(MultiRateLimiter::new(ONE_THOUSAND_SECONDS));
        let start = Instant::now();
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        for k in 0..10 {
            rate_limiter
                .throttle(k, || async {
                    COUNT.fetch_add(1, SeqCst);
                })
                .await;
        }

        assert_eq!(COUNT.load(SeqCst), 10);
        assert!(start.elapsed() < ONE_THOUSAND_SECONDS);
        Ok(())
    }

    #[tokio::test]
    async fn async_throttle_same_key() -> Result<()> {
        let rate_limiter = Arc::new(MultiRateLimiter::new(Duration::from_millis(1)));
        let start = Instant::now();
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        join_all((0..100).map(|_| {
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move {
                rate_limiter
                    .throttle("key", || async {
                        COUNT.fetch_add(1, SeqCst);
                    })
                    .await;
                Ok::<(), anyhow::Error>(())
            })
        }))
        .await;

        assert_eq!(COUNT.load(SeqCst), 100);
        assert!(start.elapsed().as_millis() > 99);
        Ok(())
    }

    #[tokio::test]
    async fn async_throttle_multi_key_get_once() -> Result<()> {
        static ONE_THOUSAND_SECONDS: Duration = Duration::from_secs(1000);
        let rate_limiter = Arc::new(MultiRateLimiter::new(ONE_THOUSAND_SECONDS));
        let start = Instant::now();
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        join_all((0..1000).map(|x| {
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move {
                rate_limiter
                    .throttle(x, || async {
                        COUNT.fetch_add(1, SeqCst);
                    })
                    .await;
                Ok::<(), anyhow::Error>(())
            })
        }))
        .await;

        assert_eq!(COUNT.load(SeqCst), 1000);
        assert!(start.elapsed() < ONE_THOUSAND_SECONDS);
        Ok(())
    }

    #[tokio::test]
    async fn async_throttle_multi_key_get_many_times() -> Result<()> {
        let period = Duration::from_nanos(100);
        let rate_limiter = Arc::new(MultiRateLimiter::new(period));
        let start = Instant::now();
        let (max, radix): (u32, u32) = (1000, 100);
        let min_wait_time = period.mul(max / radix);
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        join_all((0..max).map(|x| {
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move {
                let target = x % radix;
                rate_limiter
                    .throttle(target, || async {
                        COUNT.fetch_add(1, SeqCst);
                    })
                    .await;
                Ok::<(), anyhow::Error>(())
            })
        }))
        .await;

        assert_eq!(COUNT.load(SeqCst), max as usize);
        assert!(start.elapsed() > min_wait_time);
        Ok(())
    }
}
