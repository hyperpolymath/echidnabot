#!/usr/bin/env bash
# SPDX-License-Identifier: PMPL-1.0-or-later
# Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# apply-branch-protection.sh — set canonical branch-protection rules on `main`.
#
# Per the hyperpolymath estate memory rule "Branch protection standard":
#   PR required, 0 approvals, enforce_admins=false, linear, no force-push,
#   signatures ON (POST after PUT).
#
# Plus required status checks: the comprehensive set added across this
# session (rust-ci, cargo-audit, db-checks, codeql, hypatia-scan,
# static-analysis-gate, openssf-compliance, dogfood-gate, scorecard).
#
# CAVEAT: GitHub Free blocks branch protection on private repos. This
# script will fail with HTTP 403 unless either:
#   (a) the repository is public, OR
#   (b) the org/owner is on GitHub Pro / Team / Enterprise.
# The script is intentionally idempotent + reversible — re-running it
# with adjusted requirements is safe; it overwrites the same protection
# config each time.

set -euo pipefail

REPO="${1:-hyperpolymath/echidnabot}"
BRANCH="${2:-main}"

echo "Applying branch protection to ${REPO}@${BRANCH}…"

# Required status checks — names match the GitHub job display names from
# the workflow files at .github/workflows/*.yml.
read -r -d '' PROTECTION_BODY <<'JSON' || true
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Cargo check + clippy + fmt",
      "Dependency audit",
      "Migrations + schema drift",
      "Analyze (rust)",
      "Hypatia Neurosymbolic Analysis",
      "Static Analysis Gate",
      "OpenSSF Compliance",
      "Dogfood Gate",
      "OpenSSF Scorecard"
    ]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 0
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": true,
  "lock_branch": false,
  "allow_fork_syncing": true
}
JSON

# Apply the protection. PUT is the canonical create/replace.
echo "  PUT /repos/${REPO}/branches/${BRANCH}/protection"
if ! gh api -X PUT "repos/${REPO}/branches/${BRANCH}/protection" \
    -H "Accept: application/vnd.github+json" \
    --input - <<<"${PROTECTION_BODY}"; then
  cat <<'EOF' >&2

╭───────────────────────────────────────────────────────────────────────╮
│  Branch protection PUT failed.                                         │
│                                                                        │
│  Most common cause: this repo is private + the owning account is on    │
│  GitHub Free. Per estate memory feedback_github_free_private_ruleset_  │
│  block, branch protection on private repos requires Pro/Team/Ent.      │
│                                                                        │
│  Options:                                                              │
│    (1) Make the repo public — runs immediately.                        │
│    (2) Upgrade plan tier — also runs immediately.                      │
│    (3) Skip — keep protection in this script as documented intent      │
│        and revisit when plan changes.                                  │
╰───────────────────────────────────────────────────────────────────────╯
EOF
  exit 1
fi

# Per memory rule: "signatures ON (POST after PUT)". The signatures
# requirement is its own endpoint POST after the main protection
# config is in place.
echo "  POST /repos/${REPO}/branches/${BRANCH}/protection/required_signatures"
gh api -X POST "repos/${REPO}/branches/${BRANCH}/protection/required_signatures" \
    -H "Accept: application/vnd.github+json" \
    || echo "  (signatures POST optional — skipping if endpoint not available)"

echo "✓ Branch protection applied to ${REPO}@${BRANCH}"
