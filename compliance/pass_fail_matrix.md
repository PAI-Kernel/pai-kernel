# Compliance PASS/FAIL Matrix — MCC Upgrade

This matrix defines binary compliance outcomes for MCC invariants I1–I6.

| Test | Invariant(s) | Expected Result | Breach Class on FAIL |
|---|---|---|---|
| T1_objective_registry_integrity_add | I6 | PASS | STATE.INVALID / OBJ.INJECTION |
| T3_tier2_action_requires_consent | I1, I3 | PASS (reject) | AUTHZ.FAIL |
| T3b_tier4_action_blocked_without_dual_confirm | I3 | PASS (reject) | AUTHZ.FAIL |
| T3_tier4_dual_confirm_records_present | I3 | PASS | AUTHZ.FAIL |
| T2_delegation_allows_scoped_action | I2 | PASS | AUTHZ.FAIL |
| T2_delegation_expired_rejected | I2 | PASS (reject) | AUTHZ.FAIL |
| T4_conservative_blocks_tier2_even_with_delegation | I4 | PASS (reject) | CONS.MODE.VIOLATION |
| T4b_exit_conservative_blocked_by_drift_threshold | I4 | PASS (reject) | CONS.MODE.VIOLATION |
| T4c_exit_conservative_succeeds_when_conditions_met | I4 | PASS | CONS.MODE.VIOLATION |
| T5_verify_log_signed_ok | I5 | PASS | LOG.TAMPER |
| T5_verify_log_signature_tamper_detected | I5 | PASS (reject) | LOG.TAMPER |
| T6_drift_monotonic_and_rollback | Drift | PASS | DRIFT.OVERTHRESHOLD |

All tests are executed via `compliance/run_compliance.sh`.
