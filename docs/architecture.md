---
title: "Architecture Overview — PAI-Kernel SDK"
slug: architecture
position: 7
hidden: false
excerpt: "High-level architecture · 28-crate workspace · constitutional invariants · 4-layer enforcement · sidecar daemon model."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "architecture.md"
    path: "docs/architecture.md"
    commit: "main"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · Architecture Overview"
  last_amendment: "2026-05-09"
---

# PAI-Kernel SDK · Architecture Overview

This document describes the high-level architecture of PAI-Kernel SDK v2.2.3 for adopters evaluating fit and integrators planning deployment.

---

## What PAI-Kernel SDK is

A **constitutional governance substrate** for AI systems · enforces six non-derogable invariants (Authorship Supremacy · Cognitive Sovereignty · Consent Gating · Tier-based action gates · Conservative mode · Audit integrity) at the deployment layer.

Implementation form: a **sidecar daemon** (`pai_governance_daemon`) exposing a small REST API (5 endpoints) that AI applications call to validate actions before execution.

---

## What PAI-Kernel SDK is NOT

- **Not an AI model** · no language model · no inference engine bundled
- **Not a general policy framework** · focused on PAI-CD constitutional invariants
- **Not a multi-tenant SaaS** · single-tenant by design
- **Not a replacement for application-level governance** · provides primitives · application supplies the rest

---

## High-level diagram (sidecar pattern · two parallel lanes)

The sidecar topology is **two parallel horizontal lanes**: AI application flow on top (left → right · user input becomes action) · governance sidecar flow on bottom (left → right · API receives request · enforces · records). The lanes connect via HTTP/JSON at exactly one point — application sends each proposed action to sidecar before execution.

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│  ▶ LANE 1 · YOUR AI APPLICATION  (independent process · left → right)            │
│                                                                                  │
│    ┌────────────┐     ┌───────────┐     ┌──────────────────┐                     │
│    │ User input │────▶│ Reasoning │────▶│ Proposed action  │                     │
│    └────────────┘     └───────────┘     └────────┬─────────┘                     │
│                                                  │                               │
└──────────────────────────────────────────────────┼───────────────────────────────┘
                                                   │
                                       HTTP / JSON │
                              (pre-execution gate) │
                                                   │
                                                   ▼
