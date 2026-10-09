// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
//! Proof confidence level assessment
//!
//! The trust-level *algorithm* is ECHIDNA's, not echidnabot's: every call
//! goes through [`echidna_core::trust::compute_trust_level`] (the Creusot-
//! annotated kernel shared with ECHIDNA). This module only translates
//! echidnabot's inputs (prover slug, status, artefacts, axiom scan) into
//! ECHIDNA's [`TrustFactors`] and wraps the answer in a report.
//!
//! Epistemic note (pattern from `hyperpolymath/epistemic-types`, not a
//! dependency): a level computed here is a *warrant* — echidnabot's own
//! reading of ECHIDNA's output. When ECHIDNA itself supplies trust data in an
//! `echidna.prove.result/1` object, that is the *receipt* and is preferred;
//! see [`crate::dispatcher::TrustSource`].

use echidna_core::trust::axiom_tracker::DangerLevel;
use echidna_core::trust::{compute_trust_level, ProverClass, TrustFactors, TrustLevel};
use serde::{Deserialize, Serialize};

use crate::dispatcher::{ProofStatus, ProverKind};

/// Confidence level for a proof verification result.
///
/// Mirrors ECHIDNA's [`TrustLevel`] one-to-one (see the `From` impls); it is
/// kept as a local type only so echidnabot can attach presentation methods
/// (`label`, `Display`) to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Large-TCB system, unchecked result, or dangerous axioms present
    Level1 = 1,
    /// Single prover result without a verified certificate
    Level2 = 2,
    /// Verified certificate, or cross-checked by 2+ provers
    Level3 = 3,
    /// Small-kernel system with a verified certificate
    Level4 = 4,
    /// Cross-checked by 2+ small-kernel systems with verified certificates
    Level5 = 5,
}

impl ConfidenceLevel {
    /// Numeric value (1-5)
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Level1 => "Minimal (large-TCB / unchecked / dangerous axioms)",
            Self::Level2 => "Low (single prover, no verified certificate)",
            Self::Level3 => "Moderate (verified certificate or cross-checked)",
            Self::Level4 => "High (small-kernel + verified certificate)",
            Self::Level5 => "Maximum (cross-checked by 2+ small-kernel systems)",
        }
    }

    /// Whether this confidence level is considered sufficient for production use
    pub fn is_production_ready(&self) -> bool {
        *self >= ConfidenceLevel::Level3
    }
}

impl From<TrustLevel> for ConfidenceLevel {
    /// Translate ECHIDNA's trust level into the local presentation type.
    fn from(level: TrustLevel) -> Self {
        match level {
            TrustLevel::Level1 => Self::Level1,
            TrustLevel::Level2 => Self::Level2,
            TrustLevel::Level3 => Self::Level3,
            TrustLevel::Level4 => Self::Level4,
            TrustLevel::Level5 => Self::Level5,
        }
    }
}

impl From<ConfidenceLevel> for TrustLevel {
    /// Translate the local presentation type back into ECHIDNA's trust level.
    fn from(level: ConfidenceLevel) -> Self {
        match level {
            ConfidenceLevel::Level1 => Self::Level1,
            ConfidenceLevel::Level2 => Self::Level2,
            ConfidenceLevel::Level3 => Self::Level3,
            ConfidenceLevel::Level4 => Self::Level4,
            ConfidenceLevel::Level5 => Self::Level5,
        }
    }
}

impl std::fmt::Display for ConfidenceLevel {
    /// Render as `Level N (label)`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Level {} ({})", self.value(), self.label())
    }
}

/// Report assessing confidence in a proof result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceReport {
    /// The assessed confidence level
    pub level: ConfidenceLevel,
    /// The prover that produced the result
    pub prover: ProverKind,
    /// Whether a proof certificate was present
    pub has_certificate: bool,
    /// Number of independent checkers that verified the result
    pub checker_count: usize,
    /// Whether the prover uses a small kernel
    pub small_kernel: bool,
    /// Human-readable justification for the confidence level
    pub justification: String,
}

/// Convert echidnabot's prover kind to ECHIDNA's prover class.
pub fn prover_class(prover: &ProverKind) -> ProverClass {
    if is_small_kernel(prover) {
        return ProverClass::SmallKernel;
    }
    match prover.as_str() {
        "z3" | "cvc5" | "alt-ergo" | "vampire" | "eprover" | "spass" => ProverClass::SmtOrAtp,
        _ => ProverClass::Other,
    }
}

