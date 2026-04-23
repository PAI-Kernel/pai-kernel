use pai_governance_daemon::{BreachClass, GovError, GovernanceDaemon};

#[test]
fn drift_edge_cases() {
    let mut gov = GovernanceDaemon::new(3);
    assert!(!gov.state().conservative());
    gov.accumulate_drift(2);
    assert!(!gov.state().conservative());
    assert!(gov.state().breach_flag().is_none());

    gov.accumulate_drift(1);
    assert!(gov.state().conservative());
    assert_eq!(
        gov.state().breach_flag(),
        Some(BreachClass::DriftOverthreshold)
    );
}

#[test]
fn exit_conservative_blocked_with_breach() {
    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let vk = sk.verifying_key();
    let mut gov = GovernanceDaemon::new(10).with_author_keys("TEST_KEY", vk, Some(sk));
    gov.inference_bypass_attempt();
    gov.open_gate_for_testing();
    let err = gov.exit_conservative().err();
    assert!(matches!(err, Some(GovError::ConservativeMode)));
}
