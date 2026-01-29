;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Ecosystem position for echidnabot
;; Media-Type: application/vnd.ecosystem+scm

(ecosystem
  ((metadata
     ((version . "1.0")
      (name . "echidnabot")
      (type . "ci-bot-formal-verification")
      (purpose . "Automated proof verification in CI/CD pipelines")))

   (position-in-ecosystem
     "echidnabot bridges code hosting platforms (GitHub, GitLab, Bitbucket) and the ECHIDNA neurosymbolic theorem prover. It acts as a CI/CD orchestrator for formal verification, automatically checking proofs on every push and PR.")

   (related-projects
     ((core-dependency
        ((echidna
           ((relationship . "required-backend")
            (description . "ECHIDNA neurosymbolic theorem prover with 12 prover backends")
            (interaction . "HTTP client calls ECHIDNA REST API for proof verification")
            (url . "https://github.com/hyperpolymath/echidna")))))

      (code-platforms
        ((github
           ((relationship . "platform-integration")
            (description . "GitHub code hosting and CI/CD")
            (interaction . "Webhook receiver, Check Runs API, PR comments")
            (url . "https://github.com")))

         (gitlab
           ((relationship . "platform-integration")
            (description . "GitLab code hosting and CI/CD")
            (interaction . "Webhook receiver, commit statuses, MR notes")
            (url . "https://gitlab.com")))

         (bitbucket
           ((relationship . "platform-integration")
            (description . "Bitbucket code hosting and CI/CD")
            (interaction . "Webhook receiver, build statuses, PR comments")
            (url . "https://bitbucket.org")))))

      (theorem-provers
        ((coq
           ((relationship . "verified-via-echidna")
            (description . "Coq proof assistant")
            (interaction . "echidnabot → ECHIDNA → Coq verification")
            (url . "https://coq.inria.fr")))

         (lean4
           ((relationship . "verified-via-echidna")
            (description . "Lean 4 theorem prover")
            (interaction . "echidnabot → ECHIDNA → Lean verification")
            (url . "https://lean-lang.org")))

         (isabelle
           ((relationship . "verified-via-echidna")
            (description . "Isabelle/HOL proof assistant")
            (interaction . "echidnabot → ECHIDNA → Isabelle verification")
            (url . "https://isabelle.in.tum.de")))

         (agda
           ((relationship . "verified-via-echidna")
            (description . "Agda dependently typed language")
            (interaction . "echidnabot → ECHIDNA → Agda verification")
            (url . "https://wiki.portal.chalmers.se/agda")))

         (z3
           ((relationship . "verified-via-echidna")
            (description . "Z3 SMT solver")
            (interaction . "echidnabot → ECHIDNA → Z3 verification")
            (url . "https://github.com/Z3Prover/z3")))

         (cvc5
           ((relationship . "verified-via-echidna")
            (description . "CVC5 SMT solver")
            (interaction . "echidnabot → ECHIDNA → CVC5 verification")
            (url . "https://cvc5.github.io")))

         (metamath
           ((relationship . "verified-via-echidna")
            (description . "Metamath minimal proof checker")
            (interaction . "echidnabot → ECHIDNA → Metamath verification")
            (url . "http://us.metamath.org")))

         (hol-light
           ((relationship . "verified-via-echidna")
            (description . "HOL Light proof assistant")
            (interaction . "echidnabot → ECHIDNA → HOL Light verification")
            (url . "https://www.cl.cam.ac.uk/~jrh13/hol-light")))

         (pvs
           ((relationship . "verified-via-echidna")
            (description . "PVS verification system")
            (interaction . "echidnabot → ECHIDNA → PVS verification")
            (url . "https://pvs.csl.sri.com")))

         (acl2
           ((relationship . "verified-via-echidna")
            (description . "ACL2 theorem prover")
            (interaction . "echidnabot → ECHIDNA → ACL2 verification")
            (url . "https://www.cs.utexas.edu/users/moore/acl2")))

         (hol4
           ((relationship . "verified-via-echidna")
            (description . "HOL4 proof assistant")
            (interaction . "echidnabot → ECHIDNA → HOL4 verification")
            (url . "https://hol-theorem-prover.org")))

         (mizar
           ((relationship . "verified-via-echidna")
            (description . "Mizar proof checker")
            (interaction . "echidnabot → ECHIDNA → Mizar verification")
            (url . "http://mizar.org")))))

      (rust-ecosystem
        ((tokio
           ((relationship . "runtime-dependency")
            (description . "Async runtime for Rust")
            (usage . "Async webhook server, concurrent job execution")
            (url . "https://tokio.rs")))

         (axum
           ((relationship . "web-framework")
            (description . "Ergonomic async web framework")
            (usage . "HTTP server for webhooks and GraphQL API")
            (url . "https://github.com/tokio-rs/axum")))

         (async-graphql
           ((relationship . "graphql-framework")
            (description . "GraphQL server library for Rust")
            (usage . "Type-safe GraphQL API for job queries/mutations")
            (url . "https://github.com/async-graphql/async-graphql")))

         (sqlx
           ((relationship . "database-library")
            (description . "Async SQL toolkit with compile-time checking")
            (usage . "PostgreSQL access for job queue and state")
            (url . "https://github.com/launchbadge/sqlx")))

         (reqwest
           ((relationship . "http-client")
            (description . "HTTP client for Rust")
            (usage . "Calls to ECHIDNA API and platform APIs")
            (url . "https://github.com/seanmonstar/reqwest")))

         (octocrab
           ((relationship . "github-client")
            (description . "GitHub API client")
            (usage . "GitHub Check Runs, PR comments, issues")
            (url . "https://github.com/XAMPPRocky/octocrab")))))

      (databases
        ((postgresql
           ((relationship . "primary-database")
            (description . "PostgreSQL relational database")
            (usage . "Job queue, verification results, repository config")
            (url . "https://www.postgresql.org")))

         (sqlite
           ((relationship . "development-database")
            (description . "SQLite embedded database")
            (usage . "Local development and testing")
            (url . "https://www.sqlite.org")))))

      (containerization
        ((docker
           ((relationship . "security-isolation")
            (description . "Container runtime")
            (usage . "Isolated proof verification with resource limits")
            (url . "https://www.docker.com")))

         (podman
           ((relationship . "docker-alternative")
            (description . "Daemonless container engine")
            (usage . "Alternative to Docker for rootless containers")
            (url . "https://podman.io")))))

      (gitbot-fleet
        ((rhodibot
           ((relationship . "sibling-bot")
            (description . "RSR structural compliance bot")
            (url . "https://github.com/hyperpolymath/rhodibot")))

         (seambot
           ((relationship . "sibling-bot")
            (description . "Architectural seam hygiene bot")
            (url . "https://github.com/hyperpolymath/seambot")))

         (finishingbot
           ((relationship . "sibling-bot")
            (description . "Release readiness bot")
            (url . "https://github.com/hyperpolymath/finishingbot")))

         (glambot
           ((relationship . "sibling-bot")
            (description . "Presentation quality bot")
            (url . "https://github.com/hyperpolymath/glambot")))

         (hypatia
           ((relationship . "coordination-layer")
            (description . "Shared context manager for gitbot fleet")
            (url . "https://github.com/hyperpolymath/hypatia")))))

      (formal-verification-projects
        ((compcert
           ((relationship . "potential-user")
            (description . "Verified C compiler in Coq")
            (usage . "Could use echidnabot for CI verification of CompCert proofs")))

         (mathlib
           ((relationship . "potential-user")
            (description . "Mathematical library for Lean")
            (usage . "Could use echidnabot for CI verification of mathlib contributions")))

         (isabelle-afp
           ((relationship . "potential-user")
            (description . "Archive of Formal Proofs for Isabelle")
            (usage . "Could use echidnabot for CI verification of AFP entries")))

         (fiat-crypto
           ((relationship . "potential-user")
            (description . "Cryptographic primitives in Coq")
            (usage . "Could use echidnabot for CI verification of cryptographic proofs")))))))

   (what-this-is
     ("A CI/CD bot that automatically verifies formal proofs on every push and PR"
      "An orchestrator that bridges GitHub/GitLab/Bitbucket and ECHIDNA theorem prover"
      "A webhook server with GraphQL API for job management and monitoring"
      "A security-first system using container isolation for untrusted proof verification"
      "A multi-platform adapter supporting 3 code hosting platforms and 12 theorem provers"
      "An automated gatekeeper that can block merges when proofs fail (Regulator mode)"))

   (what-this-is-not
     ("Not a theorem prover itself - delegates to ECHIDNA for actual verification"
      "Not limited to one platform - supports GitHub, GitLab, and Bitbucket equally"
      "Not just for Coq - supports all 12 provers that ECHIDNA supports"
      "Not a replacement for human proof engineers - assists and automates, doesn't replace"
      "Not closed-source - fully open PMPL-1.0 licensed"
      "Not research-only - production-ready CI/CD integration"))))
