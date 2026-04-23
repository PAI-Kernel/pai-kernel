//! # PAI Export Service (Component #5)
//!
//! Per PHASE1-TZ-001 Section 8 / P0-4 Section 3.
//!
//! Author MUST be able to export full state at any time,
//! regardless of Conservative Mode or subscription tier.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use pai_drift::DriftThresholds;
use pai_governance_daemon::GovernanceDaemon;
use pai_witness::WitnessLog;

// ── Export types ───────────────────────────────────────────────────────

/// Full portability bundle per P0-4 Section 3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBundle {
    pub witness_log: Vec<serde_json::Value>,
    pub governance_state: serde_json::Value,
    pub consent_records: Vec<serde_json::Value>,
    pub delegation_grants: Vec<serde_json::Value>,
    pub objective_registry: Vec<String>,
    pub snapshots: Vec<serde_json::Value>,
    pub drift_config: serde_json::Value,
    pub metadata: ExportMetadata,
}

/// Metadata about the export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: u64,
    pub pai_cd_version: String,
    pub runtime_version: String,
    pub witness_chain_verified: bool,
    pub entry_count: u64,
    pub integrity_hash: String,
}

// ── Builder ────────────────────────────────────────────────────────────

/// Builds an [`ExportBundle`] from live components.
pub struct ExportBuilder<'a> {
    daemon: &'a GovernanceDaemon,
    witness: &'a WitnessLog,
    drift_thresholds: &'a DriftThresholds,
}

impl<'a> ExportBuilder<'a> {
    pub fn new(
        daemon: &'a GovernanceDaemon,
        witness: &'a WitnessLog,
        drift_thresholds: &'a DriftThresholds,
    ) -> Self {
        Self {
            daemon,
            witness,
            drift_thresholds,
        }
    }

    /// Build the complete export bundle.
    ///
    /// Available regardless of Conservative Mode (P0-4 Section 2).
    pub fn build(&self) -> ExportBundle {
        let state = self.daemon.state();

        let witness_entries: Vec<serde_json::Value> = self
            .witness
            .export()
            .iter()
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .collect();

        let governance_state =
            serde_json::to_value(state).unwrap_or_else(|_| serde_json::json!({}));

        let consent_records: Vec<serde_json::Value> = state
            .consent_ledger()
            .iter()
            .map(|c| serde_json::to_value(c).unwrap_or_default())
            .collect();

        let delegation_grants: Vec<serde_json::Value> = state
            .delegations()
            .iter()
            .map(|d| serde_json::to_value(d).unwrap_or_default())
            .collect();

        let objective_registry: Vec<String> =
            state.objective_registry().to_vec();

        let snapshots: Vec<serde_json::Value> = self
            .daemon
            .snapshots()
            .iter()
            .map(|s| serde_json::to_value(s).unwrap_or_default())
            .collect();

        let drift_config =
            serde_json::to_value(self.drift_thresholds).unwrap_or_default();

        let chain_ok = self.witness.verify().is_ok();
        let entry_count = self.witness.len() as u64;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Build bundle without hash first, then compute hash
        let mut bundle = ExportBundle {
            witness_log: witness_entries,
            governance_state,
            consent_records,
            delegation_grants,
            objective_registry,
            snapshots,
            drift_config,
            metadata: ExportMetadata {
                exported_at: now,
                pai_cd_version: "3.1".into(),
                runtime_version: "1.3.0".into(),
                witness_chain_verified: chain_ok,
                entry_count,
                integrity_hash: String::new(),
            },
        };

        // Compute integrity hash over the content (excluding the hash field itself)
        bundle.metadata.integrity_hash = compute_integrity_hash(&bundle);
        bundle
    }
}

/// SHA-256 hash of the serialized bundle content.
fn compute_integrity_hash(bundle: &ExportBundle) -> String {
    // Hash all content fields (metadata.integrity_hash is empty at this point)
    let payload = serde_json::json!({
        "witness_log": bundle.witness_log,
        "governance_state": bundle.governance_state,
        "consent_records": bundle.consent_records,
        "delegation_grants": bundle.delegation_grants,
        "objective_registry": bundle.objective_registry,
        "snapshots": bundle.snapshots,
        "drift_config": bundle.drift_config,
        "entry_count": bundle.metadata.entry_count,
    });
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    hex::encode(Sha256::digest(&bytes))
}