┌──────────────────────────────────────────────────┬───────────────────────────────┐
│  ▶ LANE 2 · PAI-KERNEL SIDECAR  (parallel process · pai_governance_daemon)       │
│                                                                                  │
│    ┌─────────────┐     ┌──────────────┐     ┌──────────────────┐                 │
│    │  REST API   │────▶│  4-Layer     │────▶│  Witness Chain   │                 │
│    │  /api/v1/.. │     │  Enforcement │     │  SQLite          │                 │
│    └─────────────┘     │              │     │  append-only     │                 │
│                        │  L1 · TLA+   │     │  hash-linked     │                 │
│                        │  L2 · SPARK  │     └──────────────────┘                 │
│                        │  L3 · Rego   │                                          │
│                        │  L4 · Runtime│                                          │
│                        └──────────────┘                                          │
└──────────────────────────────────────────────────────────────────────────────────┘
```

**Reading guide:**

- **Top lane (LANE 1)** · application's intrinsic flow · proceeds **left-to-right** · `User input → Reasoning → Proposed action`. AI application owns this flow entirely · NO sidecar coupling within the lane.
- **Bottom lane (LANE 2)** · sidecar's enforcement flow · proceeds **left-to-right in parallel** · `REST API → 4-Layer Enforcement → Witness Chain`. Independent process · runs alongside the application · NOT inline.
- **Vertical connection** · single HTTP/JSON call from application to sidecar at the «pre-execution gate» moment. Application proposes action · sidecar evaluates · response gates execution.
- **Sidecar property** · if sidecar offline · application can choose fail-closed (refuse action) OR fail-open (proceed without governance check) per adopter policy. The two lanes are independent processes communicating only via the explicit HTTP/JSON edge.

The two-lane visualization is essential to understanding why this is a **sidecar** pattern (NOT embedded library · NOT inline interceptor). The application can run without sidecar entirely (fail-open) · the sidecar can be replaced/upgraded independently · the coupling is one explicit edge · all governance state lives in the sidecar's lane (witness chain) — not entangled with application state.

### Diagram rendering notes

The diagram above uses ASCII art (Unicode box-drawing) within a fenced text block. Rendering targets:

- **GitHub web view** · monospace font in code blocks · renders correctly with proper alignment
- **Plain-text editors** (Vim · Emacs · VSCode) · same monospace rendering
- **Terminal pagers** (`less` · `bat` · `cat`) · native monospace rendering
- **HTML conversion** (mdBook · pandoc) · preserved within `<pre>` blocks

Mermaid alternative was attempted (commit `a9d85c0`) · but GitHub Mermaid renderer overrode `direction LR` directive within nested subgraphs · stacked nodes vertically inside lanes · defeating the parallel-lanes visualization. ASCII chosen for reliability across viewers.

---

## Core invariants (PAI-CD)

The framework enforces six non-derogable constitutional invariants:

| # | Invariant | Brief description |
|---|---|---|
| **I1** | **Authorship Supremacy** | No silent high-impact actions · author retains final authority |
| **I2** | **Cognitive Sovereignty** | Delegation paused in conservative mode · author shielded from manipulation |
| **I3** | **Consent Gating** | Tier 4 actions require dual confirmation · consent layered by tier |
| **I4** | **Conservative Tier Gate** | Conservative mode blocks Tier ≥2 actions · escalation requires explicit author action |
| **I5** | **Signed High Impact** | High-impact actions cryptographically signed · verifiable trail |
| **I6** | **Objective Subset** | Actions must align with declared objectives · no scope creep |

Full traceability of each invariant across all 4 enforcement layers is maintained internally as a verification matrix; planned for public release alongside the v2.3 multi-language documentation portal (~late May / early June 2026).

---

## 4-layer enforcement model

PAI-Kernel SDK enforces invariants at four layers, each providing different guarantees:

| Layer | Tool | Guarantee | Adopter visibility |
|---|---|---|---|
| **L1 · Formal model** | TLA+ + TLC model checker | Verified across exhaustive state space (millions of states) | Static · pre-release |
| **L2 · Formal proof** | SPARK + CVC5 prover | Mathematical proof · obligations discharged | Static · pre-release |
| **L3 · Policy** | OPA/Rego (regorus engine) | Runtime policy evaluation · adopter-extensible | Dynamic · adopter writes policies |
| **L4 · Runtime** | Rust runtime checks | Final gate · always-on | Dynamic · daemon enforces |

**Defense in depth:** an action must pass ALL applicable layers. A bug at one layer is caught at another. Independent layers prevent single-point-of-failure.

---

## Workspace structure (28 crates)

PAI-Kernel SDK is organized as a Cargo workspace with 28 crates:

### Kernel candidates (12 crates · constitutional primitives)

Pure substrate · minimal dependencies · TCB candidates:

| Crate | Role | LOC (approx) |
|---|---|---|
| `pai_classify` | Pre-execution output classification | 380 |
| `pai_delegation` | Delegation primitive | 440 |
| `pai_gate` | Gate decision primitive | 250 |
| `pai_influence` | Influence chain hash helper | 220 |
| `pai_interface` | Interface trait definitions | 190 |
| `pai_harness` | Test harness scaffolding | 170 |
| `pai_drift` | Drift accumulator | 480 |
| `pai_config` | TOML config schema | 360 |
| `pai_causal` | Causal telemetry graph | 730 |
| `pai_export` | State export | 460 |
| `pai_kernel` | Aggregating umbrella | 280 |
| `pai_governance_daemon` | Daemon orchestration | 1,938 |

### Trust capabilities (14 crates · safety + detection)

Higher-level capabilities · larger dependency footprint:

- `pai_api` · HTTP API (axum-based)
- `pai_attestation` · Capability attestation
- `pai_boundary` · Boundary enforcement
- `pai_compliance` · Compliance binary
- `pai_compliance_id` · Compliance identity
- `pai_evasion` · Anti-evasion detection
- `pai_pii` · PII detection
- `pai_policy` · Rego policy engine (regorus)
- `pai_provenance` · Provenance tracking
- `pai_sequence` · Sequence integrity
- `pai_storage` · SQLite witness backend
- `pai_vulnerability` · Author vulnerability assessment
- `pai_witness` · Witness chain core
- `pai_mcp` · MCP adapter
- `pai_openai_adapter` · OpenAI integration adapter

### Examples (1 crate · 5 binaries)

- `pai_examples` · 5 reference example binaries (see [`examples/README.md`](../examples/README.md))

---

## Runtime profiles

PAI-Kernel SDK supports multiple runtime configurations:

### Production daemon

```sh
pai_governance_daemon --config ./pai-kernel.toml
```

- Full feature set
- SQLite witness backend
- HTTP API exposed
- Required env vars: `PAI_AUTHOR_API_KEY` + `PAI_AUTHOR_SIGNING_KEY`
- ~5.9 MB stripped binary
- Default bind: 127.0.0.1:9100

### Demo mode (testing only)

```sh
pai_governance_daemon --demo
```

- Ephemeral keys (lost on restart)
- 127.0.0.1 force-bind
- In-memory storage (no SQLite)
- Prominent stderr WARNINGs
- No env vars required

### Verification (CLI · standalone)

```sh
pai_governance_daemon verify
```

- Witness chain integrity check
- Works without env vars · without running daemon
- Suitable for CI / batch verification

### Export (CLI · production)

```sh
pai_governance_daemon export > bundle.json
```

- Full governance bundle export
- Requires env vars
- JSON output

---

## Storage model

**Default backend:** SQLite (`./pai-kernel.db`).

**Witness chain structure:**

- Append-only
- Hash-linked (each entry references previous entry's hash)
- Cryptographically signed (Ed25519 via `PAI_AUTHOR_SIGNING_KEY`)
- Tamper-evident · daemon refuses to start if hash chain broken

**Schema:** see `pai_witness` crate source · stable across v2.2.x patches.

**Migration:** v2.2.2 → v2.2.3 preserves witness chain (no schema migration needed).

---

## Policy model

**Engine:** OPA/Rego via regorus (pure-Rust embedded interpreter).

**Policy directory:** `./policies/` (configurable via `pai-kernel.toml` `[policy]` section).

**Default policies installed:**

```text
policies/
├── constitutional/
│   ├── classification.rego
│   ├── consent.rego
│   ├── conservative.rego
│   ├── coordination.rego
│   ├── coordination_test.rego
│   ├── delegation.rego
│   └── drift.rego
└── operational/
    ├── denylist.rego
    └── tier_gates.rego
