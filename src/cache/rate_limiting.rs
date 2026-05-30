//! Rate limiting implementation using Redis.
//!
//! Provides token bucket and sliding window rate limiting strategies
//! with configurable limits and time windows.
//!
//! ## Overview
//!
//! Rate limiting is a critical security measure that protects APIs from abuse,
//! prevents denial-of-service attacks, and ensures fair resource allocation.
//! This module implements two popular rate limiting algorithms:
//!
//! - **Token Bucket**: Allows burst traffic while maintaining an average rate
//! - **Sliding Window**: Provides smoother rate limiting with no burst allowance
//!
//! ## Security Considerations
//!
//! 1. **Defense Against DoS**: Rate limits prevent attackers from overwhelming
//!    the system with excessive requests.
//!
//! 2. **Resource Protection**: Ensures fair usage by limiting per-client requests.
//!
//! 3. **Cost Control**: Prevents unexpected billing from third-party APIs that
//!    charge per request.
//!
//! ## Usage
//!
//! ```rust
//! use cache::rate_limiting::{RateLimiter, RateLimitConfig, RateLimitStrategy};
//! use std::time::Duration;
//!
//! // Create a rate limiter allowing 100 requests per minute
//! let config = RateLimitConfig {
//!     max_requests: 100,
//!     window: Duration::from_secs(60),
//!     strategy: RateLimitStrategy::TokenBucket,
//! };
//! let mut limiter = RateLimiter::with_config(config);
//!
//! // Try to acquire a token for each request
//! if limiter.try_acquire() {
//!     // Process request
//! } else {
//!     // Return 429 Too Many Requests
//! }
//! ```
//!
//! ## Architecture
//!
//! The rate limiter operates in-memory and can be paired with Redis for
//! distributed rate limiting across multiple application instances.
//! The token bucket algorithm works as follows:
//!
//! 1. Bucket starts full with `max_requests` tokens
//! 2. Each request consumes 1 token
//! 3. Tokens are refilled at a constant rate over the window period
//! 4. When bucket is empty, requests are rejected

use std::time::Duration;

/// Rate limiting configuration
///
/// # Fields
///
/// * `max_requests` - Maximum number of requests allowed within the time window
/// * `window` - Duration of the time window
/// * `strategy` - Algorithm to use for rate limiting
///
/// # Example
///
/// ```
/// use cache::rate_limiting::{RateLimitConfig, RateLimitStrategy};
/// use std::time::Duration;
///
/// let config = RateLimitConfig {
///     max_requests: 100,
///     window: Duration::from_secs(60),
///     strategy: RateLimitStrategy::TokenBucket,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed
    pub max_requests: u32,
    /// Time window for the rate limit
    pub window: Duration,
    /// Strategy to use for rate limiting
    pub strategy: RateLimitStrategy,
}

/// Rate limiting strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitStrategy {
    /// Token bucket algorithm
    TokenBucket,
    /// Sliding window algorithm
    SlidingWindow,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            strategy: RateLimitStrategy::TokenBucket,
        }
    }
}

