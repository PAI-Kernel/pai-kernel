//! # PAI Provenance (Doc 11) — Supply Chain & Artifact Provenance
//!
//! **Constitutional reference:** PAI-CD, Document 11
//!
//! ## Scope
//!
//! Defines protected artifact classification, provenance metadata,
//! trust registry management, and SBOM integration for the PAI runtime.
//!
//! ## Invariants
//!
//! | ID      | Invariant                                                        | Ref      |
//! |---------|------------------------------------------------------------------|----------|
//! | PRV-I1  | Protected artifacts must carry provenance metadata               | Doc11-P2 |
//! | PRV-I2  | Unsigned protected artifacts must not be activated               | Doc11-P1 |
//! | PRV-I3  | Trust registry must track active signing identities              | Doc11-P2 |
//! | PRV-I4  | Rollback must verify snapshot provenance against trust registry  | Doc11-P3 |
//! | PRV-I5  | SBOM must list all dependencies with protection-impact flags     | Doc11-P1 |
//! | PRV-I6  | Artifact manifest must be integrity-hashed                       | Doc11-P1 |
//! | PRV-I7  | Dependency lineage must be recursive for protected deps          | Doc11-P2 |

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

// ---------------------------------------------------------------------------
// Protected Artifact Classes (Principle 1)
// ---------------------------------------------------------------------------

/// Classification of protected artifact types per Doc 11 §P1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtectedArtifactClass {
    /// Governance enforcement middleware.
    GovernanceMiddleware,
    /// Classification rule bundles.
    ClassificationRules,
    /// Drift threshold configuration.
    DriftThresholdConfig,
    /// Objective Registry.
    ObjectiveRegistry,
    /// Telemetry Registry.
    TelemetryRegistry,
    /// Personalization ceiling configuration.
    PersonalizationCeiling,
    /// Consent policy configuration.
    ConsentPolicyConfig,
    /// Audit integrity module.
    AuditIntegrityModule,
    /// Portability profile.
    PortabilityProfile,
    /// Emergency profile.
    EmergencyProfile,
    /// TCB Manifest.
    TcbManifest,
    /// Attestation key material storage.
    AttestationKeyStorage,
    /// A dependency capable of altering protected artifact behavior.
    ProtectedDependency,
}

// ---------------------------------------------------------------------------
// Provenance Metadata (Principle 2)
// ---------------------------------------------------------------------------

/// Provenance metadata for a protected artifact (Doc 11 §P2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    /// Unique artifact identifier.
    pub artifact_id: String,
    /// Artifact class.
    pub artifact_class: ProtectedArtifactClass,
    /// Current version string.
    pub version: String,
    /// Source identifier (repository URL, build system, or declared origin).
    pub source_id: String,
    /// Build or packaging origin.
    pub build_origin: String,
    /// Build timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub build_timestamp: OffsetDateTime,
    /// Parent version identifier (version lineage).
    pub parent_version: Option<String>,
    /// Signing identity that signed this artifact.
    pub signing_identity: String,
    /// Hex-encoded SHA-256 hash of the artifact content.
    pub content_hash: String,
    /// Hex-encoded signature over the content hash.
    pub signature: String,
    /// Decision Log entry sequence number that authorized activation.
    pub activation_decision_seq: Option<u64>,
    /// Dependency identifiers (recursive for protected dependencies).
    pub dependency_ids: Vec<String>,
}

impl ProvenanceMetadata {
    /// Verify that content hash matches the given content bytes.
    pub fn verify_content_hash(&self, content: &[u8]) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let computed = hex::encode(hasher.finalize());
        computed == self.content_hash
    }

    /// Check all required fields are non-empty (PRV-I1).
    pub fn is_complete(&self) -> bool {
        !self.artifact_id.is_empty()
            && !self.version.is_empty()
            && !self.source_id.is_empty()
            && !self.build_origin.is_empty()
            && !self.signing_identity.is_empty()
            && !self.content_hash.is_empty()
            && !self.signature.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Trust Registry (Principle 2)
// ---------------------------------------------------------------------------

/// A signing identity in the trust registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEntry {
    /// Signing identity name.
    pub identity: String,
    /// Public key fingerprint (hex-encoded).
    pub fingerprint: String,
    /// Whether this identity is currently active.
    pub active: bool,
    /// When this identity was registered.
    #[serde(with = "time::serde::rfc3339")]
    pub registered_at: OffsetDateTime,
    /// When this identity was revoked (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

/// Trust registry managing signing identities (PRV-I3).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustRegistry {
    entries: Vec<TrustEntry>,
}

