;; SPDX-License-Identifier: PMPL-1.0-or-later
;; AGENTIC.scm - AI agent interaction patterns for echidnabot

(define agentic-config
  `((version . "1.0.0")
    (updated . "2026-01-29")

    (claude-code
      ((model . "claude-sonnet-4-5-20250929")
       (tools . ("read" "edit" "bash" "grep" "glob" "write"))
       (permissions . "read-all")
       (context . "echidnabot - CI/CD bot for formal proof verification")))

    (interaction-patterns
      ((code-review
         ((thoroughness . "comprehensive")
          (focus . ("security vulnerabilities in webhook handlers"
                   "race conditions in job scheduler"
                   "SQL injection in database queries"
                   "container escape vulnerabilities"
                   "webhook signature verification correctness"))
          (safety-first . #t)))

       (refactoring
         ((approach . "conservative")
          (priorities . ("maintain type safety"
                        "preserve async/await patterns"
                        "keep database transactions atomic"
                        "ensure webhook signature verification intact"))
          (avoid . ("breaking PlatformAdapter trait API"
                   "changing GraphQL schema without versioning"
                   "modifying database schema without migration"))))

       (testing
         ((coverage . "comprehensive")
          (required . ("unit tests for all adapters"
                      "webhook signature verification tests"
                      "job queue tests with PostgreSQL"
                      "GraphQL API tests"
                      "container isolation tests (when implemented)"))
          (mocking . ("GitHub/GitLab/Bitbucket APIs"
                     "ECHIDNA HTTP responses"
                     "Docker container execution"))))

       (security-review
         ((critical-paths . ("webhook signature verification"
                            "container resource limits"
                            "SQL query parameterization"
                            "secrets management"
                            "rate limiting"))
          (threat-model . ("malicious PR code execution"
                          "webhook replay attacks"
                          "DoS via webhook spam"
                          "SQL injection via repository names"
                          "container escape attempts"))))))

    (language-constraints
      ((allowed . ("rust" "toml" "sql" "asciidoc" "scheme"))
       (banned . ("typescript" "javascript" "python" "go" "makefile"))
       (rationale . "Rust for type safety, Scheme for meta, AsciiDoc for docs")))

    (architecture-guidance
      ((multi-platform-adapters
         ((principle . "Platform differences abstracted via PlatformAdapter trait")
          (implementation . "Each platform (GitHub/GitLab/Bitbucket) implements trait")
          (guidance . "Add new platforms by implementing trait, not modifying core")))

       (job-scheduler
         ((principle . "PostgreSQL for persistence, async for concurrency")
          (implementation . "sqlx with compile-time query checking")
          (guidance . "Use transactions for atomic state changes, retry with backoff")))

       (webhook-security
         ((principle . "Verify every webhook with HMAC-SHA256 before processing")
          (implementation . "Platform-specific signature verification in adapters")
          (guidance . "Reject invalid signatures immediately, log all attempts")))

       (container-isolation
         ((principle . "Every proof verification runs in isolated Docker container")
          (implementation . "Read-only filesystem, CPU/memory/time limits, no network")
          (guidance . "Pre-built images for provers, cleanup after execution")))))

    (operational-constraints
      ((database
         ((primary . "postgresql")
          (development . "sqlite")
          (migrations . "sqlx migrate")
          (backup-frequency . "daily")))

       (echidna-integration
         ((endpoint . "http://localhost:8080")
          (timeout . 300)
          (retry-strategy . "exponential-backoff")
          (fallback . "graceful degradation if ECHIDNA unavailable")))

       (webhook-endpoint
         ((binding . "0.0.0.0:3000")
          (tls . "recommended for production")
          (rate-limiting . "100 webhooks per minute per repository")
          (signature-verification . "mandatory")))

       (container-runtime
         ((preferred . "docker")
          (alternative . "podman")
          (resource-limits
            ((cpu . "2 cores per verification")
             (memory . "4GB per verification")
             (timeout . "300 seconds per verification")))))))

    (bot-mode-behaviors
      ((verifier-mode
         ((description . "Silent pass/fail checks on proof files")
          (actions . ("Create check run with status"
                     "Comment on PR only if failure"
                     "No tactic suggestions"))
          (use-case . "Teams wanting minimal bot interaction")))

       (advisor-mode
         ((description . "Helpful suggestions on proof failures")
          (actions . ("Create check run with status"
                     "Comment on PR with failure details"
                     "Suggest tactics via ECHIDNA ML API"
                     "Provide proof context and premises"))
          (use-case . "Teams learning theorem proving")))

       (consultant-mode
         ((description . "Interactive Q&A about proof state")
          (actions . ("Respond to @echidnabot questions in comments"
                     "Explain proof goals and subgoals"
                     "Suggest lemmas and tactics"
                     "Reference documentation"))
          (use-case . "Experienced teams needing occasional help")))

       (regulator-mode
         ((description . "Block PR merges when proofs fail")
          (actions . ("Create required check run (blocks merge)"
                     "Detailed failure explanations"
                     "Tactic suggestions"
                     "Override mechanism for emergencies"))
          (use-case . "Critical projects requiring proof correctness")))))

    (development-workflow
      ((local-testing
         ((setup . "docker-compose up -d postgres echidna")
          (build . "cargo build")
          (test . "cargo test")
          (run . "cargo run -- serve --port 3000")))

       (ci-pipeline
         ((stages . ("lint" "test" "build" "security-scan"))
          (lint . "cargo clippy -- -D warnings")
          (test . "cargo test --all-features")
          (security . "cargo audit")))

       (deployment
         ((environments . ("development" "staging" "production"))
          (strategy . "blue-green with health checks")
          (rollback . "automatic on health check failure")))))

    (observability
      ((metrics
         ((prometheus-endpoint . "/metrics")
          (key-metrics . ("webhook_received_total"
                         "verification_duration_seconds"
                         "verification_success_total"
                         "verification_failure_total"
                         "echidna_api_calls_total"
                         "container_spawns_total"
                         "job_queue_length"))))

       (tracing
         ((provider . "opentelemetry")
          (spans . ("webhook-processing"
                   "job-scheduling"
                   "echidna-verification"
                   "result-reporting"))
          (export . "jaeger or tempo")))

       (logging
         ((format . "json")
          (levels . "info in production, debug in development")
          (structured-fields . ("request_id" "repository" "commit_sha" "prover" "job_id"))))))))
