;; SPDX-License-Identifier: PMPL-1.0-or-later
(bot-directive
  (bot "accessibilitybot")
  (scope "WCAG 2.3 AAA accessibility compliance")
  (allow ("accessibility auditing" "ARIA suggestions" "docs updates"))
  (deny ("logic changes" "core code modification"))
  (notes "Focus on WCAG conformance, color contrast, keyboard navigation"))
