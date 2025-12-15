//! Webhook handlers for GitHub, GitLab, and Bitbucket

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;

use crate::adapters::Platform;
use crate::config::Config;
use crate::dispatcher::ProverKind;
use crate::scheduler::{JobPriority, JobScheduler, ProofJob};
use crate::store::Store;

/// Application state shared across webhook handlers
#[derive(Clone)]
pub struct WebhookState {
    pub config: Arc<Config>,
    pub store: Arc<dyn Store>,
    pub scheduler: Arc<JobScheduler>,
}

// =============================================================================
// GitHub Webhook Types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct GitHubPushEvent {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub after: String, // commit SHA
    pub before: String,
    pub repository: GitHubRepository,
    pub commits: Vec<GitHubCommit>,
    pub head_commit: Option<GitHubCommit>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubOwner,
    pub default_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCommit {
    pub id: String,
    pub message: String,
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPullRequestEvent {
    pub action: String,
    pub number: u64,
    pub pull_request: GitHubPullRequest,
    pub repository: GitHubRepository,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub head: GitHubPRHead,
    pub base: GitHubPRBase,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPRHead {
    pub sha: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPRBase {
    #[serde(rename = "ref")]
    pub git_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCheckSuiteEvent {
    pub action: String,
    pub check_suite: GitHubCheckSuite,
    pub repository: GitHubRepository,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCheckSuite {
    pub id: u64,
    pub head_sha: String,
    pub head_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPingEvent {
    pub zen: String,
    pub hook_id: u64,
    pub repository: Option<GitHubRepository>,
}

// =============================================================================
// GitLab Webhook Types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct GitLabPushEvent {
    pub object_kind: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub checkout_sha: Option<String>,
    pub after: String,
    pub before: String,
    pub project: GitLabProject,
    pub commits: Vec<GitLabCommit>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub default_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabCommit {
    pub id: String,
    pub message: String,
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabMergeRequestEvent {
    pub object_kind: String,
    pub event_type: String,
    pub project: GitLabProject,
    pub object_attributes: GitLabMergeRequestAttributes,
}

#[derive(Debug, Deserialize)]
pub struct GitLabMergeRequestAttributes {
    pub id: u64,
    pub iid: u64,
    pub action: Option<String>,
    pub state: String,
    pub last_commit: GitLabLastCommit,
}

#[derive(Debug, Deserialize)]
pub struct GitLabLastCommit {
    pub id: String,
}

// =============================================================================
// Bitbucket Webhook Types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct BitbucketPushEvent {
    pub push: BitbucketPushData,
    pub repository: BitbucketRepository,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketPushData {
    pub changes: Vec<BitbucketChange>,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketChange {
    pub new: Option<BitbucketRef>,
    pub old: Option<BitbucketRef>,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketRef {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub name: String,
    pub target: BitbucketTarget,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketTarget {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketRepository {
    pub uuid: String,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketPullRequestEvent {
    pub pullrequest: BitbucketPullRequest,
    pub repository: BitbucketRepository,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketPullRequest {
    pub id: u64,
    pub source: BitbucketPRSource,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketPRSource {
    pub commit: BitbucketPRCommit,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketPRCommit {
    pub hash: String,
}

// =============================================================================
// GitHub Webhook Handler
// =============================================================================

pub async fn handle_github_webhook<S>(
    State(state): State<S>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse
where
    S: AsRef<WebhookState> + Clone + Send + Sync,
{
    let webhook_state = state.as_ref();
    tracing::info!("Received GitHub webhook");

    // Verify signature if secret is configured
    if let Some(ref gh_config) = webhook_state.config.github {
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
            match serde_json::from_slice::<GitHubPushEvent>(&body) {
                Ok(event) => {
                    if let Err(e) = handle_github_push(webhook_state, event).await {
                        tracing::error!("Failed to handle GitHub push: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse GitHub push event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        "pull_request" => {
            match serde_json::from_slice::<GitHubPullRequestEvent>(&body) {
                Ok(event) => {
                    if let Err(e) = handle_github_pull_request(webhook_state, event).await {
                        tracing::error!("Failed to handle GitHub PR: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse GitHub PR event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        "check_suite" => {
            match serde_json::from_slice::<GitHubCheckSuiteEvent>(&body) {
                Ok(event) => {
                    if event.action == "requested" || event.action == "rerequested" {
                        if let Err(e) = handle_github_check_suite(webhook_state, event).await {
                            tracing::error!("Failed to handle GitHub check suite: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse GitHub check suite event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        "ping" => {
            match serde_json::from_slice::<GitHubPingEvent>(&body) {
                Ok(event) => {
                    tracing::info!("Received ping event - {}", event.zen);
                    if let Some(repo) = event.repository {
                        tracing::info!("Webhook configured for: {}", repo.full_name);
                    }
                }
                Err(e) => {
                    tracing::warn!("Could not parse ping event: {}", e);
                }
            }
        }
        _ => {
            tracing::debug!("Ignoring event type: {}", event_type);
        }
    }

    (StatusCode::OK, "OK")
}

async fn handle_github_push(state: &WebhookState, event: GitHubPushEvent) -> crate::Result<()> {
    let owner = &event.repository.owner.login;
    let name = &event.repository.name;
    let commit_sha = &event.after;

    tracing::info!(
        "Processing push to {}/{} at commit {}",
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    // Find repository in our database
    let repo = match state
        .store
        .get_repository_by_name(Platform::GitHub, owner, name)
        .await?
    {
        Some(r) if r.enabled && r.check_on_push => r,
        Some(r) if !r.enabled => {
            tracing::debug!("Repository {} is disabled", event.repository.full_name);
            return Ok(());
        }
        Some(_) => {
            tracing::debug!("Push checks disabled for {}", event.repository.full_name);
            return Ok(());
        }
        None => {
            tracing::debug!("Repository {} not registered", event.repository.full_name);
            return Ok(());
        }
    };

    // Collect changed files
    let changed_files: Vec<String> = event
        .commits
        .iter()
        .flat_map(|c| {
            c.added
                .iter()
                .chain(c.modified.iter())
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect();

    // Filter to proof files and determine provers
    let proof_files = filter_proof_files(&changed_files, &repo.enabled_provers);

    if proof_files.is_empty() {
        tracing::debug!("No proof files changed in push");
        return Ok(());
    }

    tracing::info!("Found {} proof files to check", proof_files.len());

    // Create jobs for each prover that has relevant files
    for prover in &repo.enabled_provers {
        let prover_files: Vec<String> = proof_files
            .iter()
            .filter(|f| file_matches_prover(f, *prover))
            .cloned()
            .collect();

        if prover_files.is_empty() {
            continue;
        }

        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, prover_files)
            .with_priority(JobPriority::Normal);

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created job {} for {:?}", job_id, prover);
            // Persist to database
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

async fn handle_github_pull_request(
    state: &WebhookState,
    event: GitHubPullRequestEvent,
) -> crate::Result<()> {
    // Only process opened, synchronize, or reopened actions
    if !["opened", "synchronize", "reopened"].contains(&event.action.as_str()) {
        tracing::debug!("Ignoring PR action: {}", event.action);
        return Ok(());
    }

    let owner = &event.repository.owner.login;
    let name = &event.repository.name;
    let commit_sha = &event.pull_request.head.sha;

    tracing::info!(
        "Processing PR #{} to {}/{} at commit {}",
        event.number,
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    // Find repository
    let repo = match state
        .store
        .get_repository_by_name(Platform::GitHub, owner, name)
        .await?
    {
        Some(r) if r.enabled && r.check_on_pr => r,
        Some(r) if !r.enabled => {
            tracing::debug!("Repository {} is disabled", event.repository.full_name);
            return Ok(());
        }
        Some(_) => {
            tracing::debug!("PR checks disabled for {}", event.repository.full_name);
            return Ok(());
        }
        None => {
            tracing::debug!("Repository {} not registered", event.repository.full_name);
            return Ok(());
        }
    };

    // For PRs, we check all enabled provers (we don't have the file diff here)
    for prover in &repo.enabled_provers {
        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, vec![])
            .with_priority(JobPriority::High); // PRs get high priority

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created PR job {} for {:?}", job_id, prover);
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

async fn handle_github_check_suite(
    state: &WebhookState,
    event: GitHubCheckSuiteEvent,
) -> crate::Result<()> {
    let owner = &event.repository.owner.login;
    let name = &event.repository.name;
    let commit_sha = &event.check_suite.head_sha;

    tracing::info!(
        "Processing check suite for {}/{} at commit {}",
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    // Find repository
    let repo = match state
        .store
        .get_repository_by_name(Platform::GitHub, owner, name)
        .await?
    {
        Some(r) if r.enabled => r,
        _ => {
            tracing::debug!("Repository {} not registered or disabled", event.repository.full_name);
            return Ok(());
        }
    };

    // Create jobs for all enabled provers
    for prover in &repo.enabled_provers {
        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, vec![])
            .with_priority(JobPriority::High);

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created check suite job {} for {:?}", job_id, prover);
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

// =============================================================================
// GitLab Webhook Handler
// =============================================================================

pub async fn handle_gitlab_webhook<S>(
    State(state): State<S>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse
where
    S: AsRef<WebhookState> + Clone + Send + Sync,
{
    let webhook_state = state.as_ref();
    tracing::info!("Received GitLab webhook");

    // Verify token if configured
    if let Some(ref gl_config) = webhook_state.config.gitlab {
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
            match serde_json::from_slice::<GitLabPushEvent>(&body) {
                Ok(event) => {
                    if let Err(e) = handle_gitlab_push(webhook_state, event).await {
                        tracing::error!("Failed to handle GitLab push: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse GitLab push event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        "Merge Request Hook" => {
            match serde_json::from_slice::<GitLabMergeRequestEvent>(&body) {
                Ok(event) => {
                    if let Err(e) = handle_gitlab_merge_request(webhook_state, event).await {
                        tracing::error!("Failed to handle GitLab MR: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse GitLab MR event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        _ => {
            tracing::debug!("Ignoring GitLab event type: {}", event_type);
        }
    }

    (StatusCode::OK, "OK")
}

async fn handle_gitlab_push(state: &WebhookState, event: GitLabPushEvent) -> crate::Result<()> {
    // Parse owner/name from path_with_namespace
    let parts: Vec<&str> = event.project.path_with_namespace.split('/').collect();
    if parts.len() < 2 {
        tracing::warn!("Invalid GitLab project path: {}", event.project.path_with_namespace);
        return Ok(());
    }
    let owner = parts[0];
    let name = parts[parts.len() - 1];
    let commit_sha = event.checkout_sha.as_ref().unwrap_or(&event.after);

    tracing::info!(
        "Processing GitLab push to {}/{} at commit {}",
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    let repo = match state
        .store
        .get_repository_by_name(Platform::GitLab, owner, name)
        .await?
    {
        Some(r) if r.enabled && r.check_on_push => r,
        _ => {
            tracing::debug!("Repository {} not registered or disabled", event.project.path_with_namespace);
            return Ok(());
        }
    };

    // Collect changed files
    let changed_files: Vec<String> = event
        .commits
        .iter()
        .flat_map(|c| {
            c.added
                .iter()
                .chain(c.modified.iter())
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect();

    let proof_files = filter_proof_files(&changed_files, &repo.enabled_provers);

    if proof_files.is_empty() {
        tracing::debug!("No proof files changed in GitLab push");
        return Ok(());
    }

    for prover in &repo.enabled_provers {
        let prover_files: Vec<String> = proof_files
            .iter()
            .filter(|f| file_matches_prover(f, *prover))
            .cloned()
            .collect();

        if prover_files.is_empty() {
            continue;
        }

        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, prover_files)
            .with_priority(JobPriority::Normal);

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created GitLab job {} for {:?}", job_id, prover);
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

async fn handle_gitlab_merge_request(
    state: &WebhookState,
    event: GitLabMergeRequestEvent,
) -> crate::Result<()> {
    // Only process open/update actions
    let action = event.object_attributes.action.as_deref().unwrap_or("");
    if !["open", "update", "reopen"].contains(&action) && event.object_attributes.state != "opened" {
        tracing::debug!("Ignoring GitLab MR action: {:?}", event.object_attributes.action);
        return Ok(());
    }

    let parts: Vec<&str> = event.project.path_with_namespace.split('/').collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let owner = parts[0];
    let name = parts[parts.len() - 1];
    let commit_sha = &event.object_attributes.last_commit.id;

    tracing::info!(
        "Processing GitLab MR !{} to {}/{} at commit {}",
        event.object_attributes.iid,
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    let repo = match state
        .store
        .get_repository_by_name(Platform::GitLab, owner, name)
        .await?
    {
        Some(r) if r.enabled && r.check_on_pr => r,
        _ => return Ok(()),
    };

    for prover in &repo.enabled_provers {
        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, vec![])
            .with_priority(JobPriority::High);

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created GitLab MR job {} for {:?}", job_id, prover);
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

// =============================================================================
// Bitbucket Webhook Handler
// =============================================================================

pub async fn handle_bitbucket_webhook<S>(
    State(state): State<S>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse
where
    S: AsRef<WebhookState> + Clone + Send + Sync,
{
    let webhook_state = state.as_ref();
    tracing::info!("Received Bitbucket webhook");

    let event_type = headers
        .get("X-Event-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!("Bitbucket event type: {}", event_type);

    match event_type {
        "repo:push" => {
            match serde_json::from_slice::<BitbucketPushEvent>(&body) {
                Ok(event) => {
                    if let Err(e) = handle_bitbucket_push(webhook_state, event).await {
                        tracing::error!("Failed to handle Bitbucket push: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse Bitbucket push event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        "pullrequest:created" | "pullrequest:updated" => {
            match serde_json::from_slice::<BitbucketPullRequestEvent>(&body) {
                Ok(event) => {
                    if let Err(e) = handle_bitbucket_pull_request(webhook_state, event).await {
                        tracing::error!("Failed to handle Bitbucket PR: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Processing failed");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse Bitbucket PR event: {}", e);
                    return (StatusCode::BAD_REQUEST, "Invalid payload");
                }
            }
        }
        _ => {
            tracing::debug!("Ignoring Bitbucket event type: {}", event_type);
        }
    }

    (StatusCode::OK, "OK")
}

async fn handle_bitbucket_push(
    state: &WebhookState,
    event: BitbucketPushEvent,
) -> crate::Result<()> {
    let parts: Vec<&str> = event.repository.full_name.split('/').collect();
    if parts.len() != 2 {
        return Ok(());
    }
    let (owner, name) = (parts[0], parts[1]);

    // Get the latest commit from the push
    let commit_sha = event
        .push
        .changes
        .first()
        .and_then(|c| c.new.as_ref())
        .map(|r| r.target.hash.clone());

    let commit_sha = match commit_sha {
        Some(sha) => sha,
        None => {
            tracing::debug!("No new commit in Bitbucket push");
            return Ok(());
        }
    };

    tracing::info!(
        "Processing Bitbucket push to {}/{} at commit {}",
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    let repo = match state
        .store
        .get_repository_by_name(Platform::Bitbucket, owner, name)
        .await?
    {
        Some(r) if r.enabled && r.check_on_push => r,
        _ => return Ok(()),
    };

    for prover in &repo.enabled_provers {
        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, vec![])
            .with_priority(JobPriority::Normal);

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created Bitbucket job {} for {:?}", job_id, prover);
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

async fn handle_bitbucket_pull_request(
    state: &WebhookState,
    event: BitbucketPullRequestEvent,
) -> crate::Result<()> {
    let parts: Vec<&str> = event.repository.full_name.split('/').collect();
    if parts.len() != 2 {
        return Ok(());
    }
    let (owner, name) = (parts[0], parts[1]);
    let commit_sha = &event.pullrequest.source.commit.hash;

    tracing::info!(
        "Processing Bitbucket PR #{} to {}/{} at commit {}",
        event.pullrequest.id,
        owner,
        name,
        &commit_sha[..8.min(commit_sha.len())]
    );

    let repo = match state
        .store
        .get_repository_by_name(Platform::Bitbucket, owner, name)
        .await?
    {
        Some(r) if r.enabled && r.check_on_pr => r,
        _ => return Ok(()),
    };

    for prover in &repo.enabled_provers {
        let job = ProofJob::new(repo.id, commit_sha.clone(), *prover, vec![])
            .with_priority(JobPriority::High);

        if let Some(job_id) = state.scheduler.enqueue(job.clone()).await? {
            tracing::info!("Created Bitbucket PR job {} for {:?}", job_id, prover);
            state.store.create_job(&job.into()).await?;
        }
    }

    Ok(())
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Verify GitHub webhook signature (HMAC-SHA256)
fn verify_github_signature(headers: &HeaderMap, body: &Bytes, secret: &str) -> Result<(), String> {
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing X-Hub-Signature-256 header")?;

    let signature = signature
        .strip_prefix("sha256=")
        .ok_or("Invalid signature format")?;

    let signature_bytes = hex::decode(signature).map_err(|_| "Invalid hex in signature")?;

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| "Invalid secret key")?;
    mac.update(body);

    mac.verify_slice(&signature_bytes)
        .map_err(|_| "Signature mismatch")?;

    Ok(())
}

/// Filter files to only those that match enabled provers
fn filter_proof_files(files: &[String], enabled_provers: &[ProverKind]) -> Vec<String> {
    files
        .iter()
        .filter(|f| {
            enabled_provers
                .iter()
                .any(|p| file_matches_prover(f, *p))
        })
        .cloned()
        .collect()
}

/// Check if a file matches a prover based on extension
fn file_matches_prover(file: &str, prover: ProverKind) -> bool {
    let file_lower = file.to_lowercase();
    prover
        .file_extensions()
        .iter()
        .any(|ext| file_lower.ends_with(ext))
}

// =============================================================================
// Router
// =============================================================================

/// Create the webhook router
pub fn webhook_router<S>() -> axum::Router<S>
where
    S: AsRef<WebhookState> + Clone + Send + Sync + 'static,
{
    use axum::routing::post;

    axum::Router::new()
        .route("/webhooks/github", post(handle_github_webhook::<S>))
        .route("/webhooks/gitlab", post(handle_gitlab_webhook::<S>))
        .route("/webhooks/bitbucket", post(handle_bitbucket_webhook::<S>))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_github_signature() {
        let secret = "test-secret";
        let body = Bytes::from(r#"{"test": "payload"}"#);

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

    #[test]
    fn test_file_matches_prover() {
        assert!(file_matches_prover("test.mm", ProverKind::Metamath));
        assert!(file_matches_prover("src/proof.lean", ProverKind::Lean));
        assert!(file_matches_prover("Theorem.v", ProverKind::Coq));
        assert!(!file_matches_prover("readme.md", ProverKind::Metamath));
    }

    #[test]
    fn test_filter_proof_files() {
        let files = vec![
            "src/main.rs".to_string(),
            "proofs/test.mm".to_string(),
            "README.md".to_string(),
            "theorems/foo.lean".to_string(),
        ];

        let provers = vec![ProverKind::Metamath, ProverKind::Lean];
        let filtered = filter_proof_files(&files, &provers);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"proofs/test.mm".to_string()));
        assert!(filtered.contains(&"theorems/foo.lean".to_string()));
    }
}
