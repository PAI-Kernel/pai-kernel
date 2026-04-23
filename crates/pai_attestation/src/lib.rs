//! # PAI Trusted Computing Base & Attestation (Doc 10)
//!
//! **Constitutional reference:** PAI-CD v3.1, Document 10
//!
//! ## Scope
//!
//! Defines the minimum TCB for constitutional enforcement and attestation
//! requirements for proving live conformance.  Makes existing protections
//! cryptographically attestable.
//!
//! ## Invariants
//!
//! | ID      | Invariant                                                          | Ref      |
//! |---------|--------------------------------------------------------------------|----------|
//! | TCB-I1  | TCB manifest must list all 9 core components                       | Doc10-P1 |
//! | TCB-I2  | Each component must have integrity hash                            | Doc10-P1 |
//! | TCB-I3  | Attestation evidence is signed and timestamped                     | Doc10-P2 |
//! | TCB-I4  | Measured state must match declared state (mismatch = breach)       | Doc10-P4 |
//! | TCB-I5  | Attestation exportable without proprietary SDK                     | Doc10-P2 |
//! | TCB-I6  | TCB manifest includes constitutional document version hash         | Doc10-P1 |

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// TCB Component Classification (Principle 1)
// ---------------------------------------------------------------------------

/// Minimum TCB components per Doc 10 §P1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TcbComponent {
    /// Governance enforcement middleware.
    GovernanceMiddleware,
    /// Capability classification engine.
    ClassificationEngine,
    /// Consent verification engine.
    ConsentEngine,
    /// Drift threshold engine.
    DriftEngine,
    /// Conservative Mode controller.
    ConservativeModeController,
    /// Audit integrity module.
    AuditIntegrityModule,
    /// Snapshot integrity verifier.
    SnapshotVerifier,
    /// Active configuration manifest.
    ConfigManifest,
    /// Attestation key material.
    AttestationKeyMaterial,
}

impl TcbComponent {
    /// All 9 required TCB components.
    pub fn all() -> [TcbComponent; 9] {
        [
            TcbComponent::GovernanceMiddleware,
            TcbComponent::ClassificationEngine,
            TcbComponent::ConsentEngine,
            TcbComponent::DriftEngine,
            TcbComponent::ConservativeModeController,
            TcbComponent::AuditIntegrityModule,
            TcbComponent::SnapshotVerifier,
            TcbComponent::ConfigManifest,
            TcbComponent::AttestationKeyMaterial,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TcbComponent::GovernanceMiddleware => "Governance Enforcement Middleware",
            TcbComponent::ClassificationEngine => "Capability Classification Engine",
            TcbComponent::ConsentEngine => "Consent Verification Engine",
            TcbComponent::DriftEngine => "Drift Threshold Engine",
            TcbComponent::ConservativeModeController => "Conservative Mode Controller",
            TcbComponent::AuditIntegrityModule => "Audit Integrity Module",
            TcbComponent::SnapshotVerifier => "Snapshot Integrity Verifier",
            TcbComponent::ConfigManifest => "Active Configuration Manifest",
            TcbComponent::AttestationKeyMaterial => "Attestation Key Material",
        }
    }
}

// ---------------------------------------------------------------------------
// TCB Manifest (Principle 1)
// ---------------------------------------------------------------------------

/// An entry in the TCB manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcbEntry {
    /// Component classification.
    pub component: TcbComponent,
    /// Stable identifier.
    pub stable_id: String,
    /// Version string.
    pub version: String,
    /// SHA-256 integrity hash (hex).
    pub integrity_hash: String,
}

/// TCB Manifest (TCB-I1, TCB-I2, TCB-I6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcbManifest {
    /// Instance identifier.
    pub instance_id: String,
    /// Constitutional document version hash.
    pub corpus_version_hash: String,
    /// Component entries.
    entries: Vec<TcbEntry>,
    /// Policy artifact hashes.
    pub policy_hashes: PolicyHashes,
    /// Manifest integrity hash (computed on finalization).
    manifest_hash: Option<String>,
}

/// Policy artifact hashes included in the TCB manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyHashes {
    pub classification_rules: String,
    pub objective_registry: String,
    pub telemetry_registry: String,
    pub consent_rules: String,
    pub drift_thresholds: String,
}

