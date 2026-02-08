;; SPDX-License-Identifier: PMPL-1.0-or-later
(bot-directive
  (bot "cipherbot")
  (scope "cryptographic hygiene and post-quantum readiness")
  (allow ("crypto analysis" "algorithm strength auditing" "pq-readiness checks"))
  (deny ("key material access" "credential handling" "core logic changes"))
  (notes "Specialist tier; audits crypto usage, never handles actual secrets"))