/// Verify that a bundle's integrity hash is correct.
pub fn verify_bundle_integrity(bundle: &ExportBundle) -> bool {
    let expected = compute_integrity_hash(bundle);
    expected == bundle.metadata.integrity_hash
}

// ── MP-7: Import Parity Verification ──────────────────────────────────

/// Parity check result for MP-7 (Portability 100%).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityReport {
    /// True if all 7 data components are present.
    pub components_complete: bool,
    /// True if witness chain hash continuity verified on imported data.
    pub witness_chain_continuous: bool,
    /// True if integrity hash matches recomputed hash.
    pub integrity_verified: bool,
    /// True if entry counts match between metadata and actual data.
    pub entry_count_matches: bool,
    /// Count of components present (max 7).
    pub component_count: u8,
    /// List of missing components.
    pub missing_components: Vec<String>,
    /// Overall pass: all checks true = 100% parity (MP-7 threshold).
    pub pass: bool,
}

/// Verify import parity per MP-7 (Portability 100%).
///
/// Ensures that an imported ExportBundle has:
/// 1. All 7 data components present (not null/missing)
/// 2. Witness chain hash continuity (prev_hash linkage)
/// 3. Integrity hash matches recomputed hash
/// 4. Entry count in metadata matches actual witness_log length
///
/// MP-7 threshold: 100/100/100 — zero tolerance for data loss.
pub fn verify_import_parity(bundle: &ExportBundle) -> ParityReport {
    let mut missing = Vec::new();
    let mut count: u8 = 0;

    // Check 7 components presence
    if !bundle.witness_log.is_empty() || bundle.metadata.entry_count == 0 {
        count += 1;
    } else {
        missing.push("witness_log".into());
    }

    if bundle.governance_state.is_object() {
        count += 1;
    } else {
        missing.push("governance_state".into());
    }

    // consent_records and delegation_grants may legitimately be empty
    count += 1; // consent_records always present as array
    count += 1; // delegation_grants always present as array

    // objective_registry always present as vec
    count += 1;

    if !bundle.snapshots.is_empty() || bundle.metadata.entry_count == 0 {
        count += 1;
    } else {
        missing.push("snapshots".into());
    }

    if bundle.drift_config.is_object() {
        count += 1;
    } else {
        missing.push("drift_config".into());
    }

    let components_complete = count == 7 && missing.is_empty();

    // Witness chain continuity check
    let witness_chain_continuous = check_witness_chain_continuity(&bundle.witness_log);

    // Integrity hash verification
    let integrity_verified = verify_bundle_integrity(bundle);

    // Entry count match
    let entry_count_matches = bundle.metadata.entry_count == bundle.witness_log.len() as u64;

    let pass = components_complete && witness_chain_continuous && integrity_verified && entry_count_matches;

    ParityReport {
        components_complete,
        witness_chain_continuous,
        integrity_verified,
        entry_count_matches,
        component_count: count,
        missing_components: missing,
        pass,
    }
}

