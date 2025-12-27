# echidnabot - Proof-Aware CI Bot
# https://just.systems/man/en/
#
# Run `just` to see all available recipes
# Run `just cookbook` to generate documentation
# Run `just combinations` to see matrix recipe options
#
# NOTE: This project uses `just` for local tasks.
#       Deployment contracts use `must` (see mustfile pattern).
#       Makefiles are FORBIDDEN per RSR policy.

set shell := ["bash", "-uc"]
set dotenv-load := true
set positional-arguments := true

# Project metadata
project := "echidnabot"
version := "0.1.0"
tier := "1"  # RSR Tier 1 (Rust)

# ═══════════════════════════════════════════════════════════════════════════════
# DEFAULT & HELP
# ═══════════════════════════════════════════════════════════════════════════════

# Show all available recipes
default:
    @just --list --unsorted

# Show detailed help for a recipe
help recipe="":
    #!/usr/bin/env bash
    if [ -z "{{recipe}}" ]; then
        just --list --unsorted
        echo ""
        echo "Usage: just help <recipe>"
        echo "       just cookbook     # Generate docs"
        echo "       just combinations # Show matrix recipes"
    else
        just --show "{{recipe}}" 2>/dev/null || echo "Recipe '{{recipe}}' not found"
    fi

# Show project info
info:
    @echo "Project: {{project}}"
    @echo "Version: {{version}}"
    @echo "RSR Tier: {{tier}}"
    @echo "Rust: $(rustc --version 2>/dev/null || echo 'not found')"
    @[ -f STATE.scm ] && grep -oP '\(phase\s+\.\s+\K[^)]+' STATE.scm | head -1 | xargs -I{} echo "Phase: {}" || true

# ═══════════════════════════════════════════════════════════════════════════════
# BUILD & COMPILE
# ═══════════════════════════════════════════════════════════════════════════════

# Build in debug mode
build *args:
    cargo build {{args}}

# Build in release mode with optimizations
build-release *args:
    cargo build --release {{args}}

# Watch for changes and rebuild
watch:
    cargo watch -x build

# Clean build artifacts [reversible: rebuild with `just build`]
clean:
    cargo clean
    rm -rf docs/_site target

# ═══════════════════════════════════════════════════════════════════════════════
# TEST & QUALITY
# ═══════════════════════════════════════════════════════════════════════════════

# Run all tests
test *args:
    cargo test {{args}}

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific test
test-one TEST:
    cargo test {{TEST}} -- --nocapture

# Run tests with coverage (requires cargo-llvm-cov)
test-coverage:
    cargo llvm-cov --html --open

# Run benchmarks
bench:
    cargo bench

# ═══════════════════════════════════════════════════════════════════════════════
# LINT & FORMAT
# ═══════════════════════════════════════════════════════════════════════════════

# Format all source files [reversible: git checkout]
fmt:
    cargo fmt

# Check formatting without changes
fmt-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Run all quality checks
quality: fmt-check lint test
    @echo "All quality checks passed!"

# Fix auto-fixable issues [reversible: git checkout]
fix:
    cargo clippy --fix --allow-dirty --allow-staged
    cargo fmt

# ═══════════════════════════════════════════════════════════════════════════════
# RUN & SERVE
# ═══════════════════════════════════════════════════════════════════════════════

# Run the bot (debug)
run *args:
    cargo run -- {{args}}

# Run the webhook server
serve port="8080":
    cargo run -- serve --port {{port}}

# Run with hot reload
dev:
    cargo watch -x 'run -- serve'

# Run with verbose logging
run-debug:
    RUST_LOG=debug cargo run -- -v serve

# ═══════════════════════════════════════════════════════════════════════════════
# DATABASE
# ═══════════════════════════════════════════════════════════════════════════════

# Initialize the database
db-init:
    cargo run -- init-db

# Run migrations
db-migrate:
    cargo sqlx migrate run

# Create new migration
db-new name:
    cargo sqlx migrate add {{name}}

