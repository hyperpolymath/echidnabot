;; SPDX-License-Identifier: PMPL-1.0-or-later
;; NEUROSYM.scm - Neurosymbolic integration config for echidnabot

(define neurosym-config
  `((version . "1.0.0")
    (updated . "2026-01-29")

    (overview
      "echidnabot bridges symbolic theorem proving (via ECHIDNA's 12 prover backends) with neural learning (via ECHIDNA's Julia ML API). The bot orchestrates CI/CD while ECHIDNA handles the actual neurosymbolic verification.")

    (symbolic-layer
      ((provider . "ECHIDNA Core")
       (provers . ("Coq" "Lean4" "Isabelle/HOL" "Agda"
                  "Z3" "CVC5" "ACL2" "PVS" "HOL4"
                  "Mizar" "HOL Light" "Metamath"))
       (reasoning . "deductive")
       (verification . "formal")
       (soundness-guarantee . "All proofs verified by established theorem provers")
       (integration-method . "HTTP API calls to ECHIDNA")
       (api-endpoint . "http://localhost:8080/api")
       (proof-validation
         ((trust-model . "Zero-trust - every proof verified by prover backend")
          (no-ml-only-proofs . "Neural suggestions must pass symbolic verification")
          (multi-prover-consensus . "Optional: require N provers to agree")))))

    (neural-layer
      ((provider . "ECHIDNA Julia ML Backend")
       (model-type . "Logistic regression (MVP), Transformers planned v2.0")
       (training-data
         ((proofs . 332)
          (tactics . 1603)
          (vocabulary . 161)))
       (accuracy
         ((top-1 . 0.65)
          (top-3 . 0.85)))
       (use-cases
         ((tactic-prediction . "Suggest likely successful tactics for proof goals")
          (premise-selection . "Planned v2.0")
          (proof-repair . "Planned v3.0")))
       (api-endpoint . "http://localhost:8090/suggest")
       (integration-method . "echidnabot → ECHIDNA Rust → Julia ML")
       (confidence-scores . "Returned with each tactic suggestion")))

    (bot-intelligence
      ((advisor-mode-ml
         ((description . "Use ECHIDNA ML to suggest tactics on proof failure")
          (workflow . ("1. Proof fails symbolic verification"
                      "2. echidnabot calls ECHIDNA ML API with proof goal"
                      "3. ML suggests top-3 tactics with confidence scores"
                      "4. echidnabot posts suggestions as PR comment"))
          (example-output . "💡 Suggested tactics:
  • Try `induction xs` to break down the list structure (confidence: 0.65)
  • Consider `rewrite app_assoc` if available (confidence: 0.28)
  • Check if `simpl` simplifies the goal (confidence: 0.23)")))

       (consultant-mode-ml
         ((description . "Answer questions about proof state using ML")
          (planned . "v2.0")
          (capabilities . ("Explain why a tactic failed"
                          "Suggest alternative approaches"
                          "Reference similar proven theorems"))))

       (learning-from-ci
         ((description . "Improve ML models by learning from CI proofs")
          (planned . "v2.0")
          (workflow . ("Successful proofs in CI → training data extraction"
                      "Failed proofs → negative examples"
                      "Periodic model retraining with new data"
                      "A/B testing of model versions")))))

    (integration-architecture
      ((layers
         ((layer-1 . "echidnabot - CI orchestration, webhooks, job queue")
          (layer-2 . "ECHIDNA Rust API - Prover dispatch, HTTP endpoints")
          (layer-3 . "ECHIDNA Julia ML - Tactic prediction, confidence scores")
          (layer-4 . "ECHIDNA Provers - Symbolic verification (12 backends)")
          (layer-5 . "Idris2 Validator - Formal soundness guarantees")))

       (data-flow-verification
         ((step-1 . "GitHub webhook → echidnabot")
          (step-2 . "echidnabot → ECHIDNA HTTP API (verification request)")
          (step-3 . "ECHIDNA → Prover backend (symbolic verification)")
          (step-4 . "Prover result → ECHIDNA")
          (step-5 . "ECHIDNA → echidnabot (verification result)")
          (step-6 . "echidnabot → GitHub Check Run (pass/fail)")))

       (data-flow-suggestion
         ((step-1 . "Proof fails verification")
          (step-2 . "echidnabot → ECHIDNA HTTP API (ML suggestion request)")
          (step-3 . "ECHIDNA Rust → Julia ML API (tactic prediction)")
          (step-4 . "Julia ML → bag-of-words encoding → logistic regression")
          (step-5 . "Top-3 tactics with confidence → ECHIDNA Rust")
          (step-6 . "ECHIDNA → echidnabot (suggestions)")
          (step-7 . "echidnabot → GitHub PR comment (💡 Suggested tactics)")))

       (separation-of-concerns
         ((echidnabot-responsibilities
            . ("Webhook handling"
              "Job scheduling and queue management"
              "Platform adapter abstraction (GitHub/GitLab/Bitbucket)"
              "Container orchestration (future)"
              "GraphQL API for job queries"
              "Check run / commit status creation"))
          (echidna-responsibilities
            . ("Proof verification via 12 prover backends"
              "ML tactic prediction via Julia backend"
              "Training data management"
              "Formal soundness guarantees via Idris2"
              "Multi-prover consensus"
              "Anomaly detection"))))))

    (trust-and-validation
      ((no-ml-shortcuts
         ((principle . "Never trust ML predictions without symbolic verification")
          (enforcement . "echidnabot only reports success if ECHIDNA prover verifies")
          (example . "Even if ML predicts tactic with 99% confidence, must be verified")))

       (soundness-theorem
         ((location . "ECHIDNA proven library - Proven.ECHIDNA.Soundness")
          (statement . "∀proof, validate(proof) = Valid ∧ verify(proof) = True ⇒ ProofValid(proof)")
          (guarantee . "Mathematical proof that accepted proofs are sound")
          (language . "Idris2 dependent types - total functions, guaranteed termination")))

       (multi-layer-validation
         ((layer-1 . "Property-based testing (PropTest) - 8 invariants")
          (layer-2 . "Idris2 formal validator - type checking, totality")
          (layer-3 . "Prover backend verification - symbolic proof checking")
          (layer-4 . "Anomaly detection - catches ML failures")))

       (ml-confidence-interpretation
         ((high-confidence-low-complexity
            . "ML confident + simple theorem → likely correct, verify anyway")
          (high-confidence-high-complexity
            . "Anomaly detected! ML shouldn't be confident on hard theorems")
          (low-confidence-any-complexity
            . "ML unsure → suggest alternatives, still verify")
          (always-verify
            . "Confidence is a hint, not a substitute for verification")))))

    (future-enhancements
      ((v2.0-transformer-models
         ((description . "Replace logistic regression with Transformer architecture")
          (benefits . ("Better capture of proof structure"
                      "Attention mechanism over premises"
                      "Improved accuracy (target: 80% top-1)"))
          (timeline . "Q4 2026")))

       (v2.0-premise-selection
         ((description . "ML suggests relevant lemmas and theorems")
          (integration . "Neural retrieval from proof libraries")
          (timeline . "Q4 2026")))

       (v3.0-proof-repair
         ((description . "Automatically fix broken proofs when dependencies change")
          (workflow . ("Detect proof failure after library update"
                      "ML suggests repair tactics"
                      "Automated verification of repairs"
                      "PR with suggested fixes"))
          (timeline . "2027")))

       (continuous-learning
         ((description . "ML models improve from CI proof data")
          (data-pipeline . ("CI proofs → training data extraction"
                           "Periodic model retraining"
                           "A/B testing of model versions"
                           "Automatic deployment of better models"))
          (privacy . "Only public proofs, respect repository privacy settings")
          (timeline . "v2.0")))))))
