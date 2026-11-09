#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Recheck legs of the receipt-boundary theory (PROOF-POLICY §2): Lean and Rocq.
# Each leg is an independent re-proof, not a transport of the Agda term. This
# script checks that each target kernel accepts its file with no axioms; the
# statement correspondence is recorded separately in docs/PROOF-STATUS.adoc.
#
# Both legs also run negative controls first: a planted axiom (Lean) and a
# planted Admitted (Rocq), each queried flush-left and indented, must make the
# leg fail, so a green run means the check can see the failure it guards
# against.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

lean_expected="version $(sed -n 's|^leanprover/lean4:v||p' proofs/theory/lean/lean-toolchain)"
rocq_expected='The Coq Proof Assistant, version 8.18.0'
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

require() {
    local tool=$1
    if ! command -v "$tool" >/dev/null; then
        echo "OPEN: $tool is required; no proof was checked." >&2
        exit 127
    fi
}

# Both legs judge the checker's own output, not the source text, so a query
# the source count misses (indented, or produced some other way) still fails
# the leg if it reports an axiom. The source count only sets a floor: there
# must be at least as many clean reports as query commands. A query inside a
# block comment raises that floor without a report, so it fails closed.

# lean_leg FILE: the kernel accepts FILE, no `#print axioms` report names an
# axiom, and the clean reports cover every query command. Prints the output.
lean_leg() {
    local out
    out=$(lean "$1" 2>&1) || { printf '%s\n' "$out"; return 1; }
    printf '%s\n' "$out"
    local queries clean tainted
    queries=$(grep -c '^[[:space:]]*#print axioms' "$1" || true)
    clean=$(grep -c 'does not depend on any axioms' <<<"$out" || true)
    tainted=$(grep -c 'depends on axioms' <<<"$out" || true)
    [[ "$tainted" -eq 0 && "$queries" -gt 0 && "$clean" -ge "$queries" ]]
}

# rocq_leg FILE: coqc accepts FILE, no `Print Assumptions` report lists
# axioms, and the closed reports cover every query command.
rocq_leg() {
    local out
    out=$(cd "$(dirname "$1")" && coqc -q "$(basename "$1")" 2>&1) || { printf '%s\n' "$out"; return 1; }
    printf '%s\n' "$out"
    local queries clean tainted
    queries=$(grep -c '^[[:space:]]*Print Assumptions' "$1" || true)
    clean=$(grep -c 'Closed under the global context' <<<"$out" || true)
    tainted=$(grep -c '^Axioms:' <<<"$out" || true)
    [[ "$tainted" -eq 0 && "$queries" -gt 0 && "$clean" -ge "$queries" ]]
}

# control LEG FILE REPORT COMPILE...: FILE must compile under COMPILE, the
# checker must print REPORT (it saw the planted axiom), and LEG must still
# refuse FILE. A compile error therefore cannot pass as a refusal.
control() {
    local leg=$1 file=$2 report=$3 out
    shift 3
    out=$("$@" 2>&1) || { printf '%s\n' "$out" >&2; echo "FAIL: control $(basename "$file") did not compile." >&2; exit 1; }
    if ! grep -q -- "$report" <<<"$out"; then
        echo "FAIL: control $(basename "$file") did not report its planted axiom." >&2
        exit 1
    fi
    if "$leg" "$file" >/dev/null; then
        echo "FAIL: $leg accepted control $(basename "$file")." >&2
        exit 1
    fi
}

printf 'Base commit (not a clean-tree claim): %s\n' "$(git rev-parse HEAD)"

require lean
lean_actual=$(lean --version | head -n 1)
if [[ "$lean_actual" != *"$lean_expected,"* ]]; then
    printf 'OPEN: expected Lean %s, observed %s.\n' "$lean_expected" "$lean_actual" >&2
    exit 1
fi
printf 'Tool: %s\n' "$lean_actual"
printf 'axiom planted : False\ntheorem uses : False := planted\n#print axioms uses\n' >"$work/Planted.lean"
printf 'axiom planted : False\ntheorem uses : False := planted\n  #print axioms uses\n' >"$work/Indented.lean"
for f in Planted Indented; do
    control lean_leg "$work/$f.lean" 'depends on axioms: \[planted\]' lean "$work/$f.lean"
done
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
printf 'Lemma planted : False.\nAdmitted.\nPrint Assumptions planted.\n' >"$work/Planted.v"
printf 'Lemma planted : False.\nAdmitted.\n  Print Assumptions planted.\n' >"$work/Indented.v"
for f in Planted Indented; do
    # shellcheck disable=SC2016 # $1 and $2 are expanded by the inner sh, not here.
    control rocq_leg "$work/$f.v" '^planted : False' sh -c 'cd "$1" && coqc -q "$2"' _ "$work" "$f.v"
done
cp proofs/theory/rocq/ReceiptBoundary.v "$work/"
sha256sum proofs/theory/rocq/ReceiptBoundary.v
printf 'Invocation: coqc -q ReceiptBoundary.v\n'
rocq_leg "$work/ReceiptBoundary.v"

printf '%s\n' 'LEAN-CHECKED and COQ-CHECKED for these source hashes only; statement correspondence is recorded in docs/PROOF-STATUS.adoc.'
