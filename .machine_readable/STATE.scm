;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Project state for echidnabot
;; Media-Type: application/vnd.state+scm

(state
  (metadata
    (version "0.1.0")
    (schema-version "1.0")
    (created "2026-01-03")
    (updated "2026-02-01")
    (project "echidnabot")
    (repo "github.com/hyperpolymath/echidnabot"))

  (project-context
    (name "echidnabot")
    (tagline "Proof-aware CI bot for automated formal verification")
    (tech-stack
      ("Rust 1.75+" "Tokio async runtime" "Axum web framework"
       "async-graphql for GraphQL API" "sqlx for PostgreSQL/SQLite"
       "octocrab for GitHub API" "reqwest for HTTP client"
       "Docker for container isolation")))

  (current-position
    (phase "active-development")
    (overall-completion 75)
    (components
      ((core-infrastructure
         ((status "complete")
          (items ("Axum HTTP server" "Webhook signature verification"
                 "GraphQL API schema" "Database models and migrations"
                 "Configuration system" "Error handling"))))

       (platform-adapters
         ((status "complete")
          (items ("PlatformAdapter trait" "GitHub adapter with octocrab"
                 "GitLab adapter" "Bitbucket adapter"
                 "Webhook receivers for all 3 platforms"
                 "Check run / commit status creation"))))

       (echidna-integration
         ((status "complete")
          (items ("HTTP client for ECHIDNA API" "Prover kind enumeration"
                 "Verification job dispatch" "Result parsing"))))

       (job-scheduler
         ((status "in-progress")
          (items ("Job queue implementation" "PostgreSQL persistence"
                 "Job status tracking" "Retry logic (TODO)"
                 "Concurrent job execution (TODO)"))))

       (container-isolation
         ((status "planned")
          (items ("Docker container spawning (TODO)"
                 "Resource limits (CPU/memory) (TODO)"
                 "Read-only filesystem setup (TODO)"
                 "Network isolation (TODO)"))))

       (bot-modes
         ((status "planned")
          (items ("Verifier mode (silent checks) (TODO)"
                 "Advisor mode (tactic suggestions) (TODO)"
                 "Consultant mode (Q&A) (TODO)"
                 "Regulator mode (merge blocking) (TODO)"))))

       (documentation
         ((status "complete")
          (items ("README.adoc with architecture"
                 "Installation instructions" "API documentation"
                 "Configuration reference" "META.scm" "ECOSYSTEM.scm"
                 "STATE.scm"))))))

    (working-features
      ("HTTP server with health checks"
       "Webhook receivers for GitHub/GitLab/Bitbucket"
       "Platform adapter abstraction for multi-platform support"
       "GraphQL API for job queries and mutations"
       "PostgreSQL database with sqlx migrations"
       "Integration with ECHIDNA API for proof verification"
       "Repository registration and configuration"
       "Job status tracking")))

  (route-to-mvp
    (milestones
      ((milestone-1
         ((name . "Core Infrastructure")
          (status . "complete")
          (completion . 100)
          (items . ("Axum server setup ✓"
                   "Webhook signature verification ✓"
                   "Database schema ✓"
                   "GraphQL API ✓"))))

       (milestone-2
         ((name . "Platform Integration")
          (status . "complete")
          (completion . 100)
          (items . ("GitHub adapter ✓"
                   "GitLab adapter ✓"
                   "Bitbucket adapter ✓"
                   "Multi-platform webhook handling ✓"))))

       (milestone-3
         ((name . "ECHIDNA Integration")
          (status . "complete")
          (completion . 100)
          (items . ("HTTP client for ECHIDNA ✓"
                   "Job dispatch to ECHIDNA ✓"
                   "Result parsing and storage ✓"))))

       (milestone-4
         ((name . "Job Scheduler and Queue")
          (status . "in-progress")
          (completion . 60)
          (items . ("Basic job queue ✓"
                   "PostgreSQL persistence ✓"
                   "Job status tracking ✓"
                   "Retry logic with backoff (TODO)"
                   "Priority queue for urgent jobs (TODO)"
                   "Concurrent job execution limits (TODO)"))))

       (milestone-5
         ((name . "Container Isolation")
          (status . "planned")
          (completion . 0)
          (items . ("Docker container spawning (TODO)"
                   "Resource limits (CPU/memory/time) (TODO)"
                   "Read-only filesystem (TODO)"
                   "Network isolation (TODO)"
                   "Pre-built prover images (TODO)"))))

       (milestone-6
         ((name . "Bot Modes Implementation")
          (status . "planned")
          (completion . 0)
          (items . ("Verifier mode (silent pass/fail) (TODO)"
                   "Advisor mode (tactic suggestions via ECHIDNA ML) (TODO)"
                   "Consultant mode (interactive Q&A) (TODO)"
                   "Regulator mode (PR merge blocking) (TODO)"))))

       (milestone-7
         ((name . "Production Hardening")
          (status . "planned")
          (completion . 0)
          (items . ("Comprehensive error recovery (TODO)"
                   "Observability (metrics, tracing) (TODO)"
                   "Rate limiting (TODO)"
                   "Deployment automation (TODO)"
                   "Docker Compose for easy setup (TODO)")))))))

  (blockers-and-issues
    (critical
      ())
    (high
      ("Container isolation not yet implemented - security risk"
       "No retry logic for failed jobs - temporary failures become permanent"
       "Concurrent job execution not limited - risk of resource exhaustion"))
    (medium
      ("Bot modes not implemented - only basic verification works"
       "No observability (metrics/tracing) - hard to debug production issues"
       "No rate limiting - vulnerable to webhook spam"))
    (low
      ("Docker Compose not set up - manual PostgreSQL setup required"
       "No pre-built prover images - container startup slow")))

  (critical-next-actions
    (immediate
      ("Implement basic container isolation with Docker"
       "Add retry logic with exponential backoff to job scheduler"
       "Fix author attribution in Cargo.toml (DONE)"
       "Restore deleted source files (DONE)"
       "Update META.scm and ECOSYSTEM.scm with comprehensive docs (DONE)"))

    (this-week
      ("Implement Verifier mode (silent pass/fail checks)"
       "Add resource limits to container execution"
       "Set up Docker Compose for PostgreSQL + echidnabot + ECHIDNA"
       "Add integration tests for webhook → verification flow"
       "Implement concurrent job execution with configurable limits"))

    (this-month
      ("Implement Advisor mode with ECHIDNA ML tactic suggestions"
       "Add observability (Prometheus metrics, OpenTelemetry tracing)"
       "Create pre-built Docker images for all 12 provers"
       "Implement Regulator mode for PR merge blocking"
       "Production deployment guide"
       "GitHub App registration and distribution")))

  (session-history
    ((session-2026-01-29
       ((date . "2026-01-29")
        (focus . "Fix build issues and update documentation")
        (accomplishments
          . ("Fixed author email in Cargo.toml (jonathan.jewell@open.ac.uk)"
            "Restored deleted source files (main.rs, graphql.rs, echidna_client.rs, sqlite.rs)"
            "Cleaned and rebuilt successfully (cargo clean && cargo build)"
            "Created comprehensive META.scm with 8 Architecture Decision Records"
            "Created comprehensive ECOSYSTEM.scm with related projects and position"
            "Updated STATE.scm with current progress (75% complete)"
            "Documented 7 milestones with completion status"
            "Identified 3 high-priority blockers (container isolation, retry logic, concurrency limits)"))
        (blockers-resolved . ("Build errors due to stale artifacts"
                             "Missing source files"
                             "Wrong author attribution"))
        (next-session-focus . "Implement container isolation and retry logic")))))

  ;; Helper functions for state queries
  (get-completion-percentage
    (lambda ()
      75))

  (get-blockers
    (lambda (priority)
      (cond
        ((eq? priority 'critical) '())
        ((eq? priority 'high)
         '("Container isolation not yet implemented"
           "No retry logic for failed jobs"
           "Concurrent job execution not limited"))
        ((eq? priority 'medium)
         '("Bot modes not implemented"
           "No observability"
           "No rate limiting"))
        ((eq? priority 'low)
         '("Docker Compose not set up"
           "No pre-built prover images")))))

  (get-milestone
    (lambda (name)
      (case name
        ((core-infrastructure)
         '((status . complete) (completion . 100)))
        ((platform-integration)
         '((status . complete) (completion . 100)))
        ((echidna-integration)
         '((status . complete) (completion . 100)))
        ((job-scheduler)
         '((status . in-progress) (completion . 60)))
        ((container-isolation)
         '((status . planned) (completion . 0)))
        ((bot-modes)
         '((status . planned) (completion . 0)))
        ((production-hardening)
         '((status . planned) (completion . 0)))))))
