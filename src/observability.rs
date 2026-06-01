// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Observability: structured logging initialisation.
//!
//! Centralises `tracing-subscriber` setup so the CLI entry point, the
//! webhook server, and any future bin/example targets all init the same
//! way. Two output formats are supported, selected via env var:
//!
//! * `ECHIDNABOT_LOG_FORMAT=text` (default) — human-friendly `fmt` layer,
//!   colour-aware on TTYs.
//! * `ECHIDNABOT_LOG_FORMAT=json` — structured JSON with flattened event
//!   fields, suitable for log aggregators (Loki, ELK, CloudWatch, etc.).
//!
//! The `RUST_LOG` env var works as usual via `EnvFilter`. A fallback
//! directive is taken from the caller (typically `"info"` or `"debug"`)
//! when `RUST_LOG` is unset or unparseable.
//!
//! # Coordination
//!
//! When the OpenTelemetry layer lands (roadmap "Production Hardening"),
//! it is expected to register an additional `tracing-subscriber` layer
//! on top of the format layer chosen here. The init function returns a
//! plain `()` for now; the OpenTelemetry agent should adapt the signature
//! to return its `TracerShutdown` guard without disturbing the format
//! selector.
//!
//! # Example
//!
//! ```no_run
//! use echidnabot::observability;
//!
//! // CLI / server entry point
//! observability::init_tracing("info");
//! tracing::info!(repo = "owner/name", "registered");
//! ```

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Env var that selects the log output format.
pub const FORMAT_ENV_VAR: &str = "ECHIDNABOT_LOG_FORMAT";

/// Supported log output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-friendly, colour-aware `fmt` output (default).
    Text,
    /// Structured JSON with flattened event fields.
    Json,
}

impl LogFormat {
    /// Resolve the format from the [`FORMAT_ENV_VAR`] environment variable.
    ///
    /// Unknown / unset values fall back to [`LogFormat::Text`]. Comparison
    /// is case-insensitive so `JSON`, `Json`, and `json` all parse the
    /// same.
    pub fn from_env() -> Self {
        match std::env::var(FORMAT_ENV_VAR)
            .ok()
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Some("json") => LogFormat::Json,
            _ => LogFormat::Text,
        }
    }
}

/// Initialise the global `tracing` subscriber.
///
/// * Filter directive is `RUST_LOG` if set + parseable, otherwise the
///   `fallback_filter` argument (e.g. `"info"` or `"debug"` — the same
///   strings accepted by `EnvFilter::new`).
/// * Output format follows `ECHIDNABOT_LOG_FORMAT`
///   (see [`LogFormat::from_env`]).
///
/// Returns silently on success. Errors during subscriber installation
/// (typically: a global subscriber was already installed) are swallowed
/// so that callers in test contexts can call this idempotently without
/// panicking.
pub fn init_tracing(fallback_filter: &str) {
    init_tracing_with_format(fallback_filter, LogFormat::from_env());
}

/// Same as [`init_tracing`] but with an explicit format selector.
///
/// Useful from tests that want to exercise both branches without
/// mutating process-wide env state.
pub fn init_tracing_with_format(fallback_filter: &str, format: LogFormat) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(fallback_filter));

    // Two arms compose distinct layer types and call `try_init` on each
    // independently. Unifying via `Box<dyn Layer<_>>` does not work here
    // because the resulting `Layered<Box<dyn Layer<_>>, _>` does not
    // implement `SubscriberInitExt` in tracing-subscriber 0.3.
    // `try_init` (not `init`) so tests / repeated calls don't panic;
    // the second call is a no-op (returns Err) which is deliberately
    // ignored.
    match format {
        LogFormat::Json => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_span_list(false);
            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(json_layer)
                .try_init();
        }
        LogFormat::Text => {
            let text_layer = tracing_subscriber::fmt::layer().compact();
            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(text_layer)
                .try_init();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_defaults_to_text_when_unset() {
        // Save + clear the env var so this test is isolated from caller
        // environment. Restore at end.
        let prev = std::env::var(FORMAT_ENV_VAR).ok();
        // SAFETY: tests are single-threaded per Rust default test harness
        // for env-var ops; this is the standard pattern for env-driven
        // unit tests in this crate.
        unsafe { std::env::remove_var(FORMAT_ENV_VAR); }
        assert_eq!(LogFormat::from_env(), LogFormat::Text);
        if let Some(v) = prev {
            unsafe { std::env::set_var(FORMAT_ENV_VAR, v); }
        }
    }

    #[test]
    fn log_format_recognises_json_case_insensitive() {
        let prev = std::env::var(FORMAT_ENV_VAR).ok();
        for v in ["json", "JSON", "Json", "jSoN"] {
            unsafe { std::env::set_var(FORMAT_ENV_VAR, v); }
            assert_eq!(LogFormat::from_env(), LogFormat::Json, "input was {v}");
        }
        match prev {
            Some(v) => unsafe { std::env::set_var(FORMAT_ENV_VAR, v) },
            None => unsafe { std::env::remove_var(FORMAT_ENV_VAR) },
        }
    }

    #[test]
    fn log_format_unknown_falls_back_to_text() {
        let prev = std::env::var(FORMAT_ENV_VAR).ok();
        unsafe { std::env::set_var(FORMAT_ENV_VAR, "yaml"); }
        assert_eq!(LogFormat::from_env(), LogFormat::Text);
        match prev {
            Some(v) => unsafe { std::env::set_var(FORMAT_ENV_VAR, v) },
            None => unsafe { std::env::remove_var(FORMAT_ENV_VAR) },
        }
    }

    #[test]
    fn init_text_does_not_panic() {
        // Idempotent: second-and-later calls inside the test process
        // return Err from try_init but don't panic.
        init_tracing_with_format("info", LogFormat::Text);
        init_tracing_with_format("debug", LogFormat::Text);
    }

    #[test]
    fn init_json_does_not_panic() {
        init_tracing_with_format("info", LogFormat::Json);
        init_tracing_with_format("trace", LogFormat::Json);
    }

    #[test]
    fn init_tracing_env_path_does_not_panic() {
        init_tracing("info");
    }
}
