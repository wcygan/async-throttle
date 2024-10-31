use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{interval, Interval};

/// [`RateLimiter`] is a tool which can control the rate at which processing happens.
///
/// # Examples
///
/// ```
/// use async_throttle::RateLimiter;
///
/// #[tokio::main]
/// async fn main() {
///     let period = std::time::Duration::from_millis(10);
///     let rate_limiter = RateLimiter::new(period);
///
///     // Takes 90ms to complete, the first iteration is instant, the next 9 iterations take 100ms
///     for _ in 0..10 {
///        rate_limiter.throttle(|| async { /* work */ }).await;
///     }
/// }
pub struct RateLimiter {
    /// The mutex that will be locked when the rate limiter is waiting for the interval to tick.
    ///
    /// It's important to use a tokio::sync::Mutex here instead of a std::sync::Mutex. The reason is
    /// that the tokio::sync::Mutex does not block & the MutexGuard is held across await points.
    ///
    /// If you tried to use std::sync::Mutex instead, you would get a compiler error when
    /// spawning tokio tasks because the MutexGuard would not be Send.
    interval: Mutex<Interval>,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::Mutex;
    /// use anyhow::Result;
    /// use std::time::Duration;
    /// use async_throttle::RateLimiter;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     RateLimiter::new(Duration::from_millis(10));
    ///     Ok(())
    /// }
    /// ```
    pub fn new(period: Duration) -> Self {
        Self {
            interval: Mutex::new(interval(period)),
        }
    }

    /// Throttles the execution of a function.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_throttle::RateLimiter;
    /// use anyhow::Result;
    /// use std::sync::Arc;
    ///
    /// async fn do_work() { /* some computation */ }
    ///
    /// async fn do_throttle(limiter: Arc<RateLimiter>) {
    ///     limiter.throttle(|| do_work()).await
    /// }
    pub async fn throttle<Fut, F, T>(&self, f: F) -> T
    where
        Fut: std::future::Future<Output = T>,
        F: FnOnce() -> Fut,
    {
        self.wait().await;
        f().await
    }

    /// Waits for the interval to tick.
    ///
    /// This is the building block for the throttle function. It works by allowing only one task to
    /// access the interval at a time. The first task to access the interval will tick it and then
    /// release the lock. The next task to access the interval will tick it and then release the
    /// lock.
    async fn wait(&self) {
        let mut interval = self.interval.lock().await;
        interval.tick().await;
    }
}
