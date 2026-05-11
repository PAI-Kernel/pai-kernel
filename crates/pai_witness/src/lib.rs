//! # PAI-Kernel WitnessLog (HAC Component #5)
//!
//! **Constitutional reference:** Decision Log Principles §P2 (Structured Decision Record),
//! §P4 (Append-Only & Tamper-Evident), §P5 (Reversibility Linkage),
//! P0-1 §7 (Audit Layer)
//!
//! ## Invariants
//!
//! | ID     | Invariant                                           | PAI-CD ref         |
//! |--------|-----------------------------------------------------|--------------------|
//! | WIT-I1 | Log MUST be append-only                             | Decision Log §P4   |
//! | WIT-I2 | Log MUST be tamper-evident via integrity hash chain | Decision Log §P4   |
//! | WIT-I3 | Deletion or rewriting MUST be rejected              | Decision Log §P4   |
//! | WIT-I4 | Each entry MUST include 10-field structured schema  | Decision Log §P2   |
//! | WIT-I5 | Incomplete entries MUST be rejected                 | Decision Log §P2   |
//! | WIT-I6 | Integrity failure MUST trigger breach signal        | Decision Log §P4   |
//! | WIT-I7 | Sequence MUST be monotonically increasing           | Append-only        |
//! | WIT-I8 | Hash chain starts at GENESIS                        | P0-1 §4.5          |
//!
//! ## Failure modes
//!
//! - **Hash collision:** SHA-256 collision probability negligible at operational volumes.
//! - **Serialization non-determinism:** No floats, BTreeMap only, fixed UTF-8 encoding.
//! - **Clock manipulation:** Sequence number is ordering authority, not timestamp.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

pub type SequenceNumber = u64;
pub type AuthorId = u64;
pub type SnapshotId = u64;
pub type Timestamp = u64;

/// 32-byte SHA-256 hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash256(pub [u8; 32]);

impl Serialize for Hash256 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let hex: String = self.0.iter().map(|b| format!("{:02x}", b)).collect();
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for Hash256 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        // Reject non-ASCII input · `hex.len()` returns byte count, multi-byte
        // UTF-8 would pass the `== 64` check but panic on byte-index slicing
        // at non-char boundaries (cargo-fuzz finding · Session #21 ·
        // witness_entry_parse target crash on `{"hash":"+\xd7\x97onfir..."}`).
        if !hex.is_ascii() || hex.len() != 64 {
            return Err(serde::de::Error::custom("expected 64-char ASCII hex string"));
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(serde::de::Error::custom)?;
        }
        Ok(Hash256(bytes))
    }
}

/// GENESIS hash constant — all zeros.
pub const GENESIS_HASH: Hash256 = Hash256([0u8; 32]);

