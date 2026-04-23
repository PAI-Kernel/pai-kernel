//! Doc 15: Runtime Capture Detection tests
//!
//! Tests the capture detector which monitors consent/delegation grant
//! concentration patterns and flags potential governance capture.

use pai_governance_daemon::{
    ConsentEvidence, ConsentRecord, DelegationGrant, ExpiryPolicy,
    GovernanceDaemon, RiskTier,
};
use time::OffsetDateTime;

fn make_daemon() -> GovernanceDaemon {
    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let vk = sk.verifying_key();
    GovernanceDaemon::new(100).with_author_keys("TEST_KEY", vk, Some(sk))
}

// D15-T01: No delegations = no capture risk
#[test]
fn d15_t01_no_delegations_no_risk() {
    let gov = make_daemon();
    let report = gov.capture_risk_report();
    assert!(!report.capture_detected);
    assert_eq!(report.delegate_count, 0);
}

// D15-T02: Single delegation = no capture
#[test]
fn d15_t02_single_delegation_ok() {
    let mut gov = make_daemon();
    gov.open_gate_for_testing();
    gov.grant_delegation(DelegationGrant {
        delegation_id: "D1".into(),
        principal: "AUTHOR".into(),
        delegate: "AGENT_A".into(),
        scope: vec!["CAP.READ".into()],
        domain_scope: vec![],
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
        revocable: true,
        revoked_at: None,
        issued_by_decision: 0,
    })
    .unwrap();

    let report = gov.capture_risk_report();
    assert!(!report.capture_detected);
    assert_eq!(report.delegate_count, 1);
}

// D15-T03: Multiple delegations to same entity = capture warning
#[test]
fn d15_t03_concentrated_delegations() {
    let mut gov = make_daemon();
    gov.open_gate_for_testing();

    // Grant consent for delegation
    gov.grant_consent(ConsentRecord {
        consent_id: "C_DEL".into(),
        capability_id: "CAP.DELEGATION.GRANT".into(),
        tier: RiskTier::Tier2,
        scope_text_hash: "scope".into(),
        granted_at: OffsetDateTime::now_utc(),
        revoked_at: None,
        granted_by: "AUTHOR".into(),
        evidence: ConsentEvidence::ManualConfirm,
        decision_seq: 0,
        capability_scope: vec![],
        expiry_policy: ExpiryPolicy::NoExpiry,
        grant_depth: 1,
    })
    .unwrap();

    // Grant 5 delegations to the same entity
    for i in 0..5 {
        gov.grant_delegation(DelegationGrant {
            delegation_id: format!("D{}", i),
            principal: "AUTHOR".into(),
            delegate: "AGENT_CAPTURE".into(),
            scope: vec![format!("CAP.ACTION.{}", i)],
            domain_scope: vec![],
            expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
            revocable: true,
            revoked_at: None,
            issued_by_decision: 0,
        })
        .unwrap();
    }

    let report = gov.capture_risk_report();
    assert!(
        !report.concentration_warnings.is_empty(),
        "concentrated delegations should produce warnings"
    );
}

// D15-T04: Report includes concentration ratio
#[test]
fn d15_t04_concentration_ratio() {
    let mut gov = make_daemon();
    gov.open_gate_for_testing();

    gov.grant_delegation(DelegationGrant {
        delegation_id: "D1".into(),
        principal: "AUTHOR".into(),
        delegate: "A".into(),
        scope: vec!["CAP.X".into()],
        domain_scope: vec![],
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
        revocable: true,
        revoked_at: None,
        issued_by_decision: 0,
    })
    .unwrap();

    gov.grant_delegation(DelegationGrant {
        delegation_id: "D2".into(),
        principal: "AUTHOR".into(),
        delegate: "B".into(),
        scope: vec!["CAP.Y".into()],
        domain_scope: vec![],
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
        revocable: true,
        revoked_at: None,
        issued_by_decision: 0,
    })
    .unwrap();

    let report = gov.capture_risk_report();
    // With 2 delegates each having 1 delegation, max concentration = 50%
    assert!(report.max_concentration_pct <= 50.0);
}

// D15-T05: Revoked delegations excluded from capture analysis
#[test]
fn d15_t05_revoked_excluded() {
    let mut gov = make_daemon();
    gov.open_gate_for_testing();

    gov.grant_consent(ConsentRecord {
        consent_id: "C_DEL".into(),
        capability_id: "CAP.DELEGATION.GRANT".into(),
        tier: RiskTier::Tier2,
        scope_text_hash: "scope".into(),
        granted_at: OffsetDateTime::now_utc(),
        revoked_at: None,
        granted_by: "AUTHOR".into(),
        evidence: ConsentEvidence::ManualConfirm,
        decision_seq: 0,
        capability_scope: vec![],
        expiry_policy: ExpiryPolicy::NoExpiry,
        grant_depth: 1,
    })
    .unwrap();

    gov.grant_delegation(DelegationGrant {
        delegation_id: "D1".into(),
        principal: "AUTHOR".into(),
        delegate: "AGENT".into(),
        scope: vec!["CAP.X".into()],
        domain_scope: vec![],
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
        revocable: true,
        revoked_at: None,
        issued_by_decision: 0,
    })
    .unwrap();

    gov.revoke_delegation("D1").unwrap();

    let report = gov.capture_risk_report();
    // After revocation, active delegate count should be 0
    assert_eq!(report.delegate_count, 0);
}
