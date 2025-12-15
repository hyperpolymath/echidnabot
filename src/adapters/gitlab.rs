//! GitLab platform adapter using reqwest

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{
    CheckConclusion, CheckRun, CheckRunId, CheckStatus, CommentId, IssueId, NewIssue,
    PlatformAdapter, PrId, RepoId,
};
use crate::error::{Error, Result};

/// GitLab API base URL
const GITLAB_API_URL: &str = "https://gitlab.com/api/v4";

/// GitLab adapter using reqwest
pub struct GitLabAdapter {
    client: reqwest::Client,
    token: String,
    base_url: String,
}

impl GitLabAdapter {
    /// Create a new GitLab adapter with a personal access token
    pub fn new(token: &str) -> Result<Self> {
        Self::with_base_url(token, GITLAB_API_URL)
    }

    /// Create adapter with custom base URL (for self-hosted GitLab)
    pub fn with_base_url(token: &str, base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("echidnabot")
            .build()
            .map_err(|e| Error::GitLab(e.to_string()))?;

        Ok(Self {
            client,
            token: token.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Create adapter from environment variable
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITLAB_TOKEN")
            .map_err(|_| Error::Config("GITLAB_TOKEN not set".to_string()))?;

        let base_url = std::env::var("GITLAB_URL")
            .unwrap_or_else(|_| GITLAB_API_URL.to_string());

        Self::with_base_url(&token, &base_url)
    }

    /// URL-encode project path (owner/name)
    fn encode_project(&self, repo: &RepoId) -> String {
        urlencoding::encode(&format!("{}/{}", repo.owner, repo.name)).to_string()
    }

    /// Make authenticated GET request
    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| Error::GitLab(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::GitLab(format!("API error {}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| Error::GitLab(e.to_string()))
    }

    /// Make authenticated POST request
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::GitLab(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::GitLab(format!("API error {}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| Error::GitLab(e.to_string()))
    }

    /// Make authenticated PUT request
    async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::GitLab(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::GitLab(format!("API error {}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| Error::GitLab(e.to_string()))
    }
}

// =============================================================================
// GitLab API Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct GitLabProject {
    id: i64,
    default_branch: Option<String>,
    http_url_to_repo: String,
}

#[derive(Debug, Deserialize)]
struct GitLabCommitStatus {
    id: i64,
    status: String,
}

#[derive(Debug, Serialize)]
struct CreateCommitStatus {
    state: &'static str,
    name: String,
    description: Option<String>,
    target_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabNote {
    id: i64,
}

#[derive(Debug, Serialize)]
struct CreateNote {
    body: String,
}

#[derive(Debug, Deserialize)]
struct GitLabIssue {
    iid: i64,
}

#[derive(Debug, Serialize)]
struct CreateIssue {
    title: String,
    description: String,
    labels: String,
}

#[async_trait]
impl PlatformAdapter for GitLabAdapter {
    async fn clone_repo(&self, repo: &RepoId, commit: &str) -> Result<PathBuf> {
        // Get project info to get the clone URL
        let project_path = self.encode_project(repo);
        let project: GitLabProject = self.get(&format!("/projects/{}", project_path)).await?;

        // Create a temporary directory for the clone
        let temp_dir = tempfile::tempdir().map_err(Error::Io)?;
        let clone_path = temp_dir.into_path();

        // Construct authenticated clone URL
        let clone_url = project.http_url_to_repo.replace(
            "https://",
            &format!("https://oauth2:{}@", self.token),
        );

        // Clone with depth=1
        let status = tokio::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &clone_url,
                clone_path.to_str().unwrap(),
            ])
            .status()
            .await
            .map_err(Error::Io)?;

        if !status.success() {
            return Err(Error::GitLab(format!(
                "Failed to clone {}",
                repo.full_name()
            )));
        }

        // Fetch and checkout specific commit if not HEAD
        if commit != "HEAD" {
            let fetch_status = tokio::process::Command::new("git")
                .current_dir(&clone_path)
                .args(["fetch", "--depth", "1", "origin", commit])
                .status()
                .await
                .map_err(Error::Io)?;

            if fetch_status.success() {
                tokio::process::Command::new("git")
                    .current_dir(&clone_path)
                    .args(["checkout", commit])
                    .status()
                    .await
                    .map_err(Error::Io)?;
            }
        }

        Ok(clone_path)
    }

    async fn create_check_run(&self, repo: &RepoId, check: CheckRun) -> Result<CheckRunId> {
        // GitLab uses commit statuses instead of check runs
        let project_path = self.encode_project(repo);

        let (state, description) = match check.status {
            CheckStatus::Queued => ("pending", Some("Proof verification queued".to_string())),
            CheckStatus::InProgress => {
                ("running", Some("Proof verification in progress".to_string()))
            }
            CheckStatus::Completed { conclusion, summary } => {
                let state = match conclusion {
                    CheckConclusion::Success => "success",
                    CheckConclusion::Failure => "failed",
                    CheckConclusion::Neutral => "success",
                    CheckConclusion::Cancelled => "canceled",
                    CheckConclusion::Skipped => "success",
                    CheckConclusion::TimedOut => "failed",
                    CheckConclusion::ActionRequired => "failed",
                };
                (state, Some(summary))
            }
        };

        let body = CreateCommitStatus {
            state,
            name: check.name.clone(),
            description,
            target_url: check.details_url,
        };

        let status: GitLabCommitStatus = self
            .post(
                &format!(
                    "/projects/{}/statuses/{}",
                    project_path, check.head_sha
                ),
                &body,
            )
            .await?;

        Ok(CheckRunId(status.id.to_string()))
    }

    async fn update_check_run(&self, _id: CheckRunId, status: CheckStatus) -> Result<()> {
        // GitLab commit statuses are immutable once created
        // We need to create a new status instead
        // For now, just log this - the caller should create a new status
        tracing::info!("GitLab commit statuses are immutable. Status update to {:?} ignored.", status);
        Ok(())
    }

    async fn create_comment(&self, repo: &RepoId, pr: PrId, body: &str) -> Result<CommentId> {
        let project_path = self.encode_project(repo);
        let mr_iid: i64 = pr
            .0
            .parse()
            .map_err(|_| Error::GitLab("Invalid MR IID".to_string()))?;

        let note_body = CreateNote {
            body: body.to_string(),
        };

        let note: GitLabNote = self
            .post(
                &format!(
                    "/projects/{}/merge_requests/{}/notes",
                    project_path, mr_iid
                ),
                &note_body,
            )
            .await?;

        Ok(CommentId(note.id.to_string()))
    }

    async fn create_issue(&self, repo: &RepoId, issue: NewIssue) -> Result<IssueId> {
        let project_path = self.encode_project(repo);

        let body = CreateIssue {
            title: issue.title,
            description: issue.body,
            labels: issue.labels.join(","),
        };

        let created: GitLabIssue = self
            .post(&format!("/projects/{}/issues", project_path), &body)
            .await?;

        Ok(IssueId(created.iid.to_string()))
    }

    async fn get_default_branch(&self, repo: &RepoId) -> Result<String> {
        let project_path = self.encode_project(repo);
        let project: GitLabProject = self.get(&format!("/projects/{}", project_path)).await?;

        Ok(project.default_branch.unwrap_or_else(|| "main".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project() {
        let adapter = GitLabAdapter {
            client: reqwest::Client::new(),
            token: "test".to_string(),
            base_url: GITLAB_API_URL.to_string(),
        };

        let repo = RepoId::new(
            super::super::Platform::GitLab,
            "hyperpolymath",
            "echidnabot",
        );
        let encoded = adapter.encode_project(&repo);
        assert_eq!(encoded, "hyperpolymath%2Fechidnabot");
    }
}
