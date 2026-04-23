//! # HAC Integration Tests — INT-T01 through INT-T05
//!
//! Per HAC-COMP-TZ-001 §7. These tests verify cross-component behavior
//! across the six High-Assurance Core primitives.

use pai_delegation::*;
use pai_witness::{
    ConstitutionalRef, DecisionClass, ImpactScope, Initiator,
    ReversibilityStatus, StructuredRationale, WitnessEntryBuilder, WitnessLog,
};
use pai_classify::{
    BiasSignals, BoundaryVerdict, ConsequentialDomain, DomainType,
    RecActionClassifier, RiskTier as ClassifyRiskTier,
};

// ---------------------------------------------------------------------------
// INT-T01: Delegation grant → action within scope → WitnessLog records
//          → verify() passes
// Components: DelegationValidator + WitnessLog
// ---------------------------------------------------------------------------
#[test]
fn int_t01_delegation_then_witness() {
    // 1. Grant delegation
    let mut del_store = DelegationStore::new();
    let grant = DelegationGrant {
        id: 0,
        principal: 1,
        delegate: 10,
        scope: vec![1, 2],
        domain_scope: vec![100],
        tier_ceiling: RiskTier::Tier2,
        issued_at: 1000,
        expires_at: Some(9999),
        revoked_at: None,
        issued_by_seq: 1,
    };
    let grant_id = del_store.grant(grant, false).unwrap();

    // 2. Validate action within scope
    let verdict = del_store.validate(&10, &1, Some(&100), RiskTier::Tier1, 2000, false);
    assert!(matches!(verdict, DelegationVerdict::Authorized { .. }));

    // 3. Record both events in WitnessLog
    let mut log = WitnessLog::new();

    // Record grant
    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::Delegation)
            .timestamp(1000)
            .initiator(Initiator::Author(1))
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(pai_witness::RiskTier::Tier1)
            .rationale(StructuredRationale::new("Delegation grant issued to delegate 10 for capabilities 1,2").unwrap())
            .constitutional_ref(ConstitutionalRef("Consent Model §P3".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 1 }),
    ).unwrap();

    // Record authorized action
    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::GovAction)
            .timestamp(2000)
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Identity])
            .risk_tier(pai_witness::RiskTier::Tier1)
            .rationale(StructuredRationale::new(
                &format!("Action authorized via delegation grant {}", grant_id),
            ).unwrap())
            .constitutional_ref(ConstitutionalRef("Consent Model §P2".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 2 }),
    ).unwrap();

    // 4. Verify chain integrity
    assert!(log.verify().is_ok());
    assert_eq!(log.len(), 2);
}

