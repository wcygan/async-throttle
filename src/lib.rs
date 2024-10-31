//! Tools for rate limiting asynchronously
//!
//! * [`MultiRateLimiter`], a key-based rate limiter
//! * [`RateLimiter`], a rate limiter
//!
//! # Examples
//!
//! ```
//! use async_throttle::MultiRateLimiter;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let period = std::time::Duration::from_secs(5);
//!     let rate_limiter = MultiRateLimiter::new(period);
//!
//!     // This completes instantly
//!     rate_limiter.throttle("foo", || computation()).await;
//!
//!     // This completes instantly
//!     rate_limiter.throttle("bar", || computation()).await;
//!
//!     // This takes 5 seconds to complete because the key "foo" is rate limited
//!     rate_limiter.throttle("foo", || computation()).await;
//! }
//!
//! async fn computation() { }
//! ```
pub use multi::MultiRateLimiter;
pub use single::RateLimiter;
mod multi;
mod single;
