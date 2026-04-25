// SPDX-License-Identifier: PMPL-1.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
//! Webhook rate limiting
//!
//! Sliding-window per-IP rate limiter applied to all webhook ingress paths.
//! The window is always 60 seconds; the limit is configurable via
//! `[server] rate_limit_rpm` (requests per minute). When the limit is
//! exceeded the handler returns 429 Too Many Requests with a `Retry-After`
//! header.
//!
//! No external crates: uses a `Mutex<HashMap<IpAddr, VecDeque<Instant>>>`
//! with lock time bounded by the dequeue sweep. Under webhook load (at most
//! a few hundred requests/minute per IP) this is negligible.

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use super::webhooks::AppState;

/// Sliding-window per-IP rate limiter (60-second window).
pub struct WebhookRateLimiter {
    state: Mutex<HashMap<IpAddr, VecDeque<Instant>>>,
    window: Duration,
    limit: u32,
}

impl WebhookRateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            window: Duration::from_secs(60),
            limit: requests_per_minute,
        }
    }

    /// Returns `true` if the request is within the allowed rate.
    pub fn check_ip(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut state = self.state.lock().expect("rate limiter mutex poisoned");
        let timestamps = state.entry(ip).or_insert_with(VecDeque::new);

        // Evict entries older than the window
        let cutoff = now - self.window;
        while timestamps.front().map(|&t| t < cutoff).unwrap_or(false) {
            timestamps.pop_front();
        }

        if (timestamps.len() as u32) < self.limit {
            timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }
}

/// Axum middleware: reject requests over the per-IP limit with 429.
///
/// Applied only to webhook routes. Health and metrics endpoints are
/// intentionally excluded so monitoring systems are never blocked.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if let Some(ref limiter) = state.rate_limiter {
        if !limiter.check_ip(peer.ip()) {
            tracing::warn!(
                ip = %peer.ip(),
                limit = limiter.limit(),
                "Webhook rate limit exceeded"
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    ("Retry-After", "60"),
                    ("X-RateLimit-Limit", &limiter.limit().to_string()),
                    ("X-RateLimit-Window", "60"),
                ],
                "Rate limit exceeded — too many webhook requests from this IP",
            )
                .into_response();
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_allows_requests_within_limit() {
        let limiter = WebhookRateLimiter::new(5);
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        for _ in 0..5 {
            assert!(limiter.check_ip(ip), "should allow up to the limit");
        }
    }

    #[test]
    fn test_blocks_over_limit() {
        let limiter = WebhookRateLimiter::new(3);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        for _ in 0..3 {
            limiter.check_ip(ip);
        }
        assert!(!limiter.check_ip(ip), "4th request should be blocked");
    }

    #[test]
    fn test_different_ips_are_independent() {
        let limiter = WebhookRateLimiter::new(2);
        let ip1 = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        limiter.check_ip(ip1);
        limiter.check_ip(ip1);
        // ip1 is now at limit, ip2 should still be free
        assert!(!limiter.check_ip(ip1), "ip1 should be blocked");
        assert!(limiter.check_ip(ip2), "ip2 should still be allowed");
    }
}