impl TcbManifest {
    pub fn new(instance_id: &str, corpus_version_hash: &str) -> Self {
        Self {
            instance_id: instance_id.into(),
            corpus_version_hash: corpus_version_hash.into(),
            entries: Vec::new(),
            policy_hashes: PolicyHashes::default(),
            manifest_hash: None,
        }
    }

    /// Add a TCB component entry.
    pub fn add_entry(&mut self, entry: TcbEntry) {
        self.manifest_hash = None;
        self.entries.push(entry);
    }

    /// Get all entries.
    pub fn entries(&self) -> &[TcbEntry] {
        &self.entries
    }

    /// Check which of the 9 required components are present (TCB-I1).
    pub fn missing_components(&self) -> Vec<TcbComponent> {
        let present: std::collections::HashSet<_> = self.entries.iter().map(|e| e.component).collect();
        TcbComponent::all()
            .into_iter()
            .filter(|c| !present.contains(c))
            .collect()
    }

    /// Whether all 9 required components are declared.
    pub fn is_complete(&self) -> bool {
        self.missing_components().is_empty()
    }

    /// Finalize with integrity hash.
    pub fn finalize(&mut self) -> String {
        let serialized = serde_json::to_string(&(&self.entries, &self.policy_hashes, &self.corpus_version_hash))
            .unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.manifest_hash = Some(hash.clone());
        hash
    }

    /// Verify integrity hash.
    pub fn verify_integrity(&self) -> bool {
        let Some(ref stored) = self.manifest_hash else {
            return false;
        };
        let serialized = serde_json::to_string(&(&self.entries, &self.policy_hashes, &self.corpus_version_hash))
            .unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize()) == *stored
    }
}

// ---------------------------------------------------------------------------
// Attestation Evidence (Principle 2)
// ---------------------------------------------------------------------------

/// Attestation evidence produced by the runtime (TCB-I3, TCB-I5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationEvidence {
    /// Instance identifier.
    pub instance_id: String,
    /// Timestamp (ISO 8601 string for portability).
    pub timestamp: String,
    /// Active TCB Manifest hash.
    pub tcb_manifest_hash: String,
    /// Governance logic version hash.
    pub governance_version_hash: String,
    /// Classification rules hash.
    pub classification_rules_hash: String,
    /// Drift threshold configuration hash.
    pub drift_threshold_hash: String,
    /// Objective Registry hash.
    pub objective_registry_hash: String,
    /// Telemetry Registry hash.
    pub telemetry_registry_hash: String,
    /// Personalization ceiling hash.
    pub personalization_ceiling_hash: String,
    /// Active Amendment set hash.
    pub amendment_set_hash: String,
    /// Emergency profile state (if any).
    pub emergency_state: Option<String>,
    /// Conservative Mode / Response Level.
    pub response_level: String,
    /// Decision Log integrity state (latest chain hash).
    pub decision_log_chain_hash: String,
    /// Hex-encoded signature over all fields.
    pub signature: String,
}

impl AttestationEvidence {
    /// Compute a content hash over all attestable fields (excluding signature).
    pub fn content_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.instance_id.as_bytes());
        hasher.update(self.timestamp.as_bytes());
        hasher.update(self.tcb_manifest_hash.as_bytes());
        hasher.update(self.governance_version_hash.as_bytes());
        hasher.update(self.classification_rules_hash.as_bytes());
        hasher.update(self.drift_threshold_hash.as_bytes());
        hasher.update(self.objective_registry_hash.as_bytes());
        hasher.update(self.telemetry_registry_hash.as_bytes());
        hasher.update(self.personalization_ceiling_hash.as_bytes());
        hasher.update(self.amendment_set_hash.as_bytes());
        hasher.update(self.response_level.as_bytes());
        hasher.update(self.decision_log_chain_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Export as JSON (TCB-I5: readable without proprietary SDK).
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Measured Enforcement State (Principle 4)
// ---------------------------------------------------------------------------

/// Declared vs. measured enforcement state for comparison (TCB-I4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementState {
    /// Hash of loaded governance version.
    pub governance_hash: String,
    /// Hash of loaded classification rules.
    pub classification_hash: String,
    /// Hash of loaded drift thresholds.
    pub threshold_hash: String,
    /// Hash of loaded Objective Registry.
    pub objective_registry_hash: String,
    /// Hash of loaded Telemetry Registry.
    pub telemetry_registry_hash: String,
    /// Hash of active portability profile.
    pub portability_hash: String,
    /// Hash of active consent policy.
    pub consent_policy_hash: String,
    /// Emergency state identifier (if any).
    pub emergency_state: Option<String>,
    /// Active response level.
    pub response_level: String,
}