impl TrustRegistry {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// Register a new signing identity.
    pub fn register(&mut self, entry: TrustEntry) {
        self.entries.push(entry);
    }

    /// Revoke a signing identity by name.
    pub fn revoke(&mut self, identity: &str) -> bool {
        for e in &mut self.entries {
            if e.identity == identity && e.active {
                e.active = false;
                e.revoked_at = Some("revoked".into());
                return true;
            }
        }
        false
    }

    /// Check if a signing identity is active.
    pub fn is_active(&self, identity: &str) -> bool {
        self.entries
            .iter()
            .any(|e| e.identity == identity && e.active)
    }

    /// List all active signing identities.
    pub fn active_identities(&self) -> Vec<&TrustEntry> {
        self.entries.iter().filter(|e| e.active).collect()
    }

    /// Number of entries (active + revoked).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Artifact Manifest (Principle 1 + SBOM hook)
// ---------------------------------------------------------------------------

/// An artifact manifest entry for the TCB/SBOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifestEntry {
    /// Artifact identifier.
    pub artifact_id: String,
    /// Artifact class.
    pub artifact_class: ProtectedArtifactClass,
    /// Version string.
    pub version: String,
    /// Content hash (SHA-256, hex).
    pub content_hash: String,
    /// Signing identity.
    pub signing_identity: String,
    /// Whether this is a direct dependency or transitive.
    pub is_direct: bool,
    /// Protection impact: true if this dependency can alter enforcement behavior.
    pub protection_impact: bool,
}

/// Artifact manifest collecting all protected artifacts (PRV-I5, PRV-I6).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactManifest {
    entries: Vec<ArtifactManifestEntry>,
    /// Integrity hash of the manifest itself (computed on finalization).
    manifest_hash: Option<String>,
}

impl ArtifactManifest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an artifact entry.
    pub fn add(&mut self, entry: ArtifactManifestEntry) {
        self.manifest_hash = None; // invalidate on mutation
        self.entries.push(entry);
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries.
    pub fn entries(&self) -> &[ArtifactManifestEntry] {
        &self.entries
    }

    /// Get entries with protection impact.
    pub fn protected_entries(&self) -> Vec<&ArtifactManifestEntry> {
        self.entries.iter().filter(|e| e.protection_impact).collect()
    }

    /// Compute and store the manifest integrity hash (PRV-I6).
    pub fn finalize(&mut self) -> String {
        let serialized = serde_json::to_string(&self.entries).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.manifest_hash = Some(hash.clone());
        hash
    }

    /// Verify the manifest integrity hash.
    pub fn verify_integrity(&self) -> bool {
        let Some(ref stored) = self.manifest_hash else {
            return false;
        };
        let serialized = serde_json::to_string(&self.entries).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let computed = hex::encode(hasher.finalize());
        computed == *stored
    }
}

// ---------------------------------------------------------------------------
// Rollback Provenance Validation (Principle 3)
// ---------------------------------------------------------------------------

/// Result of rollback provenance validation (PRV-I4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackValidation {
    /// Whether the rollback target passes provenance checks.
    pub valid: bool,
    /// Specific issues found.
    pub issues: Vec<String>,
}