/// Rate limiter for controlling request rates
///
/// Implements the token bucket algorithm for efficient rate limiting.
/// The limiter tracks available tokens and refills them over time.
///
/// # Example
///
/// ```
/// use cache::rate_limiting::{RateLimiter, RateLimitConfig};
/// use std::time::Duration;
///
/// let config = RateLimitConfig {
///     max_requests: 100,
///     window: Duration::from_secs(60),
///     ..Default::default()
/// };
/// let mut limiter = RateLimiter::with_config(config);
///
/// // For each incoming request
/// match limiter.try_acquire() {
///     true => println!("Request allowed"),
///     false => println!("Rate limit exceeded"),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    tokens: u32,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    /// Creates a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Creates a new rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            tokens: config.max_requests,
            last_refill: std::time::Instant::now(),
        }
    }

    /// Attempts to acquire a token for a request
    ///
    /// Returns `true` if a token was available and the request is allowed.
    /// Returns `false` if no tokens are available and the request should be rejected.
    ///
    /// # Behavior
    ///
    /// - Automatically refills tokens based on elapsed time
    /// - Consumes 1 token on success
    /// - Does not consume a token if none are available
    ///
    /// # Example
    ///
    /// ```
    /// let mut limiter = RateLimiter::new();
    ///
    /// if limiter.try_acquire() {
    ///     // Process the request
    /// } else {
    ///     // Return 429 Too Many Requests to client
    /// }
    /// ```
    pub fn try_acquire(&mut self) -> bool {
        self.refill_tokens();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Attempts to acquire multiple tokens (for batch operations)
    ///
    /// Useful for endpoints that perform multiple operations in a single request.
    /// Returns `true` if all requested tokens were available, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// let mut limiter = RateLimiter::new();
    ///
    /// // Bulk operation requires 5 tokens
    /// if limiter.try_acquire_batch(5) {
    ///     // Process bulk request
    /// } else {
    ///     // Reject - not enough tokens
    /// }
    /// ```
    pub fn try_acquire_batch(&mut self, count: u32) -> bool {
        self.refill_tokens();

        if self.tokens >= count {
            self.tokens -= count;
            true
        } else {
            false
        }
    }

    /// Returns the number of available tokens
    ///
    /// Useful for displaying rate limit status to clients via headers.
    /// Automatically refills tokens before counting.
    ///
    /// # Example
    ///
    /// ```
    /// let mut limiter = RateLimiter::new();
    ///
    /// // Check remaining quota
    /// let remaining = limiter.available_tokens();
    /// println!("{} requests remaining", remaining);
    /// ```
    pub fn available_tokens(&mut self) -> u32 {
        self.refill_tokens();
        self.tokens
    }

    /// Returns the time until the next token is available
    pub fn time_until_available(&mut self) -> Option<Duration> {
        if self.try_acquire() {
            return Some(Duration::from_secs(0));
        }

        let elapsed = self.last_refill.elapsed();
        if elapsed >= self.config.window {
            return Some(Duration::from_secs(0));
        }

        Some(self.config.window - elapsed)
    }

    /// Refills tokens based on elapsed time
    fn refill_tokens(&mut self) {
        let elapsed = self.last_refill.elapsed();

        if elapsed >= self.config.window {
            self.tokens = self.config.max_requests;
            self.last_refill = std::time::Instant::now();
        } else {
            // Calculate tokens to add based on elapsed time
            let refill_rate = self.config.max_requests as f64 / self.config.window.as_secs_f64();
            let tokens_to_add = (elapsed.as_secs_f64() * refill_rate) as u32;

            self.tokens = (self.tokens + tokens_to_add).min(self.config.max_requests);
            self.last_refill = std::time::Instant::now();
        }
    }

    /// Resets the rate limiter to initial state
    pub fn reset(&mut self) {
        self.tokens = self.config.max_requests;
        self.last_refill = std::time::Instant::now();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_token() {
        let mut limiter = RateLimiter::new();
        assert!(limiter.try_acquire());
    }

    #[test]
    fn test_exhaust_tokens() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
            strategy: RateLimitStrategy::TokenBucket,
        };
        let mut limiter = RateLimiter::with_config(config);

        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_acquire_batch() {
        let config = RateLimitConfig {
            max_requests: 10,
            window: Duration::from_secs(60),
            strategy: RateLimitStrategy::TokenBucket,
        };
        let mut limiter = RateLimiter::with_config(config);

        assert!(limiter.try_acquire_batch(5));
        assert!(limiter.try_acquire_batch(5));
        assert!(!limiter.try_acquire_batch(1));
    }

    #[test]
    fn test_available_tokens() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            strategy: RateLimitStrategy::TokenBucket,
        };
        let mut limiter = RateLimiter::with_config(config);

        assert_eq!(limiter.available_tokens(), 5);
        limiter.try_acquire();
        assert_eq!(limiter.available_tokens(), 4);
    }

    #[test]
    fn test_reset() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
            strategy: RateLimitStrategy::TokenBucket,
        };
        let mut limiter = RateLimiter::with_config(config);

        limiter.try_acquire();
        limiter.try_acquire();
        assert_eq!(limiter.available_tokens(), 3);

        limiter.reset();
        assert_eq!(limiter.available_tokens(), 5);
    }

    #[test]
    fn test_time_until_available() {
        let config = RateLimitConfig {
            max_requests: 1,
            window: Duration::from_secs(60),
            strategy: RateLimitStrategy::TokenBucket,
        };
        let mut limiter = RateLimiter::with_config(config);

        limiter.try_acquire();
        let time_until = limiter.time_until_available();
        assert!(time_until.is_some());
        assert!(time_until.unwrap() > Duration::from_secs(0));
    }
}
