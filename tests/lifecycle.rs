// SPDX-License-Identifier: PMPL-1.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
//! Lifecycle tests — full state-machine traversal for jobs, scheduler, and store.
//!
//! Testing taxonomy category: Lifecycle.
//! Covers: creation, active use, completion/cancellation, error recovery, cleanup.
//! All tests use in-memory or temp-file SQLite; no real ECHIDNA instance needed.

use echidnabot::adapters::Platform;
use echidnabot::dispatcher::ProverKind;
use echidnabot::scheduler::{JobResult, JobScheduler, JobStatus, ProofJob};
use echidnabot::store::{
    models::{ProofJobRecord, Repository},
    SqliteStore, Store,
};
use uuid::Uuid;

fn make_job_result(success: bool) -> JobResult {
    JobResult {
        success,
        message: if success { "Verified".to_string() } else { "Failed".to_string() },
        prover_output: if success { "proof completed" } else { "error: goal not proved" }.to_string(),
        duration_ms: 100,
        verified_files: if success { vec!["main.v".to_string()] } else { vec![] },
        failed_files: if success { vec![] } else { vec!["main.v".to_string()] },
        confidence: None,
        axioms: None,
    }
}

fn make_job(repo: Uuid, commit: &str, prover: &str) -> ProofJob {
    ProofJob::new(
        repo,
        commit.to_string(),
        ProverKind::new(prover),
        vec![format!("proofs/{prover}/main.v")],
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// ProofJob state machine
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn lifecycle_job_starts_queued() {
    let job = make_job(Uuid::new_v4(), "abc123", "coq");
    assert_eq!(job.status, JobStatus::Queued);
    assert!(job.started_at.is_none());
    assert!(job.completed_at.is_none());
}

#[test]
fn lifecycle_job_transitions_queued_to_running() {
    let mut job = make_job(Uuid::new_v4(), "abc123", "lean");
    job.start();
    assert_eq!(job.status, JobStatus::Running);
    assert!(job.started_at.is_some(), "started_at must be set after start()");
}

#[test]
fn lifecycle_job_transitions_running_to_completed() {
    let mut job = make_job(Uuid::new_v4(), "def456", "coq");
    job.start();
    job.complete(make_job_result(true));
    assert_eq!(job.status, JobStatus::Completed);
    assert!(job.completed_at.is_some(), "completed_at must be set after complete()");
}

#[test]
fn lifecycle_job_transitions_running_to_failed() {
    let mut job = make_job(Uuid::new_v4(), "fail001", "lean");
    job.start();
    job.complete(make_job_result(false));
    assert_eq!(job.status, JobStatus::Failed);
}

#[test]
fn lifecycle_job_cancel_from_queued() {
    let mut job = make_job(Uuid::new_v4(), "ghi789", "metamath");
    job.cancel();
    assert_eq!(job.status, JobStatus::Cancelled);
}

#[test]
fn lifecycle_job_duration_ms_after_completion() {
    let mut job = make_job(Uuid::new_v4(), "abc", "lean");
    job.start();
    job.complete(make_job_result(true));
    let dur = job.duration_ms();
    assert!(dur.is_some(), "duration_ms must be Some after completion");
}

#[test]
fn lifecycle_job_result_attached_after_completion() {
    let mut job = make_job(Uuid::new_v4(), "res_check", "coq");
    job.start();
    job.complete(make_job_result(true));
    assert!(job.result.is_some(), "result must be attached after complete()");
    assert!(job.result.unwrap().success);
}

// ──────────────────────────────────────────────────────────────────────────────
// JobScheduler lifecycle
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn lifecycle_scheduler_empty_at_start() {
    let sched = JobScheduler::new(4, 100);
    let stats = sched.stats().await;
    assert_eq!(stats.queued, 0);
    assert_eq!(stats.running, 0);
    assert_eq!(sched.running_count(), 0);
    assert!(sched.has_capacity());
}

#[tokio::test]
async fn lifecycle_scheduler_full_job_cycle() {
    let sched = JobScheduler::new(2, 10);
    let repo = Uuid::new_v4();

    let job = make_job(repo, "sha001", "coq");
    let job_id = sched.enqueue(job).await.unwrap().expect("first enqueue must succeed");

    assert_eq!(sched.stats().await.queued, 1);
    assert_eq!(sched.stats().await.running, 0);

    let started = sched.try_start_next().await.expect("must start a job");
    assert_eq!(started.id, job_id);
    assert_eq!(started.status, JobStatus::Running);
    assert_eq!(sched.stats().await.running, 1);
    assert_eq!(sched.stats().await.queued, 0);

    sched.complete_job(job_id, make_job_result(true)).await;

    assert_eq!(sched.stats().await.running, 0);
    assert_eq!(sched.stats().await.queued, 0);
    assert_eq!(sched.running_count(), 0);
}

#[tokio::test]
async fn lifecycle_scheduler_cancel_queued_job() {
    let sched = JobScheduler::new(1, 10);
    let repo = Uuid::new_v4();

    // Fill the single concurrent slot
    let blocker = make_job(repo, "sha_blocker", "lean");
    let blocker_id = sched.enqueue(blocker).await.unwrap().unwrap();
    sched.try_start_next().await;

    // Queue a second job
    let waiter = make_job(repo, "sha_waiter", "coq");
    let waiter_id = sched.enqueue(waiter).await.unwrap().unwrap();
    assert_eq!(sched.stats().await.queued, 1);

    let cancelled = sched.cancel_job(waiter_id).await;
    assert!(cancelled, "cancel must succeed for queued job");
    assert_eq!(sched.stats().await.queued, 0);

    let not_cancelled = sched.cancel_job(blocker_id).await;
    assert!(!not_cancelled, "cannot cancel a running job");
}

#[tokio::test]
async fn lifecycle_scheduler_max_concurrent_enforced() {
    let sched = JobScheduler::new(1, 10);
    let repo = Uuid::new_v4();

    sched.enqueue(make_job(repo, "sha1", "coq")).await.unwrap();
    sched.enqueue(make_job(repo, "sha2", "lean")).await.unwrap();

    assert!(sched.try_start_next().await.is_some());
    assert!(
        sched.try_start_next().await.is_none(),
        "must not start second job when at max_concurrent=1"
    );
    assert!(!sched.has_capacity());
}

#[tokio::test]
async fn lifecycle_scheduler_get_job_by_id() {
    let sched = JobScheduler::new(2, 10);
    let job = make_job(Uuid::new_v4(), "get_me", "z3");
    let job_id = sched.enqueue(job).await.unwrap().unwrap();

    let found = sched.get_job(job_id).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, job_id);

    sched.try_start_next().await;
    let found_running = sched.get_job(job_id).await;
    assert!(found_running.is_some(), "get_job must find running jobs");
}

#[tokio::test]
async fn lifecycle_scheduler_jobs_for_repo() {
    let sched = JobScheduler::new(4, 20);
    let repo_a = Uuid::new_v4();
    let repo_b = Uuid::new_v4();

    sched.enqueue(make_job(repo_a, "a1", "coq")).await.unwrap();
    sched.enqueue(make_job(repo_a, "a2", "lean")).await.unwrap();
    sched.enqueue(make_job(repo_b, "b1", "z3")).await.unwrap();

    let a_jobs = sched.jobs_for_repo(repo_a).await;
    let b_jobs = sched.jobs_for_repo(repo_b).await;

    assert_eq!(a_jobs.len(), 2);
    assert_eq!(b_jobs.len(), 1);
}

#[tokio::test]
async fn lifecycle_scheduler_queue_depth_under_capacity() {
    let sched = JobScheduler::new(4, 20);
    // When active < max_concurrent, queue_depth approximation is 0
    assert_eq!(sched.queue_depth(), 0);
}

// ──────────────────────────────────────────────────────────────────────────────
// Store lifecycle
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn lifecycle_store_repository_crud() {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();

    let repo = Repository::new(Platform::GitHub, "test-owner".to_string(), "test-repo".to_string());
    let repo_id = repo.id;

    store.create_repository(&repo).await.unwrap();

    let found = store.get_repository(repo_id).await.unwrap();
    assert!(found.is_some(), "registered repo must be findable");
    assert_eq!(found.unwrap().name, "test-repo");

    let by_name = store
        .get_repository_by_name(Platform::GitHub, "test-owner", "test-repo")
        .await
        .unwrap();
    assert!(by_name.is_some(), "must be findable by name");
}

#[tokio::test]
async fn lifecycle_store_proof_job_persist_and_retrieve() {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();

    let repo = Repository::new(Platform::GitHub, "owner".to_string(), "lifecycle".to_string());
    let repo_id = repo.id;
    store.create_repository(&repo).await.unwrap();

    let job = make_job(repo_id, "lifecycle001", "coq");
    let job_id = job.id;
    let record = ProofJobRecord::from(job);

    store.create_job(&record).await.unwrap();

    let jobs = store.list_jobs_for_repo(repo_id, 10).await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id.0);
    assert_eq!(jobs[0].commit_sha, "lifecycle001");
}

#[tokio::test]
async fn lifecycle_store_health_check() {
    let store = SqliteStore::new("sqlite::memory:").await.unwrap();
    let healthy = store.health_check().await.unwrap();
    assert!(healthy, "in-memory SQLite must be healthy");
}