/// Validate a snapshot's artifacts against the current trust registry (PRV-I4).
pub fn validate_rollback_provenance(
    snapshot_artifacts: &[ProvenanceMetadata],
    trust_registry: &TrustRegistry,
) -> RollbackValidation {
    let mut issues = Vec::new();

    for artifact in snapshot_artifacts {
        // Check signing identity is still active
        if !trust_registry.is_active(&artifact.signing_identity) {
            issues.push(format!(
                "Artifact '{}' signed by '{}' which is no longer in active trust registry",
                artifact.artifact_id, artifact.signing_identity
            ));
        }

        // Check provenance completeness
        if !artifact.is_complete() {
            issues.push(format!(
                "Artifact '{}' has incomplete provenance metadata",
                artifact.artifact_id
            ));
        }
    }

    RollbackValidation {
        valid: issues.is_empty(),
        issues,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_provenance() -> ProvenanceMetadata {
        let content = b"governance module v1.2";
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hex::encode(hasher.finalize());

        ProvenanceMetadata {
            artifact_id: "gov-middleware-v1.2".into(),
            artifact_class: ProtectedArtifactClass::GovernanceMiddleware,
            version: "1.2.0".into(),
            source_id: "https://github.com/pai-kernel/runtime".into(),
            build_origin: "ci/build-1234".into(),
            build_timestamp: OffsetDateTime::now_utc(),
            parent_version: Some("1.1.0".into()),
            signing_identity: "release-key-1".into(),
            content_hash: hash,
            signature: "deadbeef".into(),
            activation_decision_seq: Some(42),
            dependency_ids: vec!["dep-a".into()],
        }
    }

    fn sample_trust_registry() -> TrustRegistry {
        let mut tr = TrustRegistry::new();
        tr.register(TrustEntry {
            identity: "release-key-1".into(),
            fingerprint: "aabb".into(),
            active: true,
            registered_at: OffsetDateTime::now_utc(),
            revoked_at: None,
        });
        tr
    }

    // PRV-T01: Provenance metadata completeness check
    #[test]
    fn prv_t01_complete_provenance() {
        let p = sample_provenance();
        assert!(p.is_complete());
    }

    // PRV-T02: Incomplete provenance detected
    #[test]
    fn prv_t02_incomplete_provenance() {
        let mut p = sample_provenance();
        p.source_id = String::new();
        assert!(!p.is_complete());
    }

    // PRV-T03: Content hash verification
    #[test]
    fn prv_t03_content_hash_verify() {
        let content = b"governance module v1.2";
        let p = sample_provenance();
        assert!(p.verify_content_hash(content));
        assert!(!p.verify_content_hash(b"tampered content"));
    }

    // PRV-T04: Trust registry — register and check active
    #[test]
    fn prv_t04_trust_registry_active() {
        let tr = sample_trust_registry();
        assert!(tr.is_active("release-key-1"));
        assert!(!tr.is_active("unknown-key"));
    }

    // PRV-T05: Trust registry — revoke identity
    #[test]
    fn prv_t05_trust_registry_revoke() {
        let mut tr = sample_trust_registry();
        assert!(tr.revoke("release-key-1"));
        assert!(!tr.is_active("release-key-1"));
    }

    // PRV-T06: Trust registry — revoke nonexistent returns false
    #[test]
    fn prv_t06_revoke_nonexistent() {
        let mut tr = sample_trust_registry();
        assert!(!tr.revoke("no-such-key"));
    }

    // PRV-T07: Artifact manifest — finalize and verify integrity
    #[test]
    fn prv_t07_manifest_integrity() {
        let mut manifest = ArtifactManifest::new();
        manifest.add(ArtifactManifestEntry {
            artifact_id: "gov-mw".into(),
            artifact_class: ProtectedArtifactClass::GovernanceMiddleware,
            version: "1.0.0".into(),
            content_hash: "abc123".into(),
            signing_identity: "key-1".into(),
            is_direct: true,
            protection_impact: true,
        });
        manifest.finalize();
        assert!(manifest.verify_integrity());
    }

    // PRV-T08: Manifest integrity fails after mutation
    #[test]
    fn prv_t08_manifest_tamper_detected() {
        let mut manifest = ArtifactManifest::new();
        manifest.add(ArtifactManifestEntry {
            artifact_id: "item-a".into(),
            artifact_class: ProtectedArtifactClass::ClassificationRules,
            version: "1.0.0".into(),
            content_hash: "aaa".into(),
            signing_identity: "key".into(),
            is_direct: true,
            protection_impact: true,
        });
        manifest.finalize();
        // Tamper: add another entry without re-finalizing
        manifest.add(ArtifactManifestEntry {
            artifact_id: "item-b".into(),
            artifact_class: ProtectedArtifactClass::DriftThresholdConfig,
            version: "1.0.0".into(),
            content_hash: "bbb".into(),
            signing_identity: "key".into(),
            is_direct: false,
            protection_impact: false,
        });
        // manifest_hash was invalidated by add(), so verify should fail
        assert!(!manifest.verify_integrity());
    }

    // PRV-T09: Protected entries filter
    #[test]
    fn prv_t09_protected_entries() {
        let mut manifest = ArtifactManifest::new();
        manifest.add(ArtifactManifestEntry {
            artifact_id: "a".into(),
            artifact_class: ProtectedArtifactClass::GovernanceMiddleware,
            version: "1".into(),
            content_hash: "h".into(),
            signing_identity: "k".into(),
            is_direct: true,
            protection_impact: true,
        });
        manifest.add(ArtifactManifestEntry {
            artifact_id: "b".into(),
            artifact_class: ProtectedArtifactClass::ProtectedDependency,
            version: "1".into(),
            content_hash: "h".into(),
            signing_identity: "k".into(),
            is_direct: false,
            protection_impact: false,
        });
        assert_eq!(manifest.protected_entries().len(), 1);
    }

    // PRV-T10: Rollback validation — passes with active trust
    #[test]
    fn prv_t10_rollback_valid() {
        let artifacts = vec![sample_provenance()];
        let tr = sample_trust_registry();
        let result = validate_rollback_provenance(&artifacts, &tr);
        assert!(result.valid);
        assert!(result.issues.is_empty());
    }

    // PRV-T11: Rollback validation — fails with revoked signing identity
    #[test]
    fn prv_t11_rollback_revoked_signer() {
        let artifacts = vec![sample_provenance()];
        let mut tr = sample_trust_registry();
        tr.revoke("release-key-1");
        let result = validate_rollback_provenance(&artifacts, &tr);
        assert!(!result.valid);
        assert!(result.issues[0].contains("no longer in active trust registry"));
    }

    // PRV-T12: Rollback validation — fails with incomplete provenance
    #[test]
    fn prv_t12_rollback_incomplete() {
        let mut p = sample_provenance();
        p.signing_identity = String::new(); // make incomplete AND unknown
        let artifacts = vec![p];
        let tr = sample_trust_registry();
        let result = validate_rollback_provenance(&artifacts, &tr);
        assert!(!result.valid);
    }

    // PRV-T13: Manifest serialization roundtrip
    #[test]
    fn prv_t13_manifest_serde() {
        let mut manifest = ArtifactManifest::new();
        manifest.add(ArtifactManifestEntry {
            artifact_id: "test".into(),
            artifact_class: ProtectedArtifactClass::TcbManifest,
            version: "1.0".into(),
            content_hash: "abc".into(),
            signing_identity: "key".into(),
            is_direct: true,
            protection_impact: true,
        });
        manifest.finalize();

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: ArtifactManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(parsed.verify_integrity());
    }

    // PRV-T14: All protected artifact classes are representable
    #[test]
    fn prv_t14_all_artifact_classes() {
        let classes = [
            ProtectedArtifactClass::GovernanceMiddleware,
            ProtectedArtifactClass::ClassificationRules,
            ProtectedArtifactClass::DriftThresholdConfig,
            ProtectedArtifactClass::ObjectiveRegistry,
            ProtectedArtifactClass::TelemetryRegistry,
            ProtectedArtifactClass::PersonalizationCeiling,
            ProtectedArtifactClass::ConsentPolicyConfig,
            ProtectedArtifactClass::AuditIntegrityModule,
            ProtectedArtifactClass::PortabilityProfile,
            ProtectedArtifactClass::EmergencyProfile,
            ProtectedArtifactClass::TcbManifest,
            ProtectedArtifactClass::AttestationKeyStorage,
            ProtectedArtifactClass::ProtectedDependency,
        ];
        // 13 classes match Doc 11 §P1 enumeration
        assert_eq!(classes.len(), 13);
    }
}
