;;; STATE.scm - Conversation Checkpoint for echidnabot
;;; Format: Guile Scheme S-expressions
;;; License: MIT / Palimpsest-0.8

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; METADATA
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(metadata
  (format-version . "1.1")
  (created . "2025-12-08")
  (last-updated . "2025-12-15")
  (generator . "claude-opus-4"))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; PROJECT IDENTITY
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(project
  (name . "echidnabot")
  (description . "Proof-aware CI bot that monitors repositories for theorem proof changes and delegates verification to ECHIDNA Core")
  (repository . "hyperpolymath/echidnabot")
  (category . "bot/automation/formal-verification")
  (status . "architecture-defined")
  (completion . 5)
  (relationship . ("orchestrator-for" "ECHIDNA Core")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; CORE CONCEPT
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(concept
  (tagline . "Dependabot for theorem provers")

  (what-it-is
    "An orchestration layer that:"
    "- Receives webhooks from GitHub/GitLab/Bitbucket"
    "- Detects proof files (.agda, .v, .lean, .mm, etc.)"
    "- Delegates verification to ECHIDNA Core"
    "- Reports results via Check Runs, comments, issues"
    "- Optionally requests ML-powered tactic suggestions")

  (what-it-is-not
    "- NOT a theorem prover (ECHIDNA does that)"
    "- NOT a replacement for CI (complements it)"
    "- NOT a code quality tool (proof verification only)")

  (value-proposition
    "Maintainers of formal verification projects get:"
    "- Automatic proof regression detection"
    "- Clear failure reports on PRs"
    "- ML-assisted fix suggestions"
    "- Multi-prover support (9+ backends via ECHIDNA)"))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; TECHNOLOGY STACK
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(technology
  (language . "Rust")
  (language-rationale . "RSR-mandated for systems; matches ECHIDNA Core")

  (runtime . "tokio")
  (http-framework . "axum")
  (graphql . "async-graphql")
  (database . ("SQLite" "PostgreSQL"))
  (serialization . "serde")
  (config-format . "TOML")
  (packaging . ("Guix" "Nix"))

  (platform-clients
    (github . "octocrab")
    (gitlab . "gitlab crate")
    (bitbucket . "custom"))

  (forbidden
    ("Python" . "RSR policy")
    ("npm/node" . "RSR policy - use Deno if TS needed")
    ("Ruby" . "RSR policy")
    ("Chapel" . "Not in approved list despite AI suggestion")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; CURRENT POSITION
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(current-position
  (phase . "phase-0-foundation")
  (state . "architecture-defined")
  (branch . "claude/echidnabot-architecture-3op9d")

  (what-exists
    (git-repo . #t)
    (remote-configured . #t)
    (architecture-doc . #t)
    (source-code . #f)
    (tests . #f)
    (ci-cd . #t)
    (guix-package . "stub-only"))

  (completed-this-session
    ("Analyzed two AI architecture proposals")
    ("Identified RSR policy conflicts")
    ("Synthesized proper Rust-based architecture")
    ("Created docs/ARCHITECTURE.adoc")
    ("Updated STATE.scm with project definition"))

  (blockers
    ("Cargo project not initialized")
    ("ECHIDNA Core API not yet defined")
    ("No test repository with proof files")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; ROUTE TO MVP v1
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(mvp-v1
  (status . "planning-complete")
  (scope . "GitHub + Metamath + single repo")
  (rationale . "Metamath is ECHIDNA's easiest prover (complexity 2/5)")

  (core-features
    ("Receive GitHub push/PR webhooks")
    ("Detect .mm (Metamath) files in commits")
    ("Delegate to ECHIDNA Core for verification")
    ("Create GitHub Check Runs with pass/fail")
    ("Store job history in SQLite"))

  (milestones
    (m0 (name . "Foundation")
        (status . "in-progress")
        (tasks
          ("Create STATE.scm" . "complete")
          ("Define architecture" . "complete")
          ("Initialize Cargo project" . "pending")
          ("Set up guix.scm with Rust deps" . "pending")
          ("Create basic project structure" . "pending")))

    (m1 (name . "Webhook Infrastructure")
        (status . "pending")
        (tasks
          ("Implement axum webhook server" . "pending")
          ("Add GitHub signature verification" . "pending")
          ("Parse push/PR events" . "pending")
          ("Test with ngrok" . "pending")))

    (m2 (name . "Job Scheduling")
        (status . "pending")
        (tasks
          ("Implement job queue" . "pending")
          ("Add SQLite state store" . "pending")
          ("Implement deduplication" . "pending")
          ("Add concurrent job limits" . "pending")))

    (m3 (name . "ECHIDNA Integration")
        (status . "pending")
        (tasks
          ("Define ECHIDNA client interface" . "pending")
          ("Implement Metamath dispatcher" . "pending")
          ("Parse verification results" . "pending")
          ("Handle timeouts gracefully" . "pending")))

    (m4 (name . "Result Reporting")
        (status . "pending")
        (tasks
          ("Create GitHub Check Runs" . "pending")
          ("Update check status on completion" . "pending")
          ("Add failure annotations" . "pending")
          ("Test end-to-end flow" . "pending")))

    (m5 (name . "MVP Release")
        (status . "pending")
        (tasks
          ("GraphQL API for manual triggers" . "pending")
          ("CLI tool" . "pending")
          ("Documentation" . "pending")
          ("Deploy to test environment" . "pending")))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; KNOWN ISSUES
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(issues
  (critical)

  (high
    (issue-001
      (title . "ECHIDNA Core API not defined")
      (description . "echidnabot needs to call ECHIDNA; API contract must be established")
      (impact . "Cannot implement prover dispatcher without this")
      (resolution . "Define GraphQL/gRPC interface in ECHIDNA Core"))

    (issue-002
      (title . "No test repository with proof files")
      (description . "Need a repo with .mm files to test webhook flow")
      (resolution . "Create test-echidnabot repo with sample Metamath proofs")))

  (medium
    (issue-003
      (title . "Guix Rust tooling")
      (description . "guix.scm needs proper Rust build system inputs")
      (resolution . "Add rust, cargo, rust-src to guix.scm inputs")))

  (low
    (issue-004
      (title . "ReScript UI deferred")
      (description . "Dashboard is phase 3, not blocking MVP")
      (resolution . "Implement after core functionality works"))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; QUESTIONS RESOLVED
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(questions-resolved
  (q1 (question . "What is echidnabot?")
      (answer . "A proof-aware CI bot that orchestrates ECHIDNA for proof verification"))

  (q2 (question . "What platform(s) should it target?")
      (answer . "GitHub first, then GitLab, Bitbucket, Codeberg"))

  (q3 (question . "What are the core features for MVP?")
      (answer . "Webhook receiver, Metamath dispatcher, GitHub Check Runs"))

  (q4 (question . "What tech stack?")
      (answer . "Rust (axum + async-graphql + tokio + SQLite)"))

  (q5 (question . "Deployment model?")
      (answer . "Container-based (Docker/Podman) or Guix service")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; LONG-TERM ROADMAP
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(roadmap
  (phase-0
    (name . "Foundation")
    (status . "in-progress")
    (goals
      ("Architecture document" . "complete")
      ("Cargo project setup" . "pending")
      ("Basic project structure" . "pending")))

  (phase-1
    (name . "MVP")
    (status . "pending")
    (scope . "GitHub + Metamath")
    (goals
      ("Webhook infrastructure")
      ("Job scheduling")
      ("ECHIDNA integration")
      ("Check run reporting")))

  (phase-2
    (name . "Multi-Prover")
    (status . "future")
    (scope . "+Z3, CVC5, Lean; +GitLab")
    (goals
      ("Auto-detect proof type")
      ("Parallel prover execution")
      ("GitLab adapter")))

  (phase-3
    (name . "Intelligence")
    (status . "future")
    (scope . "+ML suggestions; +dashboard")
    (goals
      ("Tactic suggestion via ECHIDNA Julia ML")
      ("Auto-fix PR generation")
      ("ReScript web dashboard")))

  (phase-4
    (name . "Production")
    (status . "future")
    (goals
      ("PostgreSQL for scale")
      ("Horizontal worker scaling")
      ("Monitoring (Prometheus/Grafana)")
      ("Complete documentation"))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; ECHIDNA DEPENDENCY
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(echidna-dependency
  (relationship . "echidnabot orchestrates ECHIDNA Core")

  (required-endpoints
    ("verifyProof mutation" . "Submit proof content, get verification result")
    ("suggestTactics mutation" . "Get ML-powered suggestions for failed proofs")
    ("proverStatus query" . "Check if prover backend is available"))

  (echidna-status
    (provers-complete . 9)
    (provers-total . 12)
    (tier-1 . ("Agda" "Coq" "Lean" "Isabelle" "Z3" "CVC5"))
    (tier-2 . ("Metamath" "HOL Light" "Mizar"))
    (tier-3 . ("PVS" "ACL2" "HOL4"))
    (mvp-prover . "Metamath")
    (mvp-prover-rationale . "Complexity 2/5, easiest to implement")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; SESSION TRACKING
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(session
  (id . "3op9d")
  (started . "2025-12-15")
  (actions-taken
    ("Reviewed two AI architecture proposals")
    ("Identified Chapel/npm/Python policy violations")
    ("Designed Rust-based architecture")
    ("Created docs/ARCHITECTURE.adoc")
    ("Updated STATE.scm with complete project definition")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; FILES CREATED/MODIFIED THIS SESSION
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(files
  (created
    ("docs/ARCHITECTURE.adoc" . "2025-12-15"))
  (modified
    ("STATE.scm" . "2025-12-15")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; NEXT ACTIONS
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(critical-next
  (action-1 . "Initialize Cargo project: cargo init --lib")
  (action-2 . "Add dependencies to Cargo.toml")
  (action-3 . "Create src/ directory structure per ARCHITECTURE.adoc")
  (action-4 . "Update guix.scm with Rust build inputs")
  (action-5 . "Implement stub axum server that accepts webhooks")
  (action-6 . "Define ECHIDNA API contract (in ECHIDNA repo)"))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; STATISTICS
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(statistics
  (lines-of-code . 0)
  (files-source . 0)
  (files-docs . 1)
  (completion-percentage . 5)
  (architecture-defined . #t)
  (tech-stack-selected . #t))

;;; END STATE.scm
