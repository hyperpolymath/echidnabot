<!--
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# echidnabot Wiki

Welcome to the **echidnabot** wiki — your guide to proof-aware CI.

## What is echidnabot?

echidnabot is a CI bot that automatically verifies mathematical theorems and formal proofs in your codebase. When you push code containing proofs in Coq, Lean, Agda, Isabelle, Z3, or other theorem provers, echidnabot dispatches verification jobs to the [ECHIDNA](https://github.com/hyperpolymath/echidna) prover engine and reports results directly in your pull requests.

**Think of it as GitHub Actions for mathematical certainty.**

## Navigation

### For Users
- **[[Getting Started]]** — Install and configure echidnabot
- **[[Supported Provers]]** — List of theorem provers and per-prover notes
- **[FAQ](FAQ)** — Frequently asked questions

### For Developers
- **[[Architecture]]** — System design and components
- [CONTRIBUTING.md](https://github.com/hyperpolymath/echidnabot/blob/main/CONTRIBUTING.md) — How to contribute
- [ABI-FFI-README.md](https://github.com/hyperpolymath/echidnabot/blob/main/ABI-FFI-README.md) — Zig FFI + Idris2 ABI boundary
- [ROADMAP.adoc](https://github.com/hyperpolymath/echidnabot/blob/main/ROADMAP.adoc) — Phases and current state

### Reference
- [EXPLAINME.adoc](https://github.com/hyperpolymath/echidnabot/blob/main/EXPLAINME.adoc) — Receipts behind README claims
- [CHANGELOG.md](https://github.com/hyperpolymath/echidnabot/blob/main/CHANGELOG.md) — Version history
- [Issues](https://github.com/hyperpolymath/echidnabot/issues) — Bugs and feature requests
- [Discussions](https://github.com/hyperpolymath/echidnabot/discussions) — Open-ended questions

> **Wiki pages currently in this wiki:** Home, Getting-Started,
> Architecture, FAQ, Supported-Provers. Pages such as
> *Configuration-Reference*, *Platform-Integration*, *Troubleshooting*,
> and *API-Reference* are tracked in
> [RELEASE_CHECKLIST.md](https://github.com/hyperpolymath/echidnabot/blob/main/RELEASE_CHECKLIST.md)
> "Wiki → To Add". Until those land, the
> [docs/content/](https://github.com/hyperpolymath/echidnabot/tree/main/docs/content)
> directory carries the equivalent material in a single-page form.

## Why echidnabot?

### The Problem

You're writing formally verified software. Your proofs ensure correctness at a mathematical level. But your CI pipeline doesn't understand proofs:

- Tests pass, but proofs are broken
- PRs merge with unverified theorems
- No one notices until a dependent build fails
- Manual verification is slow and error-prone

### The Solution

echidnabot bridges the gap:

```
Push/PR -> echidnabot -> ECHIDNA Core -> Proof Result -> Check Run pass/fail
```

Every commit with proof files gets verified. Broken proofs block merges (in `regulator` mode). ML-powered suggestions help fix failing proofs (in `advisor` mode).

## Key Features

| Feature                | Description                                                |
| ---------------------- | ---------------------------------------------------------- |
| **Multi-Prover**       | Coq, Lean 4, Agda, Isabelle, Z3, CVC5, Metamath, HOL Light, Mizar, PVS, ACL2, HOL4 |
| **Multi-Platform**     | GitHub, GitLab, Bitbucket (Codeberg planned)               |
| **4 Bot Modes**        | Verifier, Advisor, Consultant, Regulator                   |
| **ML Suggestions**     | Tactic suggestions via ECHIDNA's Julia ML backend          |
| **Container Isolation**| podman rootless + bwrap fallback; fail-safe (refuses unsandboxed) |
| **Trust Bridge**       | 5-level confidence, SHA-256 solver integrity, axiom tracking |
| **GraphQL API**        | Query and control programmatically                         |
| **Self-Hosting**       | Run your own instance; Containerfile + Guix supplied      |

## Where echidnabot sits in the estate

- [ECHIDNA](https://github.com/hyperpolymath/echidna) — upstream theorem-proving engine (the actual provers live here; echidnabot is the orchestration layer)
- [gitbot-fleet](https://github.com/hyperpolymath/gitbot-fleet) — fleet of bots; echidnabot is the Tier-1 Verifier
- [Hypatia](https://github.com/hyperpolymath/hypatia) — estate-wide neurosymbolic CI coordinator
- [RSR](https://github.com/hyperpolymath/rhodium-standard-repositories) — repository quality standards (see [RSR_COMPLIANCE.adoc](https://github.com/hyperpolymath/echidnabot/blob/main/RSR_COMPLIANCE.adoc))

## About hyperpolymath

[hyperpolymath](https://github.com/hyperpolymath) builds politically autonomous software for ecologically and economically conscious development.

Our principles:
- **Formal correctness** — proofs over tests
- **Sustainability** — carbon-aware computing
- **Independence** — no Big Tech dependencies
- **Openness** — MPL-2.0 with Palimpsest commentary

---

*echidnabot is licensed under MPL-2.0. See [LICENSE](https://github.com/hyperpolymath/echidnabot/blob/main/LICENSE).*