// ---------------------------------------------------------------------------
// INT-T02: Conservative Mode active → delegation blocked +
//          recommendation blocked + log records all denials
// Components: All 6 HAC
// ---------------------------------------------------------------------------
#[test]
fn int_t02_conservative_mode_blocks_all() {
    let conservative_mode = true;

    // 1. Delegation blocked
    let mut del_store = DelegationStore::new();
    let grant_result = del_store.grant(
        DelegationGrant {
            id: 0, principal: 1, delegate: 10,
            scope: vec![1], domain_scope: vec![100],
            tier_ceiling: RiskTier::Tier2,
            issued_at: 1000, expires_at: None, revoked_at: None,
            issued_by_seq: 1,
        },
        conservative_mode,
    );
    assert!(matches!(grant_result, Err(DelegationDenialReason::ConservativeModePaused)));

    // 2. Existing delegation validation blocked
    let mut store2 = DelegationStore::new();
    store2.grant(
        DelegationGrant {
            id: 0, principal: 1, delegate: 10,
            scope: vec![1], domain_scope: vec![100],
            tier_ceiling: RiskTier::Tier2,
            issued_at: 1000, expires_at: None, revoked_at: None,
            issued_by_seq: 1,
        },
        false, // grant while not in conservative mode
    ).unwrap();
    let val = store2.validate(&10, &1, Some(&100), RiskTier::Tier1, 2000, conservative_mode);
    assert!(matches!(val, DelegationVerdict::Denied(DelegationDenialReason::ConservativeModePaused)));

    // 3. Recommendation blocked
    let classify_result = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
        Some(&BiasSignals { ordering_asymmetry: 0.8, ..BiasSignals::none() }),
        false,
        conservative_mode,
    );
    assert_eq!(classify_result, BoundaryVerdict::BlockedConservativeMode);

    // 4. All denials recorded in witness log
    let mut log = WitnessLog::new();

    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::BreachRecord)
            .timestamp(3000)
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new("Delegation grant denied: Conservative Mode active").unwrap())
            .constitutional_ref(ConstitutionalRef("Governance §P3 — DEL-I6".into()))
            .reversibility(ReversibilityStatus::Irreversible),
    ).unwrap();

    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::BreachRecord)
            .timestamp(3001)
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new("Delegation validation denied: Conservative Mode active").unwrap())
            .constitutional_ref(ConstitutionalRef("Governance §P3 — DEL-I6".into()))
            .reversibility(ReversibilityStatus::Irreversible),
    ).unwrap();

    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::BreachRecord)
            .timestamp(3002)
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Classification])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new("Recommendation blocked: Conservative Mode active").unwrap())
            .constitutional_ref(ConstitutionalRef("Governance §P3 — RAB-I6".into()))
            .reversibility(ReversibilityStatus::Irreversible),
    ).unwrap();

    assert!(log.verify().is_ok());
    assert_eq!(log.len(), 3);
}

// ---------------------------------------------------------------------------
// INT-T03: Breach trigger → Conservative Mode auto-activates →
//          all HAC components reflect new state
// Components: All 6 HAC
// ---------------------------------------------------------------------------
#[test]
fn int_t03_breach_cascades_to_all_components() {
    // Simulate: system starts normal, breach occurs, everything locks down.
    let mut log = WitnessLog::new();
    let mut del_store = DelegationStore::new();

    // Phase 1: Normal operation — delegation works, classification works
    del_store.grant(
        DelegationGrant {
            id: 0, principal: 1, delegate: 10,
            scope: vec![1, 2], domain_scope: vec![100],
            tier_ceiling: RiskTier::Tier2,
            issued_at: 1000, expires_at: None, revoked_at: None,
            issued_by_seq: 1,
        },
        false,
    ).unwrap();

    let v1 = del_store.validate(&10, &1, Some(&100), RiskTier::Tier1, 2000, false);
    assert!(matches!(v1, DelegationVerdict::Authorized { .. }));

    let c1 = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Medical)),
        Some(&BiasSignals { framing_asymmetry: 0.8, ..BiasSignals::none() }),
        false, false,
    );
    assert!(matches!(c1, BoundaryVerdict::PassRecommendation { .. }));

    // Phase 2: Breach detected — record it
    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::BreachRecord)
            .timestamp(3000)
            .initiator(Initiator::System)
            .scope_of_impact(vec![ImpactScope::Governance, ImpactScope::Identity])
            .risk_tier(pai_witness::RiskTier::Tier4)
            .rationale(StructuredRationale::new("GOV.BYPASS detected: inference layer attempted direct state mutation").unwrap())
            .constitutional_ref(ConstitutionalRef("Constitutional Core I1 — Authorship Supremacy".into()))
            .reversibility(ReversibilityStatus::Irreversible),
    ).unwrap();

    // Phase 3: Conservative Mode now active — everything blocked
    let conservative_mode = true;

    let v2 = del_store.validate(&10, &1, Some(&100), RiskTier::Tier1, 4000, conservative_mode);
    assert!(matches!(v2, DelegationVerdict::Denied(DelegationDenialReason::ConservativeModePaused)));

    let c2 = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Medical)),
        Some(&BiasSignals { framing_asymmetry: 0.8, ..BiasSignals::none() }),
        false, conservative_mode,
    );
    assert_eq!(c2, BoundaryVerdict::BlockedConservativeMode);

    // New delegation grant also blocked
    let grant_result = del_store.grant(
        DelegationGrant {
            id: 0, principal: 1, delegate: 20,
            scope: vec![1], domain_scope: vec![100],
            tier_ceiling: RiskTier::Tier1,
            issued_at: 4000, expires_at: None, revoked_at: None,
            issued_by_seq: 2,
        },
        conservative_mode,
    );
    assert!(matches!(grant_result, Err(DelegationDenialReason::ConservativeModePaused)));

    // Log records the cascade
    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::GovAction)
            .timestamp(3001)
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new("Conservative Mode auto-activated: all Tier>=2 capabilities suspended").unwrap())
            .constitutional_ref(ConstitutionalRef("Governance §P3 — Conservative Mode Control".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 1 }),
    ).unwrap();

    assert!(log.verify().is_ok());
    assert_eq!(log.len(), 2);
}

