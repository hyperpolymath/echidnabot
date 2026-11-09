#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Recheck legs of the receipt-boundary theory (PROOF-POLICY §2): Lean and Rocq.
# Each leg is an independent re-proof, not a transport of the Agda term. This
# script checks that each target kernel accepts its file with no axioms; the
# statement correspondence is recorded separately in docs/PROOF-STATUS.adoc.
#
# Both legs also run a negative control first: a planted axiom (Lean) and a
# planted Admitted (Rocq) must make the leg fail, so a green run means the
# check can see the failure it guards against.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

lean_expected="version $(sed -n 's|^leanprover/lean4:v||p' proofs/theory/lean/lean-toolchain)"
rocq_expected='The Coq Proof Assistant, version 8.18.0'
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

require() {
    if ! command -v "$1" >/dev/null; then
        echo "OPEN: $1 is required; no proof was checked." >&2
        exit 127
    fi
}

# lean_leg FILE: the kernel accepts FILE and every `#print axioms` line reports
# no axioms. Prints the observed output.
lean_leg() {
    local out
    out=$(lean "$1" 2>&1) || { printf '%s\n' "$out"; return 1; }
    printf '%s\n' "$out"
    local printed clean
    printed=$(grep -c '#print axioms' "$1" || true)
    clean=$(grep -c 'does not depend on any axioms' <<<"$out" || true)
    [[ "$printed" -gt 0 && "$clean" -eq "$printed" ]]
}

# rocq_leg FILE: coqc accepts FILE and every `Print Assumptions` is closed.
rocq_leg() {
    local out
    out=$(cd "$(dirname "$1")" && coqc -q "$(basename "$1")" 2>&1) || { printf '%s\n' "$out"; return 1; }
    printf '%s\n' "$out"
    local printed clean
    printed=$(grep -c '^Print Assumptions' "$1" || true)
    clean=$(grep -c 'Closed under the global context' <<<"$out" || true)
    [[ "$printed" -gt 0 && "$clean" -eq "$printed" ]]
}

printf 'Base commit (not a clean-tree claim): %s\n' "$(git rev-parse HEAD)"

require lean
lean_actual=$(lean --version | head -n 1)
if [[ "$lean_actual" != *"$lean_expected,"* ]]; then
    printf 'OPEN: expected Lean %s, observed %s.\n' "$lean_expected" "$lean_actual" >&2
    exit 1
fi
printf 'Tool: %s\n' "$lean_actual"
cat >"$work/Planted.lean" <<'EOF'
axiom planted : False
theorem uses : False := planted
#print axioms uses
EOF
if lean_leg "$work/Planted.lean" >/dev/null; then
    echo 'FAIL: the Lean leg accepted a planted axiom.' >&2
    exit 1
fi
sha256sum proofs/theory/lean/ReceiptBoundary.lean
printf 'Invocation: lean proofs/theory/lean/ReceiptBoundary.lean\n'
lean_leg proofs/theory/lean/ReceiptBoundary.lean

require coqc
rocq_actual=$(coqc --version | head -n 1)
if [[ "$rocq_actual" != "$rocq_expected" ]]; then
    printf 'OPEN: expected %s, observed %s.\n' "$rocq_expected" "$rocq_actual" >&2
    exit 1
fi
printf 'Tool: %s\n' "$rocq_actual"
cat >"$work/Planted.v" <<'EOF'
Lemma planted : False.
Admitted.
Print Assumptions planted.
EOF
if rocq_leg "$work/Planted.v" >/dev/null; then
    echo 'FAIL: the Rocq leg accepted a planted Admitted.' >&2
    exit 1
fi
cp proofs/theory/rocq/ReceiptBoundary.v "$work/"
sha256sum proofs/theory/rocq/ReceiptBoundary.v
printf 'Invocation: coqc -q ReceiptBoundary.v\n'
rocq_leg "$work/ReceiptBoundary.v"

printf '%s\n' 'LEAN-CHECKED and COQ-CHECKED for these source hashes only; statement correspondence is recorded in docs/PROOF-STATUS.adoc.'
