use crate::RateLimiter;
use backoff::backoff::Backoff;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use std::hash::Hash;
use std::time::Duration;

/// [`MultiRateLimiter`] enables key-based rate limiting, where each key has its own [`RateLimiter`].
///
/// This behavior is useful when you want to throttle a set of keys independently, for example
/// you may have a web crawler that wants to throttle its requests to each domain independently.
///
/// # Examples
///
/// ```
/// use async_throttle::MultiRateLimiter;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///    let period = std::time::Duration::from_secs(5);
///    let rate_limiter = MultiRateLimiter::new(period);
///    
///    // This completes instantly
///    rate_limiter.throttle("foo", || computation()).await;
///
///    // This completes instantly
///    rate_limiter.throttle("bar", || computation()).await;
///
///    // This takes 5 seconds to complete because the key "foo" is rate limited
///    rate_limiter.throttle("foo", || computation()).await;
/// }
///
/// async fn computation() { }
/// ```
pub struct MultiRateLimiter<K> {
    /// The period for each [`RateLimiter`] associated with a particular key
    period: Duration,

    /// The key-specific [`RateLimiter`]s
    ///
    /// The [`RateLimiter`]s are stored in a [`dashmap::DashMap`], which is a concurrent hash map.
    /// Note that keys may map to the same shard within the [`dashmap::DashMap`], so you may experience
    /// increase latency due to the spin-looping nature of [MultiRateLimiter::throttle] combined
    /// with the fallibility of [`dashmap::DashMap::try_entry`].
    rate_limiters: dashmap::DashMap<K, RateLimiter>,
}

impl<K: Eq + Hash + Clone> MultiRateLimiter<K> {
    /// Creates a new [`MultiRateLimiter`].
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            rate_limiters: dashmap::DashMap::new(),
        }
    }

    /// Throttles the execution of a function based on a key.
    /// Throttling is key-specific, so multiple keys can be throttled independently.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_throttle::MultiRateLimiter;
    /// use anyhow::Result;
    /// use std::sync::Arc;
    ///
    /// async fn do_work() { /* some computation */ }
    ///
    /// async fn throttle_by_key(the_key: u32, limiter: Arc<MultiRateLimiter<u32>>) {
    ///    limiter.throttle(the_key, || do_work()).await
    /// }
    pub async fn throttle<Fut, F, T>(&self, key: K, f: F) -> T
    where
        Fut: std::future::Future<Output = T>,
        F: FnOnce() -> Fut,
    {
        loop {
            let mut backoff = get_backoff();

            match self.rate_limiters.try_entry(key.clone()) {
                None => {
                    // Safety: `next_backoff` always returns Some(Duration)
                    tokio::time::sleep(backoff.next_backoff().unwrap()).await
                }
                Some(entry) => {
                    let rate_limiter = entry.or_insert_with(|| RateLimiter::new(self.period));
                    return rate_limiter.value().throttle(f).await;
                }
            }
        }
    }
}

fn get_backoff() -> ExponentialBackoff {
    ExponentialBackoffBuilder::default()
        .with_initial_interval(Duration::from_millis(50))
        .with_max_elapsed_time(None)
        .build()
}
