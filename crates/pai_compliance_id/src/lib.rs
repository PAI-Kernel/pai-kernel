//! # PAI Compliance Identity (G-1) & Certification Nomenclature (H-2)
//!
//! **Constitutional reference:** PAI-CD v3.1, Amendments G-1, H-2
//!
//! ## G-1: Compliance Identity
//!
//! Every PAI system must declare its compliance level:
//! - **PAI-compliant**: Implements v2.2 core invariants
//! - **PAI-native**: Built on PAI-CD runtime (SDK)
//! - **PAI-governed**: Full governance daemon with all invariants enforced
//!
//! ## H-2: Certification Nomenclature
//!
//! Three certification tiers with anti-capture provisions:
//! - **Tier A**: Full formal verification + runtime compliance
//! - **Tier B**: Runtime compliance + test suite coverage
//! - **Tier C**: Self-assessment against checklist
//!
//! Anti-capture: No single entity may certify >20% of active systems.
//!
//! ## Invariants
//!
//! | ID     | Invariant                                                       | Ref   |
//! |--------|-----------------------------------------------------------------|-------|
//! | CID-I1 | Compliance level must be declared, not inferred                  | G-1   |
//! | CID-I2 | Higher levels require all lower-level capabilities               | G-1   |
//! | CID-I3 | Certification tier must match actual verification evidence        | H-2   |
//! | CID-I4 | Anti-capture: certifier concentration check                      | H-2   |
//! | CID-I5 | Compliance identity is part of export bundle                     | G-1+P4|

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// G-1: Compliance Identity
// ---------------------------------------------------------------------------

/// PAI-CD compliance level (G-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Implements v2.2 core invariants (may be external implementation).
    PaiCompliant,
    /// Built on PAI-CD runtime SDK.
    PaiNative,
    /// Full governance daemon with all invariants enforced at runtime.
    PaiGoverned,
}

impl ComplianceLevel {
    pub fn label(&self) -> &'static str {
        match self {
            ComplianceLevel::PaiCompliant => "PAI-compliant",
            ComplianceLevel::PaiNative => "PAI-native",
            ComplianceLevel::PaiGoverned => "PAI-governed",
        }
    }
}

/// Capabilities required for each compliance level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCapabilities {
    /// v2.2 core invariants implemented (P1-P8 + Risk Tiers + Consent + etc.)
    pub core_invariants: bool,
    /// PAI-CD SDK runtime present.
    pub sdk_runtime: bool,
    /// Governance daemon active with decision log, witness chain, drift monitoring.
    pub governance_daemon: bool,
    /// Formal verification evidence (TLA+, SPARK).
    pub formal_verification: bool,
    /// Test suite coverage above threshold.
    pub test_coverage: bool,
}

impl ComplianceCapabilities {
    /// Determine the highest compliance level supported by these capabilities (CID-I2).
    pub fn assess_level(&self) -> ComplianceLevel {
        if self.core_invariants && self.sdk_runtime && self.governance_daemon {
            ComplianceLevel::PaiGoverned
        } else if self.core_invariants && self.sdk_runtime {
            ComplianceLevel::PaiNative
        } else if self.core_invariants {
            ComplianceLevel::PaiCompliant
        } else {
            // No core invariants = not even PAI-compliant.
            // We return PaiCompliant as floor but with a note.
            ComplianceLevel::PaiCompliant
        }
    }
}

/// Compliance identity declaration (G-1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIdentity {
    /// Declared compliance level.
    pub level: ComplianceLevel,
    /// PAI-CD corpus version claimed.
    pub corpus_version: String,
    /// SDK version (if PaiNative or PaiGoverned).
    pub sdk_version: Option<String>,
    /// Capabilities self-assessment.
    pub capabilities: ComplianceCapabilities,
    /// Whether the declared level matches the assessed level (CID-I1 + CID-I2).
    pub declaration_valid: bool,
}

