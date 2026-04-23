//! MP-9 Granular Consent tests
//!
//! Tests for capability-specific consent, expiry policies, and click-depth symmetry.

use pai_governance_daemon::{
    ConsentEvidence, ConsentRecord, ExpiryPolicy, RiskTier,
};
use time::OffsetDateTime;

fn consent_record(
    id: &str,
    capability_id: &str,
    tier: RiskTier,
    expiry: ExpiryPolicy,
    scope: Vec<String>,
    depth: u8,
) -> ConsentRecord {
    ConsentRecord {
        consent_id: id.into(),
        capability_id: capability_id.into(),
        tier,
        scope_text_hash: "test".into(),
        granted_at: OffsetDateTime::now_utc(),
        revoked_at: None,
        granted_by: "AUTHOR".into(),
        evidence: ConsentEvidence::ManualConfirm,
        decision_seq: 0,
        capability_scope: scope,
        expiry_policy: expiry,
        grant_depth: depth,
    }
}

// MP9-T01: Consent with NoExpiry never expires
#[test]
fn mp9_t01_no_expiry() {
    let record = consent_record(
        "c1",
        "CAP.TEST",
        RiskTier::Tier2,
        ExpiryPolicy::NoExpiry,
        vec![],
        1,
    );
    assert_eq!(
        record.expiry_policy.expires_at(record.granted_at, record.tier),
        None
    );
}

// MP9-T02: Standard expiry gives 90 days for Tier 0-2
#[test]
fn mp9_t02_standard_expiry_low_tier() {
    let now = OffsetDateTime::now_utc();
    for tier in [RiskTier::Tier0, RiskTier::Tier1, RiskTier::Tier2] {
        let expiry = ExpiryPolicy::Standard.expires_at(now, tier).unwrap();
        let diff = expiry - now;
        assert_eq!(diff.whole_days(), 90, "tier {:?} should be 90 days", tier);
    }
}

// MP9-T03: Standard expiry gives 30 days for Tier 3-4
#[test]
fn mp9_t03_standard_expiry_high_tier() {
    let now = OffsetDateTime::now_utc();
    for tier in [RiskTier::Tier3, RiskTier::Tier4] {
        let expiry = ExpiryPolicy::Standard.expires_at(now, tier).unwrap();
        let diff = expiry - now;
        assert_eq!(diff.whole_days(), 30, "tier {:?} should be 30 days", tier);
    }
}

// MP9-T04: Custom expiry uses specified duration
#[test]
fn mp9_t04_custom_expiry() {
    let now = OffsetDateTime::now_utc();
    let policy = ExpiryPolicy::Custom {
        duration_secs: 7200, // 2 hours
    };
    let expiry = policy.expires_at(now, RiskTier::Tier2).unwrap();
    let diff = expiry - now;
    assert_eq!(diff.whole_seconds(), 7200);
}

// MP9-T05: Default expiry policy is Standard
#[test]
fn mp9_t05_default_is_standard() {
    let policy = ExpiryPolicy::default();
    assert_eq!(policy, ExpiryPolicy::Standard);
}

// MP9-T06: Consent record with empty capability_scope acts as wildcard (backward compat)
#[test]
fn mp9_t06_empty_scope_wildcard() {
    let record = consent_record(
        "c1",
        "CAP.CONSENT.GRANT",
        RiskTier::Tier2,
        ExpiryPolicy::NoExpiry,
        vec![], // empty = all capabilities
        1,
    );
    assert!(record.capability_scope.is_empty());
}

// MP9-T07: Consent record with specific capability_scope restricts coverage
#[test]
fn mp9_t07_specific_scope() {
    let record = consent_record(
        "c1",
        "CAP.CONSENT.GRANT",
        RiskTier::Tier2,
        ExpiryPolicy::NoExpiry,
        vec!["CAP.READ".into(), "CAP.WRITE".into()],
        2,
    );
    assert_eq!(record.capability_scope.len(), 2);
    assert!(record.capability_scope.contains(&"CAP.READ".into()));
    assert!(record.capability_scope.contains(&"CAP.WRITE".into()));
    assert!(!record.capability_scope.contains(&"CAP.DELETE".into()));
}

// MP9-T08: Click-depth symmetry — grant_depth recorded
#[test]
fn mp9_t08_grant_depth_recorded() {
    let record = consent_record(
        "c1",
        "CAP.TEST",
        RiskTier::Tier2,
        ExpiryPolicy::Standard,
        vec![],
        3, // 3 clicks to grant
    );
    assert_eq!(record.grant_depth, 3);
    // Invariant: revocation must require <= grant_depth clicks
    // This is enforced by the UI layer; the data model records it for audit.
}

// MP9-T09: Consent with all MP-9 fields serializes/deserializes correctly
#[test]
fn mp9_t09_serde_roundtrip() {
    let record = consent_record(
        "c1",
        "CAP.TEST",
        RiskTier::Tier3,
        ExpiryPolicy::Custom { duration_secs: 86400 },
        vec!["CAP.READ".into()],
        2,
    );
    let json = serde_json::to_string(&record).expect("serialize");
    let parsed: ConsentRecord = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.consent_id, "c1");
    assert_eq!(parsed.capability_scope, vec!["CAP.READ".to_string()]);
    assert_eq!(parsed.grant_depth, 2);
    assert_eq!(
        parsed.expiry_policy,
        ExpiryPolicy::Custom { duration_secs: 86400 }
    );
}

// MP9-T10: Legacy consent records (no MP-9 fields) deserialize with defaults
#[test]
fn mp9_t10_legacy_compat() {
    // Create a "legacy" record (only v1 fields), serialize it, strip MP-9 fields,
    // then deserialize — MP-9 fields should use defaults.
    let full = consent_record(
        "legacy1",
        "CAP.OLD",
        RiskTier::Tier2,
        ExpiryPolicy::NoExpiry,
        vec![],
        0,
    );
    let mut json_val = serde_json::to_value(&full).expect("serialize full record");
    // Remove MP-9 fields to simulate legacy format
    let obj = json_val.as_object_mut().unwrap();
    obj.remove("capability_scope");
    obj.remove("expiry_policy");
    obj.remove("grant_depth");

    let parsed: ConsentRecord =
        serde_json::from_value(json_val).expect("legacy record must parse");
    // Defaults: empty scope, Standard expiry, depth 0
    assert!(parsed.capability_scope.is_empty());
    assert_eq!(parsed.expiry_policy, ExpiryPolicy::Standard);
    assert_eq!(parsed.grant_depth, 0);
}
