;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Project state for echidnabot
;; Media-Type: application/vnd.state+scm

(state
  (metadata
    (version "0.1.0")
    (schema-version "1.0")
    (created "2026-01-03")
    (updated "2026-02-08")
    (project "echidnabot")
    (repo "github.com/hyperpolymath/echidnabot"))

  (project-context
    (name "echidnabot")
    (tagline "Proof-aware CI bot for automated formal verification")
    (tech-stack
      ("Rust 1.75+" "Tokio async runtime" "Axum web framework"
       "async-graphql for GraphQL API" "sqlx for PostgreSQL/SQLite"
       "octocrab for GitHub API" "reqwest for HTTP client"
       "Podman rootless for container isolation"
       "bubblewrap (bwrap) as isolation fallback")))

  (current-position
    (phase "active-development")
    (overall-completion 90)
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
         ((status "complete")
          (items ("Job queue implementation" "PostgreSQL persistence"
                 "Job status tracking" "Retry logic with exponential backoff"
                 "Circuit breaker for ECHIDNA API protection"
                 "Concurrent job execution with configurable limits"))))

       (container-isolation
         ((status "complete")
          (items ("Podman rootless container spawning"
                 "bubblewrap (bwrap) fallback"
                 "Resource limits (CPU/memory/pids/timeout)"
                 "Read-only filesystem with writable /tmp"
                 "Network isolation (--network=none)"
                 "Fail-safe: refuses proofs without isolation"
                 "Security: --cap-drop=ALL, no-new-privileges"))))

       (bot-modes
         ((status "complete")
          (items ("Verifier mode (silent pass/fail)"
                 "Advisor mode (tactic suggestions via ECHIDNA ML)"
                 "Consultant mode (interactive Q&A, explicit mentions)"
                 "Regulator mode (PR merge blocking)"
                 "Mode parsing from .bot_directives/echidnabot.scm"
                 "Mode-dependent webhook trigger logic"))))

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
          (status . "complete")
          (completion . 100)
          (items . ("Basic job queue ✓"
                   "PostgreSQL persistence ✓"
                   "Job status tracking ✓"
                   "Retry logic with exponential backoff ✓"
                   "Circuit breaker for ECHIDNA API ✓"
                   "Priority queue for urgent jobs ✓"
                   "Concurrent job execution limits ✓"))))

       (milestone-5
         ((name . "Container Isolation")
          (status . "complete")
          (completion . 100)
          (items . ("Podman rootless container spawning ✓"
                   "bubblewrap (bwrap) fallback ✓"
                   "Resource limits (CPU/memory/pids/timeout) ✓"
                   "Read-only filesystem with /tmp ✓"
                   "Network isolation (--network=none) ✓"
                   "Fail-safe policy (no isolation = no proofs) ✓"))))

       (milestone-6
         ((name . "Bot Modes Implementation")
          (status . "complete")
          (completion . 100)
          (items . ("Verifier mode (silent pass/fail) ✓"
                   "Advisor mode (tactic suggestions) ✓"
                   "Consultant mode (interactive Q&A) ✓"
                   "Regulator mode (PR merge blocking) ✓"
                   "Mode parsing from .bot_directives ✓"
                   "Mode-dependent auto-trigger logic ✓"))))

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
      ())
    (medium
      ("No observability (metrics/tracing) - hard to debug production issues"
       "No rate limiting - vulnerable to webhook spam"
       "ECHIDNA Trust Bridge not yet implemented"))
    (low
      ("Podman Compose not set up - manual PostgreSQL setup required"
       "No pre-built prover images - container startup slow")))

  (critical-next-actions
    (immediate
      ("Implement ECHIDNA Trust Bridge (confidence levels, solver integrity, axiom tracking)"
       "Add observability (Prometheus metrics, OpenTelemetry tracing)"))

    (this-week
      ("Create pre-built Podman images for all 12 provers"
       "Set up Podman Compose for PostgreSQL + echidnabot + ECHIDNA"
       "Production deployment guide"
       "GitHub App registration and distribution"))

    (this-month
      ("Rate limiting for webhook endpoints"
       "Performance benchmarks for proof verification pipeline"
       "End-to-end integration tests with real ECHIDNA instance")))

  (session-history
    ((session-2026-02-08
       ((date . "2026-02-08")
        (focus . "Execute SONNET-TASKS.md: container isolation, bot modes, retry, tests, metadata")
        (accomplishments
          . ("Implemented Podman rootless container isolation with bwrap fallback"
            "Wired bot modes (Verifier/Advisor/Consultant/Regulator) into webhook handlers"
            "Added mode parsing from .bot_directives/echidnabot.scm"
            "Integrated retry logic with exponential backoff and circuit breaker"
            "Added 30 integration tests (97 total tests passing)"
            "Fixed Cargo.toml license to PMPL-1.0-or-later"
            "Added SPDX headers to all .rs files"
            "Updated STATE.scm to reflect actual 90% completion"))
        (blockers-resolved . ("Container isolation was empty (now Podman+bwrap)"
                             "Bot modes not connected to handlers (now wired)"
                             "Retry logic not integrated (now circuit breaker + backoff)"
                             "Zero automated tests (now 97 passing)"))
        (next-session-focus . "ECHIDNA Trust Bridge and production hardening")))

     (session-2026-01-29
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
      90))

  (get-blockers
    (lambda (priority)
      (cond
        ((eq? priority 'critical) '())
        ((eq? priority 'high) '())
        ((eq? priority 'medium)
         '("No observability"
           "No rate limiting"
           "ECHIDNA Trust Bridge not yet implemented"))
        ((eq? priority 'low)
         '("Podman Compose not set up"
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
         '((status . complete) (completion . 100)))
        ((container-isolation)
         '((status . complete) (completion . 100)))
        ((bot-modes)
         '((status . complete) (completion . 100)))
        ((production-hardening)
         '((status . planned) (completion . 0)))))))