impl ComplianceIdentity {
    /// Create a new identity declaration and validate it.
    pub fn declare(
        level: ComplianceLevel,
        corpus_version: &str,
        sdk_version: Option<&str>,
        capabilities: ComplianceCapabilities,
    ) -> Self {
        let assessed = capabilities.assess_level();
        // CID-I1: explicit declaration. CID-I2: cannot claim higher than capabilities support.
        let declaration_valid = level <= assessed;

        ComplianceIdentity {
            level,
            corpus_version: corpus_version.into(),
            sdk_version: sdk_version.map(|s| s.into()),
            capabilities,
            declaration_valid,
        }
    }

    /// Produce a machine-readable identity document.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// H-2: Certification Nomenclature
// ---------------------------------------------------------------------------

/// Certification tier (H-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CertificationTier {
    /// Self-assessment against checklist.
    TierC,
    /// Runtime compliance + test suite coverage.
    TierB,
    /// Full formal verification + runtime compliance.
    TierA,
}

impl CertificationTier {
    pub fn label(&self) -> &'static str {
        match self {
            CertificationTier::TierC => "Tier C (Self-Assessment)",
            CertificationTier::TierB => "Tier B (Runtime Compliance)",
            CertificationTier::TierA => "Tier A (Formal Verification)",
        }
    }
}

/// Evidence required for certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationEvidence {
    /// Checklist items completed (required for all tiers).
    pub checklist_completed: bool,
    /// Test suite pass rate (required for Tier B+).
    pub test_pass_rate: Option<f64>,
    /// Formal proof coverage percentage (required for Tier A).
    pub formal_proof_coverage: Option<f64>,
    /// Certifier identity.
    pub certifier: String,
}

/// Assess certification tier from evidence (CID-I3).
pub fn assess_certification(evidence: &CertificationEvidence) -> CertificationTier {
    if evidence.checklist_completed
        && evidence.test_pass_rate.is_some_and(|r| r >= 0.95)
        && evidence.formal_proof_coverage.is_some_and(|c| c >= 0.40)
    {
        CertificationTier::TierA
    } else if evidence.checklist_completed
        && evidence.test_pass_rate.is_some_and(|r| r >= 0.90)
    {
        CertificationTier::TierB
    } else {
        // Floor: TierC whether checklist is complete or not.
        CertificationTier::TierC
    }
}

