<!-- coderabbit-ai-generated:v1 -->

# Require code owner review for changes to the CI gate

`gate-pr` publishes `CI/gate` from a workflow the PR can modify. Use GitHub's
code owner review requirement as a separate merge condition. The base branch's
`.github/CODEOWNERS` covers `.github/`, including the workflows and CODEOWNERS
itself. Its wildcard also assigns an owner to the rest of the repository, so
enabling this requirement applies beyond workflow changes.

## Enable the rule

1. GitHub uses CODEOWNERS from the PR's base branch and does not let authors
   approve their own PRs. If the sole code owner also authors PRs, first arrange
   for a second trusted code owner with write access to be listed on the base
   branch for the paths they will review, including `.github/` for gate changes.
   The PR that adds that owner must land before strict enforcement, or its own
   owner-authored changes could become unmergeable.
2. In **Settings → Rules → Rulesets → Branch-Floor**, keep enforcement **Active**
   and keep the default branch target and existing required checks. Under
   **Require a pull request before merging**, enable **Require review from
   Code Owners** and keep **Dismiss stale pull request approvals when new commits
   are pushed** enabled. Remove any bypass entries that could merge without
   this review.

## Verify merge enforcement

- Read the [live rules for main](https://api.github.com/repos/hyperpolymath/echidnabot/rules/branches/main).
  The active Branch-Floor `pull_request` rule must have
  `require_code_owner_review: true` and `dismiss_stale_reviews_on_push: true`.
  Check the bypass list in Settings with admin access.
- Use a PR to `main` that makes a harmless change under `.github/workflows/`.
  Once all other merge conditions pass (including `CI/gate`), confirm that
  GitHub's merge box specifically blocks merging pending code owner review.
  Confirm a non-owner approval leaves that block in place, an eligible code
  owner's approval clears it, and a new commit that changes the diff restores
  it. Close the test PR without merging. A workflow result alone does not
  verify enforcement.

See [GitHub's CODEOWNERS documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners).
