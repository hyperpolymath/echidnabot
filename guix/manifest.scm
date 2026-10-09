;; SPDX-License-Identifier: MPL-2.0
;; Native build prerequisites. Rust/Just versions remain owned by mise and
;; rust-toolchain.toml, not by the Guix channel's Rust package version.
(use-modules (guix profiles))

(specifications->manifest
 '("bash" "coreutils" "findutils" "diffutils" "grep" "sed"
   "tar" "gzip" "xz" "unzip" "which"
   "git" "curl" "nss-certs"
   "gcc-toolchain" "make" "cmake" "pkg-config" "sqlite" "zlib"))
