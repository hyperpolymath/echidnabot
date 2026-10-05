<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# ECHIDNA Integration

echidnabot orchestrates; [ECHIDNA](https://github.com/hyperpolymath/echidna) proves. This page is the user-facing summary of `docs/ECHIDNA-INTEGRATION.adoc` in the repository, which is the authoritative record.

## Connecting

- Default endpoints: `http://127.0.0.1:8081` (REST, `echidna server`) and `http://127.0.0.1:8081/` (GraphQL, the separate `echidna-graphql` binary, which serves at `/`). Both ECHIDNA binaries default to port 8081, so run one or the other; `auto` mode tries GraphQL and falls back to REST.
- **Version handshake:** on start-up echidnabot reads `GET /api/provers` (and `/api/health` if needed). It refuses to run against an ECHIDNA older than **2.3.0**, or one that reports no version. An unreachable or warming-up (5xx) ECHIDNA only logs a warning, and every job retries the handshake before dispatching. GraphQL-only mode skips this REST check.
- Prover names are taken from ECHIDNA's own `/api/provers` list, so new ECHIDNA backends need no echidnabot change.

## Trust levels

The 5-level confidence shown in check runs and PR comments is computed by **ECHIDNA's trust kernel** (linked as a library), not by an echidnabot copy. Axioms and holes (`sorry`, `Admitted`, `postulate`, `believe_me`, ...) are found by ECHIDNA's source scanner, plus a scan of the prover's output text.

An axiom that ECHIDNA names (for example `sorryAx`) counts at full severity, even if echidnabot's own scan of the source finds nothing.

echidnabot never verifies proof certificates itself, so a certificate artefact on its own does not raise a single result above Level 2.

Each result says where its trust data came from:

| Source | Meaning |
|---|---|
| `echidna` | ECHIDNA reported the axioms itself (`echidna.prove.result/1` `trust` object), passed on unchanged |
| `local-fallback` | echidnabot derived the axioms from the proof source and prover output |

## The `echidna.prove.result/1` contract

When ECHIDNA answers in the shared contract shape, echidnabot reads it directly:

```json
{"duration_ms":12,"echidna_version":"2.3.0","goal":"t","message":"ok","prover":"Lean","schema":"echidna.prove.result/1","status":"verified","trust":{"axioms":[],"confidence":null}}
```

`status` is one of `verified`, `failed`, `error`, `timeout`, `unknown`. Older ECHIDNA response bodies are still accepted.

## Identifiers

echidnabot mints UUIDv7 ids for jobs and records. A UUIDv8 content-id helper (SHA-256 over RFC 8785 canonical JSON, the same construction ECHIDNA and proof-burrower use) is available in `src/ids.rs`. Nothing uses it yet for proof goals or results.