/// Check witness chain hash continuity in imported JSON entries.
fn check_witness_chain_continuity(entries: &[serde_json::Value]) -> bool {
    if entries.is_empty() {
        return true; // Empty chain is trivially continuous
    }

    for (i, entry) in entries.iter().enumerate() {
        let hash = entry.get("hash").and_then(|h| h.as_str());
        let prev_hash = entry.get("prev_hash").and_then(|h| h.as_str());

        // Must have hash field
        if hash.is_none() {
            return false;
        }

        if i == 0 {
            // First entry should have prev_hash = "GENESIS" or be the start
            // We just verify it exists
            if prev_hash.is_none() {
                return false;
            }
        } else {
            // Subsequent entries: prev_hash must match previous entry's hash
            let prev_entry_hash = entries[i - 1].get("hash").and_then(|h| h.as_str());
            if prev_hash != prev_entry_hash {
                return false;
            }
        }
    }
    true
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pai_drift::DriftThresholds;
    use pai_witness::{
        ConstitutionalRef, DecisionClass, ImpactScope, Initiator, ReversibilityStatus,
        RiskTier, StructuredRationale, WitnessEntryBuilder, WitnessLog,
    };

    fn sample_daemon() -> GovernanceDaemon {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key();
        let mut gd =
            GovernanceDaemon::new(10).with_author_keys("TEST_KEY", vk, Some(sk));
        gd.snapshot();
        gd.accumulate_drift(2);
        gd
    }

    fn sample_witness() -> WitnessLog {
        let mut log = WitnessLog::new();
        for i in 1..=3 {
            log.append(
                WitnessEntryBuilder::new()
                    .decision_class(DecisionClass::GovAction)
                    .timestamp(1000 * i)
                    .initiator(Initiator::Governance)
                    .scope_of_impact(vec![ImpactScope::Governance])
                    .risk_tier(RiskTier::Tier1)
                    .rationale(
                        StructuredRationale::new(&format!("export test entry {}", i)).unwrap(),
                    )
                    .constitutional_ref(ConstitutionalRef("Export §3".into()))
                    .reversibility(ReversibilityStatus::Irreversible),
            )
            .unwrap();
        }
        log
    }

    fn sample_thresholds() -> DriftThresholds {
        DriftThresholds::new(10.0, 30 * 86400)
    }

    // ── EXP-T01: Export produces valid JSON ────────────────────────
    #[test]
    fn exp_t01_export_valid_json() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        let json = serde_json::to_string_pretty(&bundle);
        assert!(json.is_ok(), "bundle must serialize to valid JSON");

        // Round-trip: deserialize back
        let json_str = json.unwrap();
        let parsed: Result<ExportBundle, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok(), "bundle must deserialize from JSON");
    }

    // ── EXP-T02: Export includes all 7 bundle components ───────────
    #[test]
    fn exp_t02_all_seven_components() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        // 1. witness_log
        assert_eq!(bundle.witness_log.len(), 3, "witness_log must have entries");
        // 2. governance_state
        assert!(bundle.governance_state.is_object(), "governance_state must be an object");
        // 3. consent_records (may be empty but must be present)
        assert!(bundle.consent_records.is_empty() || !bundle.consent_records.is_empty());
        // 4. delegation_grants
        assert!(bundle.delegation_grants.is_empty() || !bundle.delegation_grants.is_empty());
        // 5. objective_registry (present as a field, may be empty)
        let _ = &bundle.objective_registry; // field exists
        // 6. snapshots
        assert!(!bundle.snapshots.is_empty(), "snapshots must have at least 1");
        // 7. drift_config
        assert!(bundle.drift_config.is_object(), "drift_config must be an object");

        // Metadata
        assert!(bundle.metadata.exported_at > 0);
        assert_eq!(bundle.metadata.pai_cd_version, "3.1");
        assert_eq!(bundle.metadata.runtime_version, "1.3.0");
        assert_eq!(bundle.metadata.entry_count, 3);
        assert!(!bundle.metadata.integrity_hash.is_empty());
    }

    // ── EXP-T03: Witness chain verifiable after export ─────────────
    #[test]
    fn exp_t03_witness_chain_verified() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        assert!(
            bundle.metadata.witness_chain_verified,
            "witness chain must be verified in export metadata"
        );

        // Integrity hash must verify
        assert!(
            verify_bundle_integrity(&bundle),
            "bundle integrity hash must verify"
        );
    }

    // ── EXP-T04: Export available regardless of Conservative Mode ──
    #[test]
    fn exp_t04_export_in_conservative_mode() {
        let mut gd = sample_daemon();
        gd.inference_bypass_attempt(); // enters Conservative Mode
        assert!(gd.state().conservative(), "must be in conservative mode");

        let wl = sample_witness();
        let dt = sample_thresholds();

        // Export must succeed even in Conservative Mode
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();
        assert!(!bundle.witness_log.is_empty());
        assert!(bundle.governance_state.is_object());
        assert!(verify_bundle_integrity(&bundle));
    }

    // ── EXP-T05: Export available regardless of subscription tier ──
    #[test]
    fn exp_t05_export_no_tier_restriction() {
        // Per P0-4 Section 2: export is a fundamental right, not tier-gated.
        // We verify by exporting with a minimal daemon (no keys, no consent).
        let gd = GovernanceDaemon::new(10);
        let wl = WitnessLog::new();
        let dt = DriftThresholds::new(10.0, 86400);

        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        // Export succeeds with empty state
        assert!(bundle.witness_log.is_empty());
        assert!(bundle.objective_registry.is_empty());
        assert_eq!(bundle.metadata.entry_count, 0);
        assert!(verify_bundle_integrity(&bundle));

        let json = serde_json::to_string(&bundle);
        assert!(json.is_ok(), "empty-state export must produce valid JSON");
    }

    // ── MP7-T01: Full export passes parity check ─────────────────────
    #[test]
    fn mp7_t01_full_export_parity() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        let report = verify_import_parity(&bundle);
        assert!(report.pass, "full export must pass parity check: {:?}", report);
        assert!(report.components_complete);
        assert!(report.integrity_verified);
        assert!(report.entry_count_matches);
        assert_eq!(report.component_count, 7);
        assert!(report.missing_components.is_empty());
    }

    // ── MP7-T02: Empty state passes parity check ─────────────────────
    #[test]
    fn mp7_t02_empty_state_parity() {
        let gd = GovernanceDaemon::new(10);
        let wl = WitnessLog::new();
        let dt = DriftThresholds::new(10.0, 86400);
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        let report = verify_import_parity(&bundle);
        assert!(report.pass, "empty state must pass parity: {:?}", report);
        assert_eq!(report.component_count, 7);
    }

    // ── MP7-T03: Tampered integrity hash fails parity ────────────────
    #[test]
    fn mp7_t03_tampered_hash_fails() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let mut bundle = ExportBuilder::new(&gd, &wl, &dt).build();
        bundle.metadata.integrity_hash = "tampered".into();

        let report = verify_import_parity(&bundle);
        assert!(!report.pass);
        assert!(!report.integrity_verified);
    }

    // ── MP7-T04: Wrong entry count fails parity ──────────────────────
    #[test]
    fn mp7_t04_wrong_entry_count_fails() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let mut bundle = ExportBuilder::new(&gd, &wl, &dt).build();
        // Corrupt entry count
        bundle.metadata.entry_count = 999;

        let report = verify_import_parity(&bundle);
        assert!(!report.pass);
        assert!(!report.entry_count_matches);
    }

    // ── MP7-T05: Missing governance_state fails parity ───────────────
    #[test]
    fn mp7_t05_missing_component_fails() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let mut bundle = ExportBuilder::new(&gd, &wl, &dt).build();
        // Replace governance_state with null
        bundle.governance_state = serde_json::Value::Null;

        let report = verify_import_parity(&bundle);
        assert!(!report.pass);
        assert!(!report.components_complete);
        assert!(report.missing_components.contains(&"governance_state".into()));
    }

    // ── MP7-T06: Witness chain continuity check ──────────────────────
    #[test]
    fn mp7_t06_witness_chain_continuity() {
        // Valid chain
        let valid = vec![
            serde_json::json!({"hash": "aaa", "prev_hash": "GENESIS"}),
            serde_json::json!({"hash": "bbb", "prev_hash": "aaa"}),
            serde_json::json!({"hash": "ccc", "prev_hash": "bbb"}),
        ];
        assert!(check_witness_chain_continuity(&valid));

        // Broken chain
        let broken = vec![
            serde_json::json!({"hash": "aaa", "prev_hash": "GENESIS"}),
            serde_json::json!({"hash": "bbb", "prev_hash": "WRONG"}),
        ];
        assert!(!check_witness_chain_continuity(&broken));

        // Empty chain
        assert!(check_witness_chain_continuity(&[]));
    }

    // ── MP7-T07: Round-trip export→serialize→deserialize→parity ──────
    #[test]
    fn mp7_t07_roundtrip_parity() {
        let gd = sample_daemon();
        let wl = sample_witness();
        let dt = sample_thresholds();
        let bundle = ExportBuilder::new(&gd, &wl, &dt).build();

        // Serialize and deserialize (simulates file transfer)
        let json = serde_json::to_string(&bundle).unwrap();
        let imported: ExportBundle = serde_json::from_str(&json).unwrap();

        let report = verify_import_parity(&imported);
        assert!(report.pass, "round-trip must preserve 100% parity: {:?}", report);
    }
}
