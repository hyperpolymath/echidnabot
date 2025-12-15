//! GraphQL schema and resolvers

use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, ID};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::adapters::Platform as AdapterPlatform;
use crate::config::Config;
use crate::dispatcher::{
    EchidnaClient, ProverKind as DispatcherProverKind,
    echidna_client::ProverStatus as EchidnaProverStatus,
};
use crate::scheduler::{JobId, JobPriority, JobScheduler, JobStatus as SchedulerJobStatus, ProofJob as SchedulerProofJob};
use crate::store::{models::Repository as StoreRepository, Store};

/// Application context shared across GraphQL resolvers
#[derive(Clone)]
pub struct AppContext {
    pub store: Arc<dyn Store>,
    pub scheduler: Arc<JobScheduler>,
    pub echidna: Arc<EchidnaClient>,
    pub config: Arc<Config>,
}

/// GraphQL schema type
pub type EchidnabotSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Create the GraphQL schema with application context
pub fn create_schema(ctx: AppContext) -> EchidnabotSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .finish()
}

// =============================================================================
// GraphQL Types
// =============================================================================

/// Platform enum for GraphQL
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Platform {
    GitHub,
    GitLab,
    Bitbucket,
    Codeberg,
}

impl From<AdapterPlatform> for Platform {
    fn from(p: AdapterPlatform) -> Self {
        match p {
            AdapterPlatform::GitHub => Platform::GitHub,
            AdapterPlatform::GitLab => Platform::GitLab,
            AdapterPlatform::Bitbucket => Platform::Bitbucket,
            AdapterPlatform::Codeberg => Platform::Codeberg,
        }
    }
}

impl From<Platform> for AdapterPlatform {
    fn from(p: Platform) -> Self {
        match p {
            Platform::GitHub => AdapterPlatform::GitHub,
            Platform::GitLab => AdapterPlatform::GitLab,
            Platform::Bitbucket => AdapterPlatform::Bitbucket,
            Platform::Codeberg => AdapterPlatform::Codeberg,
        }
    }
}

/// Prover kind enum for GraphQL
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProverKind {
    Agda,
    Coq,
    Lean,
    Isabelle,
    Z3,
    Cvc5,
    Metamath,
    HolLight,
    Mizar,
    Pvs,
    Acl2,
    Hol4,
}

impl From<DispatcherProverKind> for ProverKind {
    fn from(p: DispatcherProverKind) -> Self {
        match p {
            DispatcherProverKind::Agda => ProverKind::Agda,
            DispatcherProverKind::Coq => ProverKind::Coq,
            DispatcherProverKind::Lean => ProverKind::Lean,
            DispatcherProverKind::Isabelle => ProverKind::Isabelle,
            DispatcherProverKind::Z3 => ProverKind::Z3,
            DispatcherProverKind::Cvc5 => ProverKind::Cvc5,
            DispatcherProverKind::Metamath => ProverKind::Metamath,
            DispatcherProverKind::HolLight => ProverKind::HolLight,
            DispatcherProverKind::Mizar => ProverKind::Mizar,
            DispatcherProverKind::Pvs => ProverKind::Pvs,
            DispatcherProverKind::Acl2 => ProverKind::Acl2,
            DispatcherProverKind::Hol4 => ProverKind::Hol4,
        }
    }
}

impl From<ProverKind> for DispatcherProverKind {
    fn from(p: ProverKind) -> Self {
        match p {
            ProverKind::Agda => DispatcherProverKind::Agda,
            ProverKind::Coq => DispatcherProverKind::Coq,
            ProverKind::Lean => DispatcherProverKind::Lean,
            ProverKind::Isabelle => DispatcherProverKind::Isabelle,
            ProverKind::Z3 => DispatcherProverKind::Z3,
            ProverKind::Cvc5 => DispatcherProverKind::Cvc5,
            ProverKind::Metamath => DispatcherProverKind::Metamath,
            ProverKind::HolLight => DispatcherProverKind::HolLight,
            ProverKind::Mizar => DispatcherProverKind::Mizar,
            ProverKind::Pvs => DispatcherProverKind::Pvs,
            ProverKind::Acl2 => DispatcherProverKind::Acl2,
            ProverKind::Hol4 => DispatcherProverKind::Hol4,
        }
    }
}

/// Job status enum for GraphQL
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<SchedulerJobStatus> for JobStatus {
    fn from(s: SchedulerJobStatus) -> Self {
        match s {
            SchedulerJobStatus::Queued => JobStatus::Queued,
            SchedulerJobStatus::Running => JobStatus::Running,
            SchedulerJobStatus::Completed => JobStatus::Completed,
            SchedulerJobStatus::Failed => JobStatus::Failed,
            SchedulerJobStatus::Cancelled => JobStatus::Cancelled,
        }
    }
}

