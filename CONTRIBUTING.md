<!--
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# Contributing to echidnabot

Thanks for your interest in contributing to **echidnabot** — the proof-aware
CI bot that bridges code platforms (GitHub / GitLab / Bitbucket) and the
[ECHIDNA](https://github.com/hyperpolymath/echidna) theorem-proving platform.

This document covers the practicalities. For project conduct, see
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md); for security reports, see
[`SECURITY.md`](SECURITY.md); for maintainership and decision rights, see
[`MAINTAINERS.adoc`](MAINTAINERS.adoc).

---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/echidnabot.git
cd echidnabot

# Reproducible environment (Guix is the estate primary; nix is deprecated 2026-06-01)
guix shell -D -f guix.scm         # Guix package (guix.scm)

# Or bring your own toolchain (Rust 1.75+, SQLite, podman/bwrap)

# Verify
cargo check
cargo test
```

### Workspace caveat — `gitbot-shared-context`

echidnabot's `Cargo.toml` carries a **path dependency** on
`gitbot-shared-context` that assumes the
[gitbot-fleet](https://github.com/hyperpolymath/gitbot-fleet) monorepo
layout (sibling directory `../../shared-context`). Building from a bare
clone of `echidnabot` alone therefore fails at the dependency-resolution
step. This is **acknowledged tech debt** — see issue
[#18](https://github.com/hyperpolymath/echidnabot/issues/18). Until that
is resolved, contributors who want a building tree should either:

1. Clone the parent
   [`gitbot-fleet`](https://github.com/hyperpolymath/gitbot-fleet) and
   work from `gitbot-fleet/bots/echidnabot/`, **or**
2. Provide a local checkout of `shared-context` at `../../shared-context`
   relative to this clone, **or**
3. Limit your changes to **documentation** (which is what this PR-class
   covers — no build required).

---

## Repository Layout

```
echidnabot/
├── src/                    # Rust source
│   ├── adapters/           # Platform adapters (GitHub, GitLab, Bitbucket)
│   ├── api/                # GraphQL + webhook handlers (axum)
│   ├── dispatcher/         # ECHIDNA HTTP client + prover enumeration
│   ├── scheduler/          # Job queue, retry, circuit breaker
│   ├── executor/           # Container isolation (podman + bwrap)
│   ├── modes/              # Bot modes (Verifier/Advisor/Consultant/Regulator)
│   ├── trust/              # Confidence levels, solver integrity, axiom tracking
│   ├── store/              # SQLite/PostgreSQL persistence
│   ├── feedback/           # Double-loop tactic-outcome recording
│   ├── fleet/              # Shared-context integration with gitbot-fleet
│   ├── abi/                # Idris2 ABI definitions (Types/Layout/Foreign)
│   ├── config.rs / error.rs / lib.rs / main.rs
├── ffi/zig/                # Zig FFI scaffold (see ABI-FFI-README.md)
├── tests/                  # Integration + property + smoke + seam test suites
├── proofs/                 # Dogfood proofs (Coq + Lean) — INCLUDES failing stubs
├── benches/                # cargo-criterion benchmarks
├── docs/                   # Long-form documentation (casket-ssg)
├── wiki/                   # Wiki source (mirrored to GitHub wiki)
├── packaging/              # Container/Guix packaging
├── .machine_readable/      # A2ML state, bot directives, contractiles
├── .well-known/            # security.txt / ai.txt / humans.txt
└── .github/workflows/      # CI — quality, codeql, scorecard, hypatia-scan
```

---

## How to Contribute

### Reporting Bugs

1. Search [existing issues](https://github.com/hyperpolymath/echidnabot/issues)
   to avoid duplicates.
2. Check whether the bug is already fixed on `main`.
3. Open a bug report using the
   [bug_report.yml template](.github/ISSUE_TEMPLATE/bug_report.yml) and
   include:
   - Environment (OS, Rust version, podman/bwrap version)
   - Steps to reproduce
   - Expected vs actual behaviour
   - Logs (`RUST_LOG=debug` output if relevant)

### Suggesting Features

1. Check [`ROADMAP.adoc`](ROADMAP.adoc) — your idea may already be tracked.
2. Open a feature request using the
   [feature_request.yml template](.github/ISSUE_TEMPLATE/feature_request.yml).
3. Include a problem statement, not just a solution.

### Good-first-issue Labels

- [`good first issue`](https://github.com/hyperpolymath/echidnabot/issues?q=is%3Aopen+label%3A%22good+first+issue%22)
- [`help wanted`](https://github.com/hyperpolymath/echidnabot/issues?q=is%3Aopen+label%3A%22help+wanted%22)
- [`documentation`](https://github.com/hyperpolymath/echidnabot/issues?q=is%3Aopen+label%3Adocumentation)

---

## Development Workflow

### Branch Naming

```
feat/<short-description>       # New features
fix/<issue-number>-<slug>      # Bug fixes
docs/<short-description>       # Documentation only
refactor/<what-changed>        # Code improvement without behaviour change
test/<what-added>              # Test additions
ci/<what-changed>              # CI / workflow changes
chore/<what-changed>           # Tooling / deps / housekeeping
security/<what-fixed>          # Security fixes
```

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <imperative description>

[optional body explaining the why]

[optional footer with Closes #N, Signed-off-by:, Co-Authored-By:]
```

Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
`build`, `ci`, `chore`, `revert`, `security`.