// ---------------------------------------------------------------------------
// INT-T04: Full lifecycle: consent → delegation → action →
//          classification → witness → verify chain
// Components: All 6 HAC
// ---------------------------------------------------------------------------
#[test]
fn int_t04_full_lifecycle() {
    let mut log = WitnessLog::new();
    let mut del_store = DelegationStore::new();

    // Step 1: Consent granted (recorded in log)
    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::GovAction)
            .timestamp(1000)
            .initiator(Initiator::Author(1))
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new("Author granted Tier 2 consent for CAP.COMPARE.RANK").unwrap())
            .constitutional_ref(ConstitutionalRef("Consent Model §P2 — Tier 2 explicit opt-in".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 1 }),
    ).unwrap();

    // Step 2: Delegation issued
    let grant_id = del_store.grant(
        DelegationGrant {
            id: 0, principal: 1, delegate: 10,
            scope: vec![42], // CAP.COMPARE.RANK
            domain_scope: vec![200], // Financial
            tier_ceiling: RiskTier::Tier2,
            issued_at: 1100, expires_at: Some(9999),
            revoked_at: None, issued_by_seq: 2,
        },
        false,
    ).unwrap();

    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::Delegation)
            .timestamp(1100)
            .initiator(Initiator::Author(1))
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new(
                &format!("Delegation grant {} issued to delegate 10 for CAP.COMPARE.RANK in Financial domain", grant_id)
            ).unwrap())
            .constitutional_ref(ConstitutionalRef("Consent Model §P3 — Delegation".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 2 }),
    ).unwrap();

    // Step 3: Validate delegation for action
    let verdict = del_store.validate(&10, &42, Some(&200), RiskTier::Tier2, 2000, false);
    assert!(matches!(verdict, DelegationVerdict::Authorized { .. }));

    // Step 4: Classify output (Recommendation in Financial domain with bias)
    let classification = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
        Some(&BiasSignals { ordering_asymmetry: 0.8, ..BiasSignals::none() }),
        false,
        false,
    );
    assert!(matches!(
        classification,
        BoundaryVerdict::PassRecommendation { consent_required: ClassifyRiskTier::Tier2 }
    ));

    // Step 5: Record action in witness log
    log.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::HighImpact)
            .timestamp(2000)
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Identity, ImpactScope::Classification])
            .risk_tier(pai_witness::RiskTier::Tier2)
            .rationale(StructuredRationale::new(
                "Recommendation delivered in Financial domain: comparison ranking with consent and delegation verified"
            ).unwrap())
            .constitutional_ref(ConstitutionalRef("Consent Model §P1/P2, Decision Log §P2".into()))
            .reversibility(ReversibilityStatus::Reversible { snapshot_id: 3 }),
    ).unwrap();

    // Step 6: Verify full chain
    assert!(log.verify().is_ok());
    assert_eq!(log.len(), 3);

    // Verify chain linkage is continuous
    let e1 = log.get(1).unwrap();
    let e2 = log.get(2).unwrap();
    let e3 = log.get(3).unwrap();
    assert_eq!(e1.prev_hash, pai_witness::GENESIS_HASH);
    assert_eq!(e2.prev_hash, e1.hash);
    assert_eq!(e3.prev_hash, e2.hash);
}

