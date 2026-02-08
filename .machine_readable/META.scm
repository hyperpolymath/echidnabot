;; SPDX-License-Identifier: PMPL-1.0-or-later
;; META.scm - Meta-level information for echidnabot
;; Media-Type: application/meta+scheme

(define echidnabot-meta
  `((metadata
      ((name . "echidnabot")
       (full-name . "ECHIDNA CI Bot - Proof-Aware Continuous Integration")
       (tagline . "Automated formal verification in CI/CD pipelines")
       (version . "0.1.0")
       (status . "active-development")
       (license . "PMPL-1.0-or-later")
       (authors . ("Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>"))
       (created . "2025-01-03")
       (updated . "2026-02-08")
       (repository . "https://github.com/hyperpolymath/echidnabot")))

    (architecture-decisions
      ((adr-001
         ((title . "Multi-Platform Adapter Pattern")
          (status . "accepted")
          (date . "2025-01-05")
          (context . "Need to support GitHub, GitLab, and Bitbucket with consistent behavior but platform-specific APIs")
          (decision . "Implement PlatformAdapter trait with unified interface for clone_repo, create_check_run, create_comment, create_issue operations. Each platform implements this trait with platform-specific API clients.")
          (consequences
            ((positive . ("Consistent behavior across platforms"
                         "Easy to add new platforms (just implement trait)"
                         "Testable in isolation with mocks"))
             (negative . ("Additional abstraction layer"
                         "Platform-specific features harder to expose"))
             (mitigation . ("Start with common subset of features"
                           "Add platform-specific extensions as needed"))))))

       (adr-002
         ((title . "GraphQL API for Job Management")
          (status . "accepted")
          (date . "2025-01-06")
          (context . "Need API for querying verification job status, submitting jobs, and monitoring provers")
          (decision . "Use async-graphql for type-safe GraphQL schema with queries for job status and mutations for submission.")
          (consequences
            ((positive . ("Type-safe schema with compile-time checking"
                         "Self-documenting API with introspection"
                         "Efficient querying (client requests only needed fields)"))
             (negative . ("GraphQL learning curve for contributors"
                         "More complex than simple REST"))
             (mitigation . ("GraphQL Playground for interactive exploration"
                           "REST endpoints still available for webhooks"))))))

       (adr-003
         ((title . "PostgreSQL/SQLite for Job Queue and State")
          (status . "accepted")
          (date . "2025-01-08")
          (context . "Need persistent storage for verification jobs, results, and repository metadata")
          (decision . "Use PostgreSQL (production) and SQLite (development) with sqlx for compile-time checked queries.")
          (consequences
            ((positive . ("ACID transactions for job state"
                         "Compile-time query validation with sqlx"
                         "SQLite for easy local development"))
             (negative . ("PostgreSQL installation required for production"
                         "Migration complexity"))
             (mitigation . ("SQLite support for local development"
                           "sqlx migrations for schema versioning"))))))

       (adr-004
         ((title . "Webhook-Driven Architecture")
          (status . "accepted")
          (date . "2025-01-10")
          (context . "Need to trigger verification on push/PR events from GitHub/GitLab/Bitbucket")
          (decision . "Implement webhook receivers for each platform that verify signatures, parse events, and enqueue verification jobs")
          (consequences
            ((positive . ("Real-time response to code changes"
                         "No polling overhead"
                         "Platform-native integration"))
             (negative . ("Requires publicly accessible endpoint"
                         "Webhook signature verification complexity"))
             (mitigation . ("HMAC-SHA256 signature verification"
                           "Job queue with retries for failed verifications"))))))

       (adr-005
         ((title . "Integration with ECHIDNA Core")
          (status . "accepted")
          (date . "2025-01-12")
          (context . "echidnabot needs to delegate actual proof verification to ECHIDNA's 12 prover backends")
          (decision . "Use HTTP client (reqwest) to call ECHIDNA REST API. echidnabot handles CI orchestration, ECHIDNA handles verification.")
          (consequences
            ((positive . ("Clear separation of concerns"
                         "Can scale ECHIDNA independently"
                         "ECHIDNA upgrades don't require bot redeployment"))
             (negative . ("Network dependency between services"
                         "Need to handle ECHIDNA unavailability"))
             (mitigation . ("Health checks for ECHIDNA availability"
                           "Circuit breaker with auto-reset"
                           "Retry with exponential backoff"))))))

       (adr-006
         ((title . "Container Isolation with Podman and Bubblewrap")
          (status . "accepted")
          (date . "2026-02-08")
          (context . "Running untrusted code from pull requests is dangerous -- need security isolation")
          (decision . "Use Podman rootless containers as primary isolation backend with bubblewrap (bwrap) as fallback. Fail-safe: refuse to run proofs if neither is available.")
          (consequences
            ((positive . ("Security: malicious code cannot escape container"
                         "Resource limits prevent DoS"
                         "Podman is rootless (no daemon required)"
                         "bwrap fallback for systems without Podman"
                         "Fail-safe policy prevents unprotected execution"))
             (negative . ("Container startup overhead"
                         "Requires Podman or bwrap installed"
                         "bwrap provides less isolation than Podman"))
             (mitigation . ("Pre-built prover images to reduce startup time (future)"
                           "bwrap uses unshare-all for namespace isolation"
                           "Documentation clearly states isolation requirements"))))))

       (adr-007
         ((title . "Multi-Prover Support via ECHIDNA")
          (status . "accepted")
          (date . "2025-01-18")
          (context . "Different projects use different theorem provers (Coq, Lean, Agda, Isabelle, Z3, etc.)")
          (decision . "Support all 12 provers that ECHIDNA supports. Repository configuration specifies which provers to use.")
          (consequences
            ((positive . ("Support for diverse formal verification projects"
                         "Users choose best prover for their domain"))
             (negative . ("Complexity of supporting 12 different prover outputs"))
             (mitigation . ("ECHIDNA abstracts prover differences"
                           "Unified error reporting via ECHIDNA API"))))))

       (adr-008
         ((title . "Bot Modes: Verifier, Advisor, Consultant, Regulator")
          (status . "accepted")
          (date . "2025-01-20")
          (context . "Different teams want different levels of bot interaction")
          (decision . "Support 4 modes via repository config (.bot_directives/echidnabot.scm): Verifier (silent pass/fail), Advisor (tactic suggestions), Consultant (interactive Q&A on explicit mention), Regulator (merge blocking).")
          (consequences
            ((positive . ("Flexible to team preferences"
                         "Can evolve from silent to advisory"
                         "Consultant mode avoids noise (trigger on mention only)"))
             (negative . ("More configuration complexity"
                         "Need to implement all 4 modes"))
             (mitigation . ("Default to Verifier mode (simplest)"
                           "Documentation with examples for each mode"))))))

       (adr-009
         ((title . "ECHIDNA Trust Bridge: Confidence, Integrity, Axioms")
          (status . "accepted")
          (date . "2026-02-08")
          (context . "Proof verification results vary in trustworthiness depending on the prover's kernel size, certificate presence, and axiom usage")
          (decision . "Implement three trust mechanisms: (1) 5-level confidence assessment based on prover kernel, certificates, and cross-checking; (2) solver binary integrity verification against SHA-256 manifest; (3) axiom usage tracking to detect sorry, Admitted, postulate, and other unsoundness indicators.")
          (consequences
            ((positive . ("Users can distinguish high-confidence from low-confidence results"
                         "Detects tampered solver binaries before proof verification"
                         "Flags incomplete proofs (sorry, Admitted) automatically"
                         "3-tier severity enables appropriate responses"))
             (negative . ("Additional complexity in result reporting"
                         "SHA-256 manifest must be maintained per deployment"
                         "Axiom pattern matching is heuristic (may miss obfuscated patterns)"))
             (mitigation . ("Confidence levels have clear documentation"
                           "Manifest is optional (Unchecked status, not failure)"
                           "Universal patterns supplement prover-specific ones"))))))))

    (development-practices
      ((code-style
         ((rust . "Follow Rust API guidelines, use clippy, rustfmt")
          (graphql . "Use async-graphql conventions, schema-first design")
          (documentation . "AsciiDoc for user docs, rustdoc for API docs")))

       (security
         ((principle . "Defense in depth")
          (practices . ("Webhook signature verification (HMAC-SHA256)"
                       "Container isolation (Podman rootless + bwrap fallback)"
                       "Read-only filesystems in containers"
                       "Resource limits (CPU/memory/pids/time)"
                       "No network access from verification containers"
                       "Fail-safe: no isolation = no proof execution"
                       "Solver integrity verification (SHA-256)"
                       "Constant-time hash comparison"
                       "Dependency scanning with cargo audit"))))

       (testing
         ((total-tests . 129)
          (unit-tests . "99 tests across 12 modules")
          (integration-tests . "30 tests covering end-to-end flows")
          (test-approach . "Mocks for external services, real for internal logic")))

       (versioning
         ((scheme . "Semantic versioning (major.minor.patch)")
          (policy . "Major: breaking API changes, Minor: new features, Patch: bugfixes")))

       (documentation
         ((formats . ("AsciiDoc for README and guides"
                     "Rustdoc for Rust API documentation"
                     "GraphQL introspection for API schema"
                     "Guile Scheme for machine-readable metadata"))))))

    (design-rationale
      ((why-multi-platform-adapters
         "Different teams use different platforms. Supporting all three via unified trait means echidnabot works everywhere without platform lock-in.")

       (why-graphql-api
         "GraphQL provides type-safe, self-documenting API with efficient querying. Clients request exactly the fields they need.")

       (why-sqlite-and-postgresql
         "SQLite for development simplicity, PostgreSQL for production ACID guarantees. sqlx supports both with compile-time query validation.")

       (why-webhooks-not-polling
         "Webhooks provide real-time response with no polling overhead. Check runs appear instantly in platform UI.")

       (why-echidna-integration
         "echidnabot handles CI orchestration, ECHIDNA handles verification. Clear separation of concerns, independent scaling.")

       (why-podman-not-docker
         "Podman is rootless (no daemon), more secure by default. bwrap fallback ensures isolation even on minimal systems. Fail-safe policy prevents unprotected execution.")

       (why-four-bot-modes
         "Teams have different needs. Verifier for minimal noise, Advisor for learning, Consultant for on-demand analysis, Regulator for enforcement.")

       (why-trust-bridge
         "Not all proof results are equally trustworthy. Small-kernel provers with certificates earn higher confidence than large-TCB systems. Axiom tracking catches incomplete proofs that pass syntactically.")))))
