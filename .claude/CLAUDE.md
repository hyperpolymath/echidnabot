## Machine-Readable Artefacts

The following files in `.machine_readable/6a2/` contain structured project metadata
(TOML-flavoured A2ML, migrated from Guile Scheme in `bdbc5a4`):

- `STATE.a2ml` - Current project state and progress
- `META.a2ml` - Architecture decisions and development practices
- `ECOSYSTEM.a2ml` - Position in the ecosystem and related projects
- `AGENTIC.a2ml` - AI agent interaction patterns (incl. `[exceptions.boj-only-mcp]`)
- `NEUROSYM.a2ml` - Neurosymbolic integration config
- `PLAYBOOK.a2ml` - Operational runbook
- `ANCHOR.a2ml` - Canonical identity + recalibration triggers

---

# CLAUDE.md - AI Assistant Instructions

## Language Policy (Hyperpolymath Standard)

### ALLOWED Languages & Tools

| Language/Tool | Use Case | Notes |
|---------------|----------|-------|
| **AffineScript** | Primary application code | Affine-typed, compiles to typed-wasm or Deno-ESM |
| **Deno** | Runtime & package management | Replaces Node/npm/bun |
| **Rust** | Performance-critical, systems, WASM | Preferred for CLI tools |
| **Tauri 2.0+** | Mobile apps (iOS/Android) | Rust backend + web UI |
| **Dioxus** | Mobile apps (native UI) | Pure Rust, React-like |
| **Gleam** | Backend services | Runs on BEAM or compiles to JS |
| **Bash/POSIX Shell** | Scripts, automation | Keep minimal |
| **JavaScript** | Only where AffineScript cannot | MCP protocol glue, Deno APIs |
| **Nickel** | Configuration language | For complex configs |
| **Guile Scheme** | Guix packages | guix.scm, manifests/*.scm |
| **A2ML (TOML)** | State/meta files | .machine_readable/6a2/*.a2ml |
| **Julia** | Batch scripts, data processing | Per RSR |
| **OCaml** | AffineScript compiler | Language-specific |
| **Ada** | Safety-critical systems | Where required |

### BANNED - Do Not Use

| Banned | Replacement |
|--------|-------------|
| TypeScript | AffineScript |
| Node.js | Deno |
| npm | Deno |
| Bun | Deno |
| pnpm/yarn | Deno |
| Go | Rust |
| Python | Julia/Rust/AffineScript |
| Java/Kotlin | Rust/Tauri/Dioxus |
| Swift | Tauri/Dioxus |
| React Native | Tauri/Dioxus |
| Flutter/Dart | Tauri/Dioxus |

### Mobile Development

**No exceptions for Kotlin/Swift** - use Rust-first approach:

1. **Tauri 2.0+** - Web UI (AffineScript) + Rust backend, MIT/Apache-2.0
2. **Dioxus** - Pure Rust native UI, MIT/Apache-2.0

Both are FOSS with independent governance (no Big Tech).

### Enforcement Rules

1. **No new TypeScript files** - Convert existing TS to AffineScript
2. **No package.json for runtime deps** - Use deno.json imports
3. **No node_modules in production** - Deno caches deps automatically
4. **No Go code** - Use Rust instead
5. **No Python anywhere** - Use Julia for data/batch, Rust for systems, AffineScript for apps
6. **No Kotlin/Swift for mobile** - Use Tauri 2.0+ or Dioxus

### Package Management

- **Sole primary**: Guix (guix.scm) — nix is deprecated estate-wide
  2026-06-01; do NOT add flake.nix/flake.lock back
- **JS deps**: Deno (deno.json imports)

### Security Requirements

- No MD5/SHA1 for security (use SHA256+)
- HTTPS only (no HTTP URLs)
- No hardcoded secrets
- SHA-pinned dependencies
- SPDX license headers on all files

---

## Repository-Specific Operating Notes (echidnabot)

### Identity

- **Role in the fleet:** Tier-1 Verifier in `gitbot-fleet`.
- **Coordinator:** Hypatia (`hyperpolymath/hypatia`) — see
  `.github/workflows/hypatia-scan.yml`.
- **Upstream engine:** `hyperpolymath/echidna` — echidnabot dispatches
  to it but is **not itself a prover**.
- **Self-mode:** `advisor` (per
  `.machine_readable/bot_directives/echidnabot.a2ml`). Do **not** flip
  to `regulator` without owner approval — the dogfood proofs in
  `proofs/` include deliberately-failing stubs that would block every
  merge.

### First-read order (when entering this repo)

1. `.machine_readable/6a2/STATE.a2ml` — blockers + completion-percentage + session notes
2. `.machine_readable/6a2/AGENTIC.a2ml` — tooling constraints; check `[exceptions.*]`
3. `.machine_readable/bot_directives/echidnabot.a2ml` — self-mode + future-direction notes
4. `EXPLAINME.adoc` — caveats on README claims (test counts, mode wiring, prover surface drift)
5. `ROADMAP.adoc` — what's actually done vs aspirational

### Build caveat — DO NOT try to `cargo build` from a bare clone

`Cargo.toml` carries a path dependency on `gitbot-shared-context` at
`../../shared-context` — that path only resolves inside the
`gitbot-fleet` monorepo layout. Standalone `git clone` of echidnabot
will **fail at dependency resolution**. This is acknowledged debt
tracked in issue #18; do not "fix" it unless explicitly asked.

For documentation work this doesn't matter. For code work, work from a
`gitbot-fleet` checkout.

### Dual-truth pattern (monorepo + standalone)

echidnabot is checked into `hyperpolymath/echidna`'s tree at
`echidna/echidnabot/` **as regular files** but is also registered in
that repo's `.gitmodules`. Changes to `.machine_readable/` files must
land in **both** the standalone repo and the monorepo copy. See the
memory note `project_gitbot_fleet_dual_truth_pattern` for the rationale
and the eventual fix path (promote to real submodule or delete
`.gitmodules` entry).

### BoJ-only-MCP exception

`echidnabot-mcp` (4 tools: `suggest_tactics`, `record_tactic_outcome`,
`list_outcome_history`, `corpus_refresh`) is a documented carve-out
from the estate-wide "all MCP through BoJ" rule. Scope: this repo
only. Sunset: when the BoJ cartridge supports the double-loop
feedback protocol. See `.machine_readable/6a2/AGENTIC.a2ml`
`[exceptions.boj-only-mcp]` for the canonical record.

### Upstream-drift warning (12 vs 113 provers)

`src/dispatcher/echidna_client.rs` enumerates **12 prover backends**.
The upstream `echidna` engine supports **113** (per its
`src/rust/provers/mod.rs::ProverKind` enum after commit `c8c0acf`).
Consumers needing the full surface today go through
`boj-server/cartridges/echidna-llm-mcp`. Drift is intentional;
do **not** "fix" by inflating the local enum.

The `ProverKind` *type* on echidnabot's side has migrated from a
12-variant enum to `ProverSlug(String)` (a newtype + alias) so new
provers can land without touching the type. Use
`ProverKind::new("slug")` to construct.

### Sensitive files — leave alone

- `proofs/coq/admitted_stub.v` and `proofs/lean/sorry_stub.lean` are
  **intentional failures** for dogfood CI. Do not "fix" them.
- `Cargo.toml`'s `gitbot-shared-context` path entry — see build
  caveat above.
- `LICENSE` and `LICENSE.txt` are intentionally identical (MPL-2.0).

### When making changes, update in lockstep

| Change                          | Update                                                  |
| ------------------------------- | ------------------------------------------------------- |
| New CLI flag                    | `README.adoc` Usage + `wiki/Getting-Started.md`         |
| New config option               | `echidnabot.example.toml` + `docs/content/configuration.md` + `wiki/Getting-Started.md` |
| New prover support              | `wiki/Supported-Provers.md` + `src/dispatcher/`         |
| New bot-mode behaviour          | `wiki/FAQ.md` + `.machine_readable/bot_directives/echidnabot.a2ml` |
| ABI/FFI surface change          | `src/abi/*.idr` + `ffi/zig/src/main.zig` + `ABI-FFI-README.md` |
| Roadmap item closed             | `ROADMAP.adoc` + `.machine_readable/6a2/STATE.a2ml`     |
| New external target / consumer  | `.machine_readable/6a2/STATE.a2ml` `[external-targets]` |

### Tests — full suite is 184

Per `STATE.a2ml` (last-updated `2026-04-26`): 137 lib + 17 lifecycle +
32 integration + 12 property + 15 seam + 8 smoke = 184/184. Older
docs (README, ROADMAP) say `129` — that was pre-7b. Treat 184 as the
truth and update stragglers when you touch them.

### Commit hygiene

- All commits **GPG-signed** with key
  `4A03639C1EB1F86C7F0C97A91835A14A2867091E`.
- Author email `6759885+hyperpolymath@users.noreply.github.com`.
- Conventional Commits format (`feat(scope): …`).
- `CHANGELOG.md` regenerates from commits via
  `standards/changelog-reusable.yml` — **do not hand-edit**.
- Co-author trailer:
  `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>`
- PR strategy: open with `gh pr create`, then immediately
  `gh pr merge <N> --auto --squash --delete-branch`.