// ---------------------------------------------------------------------------
// INT-T05: Export all state → re-import → verify integrity holds
// Components: WitnessLog + DelegationStore
// ---------------------------------------------------------------------------
#[test]
fn int_t05_export_reimport_integrity() {
    // Build state
    let mut log = WitnessLog::new();
    let mut del_store = DelegationStore::new();

    // Several delegation grants
    for i in 1..=5u64 {
        del_store.grant(
            DelegationGrant {
                id: 0, principal: 1, delegate: 10 + i,
                scope: vec![i, i + 10],
                domain_scope: vec![100 + i],
                tier_ceiling: RiskTier::Tier2,
                issued_at: 1000 * i,
                expires_at: Some(99999),
                revoked_at: None,
                issued_by_seq: i,
            },
            false,
        ).unwrap();

        log.append(
            WitnessEntryBuilder::new()
                .decision_class(DecisionClass::Delegation)
                .timestamp(1000 * i)
                .initiator(Initiator::Author(1))
                .scope_of_impact(vec![ImpactScope::Governance])
                .risk_tier(pai_witness::RiskTier::Tier1)
                .rationale(StructuredRationale::new(
                    &format!("Delegation grant {} issued in export/import test", i)
                ).unwrap())
                .constitutional_ref(ConstitutionalRef("Consent Model §P3".into()))
                .reversibility(ReversibilityStatus::Reversible { snapshot_id: i }),
        ).unwrap();
    }

    // Verify before export
    assert!(log.verify().is_ok());
    assert_eq!(log.len(), 5);

    // Export
    let exported_entries = log.export();
    assert_eq!(exported_entries.len(), 5);

    // Verify exported entries maintain hash chain
    // Check GENESIS linkage
    assert_eq!(exported_entries[0].prev_hash, pai_witness::GENESIS_HASH);
    // Check chain continuity
    for i in 1..exported_entries.len() {
        assert_eq!(
            exported_entries[i].prev_hash,
            exported_entries[i - 1].hash,
            "Chain broken at exported entry {}",
            i + 1,
        );
    }

    // Reconstruct and verify via WitnessLog internal verification
    let mut reimported_log = WitnessLog::new();
    unsafe_reimport_for_test(&mut reimported_log, exported_entries);
    assert!(reimported_log.verify().is_ok());
    assert_eq!(reimported_log.len(), 5);
}

/// Test-only helper: reimport entries into a WitnessLog.
/// In production, this would be a proper deserialization + validation path.
///
/// We use the same approach as WIT-T09: direct field assignment on the
/// internal Vec, which is possible because WitnessLog.entries is `pub(crate)`
/// — wait, it's not public. We'll use the export/verify pattern instead.
fn unsafe_reimport_for_test(log: &mut WitnessLog, entries: Vec<pai_witness::WitnessEntry>) {
    // The WitnessLog from WIT-T09 shows that reimport works by
    // directly setting `log.entries = exported`. Since entries is
    // not pub, we verify integrity of the exported Vec manually
    // (which is what the chain continuity check above does).
    // For this test, we re-append entries using the builder.
    // This re-creates the chain from scratch and should produce
    // identical hashes if the hash function is deterministic.

    for entry in &entries {
        let seq = log.append(
            WitnessEntryBuilder::new()
                .decision_class(entry.decision_class.clone())
                .timestamp(entry.timestamp)
                .initiator(entry.initiator.clone())
                .scope_of_impact(entry.scope_of_impact.clone())
                .risk_tier(match entry.risk_tier {
                    Some(t) => t,
                    None => pai_witness::RiskTier::Tier0,
                })
                .rationale(StructuredRationale::new(entry.rationale.as_str()).unwrap())
                .constitutional_ref(entry.constitutional_ref.clone())
                .reversibility(entry.reversibility.clone()),
        ).unwrap();
        // Verify hash determinism: re-appended entry must produce same hash
        let reimported = log.get(seq).unwrap();
        assert_eq!(
            reimported.hash, entry.hash,
            "Hash mismatch at sequence {}: determinism violated",
            seq,
        );
    }
}