// ---------------------------------------------------------------------------
// Entry schema — 10 fields per PAI-CD §P2
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionClass {
    HighImpact,
    Structural,
    Delegation,
    Constitutional,
    Upgrade,
    GovAction,
    BreachRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Initiator {
    Author(AuthorId),
    Governance,
    UpgradeProcess,
    System,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactScope {
    Identity,
    Governance,
    Objective,
    Personalization,
    Classification,
    Infrastructure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReversibilityStatus {
    Reversible { snapshot_id: SnapshotId },
    Irreversible,
}

/// Constitutional reference string (e.g. "Consent Model §P3").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalRef(pub String);

/// Confirmation record (optional).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationRecord {
    pub confirmed_by: AuthorId,
    pub confirmed_at: Timestamp,
}

/// Structured rationale — MUST NOT be empty or placeholder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredRationale(String);

impl StructuredRationale {
    /// Reject empty, whitespace-only, or placeholder rationales (< 10 chars).
    pub fn new(text: &str) -> Result<Self, WitnessError> {
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.len() < 10 {
            return Err(WitnessError::InsufficientRationale);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The 10-field structured decision record per PAI-CD §P2, plus integrity fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessEntry {
    // -- 10 schema fields --
    pub sequence: SequenceNumber,
    pub decision_class: DecisionClass,
    pub timestamp: Timestamp,
    pub initiator: Initiator,
    pub scope_of_impact: Vec<ImpactScope>,
    pub risk_tier: Option<RiskTier>,
    pub rationale: StructuredRationale,
    pub constitutional_ref: ConstitutionalRef,
    pub reversibility: ReversibilityStatus,
    pub confirmation: Option<ConfirmationRecord>,
    // -- integrity fields (implementation) --
    pub hash: Hash256,
    pub prev_hash: Hash256,
}

/// Risk tier (shared definition).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    Tier4,
}

// ---------------------------------------------------------------------------
// Builder — ensures required fields are present (WIT-I4, WIT-I5)
// ---------------------------------------------------------------------------

/// Builder for creating witness entries. All required fields must be set.
pub struct WitnessEntryBuilder {
    pub decision_class: Option<DecisionClass>,
    pub timestamp: Option<Timestamp>,
    pub initiator: Option<Initiator>,
    pub scope_of_impact: Option<Vec<ImpactScope>>,
    pub risk_tier: Option<RiskTier>,
    pub rationale: Option<StructuredRationale>,
    pub constitutional_ref: Option<ConstitutionalRef>,
    pub reversibility: Option<ReversibilityStatus>,
    pub confirmation: Option<ConfirmationRecord>,
}

impl WitnessEntryBuilder {
    pub fn new() -> Self {
        Self {
            decision_class: None,
            timestamp: None,
            initiator: None,
            scope_of_impact: None,
            risk_tier: None,
            rationale: None,
            constitutional_ref: None,
            reversibility: None,
            confirmation: None,
        }
    }

    pub fn decision_class(mut self, v: DecisionClass) -> Self { self.decision_class = Some(v); self }
    pub fn timestamp(mut self, v: Timestamp) -> Self { self.timestamp = Some(v); self }
    pub fn initiator(mut self, v: Initiator) -> Self { self.initiator = Some(v); self }
    pub fn scope_of_impact(mut self, v: Vec<ImpactScope>) -> Self { self.scope_of_impact = Some(v); self }
    pub fn risk_tier(mut self, v: RiskTier) -> Self { self.risk_tier = Some(v); self }
    pub fn rationale(mut self, v: StructuredRationale) -> Self { self.rationale = Some(v); self }
    pub fn constitutional_ref(mut self, v: ConstitutionalRef) -> Self { self.constitutional_ref = Some(v); self }
    pub fn reversibility(mut self, v: ReversibilityStatus) -> Self { self.reversibility = Some(v); self }
    pub fn confirmation(mut self, v: ConfirmationRecord) -> Self { self.confirmation = Some(v); self }

    /// Validate that all required fields are present.
    fn validate(&self) -> Result<(), WitnessError> {
        if self.decision_class.is_none() {
            return Err(WitnessError::MissingRequiredField("decision_class"));
        }
        if self.timestamp.is_none() {
            return Err(WitnessError::MissingRequiredField("timestamp"));
        }
        if self.initiator.is_none() {
            return Err(WitnessError::MissingRequiredField("initiator"));
        }
        if self.scope_of_impact.is_none() {
            return Err(WitnessError::MissingRequiredField("scope_of_impact"));
        }
        if self.rationale.is_none() {
            return Err(WitnessError::MissingRequiredField("rationale"));
        }
        if self.constitutional_ref.is_none() {
            return Err(WitnessError::MissingRequiredField("constitutional_ref"));
        }
        if self.reversibility.is_none() {
            return Err(WitnessError::MissingRequiredField("reversibility"));
        }
        Ok(())
    }
}

impl Default for WitnessEntryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WitnessError {
    InsufficientRationale,
    MissingRequiredField(&'static str),
    SequenceNotMonotonic { expected: SequenceNumber, got: SequenceNumber },
    HashMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessIntegrityError {
    pub broken_at: SequenceNumber,
    pub expected_prev_hash: Hash256,
    pub actual_prev_hash: Hash256,
}

// ---------------------------------------------------------------------------
// Hash computation — deterministic, canonical serialization
// ---------------------------------------------------------------------------

/// Serialize a field as length-prefixed UTF-8 bytes into the hasher.
fn hash_field(hasher: &mut Sha256, data: &[u8]) {
    let len = data.len() as u64;
    hasher.update(len.to_le_bytes());
    hasher.update(data);
}

#[allow(clippy::too_many_arguments)]
fn compute_hash(
    sequence: SequenceNumber,
    decision_class: &DecisionClass,
    timestamp: Timestamp,
    initiator: &Initiator,
    scope_of_impact: &[ImpactScope],
    risk_tier: Option<RiskTier>,
    rationale: &StructuredRationale,
    constitutional_ref: &ConstitutionalRef,
    reversibility: &ReversibilityStatus,
    confirmation: &Option<ConfirmationRecord>,
    prev_hash: &Hash256,
) -> Hash256 {
    let mut hasher = Sha256::new();

    // Fields in declaration order, deterministic.
    hash_field(&mut hasher, &sequence.to_le_bytes());
    hash_field(&mut hasher, format!("{:?}", decision_class).as_bytes());
    hash_field(&mut hasher, &timestamp.to_le_bytes());
    hash_field(&mut hasher, format!("{:?}", initiator).as_bytes());

    // Scope: sorted Debug representations for determinism.
    let mut scope_strs: Vec<String> = scope_of_impact.iter().map(|s| format!("{:?}", s)).collect();
    scope_strs.sort();
    hash_field(&mut hasher, scope_strs.join(",").as_bytes());

    hash_field(&mut hasher, format!("{:?}", risk_tier).as_bytes());
    hash_field(&mut hasher, rationale.as_str().as_bytes());
    hash_field(&mut hasher, constitutional_ref.0.as_bytes());
    hash_field(&mut hasher, format!("{:?}", reversibility).as_bytes());
    hash_field(&mut hasher, format!("{:?}", confirmation).as_bytes());

    // Chain linkage.
    hasher.update(prev_hash.0);

    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    Hash256(out)
}

// ---------------------------------------------------------------------------
// WitnessLog — append-only, tamper-evident
// ---------------------------------------------------------------------------

/// Append-only, hash-chained witness log.
///
/// The API exposes NO mutation methods — only `append`, read, and verify.
/// This enforces WIT-I1 and WIT-I3 at the type level.
pub struct WitnessLog {
    entries: Vec<WitnessEntry>,
}

impl WitnessLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append a new entry. Computes hash, links to prev_hash.
    pub fn append(&mut self, builder: WitnessEntryBuilder) -> Result<SequenceNumber, WitnessError> {
        builder.validate()?;

        let expected_seq = self.entries.len() as SequenceNumber + 1;
        let prev_hash = self.entries.last().map_or(GENESIS_HASH.clone(), |e| e.hash.clone());

        let decision_class = builder.decision_class.unwrap();
        let timestamp = builder.timestamp.unwrap();
        let initiator = builder.initiator.unwrap();
        let scope_of_impact = builder.scope_of_impact.unwrap();
        let risk_tier = builder.risk_tier;
        let rationale = builder.rationale.unwrap();
        let constitutional_ref = builder.constitutional_ref.unwrap();
        let reversibility = builder.reversibility.unwrap();
        let confirmation = builder.confirmation;

        let hash = compute_hash(
            expected_seq,
            &decision_class,
            timestamp,
            &initiator,
            &scope_of_impact,
            risk_tier,
            &rationale,
            &constitutional_ref,
            &reversibility,
            &confirmation,
            &prev_hash,
        );

        let entry = WitnessEntry {
            sequence: expected_seq,
            decision_class,
            timestamp,
            initiator,
            scope_of_impact,
            risk_tier,
            rationale,
            constitutional_ref,
            reversibility,
            confirmation,
            hash,
            prev_hash,
        };

        self.entries.push(entry);
        Ok(expected_seq)
    }

    /// Verify integrity of the full chain from GENESIS to head.
    pub fn verify(&self) -> Result<(), WitnessIntegrityError> {
        if self.entries.is_empty() {
            return Ok(());
        }
        self.verify_range(1, self.entries.len() as SequenceNumber)
    }

    /// Verify a specific range of the chain.
    pub fn verify_range(
        &self,
        from: SequenceNumber,
        to: SequenceNumber,
    ) -> Result<(), WitnessIntegrityError> {
        for seq in from..=to {
            let idx = (seq - 1) as usize;
            let entry = &self.entries[idx];

            // Check prev_hash linkage.
            let expected_prev = if seq == 1 {
                GENESIS_HASH.clone()
            } else {
                self.entries[idx - 1].hash.clone()
            };

            if entry.prev_hash != expected_prev {
                return Err(WitnessIntegrityError {
                    broken_at: seq,
                    expected_prev_hash: expected_prev,
                    actual_prev_hash: entry.prev_hash.clone(),
                });
            }

            // Recompute hash and verify.
            let recomputed = compute_hash(
                entry.sequence,
                &entry.decision_class,
                entry.timestamp,
                &entry.initiator,
                &entry.scope_of_impact,
                entry.risk_tier,
                &entry.rationale,
                &entry.constitutional_ref,
                &entry.reversibility,
                &entry.confirmation,
                &entry.prev_hash,
            );

            if entry.hash != recomputed {
                return Err(WitnessIntegrityError {
                    broken_at: seq,
                    expected_prev_hash: expected_prev,
                    actual_prev_hash: entry.prev_hash.clone(),
                });
            }
        }
        Ok(())
    }

    /// Read entry by sequence number (immutable view).
    pub fn get(&self, seq: SequenceNumber) -> Option<&WitnessEntry> {
        if seq == 0 || seq as usize > self.entries.len() {
            return None;
        }
        Some(&self.entries[(seq - 1) as usize])
    }

    /// Current chain head.
    pub fn head(&self) -> Option<&WitnessEntry> {
        self.entries.last()
    }

    /// Total entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Export full log for portability.
    pub fn export(&self) -> Vec<WitnessEntry> {
        self.entries.clone()
    }
}

impl Default for WitnessLog {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests — WIT-T01 through WIT-T12 per HAC-COMP-TZ-001 §5.5
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_builder() -> WitnessEntryBuilder {
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::GovAction)
            .timestamp(1000)
            .initiator(Initiator::Author(1))
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(RiskTier::Tier1)
            .rationale(StructuredRationale::new("Delegation grant issued for testing purposes").unwrap())
            .constitutional_ref(ConstitutionalRef("Consent Model §P3".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 1 })
    }

    // WIT-T01: Append to empty log → sequence 1, prev_hash = GENESIS
    #[test]
    fn wit_t01_append_empty_log_genesis() {
        let mut log = WitnessLog::new();
        let seq = log.append(valid_builder()).unwrap();
        assert_eq!(seq, 1);
        let entry = log.get(1).unwrap();
        assert_eq!(entry.prev_hash, GENESIS_HASH);
    }

    // WIT-T02: Append to non-empty log → prev_hash = head.hash
    #[test]
    fn wit_t02_append_chains_to_head() {
        let mut log = WitnessLog::new();
        log.append(valid_builder()).unwrap();
        let first_hash = log.head().unwrap().hash.clone();
        log.append(valid_builder().timestamp(2000)).unwrap();
        let second = log.get(2).unwrap();
        assert_eq!(second.prev_hash, first_hash);
    }

    // WIT-T03: verify() on intact chain → Ok
    #[test]
    fn wit_t03_verify_intact_chain() {
        let mut log = WitnessLog::new();
        for i in 0..5 {
            log.append(valid_builder().timestamp(1000 + i)).unwrap();
        }
        assert!(log.verify().is_ok());
    }

    // WIT-T04: Tamper with entry → verify() detects broken link
    #[test]
    fn wit_t04_tamper_detected() {
        let mut log = WitnessLog::new();
        for i in 0..3 {
            log.append(valid_builder().timestamp(1000 + i)).unwrap();
        }

        // Tamper: mutate hash of entry 2 directly.
        log.entries[1].hash.0[0] ^= 0xFF;

        let result = log.verify();
        assert!(result.is_err());
        // Entry 3 should detect mismatch (its prev_hash won't match entry 2's tampered hash).
        let err = result.unwrap_err();
        // Could break at entry 2 (recomputed hash mismatch) or entry 3 (prev_hash mismatch).
        assert!(err.broken_at == 2 || err.broken_at == 3);
    }

    // WIT-T05: Delete entry from middle → verify() fails
    #[test]
    fn wit_t05_delete_detected() {
        let mut log = WitnessLog::new();
        for i in 0..4 {
            log.append(valid_builder().timestamp(1000 + i)).unwrap();
        }

        // Simulate deletion by removing entry at index 1 (sequence 2).
        log.entries.remove(1);

        // Now entry at index 1 is old sequence 3, but verify_range expects
        // contiguous hashes — the prev_hash of what's now at index 1
        // won't match what's at index 0.
        let result = log.verify();
        assert!(result.is_err());
    }

    // WIT-T06: Empty rationale → Error(InsufficientRationale)
    #[test]
    fn wit_t06_empty_rationale_rejected() {
        assert!(StructuredRationale::new("").is_err());
        assert!(StructuredRationale::new("   ").is_err());
        assert!(StructuredRationale::new("short").is_err()); // < 10 chars
    }

    // WIT-T07: Missing required field → Error(MissingRequiredField)
    #[test]
    fn wit_t07_missing_field_rejected() {
        let mut log = WitnessLog::new();
        // Builder with no fields set.
        let result = log.append(WitnessEntryBuilder::new());
        assert!(matches!(result, Err(WitnessError::MissingRequiredField(_))));
    }

    // WIT-T08: Non-monotonic sequence → enforced by implementation
    // (sequence is computed, not user-supplied, so this tests internal consistency)
    #[test]
    fn wit_t08_sequence_monotonic() {
        let mut log = WitnessLog::new();
        for i in 0..5 {
            log.append(valid_builder().timestamp(1000 + i)).unwrap();
        }
        for i in 0..5 {
            assert_eq!(log.entries[i].sequence, (i + 1) as u64);
        }
    }

    // WIT-T09: Export + re-verify → chain still valid
    #[test]
    fn wit_t09_export_reverify() {
        let mut log = WitnessLog::new();
        for i in 0..5 {
            log.append(valid_builder().timestamp(1000 + i)).unwrap();
        }

        let exported = log.export();
        // Reconstruct a log from exported entries and verify.
        let mut reimported = WitnessLog::new();
        reimported.entries = exported;
        assert!(reimported.verify().is_ok());
    }

    // WIT-T10: 10,000 entries → verify() completes, integrity holds
    #[test]
    fn wit_t10_stress_10k_entries() {
        let mut log = WitnessLog::new();
        for i in 0..10_000u64 {
            log.append(valid_builder().timestamp(i)).unwrap();
        }
        assert_eq!(log.len(), 10_000);
        assert!(log.verify().is_ok());
    }

    // WIT-T11: Concurrent append attempt → sequential ordering enforced
    // (In single-threaded reference impl, this is enforced by &mut self.
    //  We verify that two sequential appends produce correct ordering.)
    #[test]
    fn wit_t11_sequential_ordering() {
        let mut log = WitnessLog::new();
        let s1 = log.append(valid_builder().timestamp(100)).unwrap();
        let s2 = log.append(valid_builder().timestamp(200)).unwrap();
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
        assert_eq!(log.get(2).unwrap().prev_hash, log.get(1).unwrap().hash);
    }

    // WIT-T12: Rewrite attempt via trait → no mutation method exists
    // (Verified at compile time: WitnessLog exposes no method to modify
    //  an existing entry. This test documents the API surface.)
    #[test]
    fn wit_t12_no_mutation_api() {
        let mut log = WitnessLog::new();
        log.append(valid_builder()).unwrap();

        // The only way to get an entry is via immutable reference.
        let entry = log.get(1).unwrap();
        assert_eq!(entry.sequence, 1);

        // There is no set(), update(), delete(), or &mut access method.
        // This is enforced by the type system — WIT-I3 by construction.
    }
}
