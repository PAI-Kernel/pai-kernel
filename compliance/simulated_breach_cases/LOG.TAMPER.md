# LOG.TAMPER — Simulated Breach Case

## Scenario
An institutional actor attempts to "edit history" by modifying an existing decision log entry
(e.g., changing a summary, removing a breach classification, or rewriting ratification details).

## Attack Steps
1. Take an existing decision entry `E[i]`.
2. Mutate any field covered by the hash-chain (e.g., `summary`).
3. Present the altered log as authoritative history.

## Expected System Response
- `verify_log()` MUST fail deterministically.
- Compliance harness MUST classify as `LOG.TAMPER`.
- Any further governance actions MUST be rejected until a valid log is restored.

## Canonical References
- Decision_Log_and_Witness_Principles.md (append-only decision record + witness integrity)
- Compliance_Checklist.md (no silent modification; breach classification required)
