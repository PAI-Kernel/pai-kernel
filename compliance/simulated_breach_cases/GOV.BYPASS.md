# Breach Case: GOV.BYPASS

Scenario: unauthorized protected-state mutation attempt.

Expected:
- reject mutation
- append BREACH log entry with GOV.BYPASS
- conservative mode engaged


## Formal linkage
- TLA+ action: `InferenceBypassAttempt`
- Invariant: `Inv_GovSeparation`
- Property: `GovernanceSeparation`
