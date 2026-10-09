;;
;; echidnabot - Guix Development Environment
;;
;; Usage:
;;   # Enter development environment with all dependencies
;;   guix shell -D -f guix.scm
;;
;;   # Build the package
;;   guix build -f guix.scm
;;
;; This file pins the Rust toolchain and essential crates from Guix's
;; package definitions. Git dependencies (like echidna-core) are NOT
;; pinned here because Guix cannot fetch from git repositories directly.
;; They are resolved by Cargo when you run `cargo build`.

(use-modules (guix packages)
             (guix gexp)
             (guix git-download)
             (guix build-system cargo)
             ((guix licenses) #:prefix license:)
             (gnu packages base)
             (gnu packages crates-io)
             (gnu packages rust)
             (gnu packages rust-apps)
             (gnu packages sqlite)
             (gnu packages tls)
             (gnu packages pkg-config)
             (gnu packages version-control))

;; Development shell with all native tools
(define-public echidnabot-dev
  (package
    (name "echidnabot-dev")
    (version "0.1.0")
    (source (local-file "."))
    (build-system cargo-build-system)
    (arguments
     `(;; Cargo build flags
       #:cargo-build-flags '("--release")
       #:phases
       (modify-phases %standard-phases
         (delete 'configure)  ; No configure phase for Rust
         (add-before 'build 'set-env
           (lambda _
             (setenv "RUST_BACKTRACE" "full")
             #t)))))
    (native-inputs
     (list
      ;; Rust toolchain
      rust
      cargo
      pkg-config
      
      ;; Git for git dependencies
      git
      
      ;; Build tools
      make
      cmake))
    (inputs
     (list
      ;; Runtime dependencies
      sqlite
      openssl))
    (synopsis "Proof-aware CI bot development environment")
    (description
     "Development environment for echidnabot. Includes the Rust toolchain,
Cargo, git, and system libraries needed for building echidnabot.
Git dependencies (like echidna-core) are fetched by Cargo at build time.")
    (home-page "https://github.com/hyperpolymath/echidnabot")
    (license license:mpl2.0)))

;; For building the production binary
(define-public echidnabot
  (package
    (inherit echidnabot-dev)
    (name "echidnabot")
    (arguments
     (substitute-keyword-arguments (package-arguments echidnabot-dev)
       ((#:phases phases)
        `(modify-phases ,phases
           (delete 'check)))))  ; Skip tests for production build
    (native-inputs
     (alist-delete 'git (package-native-inputs echidnabot-dev)))
    (synopsis "Proof-aware CI bot for theorem prover repositories")
    (description
     "echidnabot monitors code repositories containing formal proofs and
delegates verification to ECHIDNA Core. It integrates with GitHub, GitLab,
and Bitbucket to provide automated proof checking via webhooks.")))

;; Export the development environment as the default
 echidnabot-dev
