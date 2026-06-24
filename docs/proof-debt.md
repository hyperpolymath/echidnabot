<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
# Proof Debt

Tracks soundness-relevant escape hatches in `echidnabot` that the trusted-base reducer flags as undocumented. Each is retained pending dedicated cleanup; new occurrences should land with an inline `// TRUSTED:` annotation or an entry below.

## Current entries (placeholder)

The current 2 escape hatches were inherited from earlier scaffolding and have not yet been individually classified. Tracking issue: filed alongside this PR.

- Hatch 1: TBD (file:line)
- Hatch 2: TBD (file:line)

## How to populate

Run the standards trusted-base reducer locally; it lists the offending file:line tuples. Move each from "TBD" to a real entry with a one-line rationale. When all entries have rationale + cleanup-issue link, this file is fully load-bearing instead of a placeholder.