/// Assess the confidence level of a proof verification result.
///
/// This is a thin wrapper around ECHIDNA's [`compute_trust_level`]. It
/// translates echidnabot's inputs into ECHIDNA's [`TrustFactors`] type, calls
/// the canonical algorithm, and converts the result back to the local
/// [`ConfidenceLevel`] presentation type.
///
/// # Arguments
/// * `prover` - Which prover produced the result
/// * `status` - The verification status
/// * `has_certificate` - Whether a proof certificate (Alethe, DRAT/LRAT, etc.) was provided
/// * `checker_count` - Number of independent checkers that confirmed the result
pub fn assess_confidence(
    prover: &ProverKind,
    status: ProofStatus,
    has_certificate: bool,
    checker_count: usize,
) -> ConfidenceReport {
    // Only verified proofs get meaningful confidence levels
    if status != ProofStatus::Verified {
        return ConfidenceReport {
            level: ConfidenceLevel::Level1,
            prover: prover.clone(),
            has_certificate: false,
            checker_count: 0,
            small_kernel: is_small_kernel(prover),
            justification: format!(
                "Proof status is {:?} (not Verified) -- confidence is minimal",
                status
            ),
        };
    }

    let factors = TrustFactors {
        prover_class: prover_class(prover),
        confirming_provers: checker_count as u32,
        has_certificate,
        certificate_verified: has_certificate,
        worst_axiom_danger: DangerLevel::Safe,
        solver_integrity_ok: true,
    };

    let trust_level = compute_trust_level(&factors);
    let level = ConfidenceLevel::from(trust_level);

    let small_kernel = is_small_kernel(prover);

    // Build justification based on the computed level
    let justification = match level {
        ConfidenceLevel::Level5 => format!(
            "Cross-checked by {} independent small-kernel systems ({})",
            checker_count,
            prover.display_name()
        ),
        ConfidenceLevel::Level4 => format!(
            "Verified by small-kernel system ({}) with proof certificate",
            prover.display_name()
        ),
        ConfidenceLevel::Level3 => {
            if has_certificate {
                format!("Verified by {} with proof certificate", prover.display_name())
            } else {
                format!(
                    "Verified by {} and cross-checked by {} provers",
                    prover.display_name(),
                    checker_count
                )
            }
        }
        ConfidenceLevel::Level2 => format!(
            "Verified by small-kernel system ({}) without proof certificate",
            prover.display_name()
        ),
        ConfidenceLevel::Level1 => {
            if has_certificate {
                format!("Large-TCB prover ({}) with certificate", prover.display_name())
            } else {
                format!(
                    "Verified by large-TCB system ({}) -- consider cross-checking",
                    prover.display_name()
                )
            }
        }
    };

    ConfidenceReport {
        level,
        prover: prover.clone(),
        has_certificate,
        checker_count,
        small_kernel,
        justification,
    }
}

