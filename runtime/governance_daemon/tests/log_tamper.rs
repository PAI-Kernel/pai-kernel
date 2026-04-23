use pai_governance_daemon::{GovError, GovernanceDaemon};

#[test]
fn log_tamper_is_detected() {
    let mut gov = GovernanceDaemon::new(10);
    gov.snapshot();
    assert!(gov.verify_log().is_ok());

    // Use test-only API (enabled in dev builds) instead of touching private fields.
    gov.inject_tamper_for_testing(0, "tampered").unwrap();

    let err = gov.verify_log().err();
    assert!(matches!(err, Some(GovError::LogTampered)));
}
