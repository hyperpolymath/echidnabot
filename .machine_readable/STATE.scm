;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Project state for echidnabot
;; Media-Type: application/vnd.state+scm

(state
  (metadata
    (version "0.1.0")
    (schema-version "1.0")
    (created "2026-01-03")
    (updated "2026-04-24")
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
       "bubblewrap (bwrap) as isolation fallback"
       "clap for CLI" "hmac/sha2 for webhook verification")))

  (current-position
    (phase "active-development")
    (overall-completion 90)
    (test-count 129)
    (components
      ((core-infrastructure
         ((status "complete")
          (items ("Axum HTTP server with health checks"
                 "Webhook signature verification (HMAC-SHA256)"
                 "GraphQL API (async-graphql) with playground"
                 "Database models and migrations (sqlx, SQLite/PostgreSQL)"
                 "Configuration system (TOML + env vars)"
                 "Structured error handling (thiserror)"
                 "CLI: serve, register, check, status, init-db"))))

       (platform-adapters
         ((status "complete")
          (items ("PlatformAdapter trait with unified interface"
                 "GitHub adapter with octocrab"
                 "GitLab adapter"
                 "Bitbucket adapter"
                 "Webhook receivers for all 3 platforms"
                 "Check run / commit status creation"
                 "Codeberg enum variant (adapter not yet implemented)"))))

       (echidna-integration
         ((status "complete")
          (items ("HTTP client for ECHIDNA REST API"
                 "12-prover enumeration with tier classification"
                 "Verification job dispatch"
                 "Result parsing (Verified/Failed/Timeout/Error/Unknown)"
                 "Prover health checking"
                 "Tactic suggestion data model"))))

       (job-scheduler
         ((status "complete")
          (items ("Priority queue (Low/Normal/High/Critical)"
                 "SQLite/PostgreSQL persistence"
                 "Job status tracking (Queued/Running/Completed/Failed/Cancelled)"
                 "Retry logic with exponential backoff and jitter"
                 "Circuit breaker (5 failures -> open, 5 min auto-reset, half-open recovery)"
                 "Concurrent job limits (semaphore-based, global + per-repo)"))))

       (container-isolation
         ((status "complete")
          (items ("Podman rootless container spawning"
                 "bubblewrap (bwrap) fallback sandbox"
                 "Fail-safe: refuses proofs without isolation backend"
                 "Security: --cap-drop=ALL, --security-opt=no-new-privileges, --read-only"
                 "Resource limits (CPU/memory/pids/timeout)"
                 "Network isolation (--network=none)"
                 "OOM-kill detection (exit code 137)"
                 "Timeout enforcement with SIGKILL"))))

       (bot-modes
         ((status "complete")
          (items ("Verifier mode (silent pass/fail)"
                 "Advisor mode (detailed failures + tactic suggestions)"
                 "Consultant mode (interactive Q&A, explicit mention trigger)"
                 "Regulator mode (PR merge blocking)"
                 "Mode parsing from .bot_directives/echidnabot.scm"
                 "Mode-dependent auto-trigger logic"
                 "Result formatting bridge (PR comments, check runs, summaries)"))))

       (trust-bridge
         ((status "implemented-not-wired")
          (items ("5-level proof confidence assessment"
                 "Small-kernel prover classification (7 of 12 provers)"
                 "Solver integrity verification (SHA-256 manifest)"
                 "Constant-time hash comparison"
                 "Axiom usage tracking (sorry, Admitted, postulate, oops, etc.)"
                 "3-tier severity (unsound/warning/informational)"
                 "Prover-specific and universal axiom scanning"))
          (note "Trust modules exist in src/trust/ with full tests but are NOT yet called from the scheduler loop or process_job pipeline")))

       (automated-tests
         ((status "complete")
          (items ("129 total tests (30 integration, 99 unit)"
                 "Webhook signature verification tests"
                 "ECHIDNA client request/response tests"
                 "Bot mode resolution and formatting tests"
                 "Job lifecycle tests"
                 "Database model tests"
                 "Container executor command generation tests"
                 "Circuit breaker behavior tests"
                 "Retry logic tests"
                 "Concurrency limiter tests"
                 "Trust confidence level tests"
                 "Axiom tracker tests"
                 "Solver integrity tests"
                 "Result formatter tests"))))

       (production-hardening
         ((status "not-started")
          (items ("Observability (metrics, tracing) -- not implemented"
                 "Rate limiting -- not implemented"
                 "Deployment automation -- not implemented"
                 "Wire trust bridge into main pipeline -- not done"))))

       (double-loop-feedback
         ((status "partial")
          (items ("7b-1 DONE: tactic_outcome table, fingerprint helper, CRUD (commit 0cc4b4a)"
                 "7b-2 DONE: history-weighted reranker, Laplace smoothing, global fallback (commit b6ef652)"
                 "7b-3 DONE-with-gap: CorpusDelta writer + trigger_refresh (commit 1e26340)"
                 "7b-4 PENDING: native MCP server bin (temporary BoJ exception)"
                 "7b-5 PENDING: dogfood proofs committed to proofs/ exercising echidnabot.yml"
                 "7b-6 PENDING: 6a2 sweep + project memory on BoJ MCP exception"))
          (known-gap ("7b-3 schema mismatch: CorpusDelta emits delta_YYYY-MM-DD.jsonl with "
                      "{timestamp, prover, goal_state, chosen_tactic, succeeded, duration_ms, source}. "
                      "Echidna's merge_corpus.jl expects proof_states_*.jsonl with "
                      "{id, prover, theorem, goal, context[], source, proof_type} and consumes only "
                      "files in its hardcoded PER_PROVER_FILES whitelist. "
                      "Successful deltas are NOT auto-ingested by just corpus-refresh until "
                      "(a) CorpusDelta also emits echidna-schema rows for successes, or "
                      "(b) echidna/scripts/merge_corpus.jl is extended to include echidnabot deltas. "
                      "Coordination needed with Session A owning the retrainer."))))))

    (working-features
      ("HTTP server with health checks and GraphQL playground"
       "Webhook receivers for GitHub/GitLab/Bitbucket"
       "Platform adapter abstraction (PlatformAdapter trait)"
       "GraphQL API for job queries and mutations"
       "SQLite/PostgreSQL database with sqlx"
       "Integration with ECHIDNA API for proof verification"
       "Repository registration and configuration via CLI"
       "Job status tracking with priority queue"
       "Retry with exponential backoff and circuit breaker"
       "Container isolation (Podman/bwrap) with fail-safe policy"
       "Four bot modes with mode-dependent behavior"
       "Result formatting for PR comments and check runs"
       "Trust bridge: confidence, integrity, axiom tracking (standalone, not wired)")))

  (route-to-mvp
    (milestones
      ((milestone-1
         ((name . "Core Infrastructure")
          (status . "complete")
          (completion . 100)))

       (milestone-2
         ((name . "Platform Integration")
          (status . "complete")
          (completion . 100)))

       (milestone-3
         ((name . "ECHIDNA Integration")
          (status . "complete")
          (completion . 100)))

       (milestone-4
         ((name . "Job Scheduler and Queue")
          (status . "complete")
          (completion . 100)))

       (milestone-5
         ((name . "Container Isolation")
          (status . "complete")
          (completion . 100)))

       (milestone-6
         ((name . "Bot Modes Implementation")
          (status . "complete")
          (completion . 100)))

       (milestone-7
         ((name . "ECHIDNA Trust Bridge")
          (status . "implemented-not-wired")
          (completion . 80)
          (note . "Modules implemented and tested but not integrated into main pipeline")))

       (milestone-8
         ((name . "Production Hardening")
          (status . "not-started")
          (completion . 0)
          (items . ("Observability (TODO)"
                   "Rate limiting (TODO)"
                   "Deployment automation (TODO)"
                   "Wire trust bridge (TODO)")))))))

  (blockers-and-issues
    (critical
      ())
    (high
      ("Trust bridge not wired into scheduler/process_job -- confidence and axiom reports not included in job results"))
    (medium
      ("No observability (metrics/tracing) -- hard to debug production issues"
       "No rate limiting -- vulnerable to webhook spam"))
    (low
      ("Codeberg adapter not implemented (enum variant exists)"
       "No pre-built prover images -- container startup slow"
       "No Docker Compose or Kubernetes manifests")))

  (critical-next-actions
    (immediate
      ("Wire trust bridge into process_job pipeline (confidence + axiom + integrity reports)"
       "Add observability (Prometheus metrics, OpenTelemetry tracing)"))

    (this-week
      ("Rate limiting for webhook endpoints"
       "Docker Compose setup for PostgreSQL + echidnabot + ECHIDNA"
       "Production deployment guide"))

    (this-month
      ("Pre-built Podman images for all 12 provers"
       "Performance benchmarks"
       "Codeberg adapter implementation")))

  (session-history
    ((session-2026-02-08-docs
       ((date . "2026-02-08")
        (focus . "Update all documentation to accurately reflect current capabilities")
        (accomplishments
          . ("Updated README.adoc with accurate feature descriptions"
            "Rewrote ROADMAP.adoc: struck completed items, added remaining work"
            "Updated STATE.scm with actual 107 test count and correct completion"
            "Noted trust bridge is implemented but not wired into main pipeline"
            "Verified ECOSYSTEM.scm and META.scm accuracy"))
        (blockers-resolved . ())
        (next-session-focus . "Wire trust bridge into pipeline and add observability")))

     (session-2026-02-08
       ((date . "2026-02-08")
        (focus . "Execute SONNET-TASKS.md: container isolation, bot modes, retry, tests, metadata")
        (accomplishments
          . ("Implemented Podman rootless container isolation with bwrap fallback"
            "Wired bot modes (Verifier/Advisor/Consultant/Regulator) into webhook handlers"
            "Added mode parsing from .bot_directives/echidnabot.scm"
            "Integrated retry logic with exponential backoff and circuit breaker"
            "Implemented ECHIDNA Trust Bridge (confidence, integrity, axiom tracking)"
            "Added 129 tests (30 integration, 99 unit)"
            "Fixed Cargo.toml license to PMPL-1.0-or-later"
            "Added SPDX headers to all .rs files"))
        (blockers-resolved . ("Container isolation was empty (now Podman+bwrap)"
                             "Bot modes not connected to handlers (now wired)"
                             "Retry logic not integrated (now circuit breaker + backoff)"
                             "Zero automated tests (now 129 passing)"))
        (next-session-focus . "Wire trust bridge into pipeline and production hardening")))

     (session-2026-01-29
       ((date . "2026-01-29")
        (focus . "Fix build issues and update documentation")
        (accomplishments
          . ("Fixed author email in Cargo.toml"
            "Restored deleted source files"
            "Created META.scm, ECOSYSTEM.scm"
            "Updated STATE.scm with current progress"
            "Identified high-priority blockers"))
        (blockers-resolved . ("Build errors" "Missing source files" "Wrong author"))
        (next-session-focus . "Implement container isolation and retry logic")))))

  ;; Helper functions for state queries
  (get-completion-percentage
    (lambda ()
      90))

  (get-blockers
    (lambda (priority)
      (cond
        ((eq? priority 'critical) '())
        ((eq? priority 'high)
         '("Trust bridge not wired into pipeline"))
        ((eq? priority 'medium)
         '("No observability"
           "No rate limiting"))
        ((eq? priority 'low)
         '("Codeberg adapter not implemented"
           "No pre-built prover images"
           "No Docker Compose")))))

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
        ((trust-bridge)
         '((status . implemented-not-wired) (completion . 80)))
        ((production-hardening)
         '((status . not-started) (completion . 0)))))))
