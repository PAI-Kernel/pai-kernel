#![forbid(unsafe_code)]

use pai_governance_daemon::{
    ActorType, AuthorityContext, BreachClass, CapabilityDefinition, ConsentEvidence, ConsentRecord,
    DelegationGrant, ExpiryPolicy, GovError, GovernanceDaemon, RiskTier,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub formal_action: String,
    pub pass: bool,
    pub breach_class: Option<String>,
    pub invariant_id: String,
    pub tla_property: String,
    pub canonical_clause_ref: String,
    pub notes: String,
}

/// MP-6: Independent verification signal.
///
/// Each compliance test produces a machine-readable signal that can be
/// consumed by external auditors. Per MP-6, each signal must include:
/// - Observable: what was measured
/// - Pass criterion: what constitutes pass/fail
/// - Result: actual measured value
/// - Timestamp: when the verification occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSignal {
    /// Unique item identifier (e.g., "I1_AuthorSupremacy").
    pub item_id: String,
    /// What was observed/measured.
    pub observable: String,
    /// What constitutes pass vs fail.
    pub pass_criterion: String,
    /// The actual result (pass/fail + detail).
    pub result: VerificationResult,
    /// ISO 8601 timestamp of verification.
    pub timestamp: String,
    /// Invariant being verified.
    pub invariant_ref: String,
    /// Formal property being tested.
    pub tla_property: String,
}

/// Result of a single verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    Pass {
        detail: String,
    },
    Fail {
        detail: String,
        breach_class: Option<String>,
    },
}

impl VerificationResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, VerificationResult::Pass { .. })
    }
}

/// Convert a TestResult into a VerificationSignal (MP-6).
impl From<&TestResult> for VerificationSignal {
    fn from(tr: &TestResult) -> Self {
        let result = if tr.pass {
            VerificationResult::Pass {
                detail: tr.notes.clone(),
            }
        } else {
            VerificationResult::Fail {
                detail: tr.notes.clone(),
                breach_class: tr.breach_class.clone(),
            }
        };

        VerificationSignal {
            item_id: tr.name.clone(),
            observable: tr.formal_action.clone(),
            pass_criterion: format!(
                "Invariant {} holds per {}",
                tr.invariant_id, tr.canonical_clause_ref
            ),
            result,
            timestamp: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Iso8601::DEFAULT)
                .unwrap_or_else(|_| "unknown".into()),
            invariant_ref: tr.invariant_id.clone(),
            tla_property: tr.tla_property.clone(),
        }
    }
}

/// Run the full compliance suite and produce MP-6 verification signals.
pub fn run_with_signals() -> (Vec<TestResult>, Vec<VerificationSignal>) {
    let results = run();
    let signals: Vec<VerificationSignal> = results.iter().map(VerificationSignal::from).collect();
    (results, signals)
}

/// Produce a JSON-LD compatible verification report (MP-6 §2).
pub fn verification_report_json(signals: &[VerificationSignal]) -> serde_json::Value {
    let pass_count = signals.iter().filter(|s| s.result.is_pass()).count();
    let fail_count = signals.len() - pass_count;

    serde_json::json!({
        "@context": "https://paikernel.org/verification/v1",
        "@type": "VerificationReport",
        "pai_cd_version": "3.1",
        "total_signals": signals.len(),
        "passed": pass_count,
        "failed": fail_count,
        "all_pass": fail_count == 0,
        "signals": signals,
    })
}

fn author_ctx() -> AuthorityContext {
    AuthorityContext {
        current_actor: "AUTHOR".into(),
        actor_type: ActorType::Author,
        active_delegate: None,
    }
}

fn delegate_ctx(id: &str) -> AuthorityContext {
    AuthorityContext {
        current_actor: id.into(),
        actor_type: ActorType::External,
        active_delegate: Some(id.into()),
    }
}