```

Adopters can extend / override policies in their working directory · daemon reloads on configurable interval (`reload_interval_secs` in `pai-kernel.toml`).

---

## Cryptographic stack

Minimal · standard primitives only:

| Operation | Library | Version |
|---|---|---|
| Hashing | `sha2` (SHA-256) | 0.10 |
| Signing | `ed25519-dalek` | 2.x |
| Random | `getrandom` | 0.2 |

**No custom crypto** · all primitives delegated to industry-standard pure-Rust libraries. Verified empirically against workspace Cargo.toml (no `rsa` · `aes` · `chacha` · `argon2` · `bcrypt` · `openssl` · `ring` direct dependencies).

---

## Distribution channels

| Channel | Use case | Doc |
|---|---|---|
| Homebrew (`brew install PAI-Kernel/tap/pai-kernel`) | macOS · Linux developer machines | [`INSTALL.md`](INSTALL.md) Method 2 |
| Cargo (`cargo install pai_kernel`) | Rust toolchain users · auditors | [`INSTALL.md`](INSTALL.md) Method 3 |
| Docker (`ghcr.io/pai-kernel/pai-kernel:v2.2.3`) | Container deployments · Windows | [`INSTALL.md`](INSTALL.md) Method 4 |
| One-line installer (`curl ... install.sh`) | Quick install · all platforms | [`INSTALL.md`](INSTALL.md) Method 1 |
| Manual binary download | Air-gapped · custom paths | [`INSTALL.md`](INSTALL.md) Method 5 |
| Build from source | Auditors · contributors · custom builds | [`INSTALL.md`](INSTALL.md) Method 6 |

---

## Version compatibility

| Component | Version requirement |
|---|---|
| Rust toolchain | 1.88+ (MSRV) |
| Operating system | macOS 11+ · Linux glibc 2.31+ · Windows 10+ (Docker) |
| Architecture | x86_64 + aarch64 |
| SQLite | bundled via `rusqlite` (no system requirement) |

---

## Performance characteristics

Empirical measurements (Apple M4 · macOS arm64 · release build):

| Metric | Value |
|---|---|
| Binary size (stripped) | 5.92 MB |
| Binary size (LTO=fat + opt-level=z) | 3.16 MB |
| Cold startup | ~50 ms (demo mode) |
| Memory RSS (idle) | ~10-20 MB |
| API endpoint latency | <1 ms (loopback) |

For comprehensive runtime analysis · see internal Runtime Analysis substrate (Cycle N+1 audit).

---

## Project status + roadmap

**Current:** v2.2.3 LIVE (released 2026-04-28). See [`CHANGELOG.md`](../CHANGELOG.md) for release history.

**v2.2.3.x patches:** documentation + adopter UX improvements landing on `main` branch without version bump (see [`RELEASE_NOTES_v2.2.3.1.md`](RELEASE_NOTES_v2.2.3.1.md)).

**v2.3 cycle:** late May / early June 2026 (multi-language localization initiative · mdBook documentation portal · Constitutional Amendment if needed).

---

## See also

- [`docs/INSTALL.md`](INSTALL.md) — installation guide
- [`docs/quickstart.md`](quickstart.md) — 5-minute getting-started
- [`docs/api.md`](api.md) — REST API reference
- [`docs/audit_checklist.md`](audit_checklist.md) — verification procedures
- [`docs/KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) — current scope and constraints
- [`corpus/PAI_Constitutional_Document.md`](../corpus/PAI_Constitutional_Document.md) — constitutional foundation
- [`examples/README.md`](../examples/README.md) — reference example binaries

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
