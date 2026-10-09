;; SPDX-License-Identifier: MPL-2.0
;; Explicit development-environment channel, not an echidnabot package channel.
;; GitHub mirror revision inspected on 2026-10-09. Bump deliberately.
(use-modules (guix channels))

(list (channel
       (name 'guix)
       (url "https://github.com/guix-mirror/guix")
       (commit "927070a6f64da86a9c1694fe55c4316b2f9a4e11")
       (introduction
        (make-channel-introduction
         "9edb3f66fd807b096b48283debdcddccfea34bad"
         (openpgp-fingerprint
          "BBB0 2DDF 2CEA F6A8 0D1D E643 A2A0 6DF2 A33A 54FA")))))