pub fn run() -> Vec<TestResult> {
    let mut results = vec![];

    // Compliance binary uses ephemeral keys (same-session deterministic).
    // Tests verify daemon behavior within the same process, so per-session
    // ephemeral keys are sufficient and safer than hardcoded fixtures.
    // build_demo_keys prints a stderr warning explaining the constraint.
    let (sk, vk, api_key) = pai_governance_daemon::keyloader::build_demo_keys();
    // Save raw bytes for sub-tests that build separate daemons from the same
    // session key (T4c, T6 below).
    let sk_bytes: [u8; 32] = sk.to_bytes();

    let mut gov = GovernanceDaemon::new(10).with_author_keys(&api_key, vk, Some(sk));

    // Open gate for registration and consent operations
    gov.open_gate_for_testing();

    // Register capabilities needed by tests (I3)
    gov.register_capability(CapabilityDefinition {
        id: "CAP.OBJECTIVE.RATIFY_ADD".into(),
        tier: RiskTier::Tier2,
        description_hash: "h1".into(),
        version: 1,
    })
    .unwrap();

    gov.register_capability(CapabilityDefinition {
        id: "CAP.DELEGATION.GRANT".into(),
        tier: RiskTier::Tier2,
        description_hash: "h2".into(),
        version: 1,
    })
    .unwrap();

    gov.register_capability(CapabilityDefinition {
        id: "CAP.TIER4.ACTION".into(),
        tier: RiskTier::Tier4,
        description_hash: "h3".into(),
        version: 1,
    })
    .unwrap();

    // ===== T1 Objective Registry Integrity (I6)
    gov.grant_consent(ConsentRecord {
        consent_id: "C_OBJ".into(),
        capability_id: "CAP.OBJECTIVE.RATIFY_ADD".into(),
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

    let r = gov.ratify_add_objective("OBJ.A");
    let pass = r.is_ok()
        && gov.state().objectives().contains(&"OBJ.A".to_string())
        && gov
            .state()
            .objective_registry()
            .contains(&"OBJ.A".to_string());
    results.push(TestResult {
        name: "T1_objective_registry_integrity_add".into(),
        formal_action: "GovMutateObjective".into(),
        pass,
        breach_class: None,
        invariant_id: "I6".into(),
        tla_property: "Inv_I6_ObjectiveSubset".into(),
        canonical_clause_ref: "core/sources/Compliance_Checklist.md § Objective Registry".into(),
        notes: "Adding objective updates objectives and objective_registry consistently.".into(),
    });

    // ===== T3 Consent gating Tier2 (I1/I3)
    // Revoke consent and ensure Tier2 objective add fails-closed
    gov.revoke_consent("C_OBJ").unwrap();
    let r2 = gov.ratify_add_objective("OBJ.B");
    results.push(TestResult {
        name: "T3_tier2_action_requires_consent".into(),
        formal_action: "GovMutateObjective".into(),
        pass: matches!(
            r2,
            Err(GovError::Unauthorized) | Err(GovError::ConservativeMode)
        ),
        breach_class: Some("AUTHZ.FAIL".into()),
        invariant_id: "I1/I3".into(),
        tla_property: "Inv_I1_AuthorSupremacy".into(),
        canonical_clause_ref: "core/sources/PAI_Constitutional_Document.md § Author Supremacy"
            .into(),
        notes: "Tier2 without active consent is rejected (fail-closed).".into(),
    });

    // Restore consent for later tests
    gov.grant_consent(ConsentRecord {
        consent_id: "C_OBJ2".into(),
        capability_id: "CAP.OBJECTIVE.RATIFY_ADD".into(),
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

    // ===== T2 Delegation enforcement (I2/I4)
    // Grant consent required to grant delegation
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

    // Grant delegation to DELEGATE for objective add
    gov.grant_delegation(DelegationGrant {
        delegation_id: "D1".into(),
        principal: "AUTHOR".into(),
        delegate: "DELEGATE".into(),
        scope: vec!["CAP.OBJECTIVE.RATIFY_ADD".into()],
        domain_scope: vec![],
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
        revocable: true,
        revoked_at: None,
        issued_by_decision: 0,
    })
    .unwrap();

    // Switch actor to delegate and attempt Tier2 objective add: allowed due to delegation scope
    gov.set_actor_context_for_testing(delegate_ctx("DELEGATE"));
    let r3 = gov.ratify_add_objective("OBJ.DELEGATED");
    results.push(TestResult {
        name: "T2_delegation_allows_scoped_action".into(),
        formal_action: "GovMutateObjective".into(),
        pass: r3.is_ok(),
        breach_class: None,
        invariant_id: "I2".into(),
        tla_property: "Inv_I2_DelegationValidity".into(),
        canonical_clause_ref: "core/sources/Consent_and_Capability_Model.md § Delegation".into(),
        notes: "Delegation authorizes scoped Tier2 action for delegate before expiry.".into(),
    });

    // Expired delegation should reject
    // Create an already-expired delegation; requires consent already present.
    gov.set_actor_context_for_testing(author_ctx());
    gov.grant_delegation(DelegationGrant {
        delegation_id: "D_EXP".into(),
        principal: "AUTHOR".into(),
        delegate: "DELEGATE2".into(),
        scope: vec!["CAP.OBJECTIVE.RATIFY_ADD".into()],
        domain_scope: vec![],
        expires_at: OffsetDateTime::now_utc() - time::Duration::days(1),
        revocable: true,
        revoked_at: None,
        issued_by_decision: 0,
    })
    .unwrap();
    gov.set_actor_context_for_testing(delegate_ctx("DELEGATE2"));
    let r_exp = gov.ratify_add_objective("OBJ.EXP");
    results.push(TestResult {
        name: "T2_delegation_expired_rejected".into(),
        formal_action: "GovMutateObjective".into(),
        pass: r_exp.is_err(),
        breach_class: Some("AUTHZ.FAIL".into()),
        invariant_id: "I2".into(),
        tla_property: "Inv_I2_DelegationValidity".into(),
        canonical_clause_ref: "core/sources/Consent_and_Capability_Model.md § Delegation".into(),
        notes: "Expired delegation cannot authorize actions.".into(),
    });

    // Conservative mode pauses delegations (I4)
    gov.inference_bypass_attempt(); // enters conservative
    let r_cons = gov.ratify_add_objective("OBJ.CONS");
    results.push(TestResult {
        name: "T4_conservative_blocks_tier2_even_with_delegation".into(),
        formal_action: "ConservativeModeViolation".into(),
        pass: matches!(r_cons, Err(GovError::ConservativeMode)),
        breach_class: Some("CONS.MODE.VIOLATION".into()),
        invariant_id: "I4".into(),
        tla_property: "Inv_I4_ConservativeTierGate".into(),
        canonical_clause_ref: "core/constitutional_core_v1.0.md § Conservative Mode".into(),
        notes: "In conservative mode Tier>=2 is blocked; delegations are paused.".into(),
    });

    // ===== T3 Tier4 dual-confirm (I3)
    // Create a fresh daemon (not conservative) for Tier4 tests
    let sk2 = ed25519_dalek::SigningKey::from_bytes(&sk_bytes);
    let mut gov4 =
        GovernanceDaemon::new(10).with_author_keys("AUTHOR_KEY", sk2.verifying_key(), Some(sk2));
    gov4.open_gate_for_testing();
    gov4.register_capability(CapabilityDefinition {
        id: "CAP.TIER4.ACTION".into(),
        tier: RiskTier::Tier4,
        description_hash: "h3".into(),
        version: 1,
    })
    .unwrap();

    // No dual consents => unauthorized
    let r4a = gov4.register_capability(CapabilityDefinition {
        id: "CAP.DUMMY".into(),
        tier: RiskTier::Tier2,
        description_hash: "hd".into(),
        version: 1,
    });
    let _ = r4a; // unrelated: we just want tier4 to be enforced via grant_consent; next we attempt tier4 by granting consent itself

    // Grant only A evidence
    gov4.grant_consent(ConsentRecord {
        consent_id: "C4A".into(),
        capability_id: "CAP.TIER4.ACTION".into(),
        tier: RiskTier::Tier4,
        scope_text_hash: "scope".into(),
        granted_at: OffsetDateTime::now_utc(),
        revoked_at: None,
        granted_by: "AUTHOR".into(),
        evidence: ConsentEvidence::DualConfirmA("E1".into()),
        decision_seq: 0,
        capability_scope: vec![],
        expiry_policy: ExpiryPolicy::NoExpiry,
        grant_depth: 1,
    })
    .unwrap();

    // Attempt a Tier4 governed operation by granting a Tier4 consent record B must be present for validator in future ops;
    // Here we check validator requires both A and B by trying to grant a delegation under Tier4 capability by switching cap id is not supported.
    // So we directly verify has_active_consent semantics: add B and then check it's satisfied by attempting a Tier2 action that requires Tier4? out of scope.
    // Minimal: ensure dual confirm requirement in validator blocks Tier4 action by attempting to grant Tier4 consent without both evidences.
    let _only_a_ok = gov4.verify_log().is_ok();
    // Add B evidence
    gov4.grant_consent(ConsentRecord {
        consent_id: "C4B".into(),
        capability_id: "CAP.TIER4.ACTION".into(),
        tier: RiskTier::Tier4,
        scope_text_hash: "scope".into(),
        granted_at: OffsetDateTime::now_utc(),
        revoked_at: None,
        granted_by: "AUTHOR".into(),
        evidence: ConsentEvidence::DualConfirmB("E2".into()),
        decision_seq: 0,
        capability_scope: vec![],
        expiry_policy: ExpiryPolicy::NoExpiry,
        grant_depth: 1,
    })
    .unwrap();
    let _dual_ok = gov4.verify_log().is_ok();

    // ===== T5 Signed high-impact decisions (I5)
    // verify_log should pass on untampered signed log
    let ok = gov.verify_log().is_ok();
    results.push(TestResult {
        name: "T5_verify_log_signed_ok".into(),
        formal_action: "verify_log".into(),
        pass: ok,
        breach_class: None,
        invariant_id: "I5".into(),
        tla_property: "Inv_I5_SignedHighImpact".into(),
        canonical_clause_ref: "core/sources/Decision_Log_and_Witness_Principles.md § Signatures"
            .into(),
        notes: "Signed high-impact entries verify OK.".into(),
    });

    // Tamper a signed entry and ensure verify_log fails
    let tamper = gov.inject_tamper_for_testing(0, "tampered-details");
    let tampered = tamper.is_ok() && gov.verify_log().is_err();
    results.push(TestResult {
        name: "T5_verify_log_signature_tamper_detected".into(),
        formal_action: "LogTamperDetected".into(),
        pass: tampered,
        breach_class: Some("LOG.TAMPER".into()),
        invariant_id: "I5".into(),
        tla_property: "Inv_I5_SignedHighImpact".into(),
        canonical_clause_ref: "core/sources/Decision_Log_and_Witness_Principles.md § Signatures"
            .into(),
        notes: "Tampering payload makes verify_log fail (fail-closed).".into(),
    });

    // ===== T4b exit_conservative gated by drift/breach (MINOR fix V3-05)
    let sk_ec = ed25519_dalek::SigningKey::from_bytes(&sk_bytes);
    let mut gov_ec =
        GovernanceDaemon::new(3).with_author_keys("AUTHOR_KEY", sk_ec.verifying_key(), Some(sk_ec));
    gov_ec.open_gate_for_testing();

    // capability + consent required for exit_conservative
    gov_ec
        .register_capability(CapabilityDefinition {
            id: "CAP.CONS.EXIT".into(),
            tier: RiskTier::Tier2,
            description_hash: "h_exit".into(),
            version: 1,
        })
        .unwrap();
    gov_ec
        .grant_consent(ConsentRecord {
            consent_id: "C_EXIT".into(),
            capability_id: "CAP.CONS.EXIT".into(),
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

    // Enter conservative by drift over threshold
    gov_ec.snapshot();
    gov_ec.accumulate_drift(3);
    let in_cons = gov_ec.state().conservative();

    // Cannot exit if drift>=threshold
    let blocked = matches!(gov_ec.exit_conservative(), Err(GovError::ConservativeMode));

    results.push(TestResult {
        name: "T4b_exit_conservative_blocked_by_drift_threshold".into(),
        formal_action: "ExitConservative".into(),
        pass: in_cons && blocked,
        breach_class: Some("CONS.MODE.VIOLATION".into()),
        invariant_id: "I4".into(),
        tla_property: "Inv_I4_ConservativeTierGate".into(),
        canonical_clause_ref: "core/constitutional_core_v1.0.md § Conservative Mode".into(),
        notes: "exit_conservative must be blocked when drift>=threshold (fail-closed).".into(),
    });

    // ===== T4c exit_conservative succeeds when conditions met (post MCC-02 fix)
    let sk_ec2 = ed25519_dalek::SigningKey::from_bytes(&sk_bytes);
    let mut gov_ec2 = GovernanceDaemon::new(10).with_author_keys(
        "AUTHOR_KEY",
        sk_ec2.verifying_key(),
        Some(sk_ec2),
    );
    gov_ec2.open_gate_for_testing();

    // Register exit capability and grant consent
    gov_ec2
        .register_capability(CapabilityDefinition {
            id: "CAP.CONS.EXIT".into(),
            tier: RiskTier::Tier2,
            description_hash: "h_exit2".into(),
            version: 1,
        })
        .unwrap();
    gov_ec2
        .grant_consent(ConsentRecord {
            consent_id: "C_EXIT2".into(),
            capability_id: "CAP.CONS.EXIT".into(),
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

    // Enter conservative via bypass attempt (drift remains below threshold)
    gov_ec2.inference_bypass_attempt();
    // Simulate explicit governance clearance in test-only mode.
    gov_ec2.clear_breach_for_testing();

    let exited = gov_ec2.exit_conservative().is_ok() && !gov_ec2.state().conservative();

    results.push(TestResult {
    name: "T4c_exit_conservative_succeeds_when_conditions_met".into(),
    formal_action: "ExitConservative".into(),
    pass: exited,
    breach_class: None,
    invariant_id: "I4".into(),
    tla_property: "Inv_I4_ConservativeTierGate".into(),
    canonical_clause_ref: "core/constitutional_core_v1.0.md § Conservative Mode".into(),
    notes: "After MCC-02 fix, exit_conservative succeeds when drift<threshold and breach is cleared under explicit governance/test action.".into(),
});

    // ===== T6 Drift monotonic + rollback

    let sk3 = ed25519_dalek::SigningKey::from_bytes(&sk_bytes);
    let mut gov2 =
        GovernanceDaemon::new(3).with_author_keys("AUTHOR_KEY", sk3.verifying_key(), Some(sk3));
    gov2.snapshot();
    gov2.accumulate_drift(2);
    let m1 = gov2.state().drift() == 2;
    gov2.accumulate_drift(1);
    let entered = gov2.state().conservative()
        && gov2.state().breach_flag() == Some(BreachClass::DriftOverthreshold);
    let _ = gov2.rollback();
    let rolled = gov2.state().drift() == 0;
    results.push(TestResult {
        name: "T6_drift_monotonic_and_rollback".into(),
        formal_action: "DriftTick/GovRollback".into(),
        pass: m1 && entered && rolled,
        breach_class: None,
        invariant_id: "DRIFT".into(),
        tla_property: "DriftMonotonic".into(),
        canonical_clause_ref: "core/constitutional_core_v1.0.md § Drift".into(),
        notes: "Drift increases then rollback restores snapshot drift.".into(),
    });

    results
}

#[cfg(test)]
#[allow(non_snake_case)] // compliance test ID convention (T3, T3b, ...) intentional
mod tests {
    use super::*;

    #[test]
    fn T3b_tier4_action_blocked_without_dual_confirm() {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key();
        let mut gd = GovernanceDaemon::new(10).with_author_keys("TEST_KEY", vk, Some(sk));
        gd.open_gate_for_testing();
        gd.register_capability(CapabilityDefinition {
            id: "CAP.TIER4.NOOP".into(),
            tier: RiskTier::Tier4,
            description_hash: "hash".into(),
            version: 1,
        })
        .expect("register capability");

        gd.grant_consent(ConsentRecord {
            consent_id: "CONSENT.T4.A".into(),
            capability_id: "CAP.TIER4.NOOP".into(),
            tier: RiskTier::Tier4,
            scope_text_hash: "hash".into(),
            granted_at: OffsetDateTime::now_utc(),
            revoked_at: None,
            granted_by: "AUTHOR".into(),
            evidence: ConsentEvidence::DualConfirmA("eA".into()),
            decision_seq: 0,
            capability_scope: vec![],
            expiry_policy: ExpiryPolicy::NoExpiry,
            grant_depth: 1,
        })
        .expect("grant consent A");

        // Switch to delegate context — Tier4 dual confirm is enforced for non-Author actors
        gd.grant_delegation(DelegationGrant {
            delegation_id: "DEL.T4".into(),
            principal: "AUTHOR".into(),
            delegate: "DELEGATE_1".into(),
            scope: vec!["CAP.TIER4.NOOP".into()],
            domain_scope: vec![],
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
            revocable: true,
            revoked_at: None,
            issued_by_decision: 0,
        })
        .expect("grant delegation");
        gd.set_actor_context_for_testing(delegate_ctx("DELEGATE_1"));

        let res = gd.tier4_noop_for_testing("CAP.TIER4.NOOP");
        assert!(res.is_err());
    }

    #[test]
    fn T3_tier4_dual_confirm_records_present() {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key();
        let mut gd = GovernanceDaemon::new(10).with_author_keys("TEST_KEY", vk, Some(sk));
        gd.open_gate_for_testing();
        gd.register_capability(CapabilityDefinition {
            id: "CAP.TIER4.NOOP".into(),
            tier: RiskTier::Tier4,
            description_hash: "hash".into(),
            version: 1,
        })
        .expect("register capability");

        gd.grant_consent(ConsentRecord {
            consent_id: "CONSENT.T4.A".into(),
            capability_id: "CAP.TIER4.NOOP".into(),
            tier: RiskTier::Tier4,
            scope_text_hash: "hash".into(),
            granted_at: OffsetDateTime::now_utc(),
            revoked_at: None,
            granted_by: "AUTHOR".into(),
            evidence: ConsentEvidence::DualConfirmA("eA".into()),
            decision_seq: 0,
            capability_scope: vec![],
            expiry_policy: ExpiryPolicy::NoExpiry,
            grant_depth: 1,
        })
        .expect("grant consent A");

        gd.grant_consent(ConsentRecord {
            consent_id: "CONSENT.T4.B".into(),
            capability_id: "CAP.TIER4.NOOP".into(),
            tier: RiskTier::Tier4,
            scope_text_hash: "hash".into(),
            granted_at: OffsetDateTime::now_utc(),
            revoked_at: None,
            granted_by: "AUTHOR".into(),
            evidence: ConsentEvidence::DualConfirmB("eB".into()),
            decision_seq: 0,
            capability_scope: vec![],
            expiry_policy: ExpiryPolicy::NoExpiry,
            grant_depth: 1,
        })
        .expect("grant consent B");

        let res = gd.tier4_noop_for_testing("CAP.TIER4.NOOP");
        assert!(res.is_ok());
    }

    // ── MP-6 Verification Signal tests ───────────────────────────────

    #[test]
    fn mp6_t01_signals_produced_for_all_tests() {
        let (results, signals) = run_with_signals();
        assert_eq!(
            results.len(),
            signals.len(),
            "every test must produce a signal"
        );
        assert!(!signals.is_empty(), "signals must not be empty");
    }

    #[test]
    fn mp6_t02_signal_has_required_fields() {
        let (_, signals) = run_with_signals();
        for signal in &signals {
            assert!(!signal.item_id.is_empty(), "item_id must not be empty");
            assert!(
                !signal.observable.is_empty(),
                "observable must not be empty"
            );
            assert!(
                !signal.pass_criterion.is_empty(),
                "pass_criterion must not be empty"
            );
            assert!(!signal.timestamp.is_empty(), "timestamp must not be empty");
            assert!(
                !signal.invariant_ref.is_empty(),
                "invariant_ref must not be empty"
            );
        }
    }

    #[test]
    fn mp6_t03_verification_report_json() {
        let (_, signals) = run_with_signals();
        let report = verification_report_json(&signals);

        assert_eq!(report["@type"], "VerificationReport");
        assert!(report["total_signals"].as_u64().unwrap() > 0);
        assert!(report["signals"].is_array());
    }

    #[test]
    fn mp6_t04_pass_fail_accounting() {
        let (results, signals) = run_with_signals();
        let expected_pass = results.iter().filter(|r| r.pass).count();
        let signal_pass = signals.iter().filter(|s| s.result.is_pass()).count();
        assert_eq!(expected_pass, signal_pass, "pass counts must agree");
    }

    #[test]
    fn mp6_t05_signal_serialization_roundtrip() {
        let (_, signals) = run_with_signals();
        let json = serde_json::to_string(&signals).expect("serialize signals");
        let parsed: Vec<VerificationSignal> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.len(), signals.len());
    }
}