/// Anti-capture check (CID-I4, H-2 §3).
///
/// No single certifier may certify more than 20% of active systems.
pub fn check_anti_capture(certifier_counts: &[(String, usize)], total_systems: usize) -> Vec<String> {
    let threshold = (total_systems as f64 * 0.20).ceil() as usize;
    certifier_counts
        .iter()
        .filter(|(_, count)| *count > threshold && total_systems > 0)
        .map(|(name, count)| {
            format!(
                "CAPTURE WARNING: '{}' certifies {}/{} systems (>{} threshold)",
                name, count, total_systems, threshold
            )
        })
        .collect()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // CID-T01: Compliance level ordering
    #[test]
    fn cid_t01_level_ordering() {
        assert!(ComplianceLevel::PaiCompliant < ComplianceLevel::PaiNative);
        assert!(ComplianceLevel::PaiNative < ComplianceLevel::PaiGoverned);
    }

    // CID-T02: Capabilities assessment — full
    #[test]
    fn cid_t02_full_capabilities() {
        let caps = ComplianceCapabilities {
            core_invariants: true,
            sdk_runtime: true,
            governance_daemon: true,
            formal_verification: true,
            test_coverage: true,
        };
        assert_eq!(caps.assess_level(), ComplianceLevel::PaiGoverned);
    }

    // CID-T03: Capabilities assessment — no daemon
    #[test]
    fn cid_t03_no_daemon() {
        let caps = ComplianceCapabilities {
            core_invariants: true,
            sdk_runtime: true,
            governance_daemon: false,
            formal_verification: false,
            test_coverage: true,
        };
        assert_eq!(caps.assess_level(), ComplianceLevel::PaiNative);
    }

    // CID-T04: Capabilities assessment — core only
    #[test]
    fn cid_t04_core_only() {
        let caps = ComplianceCapabilities {
            core_invariants: true,
            sdk_runtime: false,
            governance_daemon: false,
            formal_verification: false,
            test_coverage: false,
        };
        assert_eq!(caps.assess_level(), ComplianceLevel::PaiCompliant);
    }

    // CID-T05: Declaration valid when level <= assessed
    #[test]
    fn cid_t05_valid_declaration() {
        let caps = ComplianceCapabilities {
            core_invariants: true,
            sdk_runtime: true,
            governance_daemon: true,
            formal_verification: true,
            test_coverage: true,
        };
        let id = ComplianceIdentity::declare(
            ComplianceLevel::PaiNative,
            "3.1",
            Some("1.3.0"),
            caps,
        );
        assert!(id.declaration_valid, "lower claim than capability should be valid");
    }

    // CID-T06: Declaration invalid when claiming higher than capabilities
    #[test]
    fn cid_t06_invalid_overclaim() {
        let caps = ComplianceCapabilities {
            core_invariants: true,
            sdk_runtime: true,
            governance_daemon: false, // no daemon
            formal_verification: false,
            test_coverage: true,
        };
        let id = ComplianceIdentity::declare(
            ComplianceLevel::PaiGoverned, // overclaim!
            "3.1",
            Some("1.3.0"),
            caps,
        );
        assert!(!id.declaration_valid, "overclaim must be invalid");
    }

    // CID-T07: Certification tier assessment — Tier A
    #[test]
    fn cid_t07_tier_a() {
        let evidence = CertificationEvidence {
            checklist_completed: true,
            test_pass_rate: Some(0.98),
            formal_proof_coverage: Some(0.45),
            certifier: "verifier1".into(),
        };
        assert_eq!(assess_certification(&evidence), CertificationTier::TierA);
    }

    // CID-T08: Certification tier assessment — Tier B
    #[test]
    fn cid_t08_tier_b() {
        let evidence = CertificationEvidence {
            checklist_completed: true,
            test_pass_rate: Some(0.92),
            formal_proof_coverage: None,
            certifier: "verifier2".into(),
        };
        assert_eq!(assess_certification(&evidence), CertificationTier::TierB);
    }

    // CID-T09: Certification tier assessment — Tier C
    #[test]
    fn cid_t09_tier_c() {
        let evidence = CertificationEvidence {
            checklist_completed: true,
            test_pass_rate: Some(0.80),
            formal_proof_coverage: None,
            certifier: "self".into(),
        };
        assert_eq!(assess_certification(&evidence), CertificationTier::TierC);
    }

    // CID-T10: Anti-capture detection
    #[test]
    fn cid_t10_anti_capture() {
        let counts = vec![
            ("verifier_a".into(), 25),
            ("verifier_b".into(), 5),
            ("verifier_c".into(), 10),
        ];
        // Total 100 systems, threshold = 20%
        let warnings = check_anti_capture(&counts, 100);
        assert_eq!(warnings.len(), 1, "only verifier_a exceeds 20%");
        assert!(warnings[0].contains("verifier_a"));
    }

    // CID-T11: Anti-capture — no violations
    #[test]
    fn cid_t11_anti_capture_clean() {
        let counts = vec![
            ("v1".into(), 10),
            ("v2".into(), 10),
        ];
        let warnings = check_anti_capture(&counts, 100);
        assert!(warnings.is_empty());
    }

    // CID-T12: Identity serialization roundtrip
    #[test]
    fn cid_t12_serialization() {
        let caps = ComplianceCapabilities {
            core_invariants: true,
            sdk_runtime: true,
            governance_daemon: true,
            formal_verification: true,
            test_coverage: true,
        };
        let id = ComplianceIdentity::declare(
            ComplianceLevel::PaiGoverned,
            "3.1",
            Some("1.3.0"),
            caps,
        );
        let json = serde_json::to_string(&id).expect("serialize");
        let parsed: ComplianceIdentity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.level, ComplianceLevel::PaiGoverned);
        assert!(parsed.declaration_valid);
    }

    // CID-T13: Certification tier ordering
    #[test]
    fn cid_t13_tier_ordering() {
        assert!(CertificationTier::TierC < CertificationTier::TierB);
        assert!(CertificationTier::TierB < CertificationTier::TierA);
    }
}
