// SPDX-License-Identifier: PMPL-1.0-or-later
//! Platform adapters for GitHub, GitLab, Bitbucket

use serde::{Deserialize, Serialize};

pub mod github;
pub mod gitlab;
pub mod bitbucket;

use async_trait::async_trait;
use std::path::PathBuf;

use crate::error::Result;

/// Unique identifier for a repository
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoId {
    pub platform: Platform,
    pub owner: String,
    pub name: String,
}

impl RepoId {
    pub fn new(platform: Platform, owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            platform,
            owner: owner.into(),
            name: name.into(),
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// Platform enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    GitHub,
    GitLab,
    Bitbucket,
    Codeberg,
}

/// Check run identifier
#[derive(Debug, Clone)]
pub struct CheckRunId(pub String);

/// Comment identifier
#[derive(Debug, Clone)]
pub struct CommentId(pub String);

/// Issue identifier
#[derive(Debug, Clone)]
pub struct IssueId(pub String);

/// Pull request identifier
#[derive(Debug, Clone)]
pub struct PrId(pub String);

/// Check run status
#[derive(Debug, Clone)]
pub enum CheckStatus {
    Queued,
    InProgress,
    Completed {
        conclusion: CheckConclusion,
        summary: String,
    },
}

/// Check run conclusion
#[derive(Debug, Clone)]
pub enum CheckConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
}

/// Check run to create
#[derive(Debug, Clone)]
pub struct CheckRun {
    pub name: String,
    pub head_sha: String,
    pub status: CheckStatus,
    pub details_url: Option<String>,
}

/// Issue to create
#[derive(Debug, Clone)]
pub struct NewIssue {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
}

/// Location anchor for an inline PR review comment.
///
/// Used by Consultant mode to attach failure notes directly to the
/// offending line in the diff rather than posting a general PR comment.
/// When the file or line is unknown, `line` defaults to 1.
#[derive(Debug, Clone)]
pub struct ReviewCommentLocation {
    /// The commit SHA to anchor the comment to (must be in the PR's history).
    pub commit_sha: String,
    /// Path of the file to comment on, relative to the repo root.
    pub path: String,
    /// Line number (1-based) on the RIGHT side of the diff. Defaults to 1
    /// when the prover output does not contain a parseable location.
    pub line: u32,
}

/// Build the right `PlatformAdapter` for a given platform.
///
/// Single source of truth for adapter construction — used by both
/// `main.rs::report_to_platform` (Phase 3) and
/// `api/webhooks.rs::handle_consultant_mention` (Phase 6).
///
/// Falls back to a tokenless GitHub client when no token is configured —
/// downstream call sites tolerate auth-failure as a warning, not a panic.
/// Codeberg returns a Config error (Gitea API not yet supported).
pub fn build_adapter(
    config: &crate::Config,
    platform: Platform,
) -> crate::error::Result<Box<dyn PlatformAdapter>> {
    use crate::adapters::{
        bitbucket::BitbucketAdapter, github::GitHubAdapter, gitlab::GitLabAdapter,
    };
    match platform {
        Platform::GitHub => {
            let token = config
                .github
                .as_ref()
                .and_then(|g| g.token.clone())
                .unwrap_or_default();
            Ok(Box::new(GitHubAdapter::new(&token)?))
        }
        Platform::GitLab => Ok(Box::new(GitLabAdapter::new(
            config.gitlab.as_ref().map(|g| g.url.as_str()),
        ))),
        Platform::Bitbucket => Ok(Box::new(BitbucketAdapter::new(None))),
        Platform::Codeberg => Err(crate::error::Error::Config(
            "Codeberg platform reporting not yet implemented".to_string(),
        )),
    }
}

/// Platform adapter trait
///
/// Abstracts operations across GitHub, GitLab, Bitbucket
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Clone a repository to a local path
    async fn clone_repo(&self, repo: &RepoId, commit: &str) -> Result<PathBuf>;

    /// Create a check run (GitHub) or pipeline status (GitLab)
    async fn create_check_run(&self, repo: &RepoId, check: CheckRun) -> Result<CheckRunId>;

    /// Update a check run status
    async fn update_check_run(&self, id: CheckRunId, status: CheckStatus) -> Result<()>;

    /// Create a comment on a PR/MR
    async fn create_comment(&self, repo: &RepoId, pr: PrId, body: &str) -> Result<CommentId>;

    /// Create an issue
    async fn create_issue(&self, repo: &RepoId, issue: NewIssue) -> Result<IssueId>;

    /// Get the default branch name
    async fn get_default_branch(&self, repo: &RepoId) -> Result<String>;

    /// Fetch a single file's contents from the target repo via platform API.
    ///
    /// Returns `Ok(None)` when the file does not exist (not an error —
    /// callers like the directive resolver use absence as a signal to
    /// fall through the cascade). `Err` is reserved for actual API
    /// failures (auth, rate limit, network).
    ///
    /// `branch` may be `None` to use the default branch.
    async fn get_file_contents(
        &self,
        repo: &RepoId,
        branch: Option<&str>,
        path: &str,
    ) -> Result<Option<String>>;

    /// Post an inline review comment on a specific line in the PR diff.
    ///
    /// Used by Consultant mode to anchor failure notes to the offending
    /// proof line. Returns `Err` when the file/line is not in the diff or
    /// when the platform does not support inline review comments — callers
    /// should fall back to `create_comment` on error.
    async fn create_review_comment(
        &self,
        repo: &RepoId,
        pr: PrId,
        body: &str,
        location: ReviewCommentLocation,
    ) -> Result<CommentId>;
}
