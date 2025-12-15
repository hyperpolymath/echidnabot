//! Bitbucket platform adapter using reqwest

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{
    CheckConclusion, CheckRun, CheckRunId, CheckStatus, CommentId, IssueId, NewIssue,
    PlatformAdapter, PrId, RepoId,
};
use crate::error::{Error, Result};

/// Bitbucket API base URL
const BITBUCKET_API_URL: &str = "https://api.bitbucket.org/2.0";

/// Bitbucket adapter using reqwest with app password authentication
pub struct BitbucketAdapter {
    client: reqwest::Client,
    username: String,
    app_password: String,
    base_url: String,
}

impl BitbucketAdapter {
    /// Create a new Bitbucket adapter with username and app password
    pub fn new(username: &str, app_password: &str) -> Result<Self> {
        Self::with_base_url(username, app_password, BITBUCKET_API_URL)
    }

    /// Create adapter with custom base URL (for Bitbucket Server)
    pub fn with_base_url(username: &str, app_password: &str, base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("echidnabot")
            .build()
            .map_err(|e| Error::Bitbucket(e.to_string()))?;

        Ok(Self {
            client,
            username: username.to_string(),
            app_password: app_password.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Create adapter from environment variables
    pub fn from_env() -> Result<Self> {
        let username = std::env::var("BITBUCKET_USERNAME")
            .map_err(|_| Error::Config("BITBUCKET_USERNAME not set".to_string()))?;

        let app_password = std::env::var("BITBUCKET_APP_PASSWORD")
            .map_err(|_| Error::Config("BITBUCKET_APP_PASSWORD not set".to_string()))?;

        let base_url = std::env::var("BITBUCKET_URL")
            .unwrap_or_else(|_| BITBUCKET_API_URL.to_string());

        Self::with_base_url(&username, &app_password, &base_url)
    }

    /// Make authenticated GET request
    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.app_password))
            .send()
            .await
            .map_err(|e| Error::Bitbucket(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Bitbucket(format!("API error {}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Bitbucket(e.to_string()))
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
            .basic_auth(&self.username, Some(&self.app_password))
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Bitbucket(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Bitbucket(format!("API error {}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Bitbucket(e.to_string()))
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
            .basic_auth(&self.username, Some(&self.app_password))
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Bitbucket(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Bitbucket(format!("API error {}: {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Bitbucket(e.to_string()))
    }
}

// =============================================================================
// Bitbucket API Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct BitbucketRepository {
    mainbranch: Option<BitbucketBranch>,
    links: BitbucketRepoLinks,
}

#[derive(Debug, Deserialize)]
struct BitbucketBranch {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketRepoLinks {
    clone: Vec<BitbucketCloneLink>,
}

#[derive(Debug, Deserialize)]
struct BitbucketCloneLink {
    name: String,
    href: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketBuildStatus {
    key: String,
    state: String,
}

#[derive(Debug, Serialize)]
struct CreateBuildStatus {
    state: &'static str,
    key: String,
    name: String,
    description: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitbucketComment {
    id: i64,
}

#[derive(Debug, Serialize)]
struct CreateComment {
    content: CommentContent,
}

#[derive(Debug, Serialize)]
struct CommentContent {
    raw: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketIssue {
    id: i64,
}

#[derive(Debug, Serialize)]
struct CreateIssue {
    title: String,
    content: IssueContent,
    kind: &'static str,
    priority: &'static str,
}

#[derive(Debug, Serialize)]
struct IssueContent {
    raw: String,
}

#[async_trait]
impl PlatformAdapter for BitbucketAdapter {
    async fn clone_repo(&self, repo: &RepoId, commit: &str) -> Result<PathBuf> {
        // Get repository info to get the clone URL
        let repo_info: BitbucketRepository = self
            .get(&format!(
                "/repositories/{}/{}",
                repo.owner, repo.name
            ))
            .await?;

        // Find HTTPS clone URL
        let clone_url = repo_info
            .links
            .clone
            .iter()
            .find(|link| link.name == "https")
            .map(|link| &link.href)
            .ok_or_else(|| Error::Bitbucket("No HTTPS clone URL found".to_string()))?;

        // Create a temporary directory for the clone
        let temp_dir = tempfile::tempdir().map_err(Error::Io)?;
        let clone_path = temp_dir.into_path();

        // Construct authenticated clone URL
        let auth_url = clone_url.replace(
            "https://",
            &format!("https://{}:{}@", self.username, self.app_password),
        );

        // Clone with depth=1
        let status = tokio::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &auth_url,
                clone_path.to_str().unwrap(),
            ])
            .status()
            .await
            .map_err(Error::Io)?;

        if !status.success() {
            return Err(Error::Bitbucket(format!(
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
        // Bitbucket uses build statuses (also called commit statuses)
        let (state, description) = match check.status {
            CheckStatus::Queued => ("INPROGRESS", Some("Proof verification queued".to_string())),
            CheckStatus::InProgress => {
                ("INPROGRESS", Some("Proof verification in progress".to_string()))
            }
            CheckStatus::Completed { conclusion, summary } => {
                let state = match conclusion {
                    CheckConclusion::Success => "SUCCESSFUL",
                    CheckConclusion::Failure => "FAILED",
                    CheckConclusion::Neutral => "SUCCESSFUL",
                    CheckConclusion::Cancelled => "STOPPED",
                    CheckConclusion::Skipped => "SUCCESSFUL",
                    CheckConclusion::TimedOut => "FAILED",
                    CheckConclusion::ActionRequired => "FAILED",
                };
                (state, Some(summary))
            }
        };

        // Generate a unique key for this status
        let key = format!("echidnabot-{}", check.name.replace(' ', "-").to_lowercase());

        let body = CreateBuildStatus {
            state,
            key: key.clone(),
            name: check.name,
            description,
            url: check.details_url,
        };

        let _status: BitbucketBuildStatus = self
            .post(
                &format!(
                    "/repositories/{}/{}/commit/{}/statuses/build",
                    repo.owner, repo.name, check.head_sha
                ),
                &body,
            )
            .await?;

        Ok(CheckRunId(key))
    }

    async fn update_check_run(&self, _id: CheckRunId, status: CheckStatus) -> Result<()> {
        // Bitbucket build statuses can be updated by posting with the same key
        // However, we need the repo and commit info which we don't have here
        // For now, just log this
        tracing::info!(
            "Bitbucket status update to {:?} - would need repo context to update",
            status
        );
        Ok(())
    }

    async fn create_comment(&self, repo: &RepoId, pr: PrId, body: &str) -> Result<CommentId> {
        let pr_id: i64 = pr
            .0
            .parse()
            .map_err(|_| Error::Bitbucket("Invalid PR ID".to_string()))?;

        let comment_body = CreateComment {
            content: CommentContent {
                raw: body.to_string(),
            },
        };

        let comment: BitbucketComment = self
            .post(
                &format!(
                    "/repositories/{}/{}/pullrequests/{}/comments",
                    repo.owner, repo.name, pr_id
                ),
                &comment_body,
            )
            .await?;

        Ok(CommentId(comment.id.to_string()))
    }

    async fn create_issue(&self, repo: &RepoId, issue: NewIssue) -> Result<IssueId> {
        // Note: Bitbucket Cloud has deprecated the issue tracker for new repos
        // This may not work for all repositories
        let body = CreateIssue {
            title: issue.title,
            content: IssueContent { raw: issue.body },
            kind: "bug",
            priority: "major",
        };

        let created: BitbucketIssue = self
            .post(
                &format!("/repositories/{}/{}/issues", repo.owner, repo.name),
                &body,
            )
            .await?;

        Ok(IssueId(created.id.to_string()))
    }

    async fn get_default_branch(&self, repo: &RepoId) -> Result<String> {
        let repo_info: BitbucketRepository = self
            .get(&format!(
                "/repositories/{}/{}",
                repo.owner, repo.name
            ))
            .await?;

        Ok(repo_info
            .mainbranch
            .map(|b| b.name)
            .unwrap_or_else(|| "main".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = BitbucketAdapter::new("test_user", "test_password");
        assert!(adapter.is_ok());
    }
}
