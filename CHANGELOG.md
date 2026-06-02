<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# Changelog

All notable changes to `echidnabot` will be documented in this file.

This file is generated from conventional commits by the
[`changelog-reusable.yml`](https://github.com/hyperpolymath/standards/blob/main/.github/workflows/changelog-reusable.yml)
workflow (`hyperpolymath/standards#206`). Adopt the workflow in this repo's CI to keep this file in sync automatically — see
[`templates/cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml)
for the canonical config.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- feat(observability): structured JSON logging via `tracing-subscriber` — new `src/observability.rs` module + `ECHIDNABOT_LOG_FORMAT=text|json` env var (default `text`); shared init point for CLI, server, and future OpenTelemetry layer
- feat(trust+executor): Tier-3 prover coverage (idris2/fstar/ATPs/protocol-checkers)
- feat(T3): wire bot modes into webhook response pipeline
- feat(feedback+graphql): double-loop write path, tactic GraphQL API, ProverKind fixes
- feat(hardening): per-IP webhook rate limiting + ROADMAP/STATE updates
- feat(Task E): Begin ProverKind enum→slug migration for 113-prover support
- feat(trust): wire trust bridge into dispatcher/scheduler pipeline
- feat(contractiles): add intend, bust, adjust verbs (6/6 complete)
- feat(governance): branch-protection script per estate memory rule
- feat(ci): add cargo-audit + db-checks workflows
- feat(ci): adopt 8 RSR-template workflows missing from echidnabot (#8)

### Fixed

- fix(ci): sync hypatia-scan.yml to canonical (413: env.HOME+Phase-2+SARIF) (#10)
- fix(ci): bump a2ml/k9-validate-action pins to canonical (standards#85) (#9)
- fix(ci): adopt canonical hypatia-scan.yml (env.HOME/scanner-layout + Comment-step gate) (#6)
- fix(rhodibot): automated RSR compliance fixes
- fix(corpus-delta): wire 7b-3 schema bridge — proof successes now reach training corpus
- fix(echidnabot): resolve 25 compile errors from ProverKind→ProverSlug migration
- fix(serve): warn at startup when webhook_secret is unset
- fix(serve): honour [server] section in echidnabot.toml
- fix(proofs): drop Coq .aux artefacts + gitignore build outputs
- fix(cargo): update gitbot-shared-context path after gitbot-fleet relocation

### Changed

- refactor(Task E): Continue ProverKind→ProverSlug migration integration
- refactor: eliminate all 13 production .unwrap() sites (#7)

### Documentation

- docs(flake): annotate KEEP+DEP rationale per standards#102 rule 3 (#16)
- docs(flake): annotate KEEP+DEP rationale per standards#102 rule 3 (#15)
- docs(flake): annotate KEEP+DEP rationale per standards#102 rule 3 (#13)
- docs(Task E): Document ProverKind→ProverSlug migration in STATE.a2ml
- docs(agentic): sunset BoJ-only-MCP exception — BoJ revived 2026-04-25
- docs(bot-directives): extend self-directive with mode block (Phase 7)
- docs(crg): populate external-targets / issues-fed-back / field-signal
- docs(mcp): fix stale STATE.scm reference in schema-gap caveat
- docs: cascade .scm→.a2ml refs across release/template/k9 docs
- docs(claude): update .machine_readable/ path table post .scm→.a2ml migration

### CI

- ci(rust): convert rust-ci.yml to thin wrapper (standards#174) (#19)
- ci: redistribute concurrency-cancel guard to read-only check workflows (#12)
- ci: bump actions/upload-artifact SHA to current v4 (#5)
- ci: bump actions/upload-artifact SHA to current v4 (#4)
- ci: fix workflow-linter YAML parse error + self-flag bug

## Pre-history

Prior commits to this file's introduction are recorded in git history but not formally classified into Keep-a-Changelog sections. To backfill, run `git cliff -o CHANGELOG.md` locally using the canonical [`cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml) — this is one-shot mechanical work.

---

<!-- This file was seeded by the 2026-05-26 estate tech-debt audit follow-up (Row-2 Phase 3); see [`hyperpolymath/standards/docs/audits/2026-05-26-estate-documentation-debt.md`](https://github.com/hyperpolymath/standards/blob/main/docs/audits/2026-05-26-estate-documentation-debt.md). -->