/// Proof verification status for GraphQL
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProofStatus {
    Verified,
    Failed,
    Timeout,
    Error,
    Unknown,
}

/// Prover availability status for GraphQL
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProverStatus {
    Available,
    Degraded,
    Unavailable,
    Unknown,
}

impl From<EchidnaProverStatus> for ProverStatus {
    fn from(s: EchidnaProverStatus) -> Self {
        match s {
            EchidnaProverStatus::Available => ProverStatus::Available,
            EchidnaProverStatus::Degraded => ProverStatus::Degraded,
            EchidnaProverStatus::Unavailable => ProverStatus::Unavailable,
            EchidnaProverStatus::Unknown => ProverStatus::Unknown,
        }
    }
}

/// Repository information for GraphQL
#[derive(SimpleObject, Clone)]
pub struct Repository {
    pub id: ID,
    pub platform: Platform,
    pub owner: String,
    pub name: String,
    pub enabled_provers: Vec<ProverKind>,
    pub check_on_push: bool,
    pub check_on_pr: bool,
    pub auto_comment: bool,
    pub enabled: bool,
    pub last_checked_commit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<StoreRepository> for Repository {
    fn from(r: StoreRepository) -> Self {
        Repository {
            id: ID(r.id.to_string()),
            platform: r.platform.into(),
            owner: r.owner,
            name: r.name,
            enabled_provers: r.enabled_provers.into_iter().map(Into::into).collect(),
            check_on_push: r.check_on_push,
            check_on_pr: r.check_on_pr,
            auto_comment: r.auto_comment,
            enabled: r.enabled,
            last_checked_commit: r.last_checked_commit,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Proof job information for GraphQL
#[derive(SimpleObject, Clone)]
pub struct ProofJob {
    pub id: ID,
    pub repo_id: ID,
    pub commit_sha: String,
    pub prover: ProverKind,
    pub file_paths: Vec<String>,
    pub status: JobStatus,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<SchedulerProofJob> for ProofJob {
    fn from(j: SchedulerProofJob) -> Self {
        ProofJob {
            id: ID(j.id.to_string()),
            repo_id: ID(j.repo_id.to_string()),
            commit_sha: j.commit_sha,
            prover: j.prover.into(),
            file_paths: j.file_paths,
            status: j.status.into(),
            queued_at: j.queued_at,
            started_at: j.started_at,
            completed_at: j.completed_at,
        }
    }
}

impl From<crate::store::models::ProofJobRecord> for ProofJob {
    fn from(j: crate::store::models::ProofJobRecord) -> Self {
        ProofJob {
            id: ID(j.id.to_string()),
            repo_id: ID(j.repo_id.to_string()),
            commit_sha: j.commit_sha,
            prover: j.prover.into(),
            file_paths: j.file_paths,
            status: j.status.into(),
            queued_at: j.queued_at,
            started_at: j.started_at,
            completed_at: j.completed_at,
        }
    }
}

/// Proof verification result for GraphQL
#[derive(SimpleObject, Clone)]
pub struct ProofResult {
    pub status: ProofStatus,
    pub message: String,
    pub prover_output: String,
    pub duration_ms: i32,
    pub verified_files: Vec<String>,
    pub failed_files: Vec<String>,
}

/// Prover information for GraphQL
#[derive(SimpleObject, Clone)]
pub struct ProverInfo {
    pub kind: ProverKind,
    pub name: String,
    pub tier: i32,
    pub file_extensions: Vec<String>,
    pub status: ProverStatus,
}

/// Tactic suggestion from ML for GraphQL
#[derive(SimpleObject, Clone)]
pub struct TacticSuggestion {
    pub tactic: String,
    pub confidence: f64,
    pub explanation: Option<String>,
}

/// Queue statistics for GraphQL
#[derive(SimpleObject, Clone)]
pub struct QueueStats {
    pub queued: i32,
    pub running: i32,
    pub max_concurrent: i32,
    pub max_queue_size: i32,
}

// =============================================================================
// Query Root
// =============================================================================

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a repository by platform, owner, and name
    async fn repository(
        &self,
        ctx: &Context<'_>,
        platform: Platform,
        owner: String,
        name: String,
    ) -> async_graphql::Result<Option<Repository>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let repo = app_ctx
            .store
            .get_repository_by_name(platform.into(), &owner, &name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(repo.map(Into::into))
    }

    /// List all registered repositories
    async fn repositories(
        &self,
        ctx: &Context<'_>,
        platform: Option<Platform>,
    ) -> async_graphql::Result<Vec<Repository>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let repos = app_ctx
            .store
            .list_repositories(platform.map(Into::into))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(repos.into_iter().map(Into::into).collect())
    }

    /// Get a proof job by ID
    async fn job(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<ProofJob>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let job_id = Uuid::parse_str(&id.0)
            .map_err(|_| async_graphql::Error::new("Invalid job ID"))?;
        let job = app_ctx
            .store
            .get_job(JobId(job_id))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(job.map(Into::into))
    }

    /// List jobs for a repository
    async fn jobs_for_repo(
        &self,
        ctx: &Context<'_>,
        repo_id: ID,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<ProofJob>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let uuid = Uuid::parse_str(&repo_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid repository ID"))?;
        let limit = limit.unwrap_or(20) as usize;
        let jobs = app_ctx
            .store
            .list_jobs_for_repo(uuid, limit)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(jobs.into_iter().map(Into::into).collect())
    }

    /// List available provers with their status
    async fn available_provers(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ProverInfo>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let mut provers = Vec::new();

        for prover in DispatcherProverKind::all() {
            let status = app_ctx
                .echidna
                .prover_status(prover)
                .await
                .unwrap_or(EchidnaProverStatus::Unknown);

            provers.push(ProverInfo {
                kind: prover.into(),
                name: prover.display_name().to_string(),
                tier: prover.tier() as i32,
                file_extensions: prover
                    .file_extensions()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                status: status.into(),
            });
        }

        Ok(provers)
    }

    /// Check specific prover status
    async fn prover_status(
        &self,
        ctx: &Context<'_>,
        prover: ProverKind,
    ) -> async_graphql::Result<ProverStatus> {
        let app_ctx = ctx.data::<AppContext>()?;
        let status = app_ctx
            .echidna
            .prover_status(prover.into())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(status.into())
    }

    /// Get queue statistics
    async fn queue_stats(&self, ctx: &Context<'_>) -> async_graphql::Result<QueueStats> {
        let app_ctx = ctx.data::<AppContext>()?;
        let stats = app_ctx.scheduler.stats().await;
        Ok(QueueStats {
            queued: stats.queued as i32,
            running: stats.running as i32,
            max_concurrent: stats.max_concurrent as i32,
            max_queue_size: stats.max_queue_size as i32,
        })
    }

    /// Check ECHIDNA Core connectivity
    async fn echidna_health(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app_ctx = ctx.data::<AppContext>()?;
        let healthy = app_ctx
            .echidna
            .health_check()
            .await
            .unwrap_or(false);
        Ok(healthy)
    }
}

// =============================================================================
// Mutation Root
// =============================================================================

pub struct MutationRoot;

/// Input for registering a repository
#[derive(async_graphql::InputObject)]
pub struct RegisterRepoInput {
    pub platform: Platform,
    pub owner: String,
    pub name: String,
    pub webhook_secret: Option<String>,
    pub enabled_provers: Option<Vec<ProverKind>>,
}

/// Input for repository settings
#[derive(async_graphql::InputObject)]
pub struct RepoSettingsInput {
    pub webhook_secret: Option<String>,
    pub enabled_provers: Option<Vec<ProverKind>>,
    pub check_on_push: Option<bool>,
    pub check_on_pr: Option<bool>,
    pub auto_comment: Option<bool>,
}

#[Object]
impl MutationRoot {
    /// Register a repository for monitoring
    async fn register_repository(
        &self,
        ctx: &Context<'_>,
        input: RegisterRepoInput,
    ) -> async_graphql::Result<Repository> {
        let app_ctx = ctx.data::<AppContext>()?;

        // Check if already exists
        if let Some(existing) = app_ctx
            .store
            .get_repository_by_name(input.platform.into(), &input.owner, &input.name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            return Ok(existing.into());
        }

        // Create new repository
        let provers: Vec<DispatcherProverKind> = input
            .enabled_provers
            .unwrap_or_else(|| vec![ProverKind::Metamath])
            .into_iter()
            .map(Into::into)
            .collect();

        let mut repo =
            StoreRepository::new(input.platform.into(), input.owner, input.name);
        repo.enabled_provers = provers;
        repo.webhook_secret = input.webhook_secret;

        app_ctx
            .store
            .create_repository(&repo)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(repo.into())
    }

    /// Manually trigger a proof check
    async fn trigger_check(
        &self,
        ctx: &Context<'_>,
        repo_id: ID,
        commit_sha: Option<String>,
        provers: Option<Vec<ProverKind>>,
    ) -> async_graphql::Result<ProofJob> {
        let app_ctx = ctx.data::<AppContext>()?;

        let uuid = Uuid::parse_str(&repo_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid repository ID"))?;

        let repo = app_ctx
            .store
            .get_repository(uuid)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Repository not found"))?;

        let commit = commit_sha.unwrap_or_else(|| "HEAD".to_string());
        let prover_list: Vec<DispatcherProverKind> = provers
            .map(|p| p.into_iter().map(Into::into).collect())
            .unwrap_or_else(|| repo.enabled_provers.clone());

        // Create job for first prover (return the first one)
        let prover = prover_list
            .first()
            .copied()
            .ok_or_else(|| async_graphql::Error::new("No provers specified"))?;

        let job = SchedulerProofJob::new(repo.id, commit, prover, vec![])
            .with_priority(JobPriority::Critical);

        let job_id = app_ctx
            .scheduler
            .enqueue(job.clone())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Job already exists"))?;

        // Persist to database
        app_ctx
            .store
            .create_job(&job.clone().into())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Create additional jobs for other provers
        for p in prover_list.iter().skip(1) {
            let additional_job = SchedulerProofJob::new(
                repo.id,
                job.commit_sha.clone(),
                *p,
                vec![],
            )
            .with_priority(JobPriority::Critical);

            if let Ok(Some(_)) = app_ctx.scheduler.enqueue(additional_job.clone()).await {
                let _ = app_ctx.store.create_job(&additional_job.into()).await;
            }
        }

        Ok(job.into())
    }

    /// Request ML-powered tactic suggestions
    async fn request_suggestions(
        &self,
        ctx: &Context<'_>,
        prover: ProverKind,
        context: String,
        goal_state: String,
    ) -> async_graphql::Result<Vec<TacticSuggestion>> {
        let app_ctx = ctx.data::<AppContext>()?;

        let suggestions = app_ctx
            .echidna
            .suggest_tactics(prover.into(), &context, &goal_state)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(suggestions
            .into_iter()
            .map(|s| TacticSuggestion {
                tactic: s.tactic,
                confidence: s.confidence,
                explanation: s.explanation,
            })
            .collect())
    }

    /// Update repository settings
    async fn update_repo_settings(
        &self,
        ctx: &Context<'_>,
        repo_id: ID,
        settings: RepoSettingsInput,
    ) -> async_graphql::Result<Repository> {
        let app_ctx = ctx.data::<AppContext>()?;

        let uuid = Uuid::parse_str(&repo_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid repository ID"))?;

        let mut repo = app_ctx
            .store
            .get_repository(uuid)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Repository not found"))?;

        // Update fields if provided
        if let Some(secret) = settings.webhook_secret {
            repo.webhook_secret = Some(secret);
        }
        if let Some(provers) = settings.enabled_provers {
            repo.enabled_provers = provers.into_iter().map(Into::into).collect();
        }
        if let Some(v) = settings.check_on_push {
            repo.check_on_push = v;
        }
        if let Some(v) = settings.check_on_pr {
            repo.check_on_pr = v;
        }
        if let Some(v) = settings.auto_comment {
            repo.auto_comment = v;
        }

        repo.updated_at = Utc::now();

        app_ctx
            .store
            .update_repository(&repo)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(repo.into())
    }

    /// Enable or disable repository monitoring
    async fn set_repo_enabled(
        &self,
        ctx: &Context<'_>,
        repo_id: ID,
        enabled: bool,
    ) -> async_graphql::Result<Repository> {
        let app_ctx = ctx.data::<AppContext>()?;

        let uuid = Uuid::parse_str(&repo_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid repository ID"))?;

        let mut repo = app_ctx
            .store
            .get_repository(uuid)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Repository not found"))?;

        repo.enabled = enabled;
        repo.updated_at = Utc::now();

        app_ctx
            .store
            .update_repository(&repo)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(repo.into())
    }

    /// Cancel a queued job
    async fn cancel_job(&self, ctx: &Context<'_>, job_id: ID) -> async_graphql::Result<bool> {
        let app_ctx = ctx.data::<AppContext>()?;

        let uuid = Uuid::parse_str(&job_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid job ID"))?;

        let cancelled = app_ctx.scheduler.cancel_job(JobId(uuid)).await;
        Ok(cancelled)
    }

    /// Delete a repository
    async fn delete_repository(&self, ctx: &Context<'_>, repo_id: ID) -> async_graphql::Result<bool> {
        let app_ctx = ctx.data::<AppContext>()?;

        let uuid = Uuid::parse_str(&repo_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid repository ID"))?;

        app_ctx
            .store
            .delete_repository(uuid)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }
}
