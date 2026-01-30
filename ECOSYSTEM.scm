;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Ecosystem relationships for echidnabot
;; Media type: application/vnd.ecosystem+scm

(ecosystem
  (metadata
    ((version . "1.0.0")
     (name . "echidnabot")
     (type . "git-automation")
     (purpose . "Part of hyperpolymath tool ecosystem")))
  
  (position-in-ecosystem
    "Provides git-automation functionality within the hyperpolymath suite")
  
  (related-projects
    ((vext . "sibling-tool")
     (hypatia . "potential-consumer")))
  
  (what-this-is
    "echidnabot is a specialized tool in the hyperpolymath ecosystem")
  
  (what-this-is-not
    "Not a general-purpose framework"
    "Not intended as standalone product"))