### Commit Signing

All commits **must be GPG-signed**. Configure:

```bash
git config commit.gpgsign true
git config user.signingkey <YOUR_KEY_ID>
```

Unsigned commits will fail the `commit-signing` enforcement check.

### Pull Request Checklist

Before opening a PR:

- [ ] `cargo fmt --check` is clean
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] `cargo test` passes (caveat: requires gitbot-fleet layout — see above)
- [ ] New behaviour has tests (unit + integration where appropriate)
- [ ] Docs updated when touching public surface (README / wiki / CLI help)
- [ ] `CHANGELOG.md` is **not** edited by hand — it regenerates from
      conventional commits via
      [`standards/changelog-reusable.yml`](https://github.com/hyperpolymath/standards/blob/main/.github/workflows/changelog-reusable.yml)
- [ ] SPDX header on every new source file
      (`// SPDX-License-Identifier: CC-BY-SA-4.0`)
- [ ] Commits GPG-signed

When opening the PR:

- Title matches conventional-commits style (`feat(scheduler): ...`).
- Description references closed issues (`Closes #N`).
- Auto-merge `--squash --delete-branch` is preferred for clean history.

---

## Language and Tooling Policy

Per the
[Hyperpolymath estate policy](https://github.com/hyperpolymath/standards):

| Allowed                | Banned (replacement)              |
| ---------------------- | --------------------------------- |
| Rust (primary)         | TypeScript (use AffineScript)     |
| AffineScript           | Node.js / npm / Bun (use Deno)    |
| Zig (FFI)              | Python (use Julia/Rust)           |
| Idris2 (ABI proofs)    | Go (use Rust)                     |
| Guile Scheme (Guix)    | Java/Kotlin/Swift (use Tauri/Dioxus) |
| Nickel (config)        | Jekyll (use casket-ssg)           |
| Julia (ML/data)        | Dockerfile (use Containerfile)    |

See [`.claude/CLAUDE.md`](.claude/CLAUDE.md) for the full table and
enforcement rules.

### Security Defaults

- SHA-256 or stronger only (no MD5/SHA1 for integrity)
- HTTPS / WSS / SSH only — never plain HTTP in code or docs
- SHA-pinned GitHub Action dependencies
- SPDX license headers on every source file

---

## Testing

```bash
cargo test                 # Full suite
cargo test --lib           # Unit tests only
cargo test --test seam_test       # Specific integration test
cargo test -- --nocapture  # See println! output
```

The suite is currently **184 tests** (per `STATE.a2ml` last-updated
`2026-04-26`):

- 137 lib unit tests
- 17 lifecycle integration tests
- 32 integration tests
- 12 property tests
- 15 seam tests
- 8 smoke tests

### Fuzzing

```bash
cd fuzz
cargo +nightly fuzz run webhook_parse
```

ClusterFuzzLite runs continuous fuzzing in CI; see
`.clusterfuzzlite/`.

---

## Documentation

### Files to keep in sync when touching public surface

| Change                                      | Update                                                  |
| ------------------------------------------- | ------------------------------------------------------- |
| New CLI flag / subcommand                   | `README.adoc` Usage section + `wiki/Getting-Started.md` |
| New configuration option                    | `echidnabot.example.toml` + `docs/content/configuration.md` + `wiki/Getting-Started.md` |
| New prover support                          | `wiki/Supported-Provers.md` + `src/dispatcher/`         |
| New bot mode behaviour                      | `wiki/FAQ.md` + `.machine_readable/bot_directives/echidnabot.a2ml` |
| ABI/FFI surface change                      | `src/abi/*.idr` + `ffi/zig/src/main.zig` + `ABI-FFI-README.md` |
| Roadmap item closed                         | `ROADMAP.adoc` + `.machine_readable/6a2/STATE.a2ml`     |

### Wiki

Wiki source lives in `wiki/` in this repo. The GitHub wiki at
`https://github.com/hyperpolymath/echidnabot.wiki.git` is the rendered
mirror — push there directly when the wiki is enabled. See
[`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md) for the push procedure.

### Machine-readable state

`.machine_readable/6a2/` holds A2ML descriptors consumed by Hypatia and
sibling bots. When a contribution lands a substantive change (new
external target, closed-issue feedback loop, completion-percentage
shift), update `STATE.a2ml` in the same commit.

---

## Releasing

Release procedure lives in [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md).
Contributors do not need to drive releases — flag readiness in a comment
on the relevant milestone issue.

---

## Getting Help

- **Code-level questions:** open a discussion at
  [hyperpolymath/echidnabot/discussions](https://github.com/hyperpolymath/echidnabot/discussions).
- **Bugs:** [GitHub Issues](https://github.com/hyperpolymath/echidnabot/issues).
- **Security:** see [`SECURITY.md`](SECURITY.md) — do **not** open public
  issues for vulnerabilities.

Thanks for contributing!
