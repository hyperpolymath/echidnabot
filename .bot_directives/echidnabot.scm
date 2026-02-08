;; SPDX-License-Identifier: PMPL-1.0-or-later
(bot-directive
  (bot "echidnabot")
  (scope "formal verification, fuzzing, proof checking")
  (allow ("self-analysis" "proof validation" "fuzzing" "solver integrity checks"))
  (deny ("self-modification without approval"))
  (notes "Self-scan for proof correctness; code changes require explicit approval"))