# Reset database [DESTRUCTIVE]
db-reset:
    rm -f echidnabot.db
    cargo run -- init-db

# ═══════════════════════════════════════════════════════════════════════════════
# CLI COMMANDS
# ═══════════════════════════════════════════════════════════════════════════════

# Register a repository
register repo platform="github":
    cargo run -- register --repo {{repo}} --platform {{platform}}

# Trigger a manual check
trigger repo commit="":
    cargo run -- check --repo {{repo}} {{if commit != "" { "--commit " + commit } else { "" } }}

# Show status
status-check target:
    cargo run -- status --target {{target}}

# ═══════════════════════════════════════════════════════════════════════════════
# DOCUMENTATION
# ═══════════════════════════════════════════════════════════════════════════════

# Generate all documentation
docs:
    @mkdir -p docs/generated docs/man
    cargo doc --no-deps
    just cookbook
    just man
    @echo "Documentation generated"

# Generate justfile cookbook
cookbook:
    #!/usr/bin/env bash
    mkdir -p docs
    OUTPUT="docs/just-cookbook.adoc"
    echo "= {{project}} Justfile Cookbook" > "$OUTPUT"
    echo ":toc: left" >> "$OUTPUT"
    echo ":toclevels: 3" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "Generated: $(date -Iseconds)" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    just --list --unsorted >> "$OUTPUT"
    echo "Generated: $OUTPUT"

# Generate man page
man:
    #!/usr/bin/env bash
    mkdir -p docs/man
    cat > docs/man/echidnabot.1 << 'EOF'
    .TH ECHIDNABOT 1 "2025" "{{version}}" "echidnabot Manual"
    .SH NAME
    echidnabot \- proof-aware CI bot for formal verification
    .SH SYNOPSIS
    .B echidnabot
    [COMMAND] [OPTIONS]
    .SH DESCRIPTION
    echidnabot automatically verifies mathematical theorems in your codebase.
    Integrates with GitHub/GitLab/Bitbucket to run formal verification on every
    push and PR using ECHIDNA's multi-prover backend.
    .SH COMMANDS
    .TP
    .B serve
    Start the webhook server
    .TP
    .B register
    Register a repository for verification
    .TP
    .B check
    Manually trigger proof verification
    .TP
    .B status
    Show verification status
    .SH AUTHOR
    hyperpolymath <hyperpolymath@proton.me>
    .SH SEE ALSO
    echidna(1), coq(1), lean(1)
    EOF
    echo "Generated: docs/man/echidnabot.1"

# Open docs in browser
docs-open:
    cargo doc --no-deps --open

# ═══════════════════════════════════════════════════════════════════════════════
# CONTAINER (Podman/nerdctl)
# ═══════════════════════════════════════════════════════════════════════════════

# Build container image
container-build tag="latest":
    podman build -t echidnabot:{{tag}} -f Containerfile .

# Run container
container-run tag="latest" *args:
    podman run --rm -it -p 8080:8080 echidnabot:{{tag}} {{args}}

# Push to registry
container-push registry="ghcr.io/hyperpolymath" tag="latest":
    podman tag echidnabot:{{tag}} {{registry}}/echidnabot:{{tag}}
    podman push {{registry}}/echidnabot:{{tag}}

# Scan container for vulnerabilities
container-scan tag="latest":
    trivy image echidnabot:{{tag}}

# ═══════════════════════════════════════════════════════════════════════════════
# CI & AUTOMATION
# ═══════════════════════════════════════════════════════════════════════════════

# Run full CI pipeline locally
ci: quality
    @echo "CI pipeline complete!"

# Install git hooks
install-hooks:
    @mkdir -p .git/hooks
    @cat > .git/hooks/pre-commit << 'EOF'
    #!/bin/bash
    just fmt-check || exit 1
    just lint || exit 1
    EOF
    @chmod +x .git/hooks/pre-commit
    @echo "Git hooks installed"

# ═══════════════════════════════════════════════════════════════════════════════
# SECURITY
# ═══════════════════════════════════════════════════════════════════════════════

