// SPDX-License-Identifier: PMPL-1.0-or-later
//! Retry logic with exponential backoff for transient failures
//!
//! Implements retry mechanism for:
//! - Network timeouts
//! - Temporary prover unavailability
//! - Rate limiting
//! - Transient ECHIDNA Core failures
//!
//! Strategy: Exponential backoff with jitter
//! - Retry 1: 1s
//! - Retry 2: 2s
//! - Retry 3: 4s
//! - Max retries: 3

use crate::error::{Error, Result};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub multiplier: f64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry policy
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    /// Create a new retry policy with default configuration
    pub fn new() -> Self {
        Self {
            config: RetryConfig::default(),
        }
    }

    /// Create a retry policy with custom configuration
    pub fn with_config(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a fallible operation with retries
    ///
    /// # Arguments
    /// * `operation` - Async function to execute
    /// * `is_retryable` - Function to determine if error should be retried
    ///
    /// # Returns
    /// Result of the operation, or last error if all retries exhausted
    pub async fn execute<F, Fut, T, E>(
        &self,
        mut operation: F,
        is_retryable: impl Fn(&E) -> bool,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut backoff = self.config.initial_backoff;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    attempt += 1;

                    if attempt > self.config.max_retries {
                        warn!(
                            "Operation failed after {} attempts: {}",
                            self.config.max_retries, error
                        );
                        return Err(error);
                    }

                    if !is_retryable(&error) {
                        warn!("Operation failed with non-retryable error: {}", error);
                        return Err(error);
                    }

                    // Calculate delay with optional jitter
                    let delay = if self.config.jitter {
                        let jitter_factor = 0.5 + (rand::random::<f64>() * 0.5); // 0.5-1.0
                        Duration::from_secs_f64(backoff.as_secs_f64() * jitter_factor)
                    } else {
                        backoff
                    };

                    warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying in {:?}...",
                        attempt, self.config.max_retries, error, delay
                    );

                    sleep(delay).await;

                    // Exponential backoff
                    backoff = Duration::from_secs_f64(
                        (backoff.as_secs_f64() * self.config.multiplier)
                            .min(self.config.max_backoff.as_secs_f64()),
                    );
                }
            }
        }
    }

    /// Execute with automatic retry on common transient errors
    pub async fn execute_auto<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.execute(operation, is_transient_error)
            .await
            .map_err(|e| e) // Error is already crate::Error
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Determine if an error is transient and should be retried
fn is_transient_error(error: &Error) -> bool {
    match error {
        // Network errors - always retry
        Error::Http(_) => true,

        // ECHIDNA errors - check if transient
        Error::Echidna(msg) => {
            let msg_lower = msg.to_lowercase();
            msg_lower.contains("timeout")
                || msg_lower.contains("unavailable")
                || msg_lower.contains("rate limit")
                || msg_lower.contains("temporary")
                || msg_lower.contains("503")
                || msg_lower.contains("504")
        }

        // Database errors - some are retryable
        Error::Sqlx(sqlx_err) => {
            let err_msg = sqlx_err.to_string().to_lowercase();
            err_msg.contains("connection")
                || err_msg.contains("timeout")
                || err_msg.contains("deadlock")
        }

        // Internal errors - generally not retryable
        Error::Internal(_) => false,

        // Config/validation errors - never retry
        Error::Config(_) | Error::InvalidInput(_) => false,

        // All other errors - default to not retrying for safety
        _ => false,
    }
}

/// Retry helper for async operations
///
/// # Example
/// ```no_run
/// use echidnabot::scheduler::retry::retry;
///
/// let result = retry(3, || async {
///     // Your async operation
///     Ok(())
/// }).await;
/// ```
pub async fn retry<F, Fut, T>(max_attempts: usize, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let policy = RetryPolicy::new();
    let mut config = RetryConfig::default();
    config.max_retries = max_attempts;

    RetryPolicy::with_config(config)
        .execute_auto(operation)
        .await
}

/// Retry with custom backoff
pub async fn retry_with_backoff<F, Fut, T>(
    max_attempts: usize,
    initial_backoff: Duration,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let config = RetryConfig {
        max_retries: max_attempts,
        initial_backoff,
        ..Default::default()
    };

    RetryPolicy::with_config(config)
        .execute_auto(operation)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = retry(3, || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<_, Error>(42)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only 1 attempt
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = retry(3, || {
            let counter = counter_clone.clone();
            async move {
                let attempt = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if attempt < 3 {
                    Err(Error::Http(reqwest::Error::from(std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "connection refused",
                    ))))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 3 attempts total
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = retry(3, || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(Error::Http(reqwest::Error::from(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "connection refused",
                ))))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 4); // Initial + 3 retries = 4
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = retry(3, || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(Error::InvalidInput("bad input".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retries for invalid input
    }

    #[test]
    fn test_is_transient_error() {
        // Transient errors
        assert!(is_transient_error(&Error::Http(
            reqwest::Error::from(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "timeout"
            ))
        )));
        assert!(is_transient_error(&Error::Echidna("timeout".to_string())));
        assert!(is_transient_error(&Error::Echidna("503 unavailable".to_string())));

        // Non-transient errors
        assert!(!is_transient_error(&Error::InvalidInput("bad".to_string())));
        assert!(!is_transient_error(&Error::Config("bad config".to_string())));
    }
}