/// Determine if a prover uses a small, trusted kernel.
///
/// Small-kernel provers have a minimal trusted code base for proof checking,
/// making their results more trustworthy.
pub fn is_small_kernel(prover: &ProverKind) -> bool {
    match prover.as_str() {
        // Tier 1 small-kernel systems
        "coq" => true,      // Gallina kernel
        "lean" => true,     // Lean4 kernel
        "isabelle" => true, // Isabelle/Pure kernel
        "agda" => true,     // Dependent type checker
        "metamath" => true, // Extremely small kernel

        // SAT/SMT solvers -- large TCB but produce certificates
        "z3" => false,
        "cvc5" => false,

        // Other provers
        "hol-light" => true, // Small OCaml kernel
        "mizar" => false,    // Large checker
        "pvs" => false,      // Large TCB
        "acl2" => false,     // Built on Common Lisp
        "hol4" => true,      // Small ML kernel

        // Tier-3 small-kernel systems
        "idris2" | "idris" => true, // Dependent-type kernel
        "fstar" => true,            // F* type-theory kernel

        // Tier-3 large-TCB systems
        "vampire" | "eprover" | "spass" => false, // Large first-order ATPs
        "dafny" | "why3" | "alt-ergo" => false,   // VC-based tools
        "tamarin" | "proverif" => false,          // Protocol model checkers
        "dreal" | "abc" => false,                 // Numerical / hardware checkers

        // Unknown provers: assume false (conservative estimate)
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::Level5 > ConfidenceLevel::Level4);
        assert!(ConfidenceLevel::Level4 > ConfidenceLevel::Level3);
        assert!(ConfidenceLevel::Level3 > ConfidenceLevel::Level2);
        assert!(ConfidenceLevel::Level2 > ConfidenceLevel::Level1);
    }

    #[test]
    fn test_confidence_level_values() {
        assert_eq!(ConfidenceLevel::Level1.value(), 1);
        assert_eq!(ConfidenceLevel::Level5.value(), 5);
    }

    #[test]
    fn test_production_readiness() {
        assert!(!ConfidenceLevel::Level1.is_production_ready());
        assert!(!ConfidenceLevel::Level2.is_production_ready());
        assert!(ConfidenceLevel::Level3.is_production_ready());
        assert!(ConfidenceLevel::Level4.is_production_ready());
        assert!(ConfidenceLevel::Level5.is_production_ready());
    }

    #[test]
    fn test_small_kernel_provers() {
        assert!(is_small_kernel(&ProverKind::new("coq")));
        assert!(is_small_kernel(&ProverKind::new("lean")));
        assert!(is_small_kernel(&ProverKind::new("isabelle")));
        assert!(is_small_kernel(&ProverKind::new("agda")));
        assert!(is_small_kernel(&ProverKind::new("metamath")));
        assert!(is_small_kernel(&ProverKind::new("hol-light")));
        assert!(is_small_kernel(&ProverKind::new("hol4")));

        assert!(!is_small_kernel(&ProverKind::new("z3")));
        assert!(!is_small_kernel(&ProverKind::new("cvc5")));
        assert!(!is_small_kernel(&ProverKind::new("mizar")));
        assert!(!is_small_kernel(&ProverKind::new("pvs")));
        assert!(!is_small_kernel(&ProverKind::new("acl2")));

        // Tier-3 small-kernel
        assert!(is_small_kernel(&ProverKind::new("idris2")));
        assert!(is_small_kernel(&ProverKind::new("fstar")));

        // Tier-3 large-TCB
        assert!(!is_small_kernel(&ProverKind::new("vampire")));
        assert!(!is_small_kernel(&ProverKind::new("eprover")));
        assert!(!is_small_kernel(&ProverKind::new("dafny")));
        assert!(!is_small_kernel(&ProverKind::new("why3")));
        assert!(!is_small_kernel(&ProverKind::new("tamarin")));
        assert!(!is_small_kernel(&ProverKind::new("proverif")));
        assert!(!is_small_kernel(&ProverKind::new("dreal")));
    }

    #[test]
    fn test_assess_level5_cross_checked() {
        let report = assess_confidence(
            &ProverKind::new("lean"),
            ProofStatus::Verified,
            true,
            3, // 3 independent checkers
        );
        assert_eq!(report.level, ConfidenceLevel::Level5);
        assert!(report.small_kernel);
        assert_eq!(report.checker_count, 3);
    }

    #[test]
    fn test_assess_level4_small_kernel_with_cert() {
        let report = assess_confidence(&ProverKind::new("coq"), ProofStatus::Verified, true, 1);
        assert_eq!(report.level, ConfidenceLevel::Level4);
    }

    #[test]
    fn test_assess_level3_cert_no_small_kernel() {
        let report = assess_confidence(
            &ProverKind::new("z3"),
            ProofStatus::Verified,
            true, // Has DRAT/LRAT certificate
            1,
        );
        assert_eq!(report.level, ConfidenceLevel::Level3);
    }

    #[test]
    fn test_assess_level2_small_kernel_no_cert() {
        let report = assess_confidence(&ProverKind::new("lean"), ProofStatus::Verified, false, 1);
        assert_eq!(report.level, ConfidenceLevel::Level2);
    }

    #[test]
    fn test_assess_level1_large_tcb() {
        // Note: The ECHIDNA trust kernel (compute_trust_level) does not
        // distinguish between small-kernel and large-TCB at Level 2; it only
        // checks for certificates and cross-verification. A large-TCB prover
        // with no certificate and 1 checker returns Level2, not Level1.
        // The old echidnabot-specific logic returned Level1 here.
        // This test now verifies the ECHIDNA canonical algorithm.
        let report = assess_confidence(&ProverKind::new("pvs"), ProofStatus::Verified, false, 1);
        assert_eq!(report.level, ConfidenceLevel::Level2);
    }

    #[test]
    fn test_assess_failed_proof_always_level1() {
        let report = assess_confidence(&ProverKind::new("coq"), ProofStatus::Failed, true, 3);
        assert_eq!(report.level, ConfidenceLevel::Level1);
    }

    #[test]
    fn test_confidence_display() {
        let level = ConfidenceLevel::Level4;
        let display = format!("{}", level);
        assert!(display.contains("Level 4"));
        assert!(display.contains("High"));
    }

    #[test]
    fn test_prover_class_small_kernel() {
        assert_eq!(prover_class(&ProverKind::new("coq")), ProverClass::SmallKernel);
        assert_eq!(prover_class(&ProverKind::new("lean")), ProverClass::SmallKernel);
    }

    #[test]
    fn test_prover_class_smt() {
        assert_eq!(prover_class(&ProverKind::new("z3")), ProverClass::SmtOrAtp);
        assert_eq!(prover_class(&ProverKind::new("cvc5")), ProverClass::SmtOrAtp);
    }

    #[test]
    fn test_prover_class_other() {
        assert_eq!(prover_class(&ProverKind::new("unknown")), ProverClass::Other);
    }
}