# Run security audit
security:
    @echo "=== Security Audit ==="
    cargo audit
    @command -v gitleaks >/dev/null && gitleaks detect --source . --verbose || true
    @echo "Security audit complete"

# Generate SBOM
sbom:
    @mkdir -p docs/security
    cargo sbom > docs/security/sbom.spdx.json 2>/dev/null || echo "cargo-sbom not installed"

# ═══════════════════════════════════════════════════════════════════════════════
# DEPENDENCIES
# ═══════════════════════════════════════════════════════════════════════════════

# Update dependencies
deps:
    cargo update

# Check for outdated deps
deps-outdated:
    cargo outdated

# Audit dependencies
deps-audit:
    cargo audit

# ═══════════════════════════════════════════════════════════════════════════════
# GUIX & NIX
# ═══════════════════════════════════════════════════════════════════════════════

# Enter Guix development shell (primary)
guix-shell:
    guix shell -D -f guix.scm

# Build with Guix
guix-build:
    guix build -f guix.scm

# Enter Nix development shell (fallback)
nix-shell:
    @if [ -f "flake.nix" ]; then nix develop; else echo "No flake.nix"; fi

# ═══════════════════════════════════════════════════════════════════════════════
# VALIDATION & COMPLIANCE
# ═══════════════════════════════════════════════════════════════════════════════

# Validate RSR compliance
validate-rsr:
    #!/usr/bin/env bash
    echo "=== RSR Compliance Check ==="
    MISSING=""
    for f in justfile README.adoc; do
        [ -f "$f" ] || MISSING="$MISSING $f"
    done
    for d in .well-known; do
        [ -d "$d" ] || MISSING="$MISSING $d/"
    done
    for f in .well-known/security.txt; do
        [ -f "$f" ] || MISSING="$MISSING $f"
    done
    if [ ! -f "guix.scm" ] && [ ! -f "flake.nix" ]; then
        MISSING="$MISSING guix.scm/flake.nix"
    fi
    if [ -n "$MISSING" ]; then
        echo "MISSING:$MISSING"
        exit 1
    fi
    echo "RSR compliance: PASS"

# Validate STATE.scm syntax
validate-state:
    @if [ -f "STATE.scm" ]; then \
        guile -c "(primitive-load \"STATE.scm\")" 2>/dev/null && echo "STATE.scm: valid" || echo "STATE.scm: INVALID"; \
    fi

# Full validation
validate: validate-rsr validate-state
    @echo "All validations passed!"

# ═══════════════════════════════════════════════════════════════════════════════
# STATE MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════════════

# Update STATE.scm timestamp
state-touch:
    @if [ -f "STATE.scm" ]; then \
        sed -i 's/(updated . "[^"]*")/(updated . "'"$(date -Iseconds)"'")/' STATE.scm && \
        echo "STATE.scm timestamp updated"; \
    fi

# Show current phase
state-phase:
    @grep -oP '\(phase\s+\.\s+\K[^)]+' STATE.scm 2>/dev/null | head -1 || echo "unknown"

# ═══════════════════════════════════════════════════════════════════════════════
# RELEASE
# ═══════════════════════════════════════════════════════════════════════════════

# Create release build with all checks
release: quality build-release
    strip target/release/echidnabot
    @echo "Release build ready: target/release/echidnabot"

# Publish to crates.io (dry run)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# ═══════════════════════════════════════════════════════════════════════════════
# COMBINATORIC MATRIX RECIPES
# ═══════════════════════════════════════════════════════════════════════════════

# Build matrix: [debug|release] × [features]
build-matrix mode="debug" features="":
    @echo "Build: mode={{mode}} features={{features}}"
    @if [ "{{mode}}" = "release" ]; then cargo build --release; else cargo build; fi

# Test matrix: [unit|integration|all] × [verbosity]
test-matrix suite="all" verbose="false":
    @echo "Test: suite={{suite}} verbose={{verbose}}"
    @if [ "{{verbose}}" = "true" ]; then cargo test -- --nocapture; else cargo test; fi

