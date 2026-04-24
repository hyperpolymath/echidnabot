// SPDX-License-Identifier: PMPL-1.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
//! Corpus-delta writer — feeds successful proofs back into ECHIDNA's
//! training_data for the Julia retrainer.
//!
//! Pipeline:
//!   proof succeeds → echidnabot records DeltaRow → appended to
//!   `{training_data_dir}/delta_YYYY-MM-DD.jsonl` → ECHIDNA's
//!   `just corpus-refresh` consumes the delta via its extract-corpora
//!   / retrain steps (see echidna repo commit 055e13e + 3f32c29).
//!
//! Writes are append-only JSONL. One line per attempt. The retrainer is
//! NOT invoked per-record by default — use `auto_trigger_threshold` to
//! batch, or call `trigger_refresh` explicitly (e.g. from an MCP tool).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::dispatcher::ProverKind;
use crate::error::{Error, Result};

/// Provenance of a delta row — where did the proof attempt originate?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeltaSource {
    /// Webhook-driven CI run (GitHub/GitLab/Bitbucket)
    Webhook,
    /// Invoked via the MCP tool surface
    Mcp,
    /// Invoked from the CLI binary
    Cli,
}

/// One row of training-delta JSONL. Serialised directly to the delta file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaRow {
    pub timestamp: DateTime<Utc>,
    pub prover: ProverKind,
    pub goal_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub chosen_tactic: String,
    pub succeeded: bool,
    pub duration_ms: i64,
    pub source: DeltaSource,
}

