# PAI-CD v3.1 SDK — Reference Implementation

[![CI](https://github.com/PAI-Kernel/pai-kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/PAI-Kernel/pai-kernel/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-286%20passing-brightgreen)]()
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-see%20LICENSE-blue)]()
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

The PAI-Kernel SDK implements the full PAI-CD v3.1 normative framework —
a constitutional governance runtime for Personal Authorial Intelligence.

**30 crates · 286 tests · 0 clippy warnings · `#![forbid(unsafe_code)]` everywhere**

## Architecture

### HAC Core (6 primitives)

| # | Component | Crate | Tests | Invariants |
|---|-----------|-------|-------|------------|
| 1 | ConsentGate | `pai_gate` | 2 | Consent not inferred from silence |
| 2 | ConservativeModeFSM | `governance_daemon` | 6 | Fail-closed, breach auto-activation |
| 3 | GovernanceVersionBinding | `governance_daemon` | — | Constitutional version immutability |
| 4 | DelegationValidator | `pai_delegation` | 12 | Explicit scope, expiry, revocability |
| 5 | WitnessLog | `pai_witness` | 12 | Append-only, hash-chained, tamper-evident |
| 6 | RecActionBoundary | `pai_classify` | 18 | Recommendation vs Informational boundary |

### v3.1 Compliance Layer (15 gap items)

| Ref | Feature | Crate | Tests |
|-----|---------|-------|-------|
| MP-1 | Anti-Evasion Detection | `pai_evasion` | 14 |
| MP-3 | Quantitative Bias Thresholds | `pai_classify` | 8 |
| MP-5 | System Boundary Declaration | `pai_boundary` | 10 |
| MP-6 | Independent Verification Signals | `compliance_test_suite` | 5 |
| MP-7 | 100% Portability Parity | `pai_export` | 7 |
| MP-8 | Tier Aggregation (Sequence) | `pai_sequence` | 12 |
| MP-9 | Granular Consent & Expiry | `governance_daemon` | 10 |
| G-1/H-2 | Compliance Identity & Certification | `pai_compliance_id` | 13 |
| Doc 10 | TCB & Attestation | `pai_attestation` | 12 |
| Doc 11 | Supply Chain Provenance | `pai_provenance` | 14 |
| Doc 14 | Causal Telemetry | `pai_causal` | 13 |
| Doc 15 | Runtime Capture Detection | `governance_daemon` | 5 |
| Doc 18 | Graduated Response | `governance_daemon` | 12 |
| Doc 20 | Author Vulnerability Protection | `pai_vulnerability` | 15 |
| B2.5 | PII Detection Baseline | `pai_pii` | 16 |

### Supporting Infrastructure

| Crate | Purpose | Tests |
|-------|---------|-------|
| `pai_api` | HTTP API layer | 8 |
| `pai_config` | Configuration management | 5 |
| `pai_drift` | Drift detection engine | 6 |
| `pai_export` | Portability bundle export/import | 12 |
| `pai_harness` | Test harness utilities | — |
| `pai_influence` | Influence event tracking | — |
| `pai_interface` | Kernel context interface | — |
| `pai_policy` | OPA/Rego policy engine | 10 |
| `pai_storage` | Persistence layer | 8 |
| `pai_mcp` | MCP adapter | 5 |
| `pai_openai_adapter` | OpenAI adapter | 4 |
| `pai_examples` | Runnable SDK examples (5 binaries) | — |

## Formal Verification

- **TLA+**: 464K states, 7 invariants verified (StateModel)
- **SPARK**: ConsentGate formally proven (no runtime exceptions)

## Repository Structure

```
corpus/          — PAI-CD v3.1 constitutional documents (42 files)
formal/          — TLA+ state model + SPARK proofs
crates/          — SDK primitives (22 crates)
runtime/         — Governance daemon + kernel binary
compliance/      — Compliance test suite + integration tests
adapters/        — MCP + OpenAI protocol adapters
```

## Quick Start

```bash
cargo test --workspace          # 262 tests
cargo clippy --workspace        # 0 warnings
cargo run -p pai_compliance > compliance_report.json
```

## Pre-Audit

```bash
bash scripts/pre_audit.sh       # Full pre-audit gate suite
bash scripts/generate_sbom.sh   # CycloneDX SBOM
```

## External Anchoring

```bash
bash scripts/timestamp_hash.sh  # Public timestamp proof for corpus hash
```