/// Compare declared and measured enforcement states (TCB-I4).
///
/// Any mismatch constitutes breach per Doc 10 §P4.
pub fn compare_enforcement_state(
    declared: &EnforcementState,
    measured: &EnforcementState,
) -> EnforcementComparison {
    let mut mismatches = Vec::new();

    let checks: &[(&str, &str, &str)] = &[
        ("governance", &declared.governance_hash, &measured.governance_hash),
        ("classification", &declared.classification_hash, &measured.classification_hash),
        ("thresholds", &declared.threshold_hash, &measured.threshold_hash),
        ("objective_registry", &declared.objective_registry_hash, &measured.objective_registry_hash),
        ("telemetry_registry", &declared.telemetry_registry_hash, &measured.telemetry_registry_hash),
        ("portability", &declared.portability_hash, &measured.portability_hash),
        ("consent_policy", &declared.consent_policy_hash, &measured.consent_policy_hash),
        ("response_level", &declared.response_level, &measured.response_level),
    ];

    for (name, decl, meas) in checks {
        if decl != meas {
            mismatches.push(format!(
                "{}: declared='{}' measured='{}'",
                name,
                &decl[..decl.len().min(16)],
                &meas[..meas.len().min(16)]
            ));
        }
    }

    // Emergency state comparison
    if declared.emergency_state != measured.emergency_state {
        mismatches.push(format!(
            "emergency_state: declared={:?} measured={:?}",
            declared.emergency_state, measured.emergency_state
        ));
    }

    EnforcementComparison {
        matches: mismatches.is_empty(),
        mismatches,
    }
}

/// Result of enforcement state comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementComparison {
    /// True if declared matches measured.
    pub matches: bool,
    /// Specific mismatches found.
    pub mismatches: Vec<String>,
}

// ---------------------------------------------------------------------------
// Helper: hash arbitrary bytes
// ---------------------------------------------------------------------------