impl DeltaRow {
    pub fn new(
        prover: ProverKind,
        goal_state: String,
        chosen_tactic: String,
        succeeded: bool,
        duration_ms: i64,
        source: DeltaSource,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            prover,
            goal_state,
            context: None,
            chosen_tactic,
            succeeded,
            duration_ms,
            source,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

/// Default subprocess: `just corpus-refresh` run in `echidna_root`.
const DEFAULT_TRIGGER_PROGRAM: &str = "just";
const DEFAULT_TRIGGER_ARGS: &[&str] = &["corpus-refresh"];

/// Writer for training-delta rows and manager for retrain triggers.
pub struct CorpusDelta {
    training_data_dir: PathBuf,
    echidna_root: Option<PathBuf>,
    trigger_program: String,
    trigger_args: Vec<String>,
    auto_trigger_threshold: Option<u32>,
    counter: Arc<Mutex<u32>>,
}

impl CorpusDelta {
    /// Create a writer that appends to `{training_data_dir}/delta_YYYY-MM-DD.jsonl`.
    /// No trigger is configured — `trigger_refresh` will error until one is set.
    pub fn new(training_data_dir: PathBuf) -> Self {
        Self {
            training_data_dir,
            echidna_root: None,
            trigger_program: DEFAULT_TRIGGER_PROGRAM.to_string(),
            trigger_args: DEFAULT_TRIGGER_ARGS.iter().map(|s| s.to_string()).collect(),
            auto_trigger_threshold: None,
            counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Configure `trigger_refresh` to run `just corpus-refresh` in `echidna_root`.
    pub fn with_trigger(mut self, echidna_root: PathBuf) -> Self {
        self.echidna_root = Some(echidna_root);
        self
    }

    /// Override the default trigger command (primarily for testing / custom pipelines).
    pub fn with_trigger_command(mut self, program: String, args: Vec<String>) -> Self {
        self.trigger_program = program;
        self.trigger_args = args;
        self
    }

    /// Fire `trigger_refresh` automatically every N successful records.
    pub fn with_auto_trigger(mut self, threshold: u32) -> Self {
        self.auto_trigger_threshold = Some(threshold);
        self
    }

    /// Append a delta row to today's JSONL file. Returns the file path written.
    /// If auto-trigger is configured and the row is a success, advances the
    /// counter and may fire `trigger_refresh`.
    pub async fn record(&self, row: &DeltaRow) -> Result<PathBuf> {
        fs::create_dir_all(&self.training_data_dir)
            .await
            .map_err(|e| Error::Internal(format!("create training_data_dir: {}", e)))?;

        let path = self.delta_path_for(row.timestamp);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(|e| Error::Internal(format!("open delta file: {}", e)))?;

        let mut line = serde_json::to_string(row)?;
        line.push('\n');
        file.write_all(line.as_bytes())
            .await
            .map_err(|e| Error::Internal(format!("write delta: {}", e)))?;
        file.flush()
            .await
            .map_err(|e| Error::Internal(format!("flush delta: {}", e)))?;

        if row.succeeded {
            if let Some(threshold) = self.auto_trigger_threshold {
                let mut counter = self.counter.lock().await;
                *counter += 1;
                if *counter >= threshold {
                    *counter = 0;
                    drop(counter);
                    let _ = self.trigger_refresh().await?;
                }
            }
        }

        Ok(path)
    }

    /// Invoke the configured trigger command (default: `just corpus-refresh`).
    /// Errors if `echidna_root` was never set.
    pub async fn trigger_refresh(&self) -> Result<RefreshStatus> {
        let cwd = self.echidna_root.as_ref().ok_or_else(|| {
            Error::Internal("corpus refresh: echidna_root not configured".to_string())
        })?;

        let output = Command::new(&self.trigger_program)
            .args(&self.trigger_args)
            .current_dir(cwd)
            .output()
            .await
            .map_err(|e| {
                Error::Internal(format!(
                    "corpus refresh: spawn {:?} failed: {}",
                    self.trigger_program, e
                ))
            })?;

        Ok(RefreshStatus {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    /// Path of the delta file for a given timestamp (UTC date).
    pub fn delta_path_for(&self, ts: DateTime<Utc>) -> PathBuf {
        self.training_data_dir
            .join(format!("delta_{}.jsonl", ts.format("%Y-%m-%d")))
    }

    /// Current value of the auto-trigger counter (for observability / tests).
    pub async fn counter_value(&self) -> u32 {
        *self.counter.lock().await
    }

    pub fn training_data_dir(&self) -> &Path {
        &self.training_data_dir
    }
}

/// Outcome of a corpus-refresh subprocess invocation.
#[derive(Debug, Clone)]
pub struct RefreshStatus {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use uuid::Uuid;

    fn tmp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("echidnabot-corpus-{}", Uuid::new_v4()))
    }

    fn sample_row(succeeded: bool) -> DeltaRow {
        DeltaRow::new(
            ProverKind::Coq,
            "forall x : nat, x + 0 = x".to_string(),
            "rewrite plus_n_O".to_string(),
            succeeded,
            42,
            DeltaSource::Mcp,
        )
    }

    async fn read_file_to_string(path: &Path) -> String {
        let mut f = tokio::fs::File::open(path).await.unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).await.unwrap();
        s
    }

    #[tokio::test]
    async fn record_writes_jsonl_line_to_dated_file() {
        let dir = tmp_dir();
        let cd = CorpusDelta::new(dir.clone());
        let row = sample_row(true);

        let path = cd.record(&row).await.unwrap();
        assert!(path.file_name().unwrap().to_string_lossy().starts_with("delta_"));
        assert!(path.file_name().unwrap().to_string_lossy().ends_with(".jsonl"));

        let contents = read_file_to_string(&path).await;
        assert_eq!(contents.lines().count(), 1);
        let parsed: DeltaRow = serde_json::from_str(contents.trim()).unwrap();
        assert_eq!(parsed.chosen_tactic, "rewrite plus_n_O");
        assert!(parsed.succeeded);
        assert_eq!(parsed.source, DeltaSource::Mcp);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn record_appends_additional_rows() {
        let dir = tmp_dir();
        let cd = CorpusDelta::new(dir.clone());
        let path = cd.record(&sample_row(true)).await.unwrap();
        let path2 = cd.record(&sample_row(false)).await.unwrap();
        assert_eq!(path, path2);

        let contents = read_file_to_string(&path).await;
        assert_eq!(contents.lines().count(), 2);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn trigger_refresh_errors_without_echidna_root() {
        let dir = tmp_dir();
        let cd = CorpusDelta::new(dir.clone());
        let err = cd.trigger_refresh().await.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("echidna_root not configured"), "msg: {}", msg);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn trigger_refresh_runs_configured_command() {
        let dir = tmp_dir();
        // Use `/bin/true` — always succeeds, portable on Linux CI.
        let cd = CorpusDelta::new(dir.clone())
            .with_trigger(std::env::temp_dir())
            .with_trigger_command("/bin/true".to_string(), vec![]);
        let status = cd.trigger_refresh().await.unwrap();
        assert!(status.success);
        assert_eq!(status.exit_code, Some(0));
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn trigger_refresh_reports_failure_exit_code() {
        let dir = tmp_dir();
        let cd = CorpusDelta::new(dir.clone())
            .with_trigger(std::env::temp_dir())
            .with_trigger_command("/bin/false".to_string(), vec![]);
        let status = cd.trigger_refresh().await.unwrap();
        assert!(!status.success);
        assert_eq!(status.exit_code, Some(1));
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn auto_trigger_fires_at_threshold_and_resets_counter() {
        let dir = tmp_dir();
        let cd = CorpusDelta::new(dir.clone())
            .with_trigger(std::env::temp_dir())
            .with_trigger_command("/bin/true".to_string(), vec![])
            .with_auto_trigger(2);

        cd.record(&sample_row(true)).await.unwrap();
        assert_eq!(cd.counter_value().await, 1);
        cd.record(&sample_row(true)).await.unwrap(); // triggers → resets
        assert_eq!(cd.counter_value().await, 0);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn failed_rows_do_not_advance_auto_trigger_counter() {
        let dir = tmp_dir();
        let cd = CorpusDelta::new(dir.clone())
            .with_trigger(std::env::temp_dir())
            .with_trigger_command("/bin/true".to_string(), vec![])
            .with_auto_trigger(3);

        cd.record(&sample_row(false)).await.unwrap();
        cd.record(&sample_row(false)).await.unwrap();
        assert_eq!(cd.counter_value().await, 0);
        cd.record(&sample_row(true)).await.unwrap();
        assert_eq!(cd.counter_value().await, 1);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
