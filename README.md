<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# echidnabot — Proof-Aware CI Bot

[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github)](https://github.com/sponsors/hyperpolymath)
[![License: PMPL-1.0](https://img.shields.io/badge/License-PMPL--1.0-blue.svg)](https://github.com/hyperpolymath/palimpsest-license)
[![GitHub Release](https://img.shields.io/github/v/release/hyperpolymath/echidnabot?include_prereleases)](https://github.com/hyperpolymath/echidnabot/releases)

> **The canonical README is [`README.adoc`](README.adoc).** This Markdown
> file exists for renderers that prefer `.md` (some package indexes, some
> doc tooling); it is a thin summary that links into the AsciiDoc
> version for the full story.

---

## What is it?

A formal-verification CI bot that orchestrates the
[ECHIDNA](https://github.com/hyperpolymath/echidna) theorem-proving
platform for automatic proof verification on every push and pull
request. Written in Rust on Tokio/Axum.

Part of the [gitbot-fleet](https://github.com/hyperpolymath/gitbot-fleet)
(Tier-1 Verifier role), coordinated by
[Hypatia](https://github.com/hyperpolymath/hypatia).

## Why does it exist?

You're writing formally verified software — proofs in Coq, Lean, Agda,
or Isabelle. But your CI pipeline doesn't understand proofs:

- Tests pass, but **proofs are broken**.
- PRs merge with **unverified theorems**.
- No one notices until a dependent build fails.
- Manual verification is **slow and error-prone**.

echidnabot bridges the gap. Push proof files; get verified.

## Features (high-level)

- **12 provers via ECHIDNA** — Coq, Lean 4, Agda, Isabelle/HOL, Z3,
  CVC5, Metamath, HOL Light, Mizar, PVS, ACL2, HOL4
  (see [`wiki/Supported-Provers.md`](wiki/Supported-Provers.md);
  upstream supports 113, drift documented in
  [`EXPLAINME.adoc`](EXPLAINME.adoc)).
- **3 platforms** — GitHub, GitLab, Bitbucket (Codeberg planned).
- **4 bot modes** — Verifier / Advisor / Consultant / Regulator
  (configured via `.machine_readable/bot_directives/echidnabot.a2ml`).
- **Container isolation** — podman rootless with bwrap fallback;
  fail-safe (refuses to run proofs without isolation).
- **Trust bridge** — 5-level confidence, SHA-256 solver-integrity
  verification, axiom-usage tracking.
- **Retry + circuit breaker** — exponential backoff, opens after
  5 failures, auto-resets after 5 minutes.
- **184 tests** (137 lib + 17 lifecycle + 32 integration + 12 property
  + 15 seam + 8 smoke).

## Install / configure / run

```bash
# Build (requires gitbot-fleet layout — see CONTRIBUTING.md)
cargo build --release

# Initialise database
echidnabot init-db

# Start the webhook server
export DATABASE_URL=sqlite:echidnabot.db
export ECHIDNA_URL=http://localhost:8080
echidnabot serve --port 8080
```

Full instructions: [`wiki/Getting-Started.md`](wiki/Getting-Started.md)
and [`docs/content/configuration.md`](docs/content/configuration.md).

## Documentation map

| Audience           | File                                                                  |
| ------------------ | --------------------------------------------------------------------- |
| Users (overview)   | [`README.adoc`](README.adoc) — full canonical README                  |
| Users (setup)      | [`wiki/Getting-Started.md`](wiki/Getting-Started.md)                  |
| Users (config)     | [`docs/content/configuration.md`](docs/content/configuration.md)      |
| Users (FAQ)        | [`wiki/FAQ.md`](wiki/FAQ.md)                                          |
| Devs (architecture)| [`wiki/Architecture.md`](wiki/Architecture.md)                        |
| Devs (contrib)     | [`CONTRIBUTING.md`](CONTRIBUTING.md)                                  |
| Devs (ABI/FFI)     | [`ABI-FFI-README.md`](ABI-FFI-README.md)                              |
| Devs (roadmap)     | [`ROADMAP.adoc`](ROADMAP.adoc)                                        |
| Devs (claims)      | [`EXPLAINME.adoc`](EXPLAINME.adoc) — receipts behind README claims    |
| AI assistants      | [`.claude/CLAUDE.md`](.claude/CLAUDE.md) + [`0-AI-MANIFEST.a2ml`](0-AI-MANIFEST.a2ml) |
| Security           | [`SECURITY.md`](SECURITY.md) + [`.well-known/security.txt`](.well-known/security.txt) |
| Releases           | [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md)                        |
| Compliance         | [`RSR_COMPLIANCE.adoc`](RSR_COMPLIANCE.adoc)                          |

## License

MPL-2.0 (Palimpsest License). See [`LICENSE`](LICENSE) and
[`PALIMPSEST.adoc`](PALIMPSEST.adoc).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Security

Vulnerabilities → [`SECURITY.md`](SECURITY.md). **Do not** open public
issues for security reports.

---

**Maintainer:** Jonathan D.A. Jewell —
[hyperpolymath](https://github.com/hyperpolymath)
