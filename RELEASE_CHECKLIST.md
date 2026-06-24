<!--
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# echidnabot Release Checklist

Procedure for cutting a release of echidnabot, plus standing checklist
of items that should already be in place before any release ships.

**Current target:** v0.2.0 (production hardening — observability,
deployment automation; see [`ROADMAP.adoc`](ROADMAP.adoc) "Remaining
Work").

**Last reviewed:** 2026-06-01

## Repository Setup

### Done
- [x] README.adoc - SEO-optimized, project-focused
- [x] BRANDING.md - Visual identity and LLM art prompts
- [x] Justfile - RSR canonical task runner
- [x] Nickel configuration (config/echidnabot.ncl)
- [x] MCP configuration (.claude/settings/mcp.json)
- [x] STATE.a2ml - Project checkpoint
- [x] META.a2ml - Dublin Core metadata
- [x] ECOSYSTEM.a2ml - Dependency graph
- [x] GitHub topics file (.github/topics.txt)

### To Apply Manually
- [ ] **Apply GitHub Topics** - Go to repo Settings → About → Topics and add:
  ```
  theorem-prover, formal-verification, proof-assistant, ci-cd, rust, coq,
  lean4, agda, isabelle, z3, smt, formal-methods, type-theory, github-app,
  automation, mathematics, logic, webhooks, hacktoberfest
  ```
- [ ] **Update GitHub Description** - Set to:
  > Proof-aware CI bot that verifies mathematical theorems on every push. Coq, Lean, Agda, Isabelle, Z3 support. Rust + Tokio + GraphQL.

## Wiki ✅

### Done
- [x] wiki/Home.md
- [x] wiki/Getting-Started.md
- [x] wiki/Architecture.md
- [x] wiki/Supported-Provers.md
- [x] wiki/FAQ.md

### To Add
- [ ] wiki/Configuration-Reference.md - All config options
- [ ] wiki/API-Reference.md - GraphQL schema documentation
- [ ] wiki/Platform-Integration.md - GitHub/GitLab/Bitbucket setup
- [ ] wiki/Troubleshooting.md - Common issues
- [ ] wiki/Changelog.md - Version history
- [ ] wiki/Roadmap.md - Future plans

### To Do Manually
- [ ] **Enable Wiki** in GitHub repo settings
- [ ] **Push wiki/** to the wiki repo:
  ```bash
  git clone https://github.com/hyperpolymath/echidnabot.wiki.git
  cp wiki/*.md echidnabot.wiki/
  cd echidnabot.wiki && git add . && git commit -m "Initial wiki" && git push
  ```

## CI/CD ✅

### Done
- [x] .github/workflows/quality.yml - Rust build/test/lint
- [x] .github/workflows/docs.yml - casket-ssg documentation
- [x] .github/workflows/echidnabot.yml - Self-referential proof checking
- [x] .github/workflows/codeql.yml - Security scanning
- [x] .github/workflows/scorecard.yml - OSSF Scorecard

### To Add/Verify
- [ ] Ensure all workflows pass on main branch
- [ ] Add release workflow for crates.io publishing
- [ ] Add container publishing to ghcr.io

## Documentation

### Done
- [x] README.adoc + README.md (Markdown summary linking into .adoc)
- [x] EXPLAINME.adoc — receipts behind README claims
- [x] ROADMAP.adoc — phases + completion
- [x] CONTRIBUTING.md
- [x] SECURITY.md
- [x] CODE_OF_CONDUCT.md
- [x] MAINTAINERS.adoc
- [x] CHANGELOG.md (auto-generated from conventional commits)
- [x] CITATION.cff + codemeta.json
- [x] ABI-FFI-README.md — Zig FFI + Idris2 ABI boundary
- [x] 0-AI-MANIFEST.a2ml + .claude/CLAUDE.md — AI assistant pointers
- [x] RSR_OUTLINE.adoc + RSR_COMPLIANCE.adoc
- [x] docs/content/{index,getting-started,configuration,api}.md (casket-ssg)

### To Add
- [ ] docs/DEPLOYMENT.md — Production deployment guide (k8s, helm, docker-compose)
- [ ] Man pages (docs/man/echidnabot.1) via Mustfile recipe

## Branding Assets 📝

### To Create (using LLM prompts in BRANDING.md)
- [ ] **Avatar** (512x512) - Geometric echidna logo
- [ ] **Banner** (1280x640) - GitHub social preview
- [ ] **Favicon** (32x32, 16x16) - For docs site

### To Apply
- [ ] Upload avatar to GitHub org/repo
- [ ] Set social preview image in repo settings
- [ ] Add favicon to docs site

## Code Quality 🔄

### To Complete
- [ ] Run `cargo fmt` on all files
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Achieve 50%+ test coverage
- [ ] Add integration tests
- [ ] Run `cargo audit` and fix vulnerabilities
- [ ] Run `cargo deny check` for license compliance

## Core Functionality

### Phase 1 (MVP) — landed in v0.1.0
- [x] GitHub webhook handler with signature verification (HMAC-SHA256)
- [x] GitLab + Bitbucket webhook handlers
- [x] Proof file detection (by extension across 7 file types)
- [x] ECHIDNA Core dispatcher client (REST + GraphQL)
- [x] GitHub Check Run reporter
- [x] SQLite + PostgreSQL persistence
- [x] CLI: `serve`, `register`, `check`, `status`, `init-db`

### Phase 2 (Multi-Prover) — landed in v0.1.0
- [x] Auto-detect prover from file extension
- [x] 12-prover surface (Coq, Lean 4, Agda, Isabelle, Z3, CVC5, Metamath, HOL Light, Mizar, PVS, ACL2, HOL4)
- [x] `ProverKind` slug newtype — open-ended for 113 upstream provers
- [x] Parallel proof checking (semaphore-bounded)
- [x] Aggregated results

### Phase 3 (Hardening) — see ROADMAP.adoc
- [x] Container isolation (podman + bwrap, fail-safe)
- [x] Retry + circuit breaker (5-failures-then-open, 5-min reset)
- [x] Trust bridge (confidence levels, solver integrity, axiom tracking)
- [x] Per-IP webhook rate limiting
- [x] Prometheus `/metrics` endpoint
- [x] Double-loop feedback (tactic-outcome recording + corpus delta)
- [ ] OpenTelemetry distributed tracing
- [ ] Structured JSON logging end-to-end
- [ ] Graceful shutdown (finish in-progress jobs before exit)

## Security ✅

### Done
- [x] SECURITY.md policy
- [x] .well-known/security.txt
- [x] HMAC-SHA256 webhook verification (code exists)
- [x] No hardcoded secrets
- [x] SHA-pinned GitHub Actions

### To Verify
- [ ] Run TruffleHog scan: no secrets in history
- [ ] Run CodeQL: no critical findings
- [ ] OSSF Scorecard: 7+ score

## Packaging 🔄

### Done
- [x] Cargo.toml metadata complete
- [x] guix.scm package definition
- [x] Containerfile for Docker/Podman
- [x] Justfile for task automation

### To Add
- [ ] cargo-deb configuration
- [ ] cargo-rpm configuration
- [ ] Homebrew formula (optional)

(Nix flake intentionally NOT planned: nix is deprecated estate-wide
as of 2026-06-01.)

## Release Process

### Pre-Release
1. [ ] All tests passing
2. [ ] Changelog updated
3. [ ] Version bumped in Cargo.toml
4. [ ] STATE.a2ml updated
5. [ ] Documentation reviewed

### Release
1. [ ] Create git tag: `git tag -s v0.1.0 -m "Release 0.1.0"`
2. [ ] Push tag: `git push origin v0.1.0`
3. [ ] GitHub release created with notes
4. [ ] Publish to crates.io: `cargo publish`
5. [ ] Container pushed to ghcr.io
6. [ ] Announce on relevant channels

### Post-Release
1. [ ] Verify crates.io page
2. [ ] Verify container works
3. [ ] Update roadmap
4. [ ] Start next milestone

## External Integration

### GitHub
- [ ] Enable GitHub Discussions
- [ ] Set up issue templates (if not present)
- [ ] Configure branch protection rules
- [ ] Enable Dependabot

### Marketing
- [ ] Post to Hacker News (when ready)
- [ ] Post to r/rust, r/programming
- [ ] Post to Coq, Lean, Agda communities
- [ ] Add to Awesome lists (awesome-rust, etc.)

## Metrics

### Success Criteria for v1.0
- [ ] 100+ GitHub stars
- [ ] 5+ external contributors
- [ ] 3+ production users
- [ ] 80%+ test coverage
- [ ] OSSF Scorecard 8+

---

## Priority Order

1. **Immediate** (before merge)
   - Apply GitHub topics manually
   - Update GitHub description
   - Enable wiki and push content

2. **This Week**
   - Create branding assets
   - Add missing wiki pages
   - Complete Phase 1 functionality

3. **This Month**
   - Achieve MVP release (v0.2)
   - 50% test coverage

4. **Next Quarter**
   - v1.0 production release
   - Multi-platform support
   - ML tactic suggestions
