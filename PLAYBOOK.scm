; SPDX-License-Identifier: PMPL-1.0-or-later
; PLAYBOOK.scm - Operational playbook for echidnabot

(playbook
  (metadata
    (version "1.0.0")
    (created "2026-01-30"))
  (quick-start
    (prerequisites "Rust 1.70+" "cargo")
    (steps
      (step 1 "Clone" "git clone https://github.com/hyperpolymath/echidnabot")
      (step 2 "Build" "cargo build --release")
      (step 3 "Test" "cargo test")))
  (common-tasks
    (development
      (task "Test" (command "cargo test"))
      (task "Lint" (command "cargo clippy"))))
  (maintenance
    (monthly (task "Update deps" "cargo update"))))
