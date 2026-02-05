;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Current project state for echidnabot

(define project-state
  `((metadata
      ((version . "0.1.0")
       (schema-version . "1")
       (created . "2026-01-03")
       (updated . "2026-02-05")
       (project . "echidnabot")
       (repo . "hyperpolymath/echidnabot")))
    (project-context
      ((name . "echidnabot")
       (tagline . "Proof-aware CI bot for formal verification in PRs and pushes")
       (tech-stack . ("Rust 1.75+" "Tokio async runtime" "Axum web framework"
                      "async-graphql" "sqlx (PostgreSQL/SQLite)" "octocrab (GitHub API)"
                      "reqwest HTTP client"))))
    (current-position
      ((phase . "Active development — core infrastructure complete, integration in progress")
       (overall-completion . 75)
       (components
         . (("HTTP Server & Health" . ((status . "complete") (completion . 100)))
            ("Webhook Receivers (3 platforms)" . ((status . "complete") (completion . 100)))
            ("Platform Adapters (GitHub/GitLab/Bitbucket)" . ((status . "complete") (completion . 100)))
            ("GraphQL API (jobs/mutations)" . ((status . "complete") (completion . 100)))
            ("Database Layer (sqlx migrations)" . ((status . "complete") (completion . 100)))
            ("ECHIDNA API Integration" . ((status . "complete") (completion . 100)))
            ("Repository Registration & Config" . ((status . "complete") (completion . 100)))
            ("Job Status Tracking" . ((status . "complete") (completion . 100)))
            ("Job Scheduler & Queue" . ((status . "partial") (completion . 60)))
            ("Container Isolation" . ((status . "planned") (completion . 0)))
            ("Bot Modes (Verifier/Advisor/Consultant/Regulator)" . ((status . "planned") (completion . 0)))
            ("Retry Logic (exponential backoff)" . ((status . "planned") (completion . 0)))
            ("Concurrent Job Limits" . ((status . "planned") (completion . 0)))
            ("Production Hardening" . ((status . "planned") (completion . 0)))))
       (working-features . ("HTTP server with health checks"
                           "Webhook receivers for GitHub, GitLab, Bitbucket"
                           "Platform adapter abstraction for multi-forge support"
                           "GraphQL API for job queries and mutations"
                           "PostgreSQL/SQLite database with sqlx migrations"
                           "ECHIDNA API integration for multi-prover verification"
                           "Repository registration and per-repo configuration"
                           "Job status tracking and lifecycle management"))))
    (route-to-mvp
      ((milestones
        ((v0.1 . ((items . ("Core infrastructure (HTTP, webhooks, DB)" "Platform adapters"
                            "GraphQL API" "ECHIDNA integration" "Job tracking"))
                  (status . "complete")))
         (v0.2 . ((items . ("Job scheduler with queue" "Container isolation (Docker)"
                            "Retry logic with exponential backoff" "Concurrent job limits"))
                  (status . "in-progress")))
         (v1.0 . ((items . ("Bot modes (Verifier/Advisor/Consultant/Regulator)"
                            "Production hardening" "Fleet integration"
                            "Monitoring and alerting"))
                  (status . "planned")))))))
    (blockers-and-issues
      ((critical . ())
       (high . (("Container isolation not implemented" . "Security risk: prover execution is unsandboxed")
                ("No retry logic for failed jobs" . "Transient failures are permanent")
                ("No concurrent job limits" . "Could overwhelm prover backends")))
       (medium . (("Bot modes not implemented" . "Only one operating mode available")
                  ("Fleet integration pending" . "Not yet connected to gitbot-shared-context")))
       (low . ())))
    (critical-next-actions
      ((immediate . ("Implement Docker container isolation for prover execution"
                     "Add retry logic with exponential backoff"))
       (this-week . ("Add concurrent job limits"
                     "Test multi-prover verification end-to-end"))
       (this-month . ("Implement bot modes (Verifier, Advisor, Consultant, Regulator)"
                      "Integrate with gitbot-fleet shared-context"
                      "Production hardening and monitoring"))))
    (session-history
      (((date . "2026-02-05")
        (session . "opus-checkpoint-update")
        (accomplishments . ("Updated STATE.scm from stub to comprehensive state"
                           "Updated ECOSYSTEM.scm with full relationship mapping"
                           "Updated META.scm with domain-specific ADRs")))
       ((date . "2026-02-01")
        (session . "maintenance")
        (accomplishments . ("Fixed author email in Cargo.toml"
                           "Restored deleted source files"
                           "Created comprehensive .machine_readable/ META.scm"
                           "Cleaned and rebuilt successfully"
                           "Identified 3 high-priority blockers")))
       ((date . "2026-01-29")
        (session . "infrastructure")
        (accomplishments . ("Created comprehensive .machine_readable/ STATE.scm"
                           "Created comprehensive .machine_readable/ ECOSYSTEM.scm"
                           "Documented all 8 ADRs")))))))