# Show all matrix combinations
combinations:
    @echo "=== Combinatoric Matrix Recipes ==="
    @echo ""
    @echo "Build:  just build-matrix [debug|release] [features]"
    @echo "Test:   just test-matrix [unit|integration|all] [true|false]"
    @echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# ECHIDNA SMART CONTRACT FUZZING
# ═══════════════════════════════════════════════════════════════════════════════

# Default Solidity and Echidna versions
solc_version := "0.8.19"
echidna_version := "2.2.3"

# Install Echidna (Linux x86_64)
echidna-install:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v echidna &>/dev/null; then
        echo "Echidna already installed: $(echidna --version)"
    else
        echo "Installing Echidna {{echidna_version}}..."
        wget -q "https://github.com/crytic/echidna/releases/download/v{{echidna_version}}/echidna-{{echidna_version}}-x86_64-linux.tar.gz"
        tar -xzf "echidna-{{echidna_version}}-x86_64-linux.tar.gz"
        sudo mv echidna /usr/local/bin/
        rm "echidna-{{echidna_version}}-x86_64-linux.tar.gz"
        echo "Installed: $(echidna --version)"
    fi

# Install solc compiler
solc-install:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v solc &>/dev/null; then
        echo "solc already installed: $(solc --version | head -2 | tail -1)"
    else
        echo "Installing solc {{solc_version}}..."
        wget -q "https://github.com/ethereum/solidity/releases/download/v{{solc_version}}/solc-static-linux"
        chmod +x solc-static-linux
        sudo mv solc-static-linux /usr/local/bin/solc
        echo "Installed: $(solc --version | head -2 | tail -1)"
    fi

# Run Echidna property tests on a contract
echidna-test contract="TokenEchidnaTest":
    echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-config.yaml

# Run Echidna with CI configuration (faster)
echidna-ci contract="TokenEchidnaTest":
    echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-ci.yaml --format json

# Run Echidna assertion tests
echidna-assertion contract="TokenEchidnaTest":
    echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-assertion.yaml

# Generate Echidna test contract from source
echidna-gen contract:
    deno run --allow-read --allow-write scripts/echidna-gen.js contracts/{{contract}}.sol

# Run all Echidna tests (property + assertion)
echidna-all contract="TokenEchidnaTest":
    @echo "=== Property Tests ==="
    just echidna-test {{contract}}
    @echo ""
    @echo "=== Assertion Tests ==="
    just echidna-assertion {{contract}}

# Check Echidna installation and dependencies
echidna-check:
    @echo "Checking Echidna dependencies..."
    @command -v solc &>/dev/null && echo "✓ solc: $(solc --version | head -2 | tail -1)" || echo "✗ solc: not installed (run: just solc-install)"
    @command -v echidna &>/dev/null && echo "✓ echidna: $(echidna --version)" || echo "✗ echidna: not installed (run: just echidna-install)"
    @command -v deno &>/dev/null && echo "✓ deno: $(deno --version | head -1)" || echo "✗ deno: not installed"

# ═══════════════════════════════════════════════════════════════════════════════
# ECHIDNA SAFEGUARDS (Rate Limits & No-Flaky Mode)
# ═══════════════════════════════════════════════════════════════════════════════

# Default safeguard settings
default_timeout := "600"
default_verification_passes := "3"

