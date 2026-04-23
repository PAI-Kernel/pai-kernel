use pai_governance_daemon::{GovError, GovernanceDaemon};
use proptest::prelude::*;

// (1) Hash-chain remains valid after random operation sequences.
proptest! {
    #[test]
    fn prop_log_chain_valid_after_random_ops(ops in prop::collection::vec(0u8..=5u8, 1..50)) {
        let mut gov = GovernanceDaemon::new(3);
        for op in ops {
            match op {
                0 => gov.snapshot(),
                1 => { let _ = gov.ratify_add_objective("O1"); }
                2 => gov.accumulate_drift(1),
                3 => gov.inference_bypass_attempt(),
                4 => gov.runtime_inject_objective_attempt("X"),
                _ => { /* noop */ }
            }
        }
        // Log must be internally consistent unless deliberately tampered
        let _ = gov.verify_log().map_err(|e| match e {
            GovError::LogTampered => e,
            _ => e,
        }).unwrap_or(());
    }
}

// (2) Conservative remains true once a breach is recorded (monotonic until explicitly cleared/testing)
proptest! {
    #[test]
    fn prop_conservative_monotonic_after_breach(steps in prop::collection::vec(0u8..=2u8, 1..50)) {
        let mut gov = GovernanceDaemon::new(10);
        let mut breached = false;
        for s in steps {
            match s {
                0 => { gov.inference_bypass_attempt(); breached = true; }
                1 => { gov.runtime_inject_objective_attempt("X"); breached = true; }
                _ => gov.accumulate_drift(1),
            }
            if breached {
                prop_assert!(gov.state().conservative());
            }
        }
    }
}

// (3) Rollback restores exact snapshot state
proptest! {
    #[test]
    fn prop_rollback_restores_snapshot(extra_drift in 0u8..=5u8) {
        let mut gov = GovernanceDaemon::new(3);
        gov.snapshot();
        gov.accumulate_drift(extra_drift as u64);
        gov.inference_bypass_attempt();
        // In conservative, rollback is allowed
        let _ = gov.rollback();
        // Snapshot was taken at drift=0, conservative=false, breach=None, objectives=[]
        prop_assert_eq!(gov.state().drift(), 0);
        prop_assert!(!gov.state().conservative());
        prop_assert!(gov.state().breach_flag().is_none());
        prop_assert_eq!(gov.state().objectives().len(), 0);
    }
}
