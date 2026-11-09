#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Recheck legs of the receipt-boundary theory (PROOF-POLICY §2): Lean and Rocq.
# Each leg is an independent re-proof, not a transport of the Agda term. This
# script checks that each target kernel accepts its file and that each named
# theorem below depends on no axioms; the statement correspondence is recorded
# separately in docs/PROOF-STATUS.adoc.
#
# The axiom queries are not taken from the proof file. Each leg compiles the
# file as a module, then asks about a fixed list of names from a query file
# this script writes. What the proof file prints, or leaves unqueried, cannot
# change the verdict. Both legs first run a negative control (a planted axiom,
# a planted Admitted) that must compile, be reported, and still be refused.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# The theorems the T-RECEIPT-LEAN and T-RECEIPT-ROCQ rows claim. Keep these
# in step with docs/PROOF-STATUS.adoc.
lean_names=(
    EchidnabotReceiptBoundary.receiveWithSoundness
    EchidnabotReceiptBoundary.noUnconditionalReflection
    EchidnabotReceiptBoundary.noSoundnessForEmptyToken
)
rocq_names=(receive_with_soundness no_unconditional_reflection no_soundness_for_empty_token)

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

# lean_leg FILE NAME...: Lean compiles FILE as a module, and a separate query
# file reports each NAME as depending on no axioms. Prints the query output.
lean_leg() {
    local file=$1 dir mod out name
    shift
    dir=$(mktemp -d -p "$work")
    mod=$(basename "$file" .lean)
    cp "$file" "$dir/$mod.lean"
    (cd "$dir" && lean -o "$mod.olean" "$mod.lean" >/dev/null 2>&1) || { echo "lean rejected $file" >&2; return 1; }
    { printf 'import %s\n' "$mod"; printf '#print axioms %s\n' "$@"; } >"$dir/Query.lean"
    out=$(cd "$dir" && LEAN_PATH="$dir" lean Query.lean 2>&1) || { printf '%s\n' "$out"; return 1; }
    printf '%s\n' "$out"
    if grep -q 'depends on axioms' <<<"$out"; then return 1; fi
    for name in "$@"; do
        grep -qxF "'$name' does not depend on any axioms" <<<"$out" || return 1
    done
}

# rocq_leg FILE NAME...: coqc compiles FILE as a module, and one query file
# per NAME reports it closed under the global context. Prints the output.
rocq_leg() {
    local file=$1 dir mod out name
    shift
    dir=$(mktemp -d -p "$work")
    mod=$(basename "$file" .v)
    cp "$file" "$dir/$mod.v"
    (cd "$dir" && coqc -q -Q . Recheck "$mod.v" >/dev/null 2>&1) || { echo "coqc rejected $file" >&2; return 1; }
    for name in "$@"; do
        printf 'From Recheck Require %s.\nPrint Assumptions Recheck.%s.%s.\n' "$mod" "$mod" "$name" >"$dir/Query.v"
        out=$(cd "$dir" && coqc -q -Q . Recheck Query.v 2>&1) || { printf '%s\n' "$out"; return 1; }
        printf '%s: %s\n' "$name" "$out"
        [[ "$out" == 'Closed under the global context' ]] || return 1
    done
}

# control LEG FILE NAME REPORT: LEG must refuse FILE, and its query output must
# contain REPORT, so the refusal is the axiom check and not a compile error.
control() {
    local leg=$1 file=$2 name=$3 report=$4 out
    if out=$("$leg" "$file" "$name" 2>&1); then
        echo "FAIL: $leg accepted control $(basename "$file")." >&2
        exit 1
    fi
    if ! grep -q -- "$report" <<<"$out"; then
        printf '%s\n' "$out" >&2
        echo "FAIL: control $(basename "$file") was refused without reporting its planted axiom." >&2
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
printf 'axiom planted : False\ntheorem uses : False := planted\n' >"$work/Planted.lean"
control lean_leg "$work/Planted.lean" uses "'uses' depends on axioms: \[planted\]"
sha256sum proofs/theory/lean/ReceiptBoundary.lean
printf 'Invocation: lean -o ReceiptBoundary.olean ReceiptBoundary.lean, then #print axioms for:'
printf ' %s' "${lean_names[@]}"
printf '\n'
lean_leg proofs/theory/lean/ReceiptBoundary.lean "${lean_names[@]}"

require coqc
rocq_actual=$(coqc --version | head -n 1)
if [[ "$rocq_actual" != "$rocq_expected" ]]; then
    printf 'OPEN: expected %s, observed %s.\n' "$rocq_expected" "$rocq_actual" >&2
    exit 1
fi
printf 'Tool: %s\n' "$rocq_actual"
printf 'Lemma planted : False.\nAdmitted.\n' >"$work/Planted.v"
control rocq_leg "$work/Planted.v" planted '^Planted.planted : False'
sha256sum proofs/theory/rocq/ReceiptBoundary.v
printf 'Invocation: coqc -q -Q . Recheck ReceiptBoundary.v, then Print Assumptions for:'
printf ' %s' "${rocq_names[@]}"
printf '\n'
rocq_leg proofs/theory/rocq/ReceiptBoundary.v "${rocq_names[@]}"

printf '%s\n' 'LEAN-CHECKED and COQ-CHECKED for these source hashes only; statement correspondence is recorded in docs/PROOF-STATUS.adoc.'
