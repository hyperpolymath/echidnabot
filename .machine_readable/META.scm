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
       (updated . "2026-01-29")
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
                         "Platform differences abstracted away"
                         "Testable in isolation with mocks"))
             (negative . ("Additional abstraction layer"
                         "Platform-specific features harder to expose"
                         "Trait may need evolution as platforms differ"))
             (mitigation . ("Start with common subset of features"
                           "Add platform-specific extensions as needed"
                           "Use feature flags for platform-specific code"))))))

       (adr-002
         ((title . "GraphQL API for Job Management")
          (status . "accepted")
          (date . "2025-01-06")
          (context . "Need API for querying verification job status, submitting jobs, and monitoring provers")
          (decision . "Use async-graphql for type-safe GraphQL schema with queries for job status and mutations for submission. Provides better introspection than REST.")
          (consequences
            ((positive . ("Type-safe schema with compile-time checking"
                         "Self-documenting API with introspection"
                         "Efficient querying (client requests only needed fields)"
                         "Real-time updates via subscriptions (future)"))
             (negative . ("GraphQL learning curve for contributors"
                         "More complex than simple REST"
                         "Additional dependency"))
             (mitigation . ("Provide example queries in documentation"
                           "GraphQL Playground for interactive exploration"
                           "REST endpoints still available for webhooks"))))))

       (adr-003
         ((title . "PostgreSQL for Job Queue and State")
          (status . "accepted")
          (date . "2025-01-08")
          (context . "Need persistent storage for verification jobs, results, and repository metadata")
          (decision . "Use PostgreSQL with sqlx for compile-time checked queries. Provides ACID guarantees and rich querying.")
          (consequences
            ((positive . ("ACID transactions for job state"
                         "Complex queries for job history and statistics"
                         "Compile-time query validation with sqlx macros"
                         "Well-understood operational model"))
             (negative . ("PostgreSQL installation required"
                         "More heavyweight than SQLite for development"
                         "Migration complexity"))
             (mitigation . ("SQLite support for local development"
                           "Docker Compose for easy PostgreSQL setup"
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
                         "Platform-native integration"
                         "Scales with event volume"))
             (negative . ("Requires publicly accessible endpoint"
                         "Webhook signature verification complexity"
                         "Event delivery not guaranteed (need retry logic)"))
             (mitigation . ("HMAC signature verification for all platforms"
                           "Job queue with retries for failed verifications"
                           "Health checks and monitoring for webhook endpoint"))))))

       (adr-005
         ((title . "Integration with ECHIDNA Core")
          (status . "accepted")
          (date . "2025-01-12")
          (context . "echidnabot needs to delegate actual proof verification to ECHIDNA's 12 prover backends")
          (decision . "Use HTTP client (reqwest) to call ECHIDNA REST API at configured URL. echidnabot handles CI orchestration, ECHIDNA handles verification.")
          (consequences
            ((positive . ("Clear separation of concerns"
                         "Can scale ECHIDNA independently"
                         "Multiple echidnabot instances can share ECHIDNA"
                         "ECHIDNA upgrades don't require bot redeployment"))
             (negative . ("Network dependency between services"
                         "Additional latency for HTTP roundtrip"
                         "Need to handle ECHIDNA unavailability"))
             (mitigation . ("Health checks for ECHIDNA availability"
                           "Graceful degradation if ECHIDNA down"
                           "Configurable timeouts and retries"
                           "Local ECHIDNA instance for development"))))))

       (adr-006
         ((title . "Container Isolation for Proof Verification")
          (status . "accepted")
          (date . "2025-01-15")
          (context . "Running untrusted code from pull requests is dangerous - need security isolation")
          (decision . "Each verification runs in isolated Docker container with read-only filesystem, limited CPU/memory, no network access")
          (consequences
            ((positive . ("Security: malicious code can't escape container"
                         "Resource limits prevent DoS"
                         "Reproducible environment for proofs"
                         "Easy cleanup after verification"))
             (negative . ("Docker dependency"
                         "Container startup overhead"
                         "More complex local development"))
             (mitigation . ("Pre-built prover images to reduce startup time"
                           "Container caching and reuse"
                           "Development mode without containers for fast iteration"))))))

       (adr-007
         ((title . "Multi-Prover Support via ECHIDNA")
          (status . "accepted")
          (date . "2025-01-18")
          (context . "Different projects use different theorem provers (Coq, Lean, Agda, Isabelle, Z3, etc.)")
          (decision . "Support all 12 provers that ECHIDNA supports. Repository configuration specifies which provers to use.")
          (consequences
            ((positive . ("Support for diverse formal verification projects"
                         "Users choose best prover for their domain"
                         "Cross-prover validation possible"))
             (negative . ("Complexity of supporting 12 different prover outputs"
                         "Each prover has different error formats"))
             (mitigation . ("ECHIDNA abstracts prover differences"
                           "Unified error reporting via ECHIDNA API"
                           "Repository config enables only needed provers"))))))

       (adr-008
         ((title . "Bot Modes: Verifier, Advisor, Consultant, Regulator")
          (status . "accepted")
          (date . "2025-01-20")
          (context . "Different teams want different levels of bot interaction - some want silent checks, others want suggestions")
          (decision . "Support 4 modes via repository config: Verifier (silent pass/fail), Advisor (suggests tactics on failure), Consultant (answers questions), Regulator (blocks merge on failure)")
          (consequences
            ((positive . ("Flexible to team preferences"
                         "Can evolve from silent to advisory as team learns"
                         "Prevents merge of broken proofs when needed"))
             (negative . ("More configuration complexity"
                         "Need to implement all 4 modes"
                         "User education needed"))
             (mitigation . ("Default to Verifier mode (simplest)"
                           "Documentation with examples for each mode"
                           "Gradual rollout: Verifier first, others as features mature"))))))))

    (development-practices
      ((code-style
         ((rust . "Follow Rust API guidelines, use clippy, rustfmt")
          (graphql . "Use async-graphql conventions, schema-first design")
          (documentation . "AsciiDoc for user docs, rustdoc for API docs")))

       (security
         ((principle . "Defense in depth")
          (practices . ("Webhook signature verification (HMAC-SHA256)"
                       "Container isolation for proof execution"
                       "Read-only filesystems in containers"
                       "Resource limits (CPU/memory/time)"
                       "No network access from verification containers"
                       "Input sanitization for all external data"
                       "Dependency scanning with cargo audit"))))

       (testing
         ((unit-tests . "Every module has unit tests")
          (integration-tests . "End-to-end webhook → verification flow")
          (mock-adapters . "Platform adapters tested with mocks")
          (database-tests . "Test migrations and queries with test DB")))

       (versioning
         ((scheme . "Semantic versioning (major.minor.patch)")
          (policy . "Major: breaking API changes, Minor: new features, Patch: bugfixes")
          (compatibility . "Maintain API compatibility within major version")))

       (documentation
         ((formats . ("AsciiDoc for README and guides"
                     "Rustdoc for Rust API documentation"
                     "GraphQL introspection for API schema"))
          (audiences . ("Developers: API reference, architecture"
                       "Users: Setup guides, configuration reference"
                       "Platform admins: Deployment, security"))))))

    (design-rationale
      ((why-multi-platform-adapters
         "Different teams use different platforms (GitHub, GitLab, Bitbucket). Supporting all three via unified trait means echidnabot works everywhere without platform lock-in.")

       (why-graphql-api
         "GraphQL provides type-safe, self-documenting API with efficient querying. Clients can request exactly the fields they need, reducing bandwidth. Introspection enables auto-generated client libraries.")

       (why-postgresql
         "PostgreSQL provides ACID transactions for job state consistency, rich querying for analytics, and compile-time query validation via sqlx. Better than Redis/queue for complex state.")

       (why-webhooks-not-polling
         "Webhooks provide real-time response to code changes with no polling overhead. Platform-native integration means check runs appear instantly in GitHub/GitLab UI.")

       (why-echidna-integration
         "echidnabot handles CI orchestration (webhooks, job queue, platform APIs), ECHIDNA handles verification (12 provers, ML suggestions, soundness). Clear separation of concerns.")

       (why-container-isolation
         "Running untrusted code from PRs is dangerous. Docker containers provide security isolation with resource limits. Read-only filesystem + no network prevents malicious code from causing harm.")

       (why-multi-prover-support
         "Different projects use different provers. Coq for CompCert, Lean for mathlib, Z3 for SMT problems. Supporting all 12 via ECHIDNA means echidnabot works for all formal verification projects.")

       (why-bot-modes
         "Teams have different needs. Some want silent checks, others want suggestions. Modes let teams choose: Verifier (silent), Advisor (helpful), Consultant (interactive), Regulator (blocking).")))))