/// Compute SHA-256 hex hash of bytes (utility for callers).
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> TcbManifest {
        let mut m = TcbManifest::new("instance-001", "corpus-v3.1-hash");
        for comp in TcbComponent::all() {
            m.add_entry(TcbEntry {
                component: comp,
                stable_id: format!("tcb-{:?}", comp),
                version: "1.0.0".into(),
                integrity_hash: sha256_hex(format!("{:?}", comp).as_bytes()),
            });
        }
        m
    }

    fn sample_state() -> EnforcementState {
        EnforcementState {
            governance_hash: "aaa".into(),
            classification_hash: "bbb".into(),
            threshold_hash: "ccc".into(),
            objective_registry_hash: "ddd".into(),
            telemetry_registry_hash: "eee".into(),
            portability_hash: "fff".into(),
            consent_policy_hash: "ggg".into(),
            emergency_state: None,
            response_level: "Normal".into(),
        }
    }

    // TCB-T01: Complete manifest with all 9 components
    #[test]
    fn tcb_t01_complete_manifest() {
        let m = sample_manifest();
        assert!(m.is_complete());
        assert!(m.missing_components().is_empty());
    }

    // TCB-T02: Incomplete manifest detects missing components
    #[test]
    fn tcb_t02_incomplete_manifest() {
        let mut m = TcbManifest::new("x", "y");
        // Only add 3 of 9
        for comp in &TcbComponent::all()[..3] {
            m.add_entry(TcbEntry {
                component: *comp,
                stable_id: "x".into(),
                version: "1".into(),
                integrity_hash: "h".into(),
            });
        }
        assert!(!m.is_complete());
        assert_eq!(m.missing_components().len(), 6);
    }

    // TCB-T03: Manifest integrity hash
    #[test]
    fn tcb_t03_manifest_integrity() {
        let mut m = sample_manifest();
        m.finalize();
        assert!(m.verify_integrity());
    }

    // TCB-T04: Manifest tamper detection
    #[test]
    fn tcb_t04_manifest_tamper() {
        let mut m = sample_manifest();
        m.finalize();
        m.entries.push(TcbEntry {
            component: TcbComponent::GovernanceMiddleware,
            stable_id: "tampered".into(),
            version: "1".into(),
            integrity_hash: "bad".into(),
        });
        assert!(!m.verify_integrity());
    }

    // TCB-T05: Attestation evidence content hash
    #[test]
    fn tcb_t05_attestation_hash() {
        let evidence = AttestationEvidence {
            instance_id: "i1".into(),
            timestamp: "2026-04-13T00:00:00Z".into(),
            tcb_manifest_hash: "mh".into(),
            governance_version_hash: "gh".into(),
            classification_rules_hash: "ch".into(),
            drift_threshold_hash: "dh".into(),
            objective_registry_hash: "oh".into(),
            telemetry_registry_hash: "th".into(),
            personalization_ceiling_hash: "ph".into(),
            amendment_set_hash: "ah".into(),
            emergency_state: None,
            response_level: "Normal".into(),
            decision_log_chain_hash: "dlh".into(),
            signature: "sig".into(),
        };
        let hash = evidence.content_hash();
        assert!(!hash.is_empty());
        // Deterministic
        assert_eq!(hash, evidence.content_hash());
    }

    // TCB-T06: Attestation JSON export (TCB-I5)
    #[test]
    fn tcb_t06_attestation_json_export() {
        let evidence = AttestationEvidence {
            instance_id: "i1".into(),
            timestamp: "2026-04-13T00:00:00Z".into(),
            tcb_manifest_hash: "mh".into(),
            governance_version_hash: "gh".into(),
            classification_rules_hash: "ch".into(),
            drift_threshold_hash: "dh".into(),
            objective_registry_hash: "oh".into(),
            telemetry_registry_hash: "th".into(),
            personalization_ceiling_hash: "ph".into(),
            amendment_set_hash: "ah".into(),
            emergency_state: None,
            response_level: "Normal".into(),
            decision_log_chain_hash: "dlh".into(),
            signature: "sig".into(),
        };
        let json = evidence.to_json();
        assert_eq!(json["instance_id"], "i1");
        assert_eq!(json["response_level"], "Normal");
    }

    // TCB-T07: Enforcement state comparison — match
    #[test]
    fn tcb_t07_state_match() {
        let s = sample_state();
        let result = compare_enforcement_state(&s, &s.clone());
        assert!(result.matches);
        assert!(result.mismatches.is_empty());
    }

    // TCB-T08: Enforcement state comparison — mismatch detected
    #[test]
    fn tcb_t08_state_mismatch() {
        let declared = sample_state();
        let mut measured = sample_state();
        measured.governance_hash = "TAMPERED".into();
        let result = compare_enforcement_state(&declared, &measured);
        assert!(!result.matches);
        assert!(result.mismatches[0].contains("governance"));
    }

    // TCB-T09: Emergency state mismatch
    #[test]
    fn tcb_t09_emergency_mismatch() {
        let declared = sample_state();
        let mut measured = sample_state();
        measured.emergency_state = Some("active".into());
        let result = compare_enforcement_state(&declared, &measured);
        assert!(!result.matches);
        assert!(result.mismatches[0].contains("emergency_state"));
    }

    // TCB-T10: All 9 TCB components accessible
    #[test]
    fn tcb_t10_all_components() {
        let all = TcbComponent::all();
        assert_eq!(all.len(), 9);
        for c in &all {
            assert!(!c.label().is_empty());
        }
    }

    // TCB-T11: Manifest serialization roundtrip
    #[test]
    fn tcb_t11_manifest_serde() {
        let mut m = sample_manifest();
        m.finalize();
        let json = serde_json::to_string(&m).unwrap();
        let parsed: TcbManifest = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_complete());
        assert!(parsed.verify_integrity());
    }

    // TCB-T12: sha256_hex utility
    #[test]
    fn tcb_t12_sha256_utility() {
        let hash1 = sha256_hex(b"hello");
        let hash2 = sha256_hex(b"hello");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, sha256_hex(b"world"));
        assert_eq!(hash1.len(), 64); // 32 bytes hex
    }
}
