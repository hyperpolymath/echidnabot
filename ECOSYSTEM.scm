;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Ecosystem relationships for echidnabot
;; Media type: application/vnd.ecosystem+scm

(ecosystem
  (version "1.0.0")
  (name "echidnabot")
  (type "verification-bot")
  (purpose "Proof-aware CI bot that orchestrates ECHIDNA's multi-prover
    verification system for theorem proving in PRs and pushes. Bridges
    the gap between formal verification tools and development workflows.")

  (position-in-ecosystem
    (role "verifier-tier-bot")
    (layer "formal-verification")
    (fleet-tier "verifier")
    (execution-order 2)
    (description "Runs as a Verifier-tier bot in the gitbot-fleet. Provides
      formal mathematical verification that no other bot can offer.
      Its findings inform seambot (seam contracts can be proven) and
      finishbot (proof status is part of release readiness)."))

  (related-projects
    (core-dependency
      (echidna
        (relationship "backend-engine")
        (description "ECHIDNA is the neurosymbolic theorem prover that echidnabot
          orchestrates. Echidnabot is the CI/CD interface; ECHIDNA is the
          verification engine with 12 prover backends.")
        (integration "HTTP API calls to ECHIDNA's Rust core for prover execution")
        (repo "hyperpolymath/echidna")))
    (parent
      (gitbot-fleet
        (relationship "fleet-member")
        (description "Echidnabot is one of six specialized bots. It is the only
          bot providing formal mathematical verification.")
        (integration "Publishes verification findings via shared-context API")))
    (engine
      (hypatia
        (relationship "rules-engine")
        (description "Hypatia determines which repositories need formal verification
          and configures echidnabot's prover selection per repo.")
        (integration "Receives execution instructions and prover configurations")))
    (executor
      (robot-repo-automaton
        (relationship "fix-executor")
        (description "When echidnabot identifies proof failures with known fixes,
          robot-repo-automaton can apply tactic suggestions.")
        (integration "Sends FixRequest actions for auto-fixable proof issues")))
    (test-case
      (absolute-zero
        (relationship "validation-target")
        (description "Absolute Zero's 81 Qed + 19 Admitted Coq proofs serve as
          the primary test case for echidnabot's verification capabilities.
          Echidnabot could attempt the 19 Admitted proofs automatically.")
        (repo "hyperpolymath/absolute-zero")))
    (siblings
      (rhodibot
        (relationship "peer-verifier")
        (description "Both are Verifier-tier bots. Rhodibot checks structural
          compliance, echidnabot checks mathematical correctness."))
      (sustainabot
        (relationship "peer-verifier")
        (description "Both are Verifier-tier bots. Sustainabot checks ecological
          efficiency, echidnabot checks formal correctness."))
      (seambot
        (relationship "consumer")
        (description "Seambot can use echidnabot's verification results to prove
          seam contracts hold. E.g. 'this API always returns valid JSON'."))
      (finishingbot
        (relationship "consumer")
        (description "Finishbot uses echidnabot's proof status as part of release
          readiness. Repos with failing proofs cannot pass release gate.")))
    (infrastructure
      (git-private-farm
        (relationship "propagation")
        (description "Verification results propagate across all forges via mirroring."))
      (bot-directives
        (relationship "configuration")
        (description ".bot_directives/echidnabot.scm per-repo config constrains
          what echidnabot can do (e.g. allowed prover backends, analysis scope)."))))

  (what-this-is
    "A CI/CD bot for formal verification of mathematical proofs in PRs"
    "An orchestrator for ECHIDNA's 12 theorem prover backends"
    "A multi-platform bot supporting GitHub, GitLab, and Bitbucket"
    "A Verifier-tier bot in the gitbot-fleet ecosystem"
    "A bridge between formal methods and development workflows")

  (what-this-is-not
    "Not the theorem prover itself — that is ECHIDNA"
    "Not a general CI bot — it specifically handles formal verification"
    "Not a code quality tool — it verifies mathematical proofs"
    "Not a standalone tool — it integrates with gitbot-fleet"
    "Not limited to one proof assistant — it supports 12 via ECHIDNA"))