# Run Echidna in no-flaky mode (deterministic, multi-pass verification)
echidna-no-flaky contract="TokenEchidnaTest" seed="42":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== No-Flaky Mode: {{contract}} ==="
    echo "Seed: {{seed}}"
    echo "Verification passes: {{default_verification_passes}}"
    echo ""

    PASS_COUNT=0
    FAIL_COUNT=0

    for i in $(seq 1 {{default_verification_passes}}); do
        echo "--- Pass $i/{{default_verification_passes}} ---"
        PASS_SEED=$(({{seed}} + i))

        if timeout {{default_timeout}} echidna contracts/{{contract}}.sol \
            --contract {{contract}} \
            --config echidna/echidna-no-flaky.yaml \
            --seed $PASS_SEED \
            --format text 2>&1; then
            PASS_COUNT=$((PASS_COUNT + 1))
            echo "Pass $i: SUCCESS"
        else
            FAIL_COUNT=$((FAIL_COUNT + 1))
            echo "Pass $i: FAILURE"
        fi
        echo ""
    done

    echo "=== Verification Summary ==="
    echo "Passed: $PASS_COUNT / {{default_verification_passes}}"
    echo "Failed: $FAIL_COUNT / {{default_verification_passes}}"

    if [ $FAIL_COUNT -gt 0 ]; then
        echo ""
        echo "ERROR: Test is FLAKY - results differ between runs"
        exit 1
    else
        echo ""
        echo "SUCCESS: Test is STABLE - all passes agree"
    fi

# Run Echidna with timeout (rate-limited)
echidna-timeout contract="TokenEchidnaTest" timeout="300":
    timeout {{timeout}} echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-ci.yaml || \
        ([ $? -eq 124 ] && echo "ERROR: Test timed out after {{timeout}}s" && exit 1)

# Run Echidna with fixed seed for reproducibility
echidna-seed contract="TokenEchidnaTest" seed="12345":
    echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-config.yaml --seed {{seed}}

# Verify test stability (quick check)
echidna-verify-stable contract="TokenEchidnaTest":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Quick stability check for {{contract}}..."

    # Run twice with different seeds
    RESULT1=$(timeout 120 echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-ci.yaml --seed 42 --format json 2>&1 || echo "FAIL")
    RESULT2=$(timeout 120 echidna contracts/{{contract}}.sol --contract {{contract}} --config echidna/echidna-ci.yaml --seed 43 --format json 2>&1 || echo "FAIL")

    if echo "$RESULT1" | grep -q "FAIL" && echo "$RESULT2" | grep -q "FAIL"; then
        echo "Both runs failed - test has consistent failures"
    elif echo "$RESULT1" | grep -q "passed" && echo "$RESULT2" | grep -q "passed"; then
        echo "Both runs passed - test appears stable"
    else
        echo "WARNING: Inconsistent results - test may be flaky!"
        exit 1
    fi

# Show current safeguard configuration
echidna-safeguards:
    @echo "=== Echidna Safeguard Configuration ==="
    @echo ""
    @echo "Rate Limits:"
    @echo "  - Default timeout: {{default_timeout}}s"
    @echo "  - Max parallel contracts (CI): 3"
    @echo "  - Max concurrent jobs (CI): 2"
    @echo ""
    @echo "No-Flaky Mode:"
    @echo "  - Verification passes: {{default_verification_passes}}"
    @echo "  - Uses deterministic config: echidna/echidna-no-flaky.yaml"
    @echo "  - Single worker for reproducibility"
    @echo "  - Fixed sender addresses"
    @echo ""
    @echo "Commands:"
    @echo "  just echidna-no-flaky [contract] [seed]  # Multi-pass verification"
    @echo "  just echidna-timeout [contract] [secs]   # Rate-limited run"
    @echo "  just echidna-seed [contract] [seed]      # Reproducible run"
    @echo "  just echidna-verify-stable [contract]    # Quick stability check"

# ═══════════════════════════════════════════════════════════════════════════════
# UTILITIES
# ═══════════════════════════════════════════════════════════════════════════════

# Count lines of code
loc:
    @find src -name "*.rs" | xargs wc -l 2>/dev/null | tail -1 || echo "0"

# Show TODO comments
todos:
    @grep -rn "TODO\|FIXME\|XXX" src/ 2>/dev/null || echo "No TODOs"

# Open in editor
edit:
    ${EDITOR:-code} .

# Git status
status:
    @git status --short

# Recent commits
log count="10":
    @git log --oneline -{{count}}
