# Threat Surface Analysis (MAD-TR v1.0.0)

This document enumerates the minimal threat surface for the reference governance daemon.
It is **not** a complete production threat model, but it is designed to be audit-friendly.

## Trust Boundaries
- **Inference Runtime** (untrusted): may attempt to influence, bypass, or mutate governance state.
- **Governance Daemon** (trusted core): enforces invariants, produces append-only decision log, triggers Conservative Mode.
- **Storage/WAL** (untrusted medium): must be tamper-evident; integrity verified on load and before mutation.
- **Operator Environment** (partially trusted): provides optional HMAC key, GPG verification key, and build inputs.

## Assets
- Governance objectives (protected state)
- Drift accumulator and threshold parameters
- Conservative mode latch + breach classification
- Decision log (append-only, hash/HMAC chained)
- Snapshots (rollback safety)

## Threats and Mitigations

### T1 — Governance bypass (GOV.BYPASS)
**Threat:** inference attempts to mutate protected state.
**Mitigation:** runtime exposes no mutable public fields; explicit bypass attempt API records breach and forces conservative.
**Tests:** `governance_bypass_attempt` (compliance suite).

### T2 — Objective injection (OBJ.INJECTION)
**Threat:** untrusted runtime injects objectives.
**Mitigation:** injection attempt does not mutate objectives; breach flagged; conservative forced.
**Tests:** `unauthorized_objective_injection`.

### T3 — Decision log tampering (LOG.TAMPER)
**Threat:** offline/online modification of log entries or WAL.
**Mitigation:** chained hash/HMAC (optional keyed) over canonical serialization; strict monotonic sequence; verify on recover and before mutation.
**Tests:** `log_tamper_detected`.

### T4 — Drift threshold manipulation / delayed conservative (DRIFT.OVERTHRESHOLD)
**Threat:** drift crosses threshold without entering conservative.
**Mitigation:** deterministic check in `accumulate_drift`; conservative mode latch; compliance test.
**Tests:** `drift_accumulation_triggers_conservative`.

### T5 — Conservative mode violations (CONS.MODE.VIOLATION)
**Threat:** prohibited governance mutation while conservative is active.
**Mitigation:** ratification fails closed; breach class set; exit_conservative blocked until cleared.
**Tests:** `conservative_mode_enforcement`.

### T6 — Crash / state loss
**Threat:** process crash loses decision log / state and breaks accountability.
**Mitigation:** append-only WAL (JSONL) with fsync; recover() replays and verifies.
**Residual risk:** snapshot persistence is not yet implemented (future).

## Residual Risks (Explicit)
- No network/auth layer (out of scope).
- No confidentiality guarantees; this is integrity-only.
- Optional HMAC key management is operator responsibility.

## Out-of-scope for MAD-TR v1.0.0
- TEEs, hardware roots of trust
- Multi-operator quorum signing
- Distributed replication

