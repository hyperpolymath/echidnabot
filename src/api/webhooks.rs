//! Webhook handlers for GitHub, GitLab, and Bitbucket

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

use crate::config::Config;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

/// Create webhook router
pub fn webhook_router(state: AppState) -> Router {
    Router::new()
        .route("/webhooks/github", post(handle_github_webhook))
        .route("/webhooks/gitlab", post(handle_gitlab_webhook))
        .route("/webhooks/bitbucket", post(handle_bitbucket_webhook))
        .with_state(state)
}

/// GitHub webhook handler
async fn handle_github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("Received GitHub webhook");

    // Verify signature if secret is configured
    if let Some(ref gh_config) = state.config.github {
        if let Some(ref secret) = gh_config.webhook_secret {
            if let Err(e) = verify_github_signature(&headers, &body, secret) {
                tracing::warn!("GitHub webhook signature verification failed: {}", e);
                return (StatusCode::UNAUTHORIZED, "Invalid signature");
            }
        }
    }

    // Parse event type
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!("GitHub event type: {}", event_type);

    match event_type {
        "push" => {
            // TODO: Parse push payload, extract commits, queue proof checks
            tracing::info!("Received push event");
        }
        "pull_request" => {
            // TODO: Parse PR payload, queue proof checks
            tracing::info!("Received pull_request event");
        }
        "check_suite" => {
            // TODO: Handle check suite requests
            tracing::info!("Received check_suite event");
        }
        "ping" => {
            tracing::info!("Received ping event - webhook configured correctly");
        }
        _ => {
            tracing::debug!("Ignoring event type: {}", event_type);
        }
    }

    (StatusCode::OK, "OK")
}

/// GitLab webhook handler
async fn handle_gitlab_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("Received GitLab webhook");

    // Verify token if configured
    if let Some(ref gl_config) = state.config.gitlab {
        if let Some(ref secret) = gl_config.webhook_secret {
            let token = headers
                .get("X-Gitlab-Token")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if token != secret {
                tracing::warn!("GitLab webhook token mismatch");
                return (StatusCode::UNAUTHORIZED, "Invalid token");
            }
        }
    }

    // Parse event type
    let event_type = headers
        .get("X-Gitlab-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!("GitLab event type: {}", event_type);

    match event_type {
        "Push Hook" => {
            tracing::info!("Received push hook");
            // TODO: Parse and queue
        }
        "Merge Request Hook" => {
            tracing::info!("Received merge request hook");
            // TODO: Parse and queue
        }
        _ => {
            tracing::debug!("Ignoring event type: {}", event_type);
        }
    }

    let _ = body; // Silence unused warning until implemented

    (StatusCode::OK, "OK")
}

/// Bitbucket webhook handler
async fn handle_bitbucket_webhook(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("Received Bitbucket webhook");

    let event_type = headers
        .get("X-Event-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!("Bitbucket event type: {}", event_type);

    // TODO: Implement Bitbucket webhook handling
    let _ = body;

    (StatusCode::OK, "OK")
}

/// Verify GitHub webhook signature (HMAC-SHA256)
fn verify_github_signature(headers: &HeaderMap, body: &Bytes, secret: &str) -> Result<(), String> {
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing X-Hub-Signature-256 header")?;

    // Signature format: "sha256=<hex>"
    let signature = signature
        .strip_prefix("sha256=")
        .ok_or("Invalid signature format")?;

    let signature_bytes = hex::decode(signature).map_err(|_| "Invalid hex in signature")?;

    // Compute expected signature
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| "Invalid secret key")?;
    mac.update(body);

    mac.verify_slice(&signature_bytes)
        .map_err(|_| "Signature mismatch")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_github_signature() {
        let secret = "test-secret";
        let body = Bytes::from(r#"{"test": "payload"}"#);

        // Compute expected signature
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(&body);
        let expected = hex::encode(mac.finalize().into_bytes());

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            format!("sha256={}", expected).parse().unwrap(),
        );

        assert!(verify_github_signature(&headers, &body, secret).is_ok());
    }
}
