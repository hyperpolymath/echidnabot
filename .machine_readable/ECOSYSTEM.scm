;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Ecosystem position for echidnabot
;; Media-Type: application/vnd.ecosystem+scm

(ecosystem
  ((metadata
     ((version . "1.1")
      (name . "echidnabot")
      (type . "ci-bot-formal-verification")
      (purpose . "Automated proof verification in CI/CD pipelines")
      (updated . "2026-02-08")))

   (position-in-ecosystem
     "echidnabot bridges code hosting platforms (GitHub, GitLab, Bitbucket) and the ECHIDNA neurosymbolic theorem prover. It acts as a CI/CD orchestrator for formal verification, automatically checking proofs on every push and PR. It runs proof verification in isolated containers (Podman/bwrap), supports four bot operating modes, and provides trust-level confidence assessments via the ECHIDNA Trust Bridge.")

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
            (interaction . "Webhook receiver, Check Runs API, PR comments via octocrab")
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
            (tier . 1)
            (small-kernel . #t)
            (description . "Coq proof assistant")
            (url . "https://coq.inria.fr")))

         (lean4
           ((relationship . "verified-via-echidna")
            (tier . 1)
            (small-kernel . #t)
            (description . "Lean 4 theorem prover")
            (url . "https://lean-lang.org")))

         (isabelle
           ((relationship . "verified-via-echidna")
            (tier . 1)
            (small-kernel . #t)
            (description . "Isabelle/HOL proof assistant")
            (url . "https://isabelle.in.tum.de")))

         (agda
           ((relationship . "verified-via-echidna")
            (tier . 1)
            (small-kernel . #t)
            (description . "Agda dependently typed language")
            (url . "https://wiki.portal.chalmers.se/agda")))

         (z3
           ((relationship . "verified-via-echidna")
            (tier . 1)
            (small-kernel . #f)
            (description . "Z3 SMT solver (large-TCB, produces certificates)")
            (url . "https://github.com/Z3Prover/z3")))

         (cvc5
           ((relationship . "verified-via-echidna")
            (tier . 1)
            (small-kernel . #f)
            (description . "CVC5 SMT solver (large-TCB, produces certificates)")
            (url . "https://cvc5.github.io")))

         (metamath
           ((relationship . "verified-via-echidna")
            (tier . 2)
            (small-kernel . #t)
            (description . "Metamath minimal proof checker")
            (url . "http://us.metamath.org")))

         (hol-light
           ((relationship . "verified-via-echidna")
            (tier . 2)
            (small-kernel . #t)
            (description . "HOL Light proof assistant")
            (url . "https://www.cl.cam.ac.uk/~jrh13/hol-light")))

         (mizar
           ((relationship . "verified-via-echidna")
            (tier . 2)
            (small-kernel . #f)
            (description . "Mizar proof checker")
            (url . "http://mizar.org")))

         (pvs
           ((relationship . "verified-via-echidna")
            (tier . 3)
            (small-kernel . #f)
            (description . "PVS verification system")
            (url . "https://pvs.csl.sri.com")))

         (acl2
           ((relationship . "verified-via-echidna")
            (tier . 3)
            (small-kernel . #f)
            (description . "ACL2 theorem prover")
            (url . "https://www.cs.utexas.edu/users/moore/acl2")))

         (hol4
           ((relationship . "verified-via-echidna")
            (tier . 3)
            (small-kernel . #t)
            (description . "HOL4 proof assistant")
            (url . "https://hol-theorem-prover.org")))))

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
            (usage . "SQLite/PostgreSQL access for job queue and state")
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
           ((relationship . "production-database")
            (description . "PostgreSQL relational database")
            (usage . "Job queue, verification results, repository config (production)")
            (url . "https://www.postgresql.org")))

         (sqlite
           ((relationship . "development-database")
            (description . "SQLite embedded database")
            (usage . "Local development and testing")
            (url . "https://www.sqlite.org")))))

      (containerization
        ((podman
           ((relationship . "primary-isolation")
            (description . "Rootless container engine (preferred backend)")
            (usage . "Isolated proof verification with resource limits, no-network, cap-drop-all")
            (url . "https://podman.io")))

         (bubblewrap
           ((relationship . "fallback-isolation")
            (description . "Lightweight sandbox using Linux namespaces")
            (usage . "Fallback when Podman is not available; unshare-all, read-only binds")
            (url . "https://github.com/containers/bubblewrap")))))

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
            (url . "https://github.com/hyperpolymath/hypatia")))

         (gitbot-shared-context
           ((relationship . "shared-library")
            (description . "Shared Rust crate for fleet coordination")
            (usage . "Compile-time dependency for fleet context types")
            (url . "https://github.com/hyperpolymath/gitbot-fleet")))))

      (formal-verification-projects
        ((compcert
           ((relationship . "potential-user")
            (description . "Verified C compiler in Coq")))

         (mathlib
           ((relationship . "potential-user")
            (description . "Mathematical library for Lean")))

         (isabelle-afp
           ((relationship . "potential-user")
            (description . "Archive of Formal Proofs for Isabelle")))

         (fiat-crypto
           ((relationship . "potential-user")
            (description . "Cryptographic primitives in Coq")))))))

   (what-this-is
     ("A CI/CD bot that automatically verifies formal proofs on every push and PR"
      "An orchestrator bridging GitHub/GitLab/Bitbucket and the ECHIDNA theorem prover"
      "A webhook server with GraphQL API for job management"
      "A security-first system using Podman/bwrap container isolation"
      "A multi-platform adapter supporting 3 code platforms and 12 theorem provers"
      "An automated gatekeeper that can block merges when proofs fail (Regulator mode)"
      "A trust assessment system for proof confidence levels and axiom tracking"))

   (what-this-is-not
     ("Not a theorem prover itself -- delegates to ECHIDNA for verification"
      "Not limited to one platform -- supports GitHub, GitLab, and Bitbucket"
      "Not just for Coq -- supports all 12 provers that ECHIDNA supports"
      "Not a replacement for human proof engineers -- assists and automates"
      "Not closed-source -- fully open PMPL-1.0-or-later licensed"))))
