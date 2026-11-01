#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Strict local theory check. This does NOT check Rust refinement or routing.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

expected='Agda version 2.8.0.2'
if ! command -v agda >/dev/null; then
    echo 'OPEN: Agda 2.8.0.2 is required; no proof was checked.' >&2
    exit 127
fi
actual=$(agda --version)
if [[ "$actual" != "$expected" ]]; then
    printf 'OPEN: expected %s, observed %s. Review the pin explicitly.\n' "$expected" "$actual" >&2
    exit 1
fi
printf 'Tool: %s\nBase commit (not a clean-tree claim): %s\n' "$actual" "$(git rev-parse HEAD)"
printf 'Working-tree source hashes:\n'
sha256sum proofs/agda/ReceiptBoundary.agda proofs/agda/FixtureIdentity.agda
for source in proofs/agda/ReceiptBoundary.agda proofs/agda/FixtureIdentity.agda; do
    # Ignore old interfaces so a previous, weaker-profile check is not reused.
    command=(agda --safe --without-K --cubical-compatible --exact-split
        --no-sized-types --no-guardedness --no-libraries --no-default-libraries
        --ignore-interfaces -i proofs/agda "$source")
    printf 'Invocation:'
    printf ' %q' "${command[@]}"
    printf '\n'
    "${command[@]}"
done
printf '%s\n' 'AGDA-VERIFIED for these source hashes only; independent rechecks and Rust refinement remain OPEN.'
