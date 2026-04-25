// SPDX-License-Identifier: PMPL-1.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
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

use serde::Deserialize;

use crate::adapters::Platform;
use crate::config::Config;
use crate::error::Result;
use crate::modes;
use crate::scheduler::{JobPriority, JobScheduler, ProofJob};
use crate::store::Store;
use crate::store::models::ProofJobRecord;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub store: Arc<dyn Store>,
    pub scheduler: Arc<JobScheduler>,
}

/// Create webhook router
pub fn webhook_router() -> Router<AppState> {
    Router::new()
        .route("/webhooks/github", post(handle_github_webhook))
        .route("/webhooks/gitlab", post(handle_gitlab_webhook))
        .route("/webhooks/bitbucket", post(handle_bitbucket_webhook))
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

    // Parse event type + traceability id
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    tracing::info!("GitHub event type: {}", event_type);

    match event_type {
        "push" => {
            tracing::info!("Received push event");
            if let Ok(payload) = serde_json::from_slice::<GitHubPushPayload>(&body) {
                let (owner, name) = split_full_name(&payload.repository.full_name);
                let _ = enqueue_repo_jobs(
                    &state,
                    Platform::GitHub,
                    &owner,
                    &name,
                    &payload.after,
                    JobPriority::Normal,
                    RepoEventKind::Push,
                    None,
                    delivery_id.clone(),
                )
                .await;
            }
        }
        "pull_request" => {
            tracing::info!("Received pull_request event");
            if let Ok(payload) = serde_json::from_slice::<GitHubPullRequestPayload>(&body) {
                let (owner, name) = split_full_name(&payload.repository.full_name);
                let _ = enqueue_repo_jobs(
                    &state,
                    Platform::GitHub,
                    &owner,
                    &name,
                    &payload.pull_request.head.sha,
                    JobPriority::High,
                    RepoEventKind::PullRequest,
                    Some(payload.pull_request.number),
                    delivery_id.clone(),
                )
                .await;
            }
        }
        "check_suite" => {
            tracing::info!("Received check_suite event");
            if let Ok(payload) = serde_json::from_slice::<GitHubCheckSuitePayload>(&body) {
                let (owner, name) = split_full_name(&payload.repository.full_name);
                let _ = enqueue_repo_jobs(
                    &state,
                    Platform::GitHub,
                    &owner,
                    &name,
                    &payload.check_suite.head_sha,
                    JobPriority::High,
                    RepoEventKind::PullRequest,
                    None, // check_suite payload doesn't carry the PR number directly
                    delivery_id.clone(),
                )
                .await;
            }
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

    // Parse event type + traceability id
    let event_type = headers
        .get("X-Gitlab-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let delivery_id = headers
        .get("X-Gitlab-Webhook-UUID")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    tracing::info!("GitLab event type: {}", event_type);

    match event_type {
        "Push Hook" => {
            tracing::info!("Received push hook");
            if let Ok(payload) = serde_json::from_slice::<GitLabPushPayload>(&body) {
                let (owner, name) = split_full_name(&payload.project.path_with_namespace);
                let commit = payload.checkout_sha.unwrap_or(payload.after);
                let _ = enqueue_repo_jobs(
                    &state,
                    Platform::GitLab,
                    &owner,
                    &name,
                    &commit,
                    JobPriority::Normal,
                    RepoEventKind::Push,
                    None,
                    delivery_id.clone(),
                )
                .await;
            }
        }
        "Merge Request Hook" => {
            tracing::info!("Received merge request hook");
            if let Ok(payload) = serde_json::from_slice::<GitLabMergeRequestPayload>(&body) {
                let (owner, name) = split_full_name(&payload.project.path_with_namespace);
                let mr_iid = payload.object_attributes.iid;
                let commit = payload
                    .object_attributes
                    .last_commit
                    .map(|c| c.id)
                    .unwrap_or_else(|| payload.object_attributes.last_commit_id);
                let _ = enqueue_repo_jobs(
                    &state,
                    Platform::GitLab,
                    &owner,
                    &name,
                    &commit,
                    JobPriority::High,
                    RepoEventKind::PullRequest,
                    mr_iid,
                    delivery_id.clone(),
                )
                .await;
            }
        }
        _ => {
            tracing::debug!("Ignoring event type: {}", event_type);
        }
    }

    (StatusCode::OK, "OK")
}

/// Bitbucket webhook handler
async fn handle_bitbucket_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("Received Bitbucket webhook");

    let event_type = headers
        .get("X-Event-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let delivery_id = headers
        .get("X-Hook-UUID")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    tracing::info!("Bitbucket event type: {}", event_type);

    if event_type.starts_with("repo:push") {
        if let Ok(payload) = serde_json::from_slice::<BitbucketPushPayload>(&body) {
            let (owner, name) = split_full_name(&payload.repository.full_name);
            if let Some(commit) = payload
                .push
                .changes
                .first()
                .and_then(|c| c.new_target.as_ref())
                .map(|t| t.hash.clone())
            {
                let _ = enqueue_repo_jobs(
                    &state,
                    Platform::Bitbucket,
                    &owner,
                    &name,
                    &commit,
                    JobPriority::Normal,
                    RepoEventKind::Push,
                    None,
                    delivery_id.clone(),
                )
                .await;
            }
        }
    }

    (StatusCode::OK, "OK")
}

#[derive(Clone, Copy)]
enum RepoEventKind {
    Push,
    PullRequest,
}

/// Enqueue proof jobs for a registered repository.
///
/// `pr_number` is populated for pull_request events (None for push events).
/// Threads through to ProofJob so the result-reporter can comment on the
/// originating PR rather than the commit page.
///
/// `delivery_id` is the platform-specific webhook traceability id —
/// `X-GitHub-Delivery`, `X-Gitlab-Webhook-UUID`, or `X-Hook-UUID` — so a
/// stored job can be correlated back to the exact webhook that produced it.
async fn enqueue_repo_jobs(
    state: &AppState,
    platform: Platform,
    owner: &str,
    name: &str,
    commit: &str,
    priority: JobPriority,
    event_kind: RepoEventKind,
    pr_number: Option<u64>,
    delivery_id: Option<String>,
) -> Result<()> {
    let repo = match state
        .store
        .get_repository_by_name(platform, owner, name)
        .await?
    {
        Some(repo) => repo,
        None => {
            tracing::info!("Repository not registered: {}/{}", owner, name);
            return Ok(());
        }
    };

    if !repo.enabled {
        tracing::info!("Repository {} is disabled", repo.full_name());
        return Ok(());
    }

    // Determine bot mode via cascade:
    //   1. target-repo `.machine_readable/bot_directives/echidnabot.a2ml`
    //      (post-clone resolution — passed as `Some(content)`; not yet
    //      reachable here, so always None until the executor lands)
    //   2. `repositories.mode` column (per-repo)
    //   3. `BotMode::default()` (= Verifier)
    //
    // TODO(phase-3): when the sandboxed executor clones the target repo, it
    // will read the directive file and pass content into resolve_mode.
    let directive_content: Option<&str> = None;
    let mode = modes::resolve_mode(&repo, directive_content);
    let is_pr = matches!(event_kind, RepoEventKind::PullRequest);

    tracing::info!(
        "Bot mode: {} (repo: {}, event: {})",
        mode,
        repo.full_name(),
        if is_pr { "pull_request" } else { "push" },
    );

    // Consultant mode only triggers on explicit @echidnabot mentions
    if !modes::should_auto_trigger(mode, is_pr) {
        tracing::info!(
            "Mode {} does not auto-trigger for this event; skipping",
            mode,
        );
        return Ok(());
    }

    let should_enqueue = match event_kind {
        RepoEventKind::Push => repo.check_on_push,
        RepoEventKind::PullRequest => repo.check_on_pr,
    };

    if !should_enqueue {
        return Ok(());
    }

    for prover in &repo.enabled_provers {
        let job = ProofJob::new(repo.id, commit.to_string(), *prover, Vec::new())
            .with_priority(priority)
            .with_context(pr_number, delivery_id.clone());
        let record = ProofJobRecord::from(job.clone());
        state.store.create_job(&record).await?;
        let _ = state.scheduler.enqueue(job).await?;
    }

    tracing::info!(
        "Enqueued {} job(s) for {} in {} mode",
        repo.enabled_provers.len(),
        repo.full_name(),
        mode,
    );

    Ok(())
}

fn split_full_name(full_name: &str) -> (String, String) {
    let mut parts = full_name.splitn(2, '/');
    let owner = parts.next().unwrap_or_default().to_string();
    let name = parts.next().unwrap_or_default().to_string();
    (owner, name)
}

#[derive(Deserialize)]
struct GitHubPushPayload {
    after: String,
    repository: GitHubRepo,
}

#[derive(Deserialize)]
struct GitHubPullRequestPayload {
    pull_request: GitHubPullRequest,
    repository: GitHubRepo,
}

#[derive(Deserialize)]
struct GitHubCheckSuitePayload {
    check_suite: GitHubCheckSuite,
    repository: GitHubRepo,
}

#[derive(Deserialize)]
struct GitHubRepo {
    full_name: String,
}

#[derive(Deserialize)]
struct GitHubPullRequest {
    /// PR number — used to comment back on the originating PR rather
    /// than the commit page.
    number: u64,
    head: GitHubHead,
}

#[derive(Deserialize)]
struct GitHubCheckSuite {
    head_sha: String,
}

#[derive(Deserialize)]
struct GitHubHead {
    sha: String,
}

#[derive(Deserialize)]
struct GitLabPushPayload {
    after: String,
    checkout_sha: Option<String>,
    project: GitLabProject,
}

#[derive(Deserialize)]
struct GitLabMergeRequestPayload {
    object_attributes: GitLabMergeAttributes,
    project: GitLabProject,
}

#[derive(Deserialize)]
struct GitLabMergeAttributes {
    last_commit_id: String,
    last_commit: Option<GitLabCommit>,
    /// GitLab's per-project MR identifier (the human-facing !N number).
    /// Equivalent to GitHub's PR number for plumbing purposes.
    iid: Option<u64>,
}

#[derive(Deserialize)]
struct GitLabCommit {
    id: String,
}

#[derive(Deserialize)]
struct GitLabProject {
    path_with_namespace: String,
}

#[derive(Deserialize)]
struct BitbucketPushPayload {
    repository: BitbucketRepo,
    push: BitbucketPush,
}

#[derive(Deserialize)]
struct BitbucketRepo {
    full_name: String,
}

#[derive(Deserialize)]
struct BitbucketPush {
    changes: Vec<BitbucketChange>,
}

#[derive(Deserialize)]
struct BitbucketChange {
    #[serde(rename = "new")]
    new_target: Option<BitbucketTarget>,
}

#[derive(Deserialize)]
struct BitbucketTarget {
    hash: String,
}

/// Verify GitHub webhook signature (HMAC-SHA256)
fn verify_github_signature(
    headers: &HeaderMap,
    body: &Bytes,
    secret: &str,
) -> std::result::Result<(), String> {
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Missing X-Hub-Signature-256 header".to_string())?;

    // Signature format: "sha256=<hex>"
    let signature = signature
        .strip_prefix("sha256=")
        .ok_or_else(|| "Invalid signature format".to_string())?;

    let signature_bytes =
        hex::decode(signature).map_err(|_| "Invalid hex in signature".to_string())?;

    // Compute expected signature
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| "Invalid secret key".to_string())?;
    mac.update(body);

    mac.verify_slice(&signature_bytes)
        .map_err(|_| "Signature mismatch".to_string())?;

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
