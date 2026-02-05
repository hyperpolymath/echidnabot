;; SPDX-License-Identifier: PMPL-1.0-or-later
;; META.scm - Meta-level information for echidnabot
;; Media type: application/vnd.meta+scm

(define meta
  `((metadata
      ((version . "1.0.0")
       (schema-version . "1.0")
       (created . "2026-01-03")
       (updated . "2026-02-05")
       (project . "echidnabot")))
    (architecture-decisions
      ((adr-001 . ((status . "accepted")
                   (date . "2026-01-03")
                   (context . "Need a CI bot for formal verification")
                   (decision . "Build echidnabot in Rust with Tokio/Axum for async
                     webhook handling. Use ECHIDNA as the verification backend
                     via HTTP API calls.")
                   (consequences . "High performance, memory safety. Tokio enables
                     concurrent job processing. Axum provides ergonomic HTTP.")))
       (adr-002 . ((status . "accepted")
                   (date . "2026-01-03")
                   (context . "Need to support multiple code forges")
                   (decision . "Multi-platform adapter pattern with trait-based
                     abstraction. GitHub (octocrab), GitLab, Bitbucket adapters
                     implement common PlatformAdapter trait.")
                   (consequences . "Bot works across all three major forges.
                     Adding new forges requires only a new adapter implementation.")))
       (adr-003 . ((status . "accepted")
                   (date . "2026-01-10")
                   (context . "Need flexible job management API")
                   (decision . "GraphQL API via async-graphql for job queries and
                     mutations. Provides flexible querying without REST endpoint
                     proliferation.")
                   (consequences . "Clients can request exactly the fields they need.
                     Schema is self-documenting. Playground available for debugging.")))
       (adr-004 . ((status . "accepted")
                   (date . "2026-01-10")
                   (context . "Need persistent job state")
                   (decision . "sqlx with PostgreSQL for production, SQLite for
                     development/testing. Migration-based schema management.")
                   (consequences . "Type-safe SQL queries. Smooth dev-to-prod transition.
                     Migrations ensure schema consistency.")))
       (adr-005 . ((status . "accepted")
                   (date . "2026-01-15")
                   (context . "ECHIDNA supports 12 theorem provers")
                   (decision . "Echidnabot delegates all prover execution to ECHIDNA
                     via HTTP API. Per-repo configuration determines which provers
                     to use (Coq, Lean, Isabelle, Z3, etc.).")
                   (consequences . "Echidnabot stays thin — it handles CI workflow,
                     ECHIDNA handles verification. Prover selection is configurable
                     per repository.")))
       (adr-006 . ((status . "proposed")
                   (date . "2026-02-05")
                   (context . "Prover execution must be sandboxed")
                   (decision . "Docker container isolation for all prover execution.
                     Each verification job runs in its own container with resource
                     limits (CPU, memory, time).")
                   (consequences . "Security: untrusted proof code cannot affect host.
                     Resource limits prevent runaway provers. Adds Docker dependency.")))
       (adr-007 . ((status . "proposed")
                   (date . "2026-02-05")
                   (context . "Different repos need different verification behavior")
                   (decision . "Four bot modes: Verifier (blocks on proof failure),
                     Advisor (suggests improvements), Consultant (reports only),
                     Regulator (enforces policies). Mode configured per-repo via
                     .bot_directives/echidnabot.scm.")
                   (consequences . "Flexible operation. Repos can opt into strict
                     verification or advisory mode. Regulator mode enables policy
                     enforcement.")))
       (adr-008 . ((status . "proposed")
                   (date . "2026-02-05")
                   (context . "Need to connect echidnabot to ECHIDNA's correctness certification")
                   (decision . "Generate proof certificates for verified theorems.
                     Certificates include prover output, ML confidence, anomaly flags,
                     cross-validation results, and SHA-256 hash for tamper-evidence.")
                   (consequences . "Users get machine-checkable proof of correctness.
                     Certificates can be published alongside code. Enables regulatory
                     compliance for safety-critical systems.")))))
    (development-practices
      ((code-style . "Rust with rustfmt. Modules: adapters/, api/, dispatcher/,
         scheduler/, store/. 18+ modules covering full CI bot lifecycle.")
       (security . "JWT for forge authentication. Container isolation for prover
         execution. No secrets in source. cargo-audit for dependency scanning.")
       (testing . "cargo test. Integration tests for webhook handling. End-to-end
         tests with ECHIDNA API mock.")
       (versioning . "Semantic versioning")
       (documentation . "Rustdoc. README.adoc. STATE.scm for project tracking.")
       (branching . "GitHub Flow. Feature branches. PRs required.")))
    (design-rationale
      ((why-separate-from-echidna
        . "ECHIDNA is the verification engine; echidnabot is the CI interface.
           Separation of concerns: ECHIDNA handles theorem proving, echidnabot
           handles webhooks, job management, and forge integration. Different
           deployment models (ECHIDNA runs as service, echidnabot runs as GitHub App).")
       (why-multi-prover
        . "Different proof assistants have different strengths. Coq for dependent
           types, Lean for modern tactics, Z3 for SMT, Isabelle for automation.
           Multi-prover support lets each repo use the right tool.")
       (why-graphql
        . "Verification jobs have complex state (prover output, ML confidence,
           anomaly flags, cross-validation). GraphQL lets clients query exactly
           the data they need without REST endpoint proliferation.")))))
