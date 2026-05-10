# PAI-Kernel SDK · Examples

This directory contains five reference example binaries demonstrating individual PAI-Kernel SDK invariants and capabilities. Each example is self-contained, runs in seconds, and prints structured output to illustrate the underlying constitutional primitive.

## Prerequisites

```sh
# Build all examples (one-time · ~2-3 min on first compile)
cargo build -p pai_examples --release
```

## Examples

### 1. `causal_graph` · Causal telemetry graph

Demonstrates building a causal graph that tracks how telemetry flows through variables to protected surfaces, detecting violations of forbidden information paths.

```sh
cargo run -p pai_examples --bin causal_graph
```

**What it shows:** Taint propagation through telemetry channels · forbidden path detection · ProtectedSurface enforcement.

### 2. `classify_output` · Pre-execution output classification

Demonstrates the `RecActionBoundary` · every output MUST be classified before delivery as either Informational or Recommendation.

```sh
cargo run -p pai_examples --bin classify_output
```

**What it shows:** Bias signal detection · Consequential vs non-Consequential domain classification · Recommendation vs Informational verdict.

### 3. `compliance_identity` · Compliance identity & certification

Demonstrates declaring compliance level and assessing certification tier with anti-capture threshold enforcement.

```sh
cargo run -p pai_examples --bin compliance_identity
```

**What it shows:** ComplianceLevel declaration · CertificationEvidence assessment · anti-capture check (prevents self-certification capture).

### 4. `evasion_audit` · Anti-evasion proxy correlation audit

Demonstrates detecting whether proxy metrics correlate with prohibited optimization targets at statistical significance p < 0.05.

```sh
cargo run -p pai_examples --bin evasion_audit
```

**What it shows:** Proxy metric correlation analysis · ProhibitedTarget detection · audit verdict (correlation suspicious vs acceptable).

### 5. `vulnerability_check` · Author vulnerability assessment

Demonstrates detecting when an Author's expressed preference may not reflect genuine autonomous choice, and the Verification Pause Protocol response flow.

```sh
cargo run -p pai_examples --bin vulnerability_check
```

**What it shows:** VulnerabilityCategory assessment · VerificationPause invocation · PauseResponse flow.

## Run all examples

```sh
for ex in causal_graph classify_output compliance_identity evasion_audit vulnerability_check; do
  echo "=== $ex ==="
  cargo run -p pai_examples --bin "$ex" --release --quiet
  echo
done
```

## Source code

Each example is a single file under `examples/src/bin/` · ~100-200 lines · readable as a standalone tutorial for the corresponding invariant.

## See also

- [`docs/INSTALL.md`](../docs/INSTALL.md) — full installation guide
- [`docs/quickstart.md`](../docs/quickstart.md) — 5-minute getting-started path
- [`docs/RELEASE_NOTES_v2.2.3.md`](../docs/RELEASE_NOTES_v2.2.3.md) — current release notes
- [`Glossary.md`](../Glossary.md) — terminology reference
