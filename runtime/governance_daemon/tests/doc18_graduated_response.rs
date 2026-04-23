//! Doc 18: Graduated Response tests
//!
//! Tests the 5-state governance mode machine:
//! Normal → Warning → Restricted → Conservative → Breach

use pai_governance_daemon::{GovernanceDaemon, GovernanceMode};

fn make_daemon(threshold: u64) -> GovernanceDaemon {
    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let vk = sk.verifying_key();
    GovernanceDaemon::new(threshold).with_author_keys("TEST_KEY", vk, Some(sk))
}

// D18-T01: Fresh daemon starts in Normal mode
#[test]
fn d18_t01_starts_normal() {
    let gov = make_daemon(100);
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Normal);
    assert!(!gov.state().conservative());
}

// D18-T02: Drift at 75% triggers Warning
#[test]
fn d18_t02_warning_at_75pct() {
    let mut gov = make_daemon(100);
    gov.accumulate_drift(75);
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Warning);
    // Warning is advisory — conservative flag NOT set
    assert!(!gov.state().conservative());
}

// D18-T03: Drift at 90% triggers Restricted
#[test]
fn d18_t03_restricted_at_90pct() {
    let mut gov = make_daemon(100);
    gov.accumulate_drift(90);
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Restricted);
    // Restricted sets conservative flag for backward compat
    assert!(gov.state().conservative());
}

// D18-T04: Drift at 100% triggers Conservative
#[test]
fn d18_t04_conservative_at_100pct() {
    let mut gov = make_daemon(100);
    gov.accumulate_drift(100);
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Conservative);
    assert!(gov.state().conservative());
    assert!(gov.state().breach_flag().is_some());
}

// D18-T05: Breach attempt sets Breach mode
#[test]
fn d18_t05_breach_mode() {
    let mut gov = make_daemon(100);
    gov.inference_bypass_attempt();
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Breach);
    assert!(gov.state().conservative());
}

// D18-T06: Mode ordering is monotonic during escalation
#[test]
fn d18_t06_monotonic_escalation() {
    let mut gov = make_daemon(100);
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Normal);

    gov.accumulate_drift(10); // 10% — still Normal
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Normal);

    gov.accumulate_drift(66); // 76% — Warning
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Warning);

    gov.accumulate_drift(15); // 91% — Restricted
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Restricted);

    gov.accumulate_drift(10); // 101% — Conservative
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Conservative);
}

// D18-T07: GovernanceMode ordering (PartialOrd)
#[test]
fn d18_t07_mode_ordering() {
    assert!(GovernanceMode::Normal < GovernanceMode::Warning);
    assert!(GovernanceMode::Warning < GovernanceMode::Restricted);
    assert!(GovernanceMode::Restricted < GovernanceMode::Conservative);
    assert!(GovernanceMode::Conservative < GovernanceMode::Breach);
}

// D18-T08: is_conservative() backward compat
#[test]
fn d18_t08_is_conservative_compat() {
    assert!(!GovernanceMode::Normal.is_conservative());
    assert!(!GovernanceMode::Warning.is_conservative());
    assert!(GovernanceMode::Restricted.is_conservative());
    assert!(GovernanceMode::Conservative.is_conservative());
    assert!(GovernanceMode::Breach.is_conservative());
}

// D18-T09: Snapshot preserves governance_mode
#[test]
fn d18_t09_snapshot_preserves_mode() {
    let mut gov = make_daemon(100);
    gov.accumulate_drift(76); // Warning
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Warning);

    gov.snapshot();
    // Mode should still be Warning
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Warning);
}

// D18-T10: Low drift does not skip to higher modes
#[test]
fn d18_t10_no_skip() {
    let mut gov = make_daemon(100);
    // Small increments should not jump to Warning
    for _ in 0..7 {
        gov.accumulate_drift(10);
    }
    // 70% — still Normal
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Normal);

    gov.accumulate_drift(6); // 76% — Warning
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Warning);
}

// D18-T11: Drift at exactly threshold boundary
#[test]
fn d18_t11_exact_threshold() {
    let mut gov = make_daemon(4);
    gov.accumulate_drift(3); // 75% exactly — Warning
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Warning);

    let mut gov2 = make_daemon(10);
    gov2.accumulate_drift(9); // 90% exactly — Restricted
    assert_eq!(gov2.state().governance_mode(), GovernanceMode::Restricted);
}

// D18-T12: Mode transitions cannot go backward via drift
#[test]
fn d18_t12_no_backward_via_drift() {
    let mut gov = make_daemon(100);
    gov.accumulate_drift(90); // Restricted
    assert_eq!(gov.state().governance_mode(), GovernanceMode::Restricted);

    // Adding 0 drift should not change mode
    gov.accumulate_drift(0);
    assert!(gov.state().governance_mode() >= GovernanceMode::Restricted);
}
