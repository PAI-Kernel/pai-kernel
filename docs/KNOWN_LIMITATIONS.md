# Known Limitations — PAI-Kernel v2.2.3

**Release:** PAI-CD v2.2.3 · `PAI-Kernel/pai-kernel@v2.2.3`
**Date:** 2026-04-28 (binary release) · 2026-05-10 (current documentation refresh)
**Status:** Early preview — invitation-only distribution
**Authority:** Adopter transparency commitment

---

## Read this first

PAI-Kernel began public release with v2.2.1 (2026-04-23, the first public release tag). The current release is v2.2.3 (2026-04-28). The framework remains an **early preview**, not a production-ready product. This document describes what is intentionally incomplete, what is structurally known to be weak, and where work is actively in progress.

Framework users should read this alongside the [Constitutional Core](https://corpus.paikernel.org/docs/constitutional-core) and [Bill of Authorial Rights](https://corpus.paikernel.org/docs/bill-of-authorial-rights).

---

## 1. Release scope and snapshot nature

### 1.1 v2.2 is a citationally-stable freeze

The published framework is **PAI-CD v2.2 Freeze Edition** (March 2026 snapshot). This is the corpus baseline cited by the SSRN paper (DOI 10.2139/ssrn.6512218).

**Implication:** external citations referencing the current release should use v2.2.3 (per `CITATION.cff`). Adopters integrating against this release bind to v2.2 corpus semantics with the v2.2.3 SDK runtime configuration.

### 1.2 Internal canonical evolves beyond v2.2

The PAI-CD framework continues to develop beyond the v2.2 published corpus. Subsequent public releases (further v2.2.x patches · future minor versions) will publish refined and extended content as the development trajectory progresses.

**Implication:** do not expect v2.2.3 published content to auto-update as the framework evolves. Future releases publish via explicit version increments. v2.2.1 · v2.2.2 · v2.2.3 each remain retrievable under their own tags.

### 1.3 SDK v1.3.2 vs corpus v2.2 scope gap

The Rust SDK is at **v1.3.2** — reflecting implementation work beyond the v2.2 corpus baseline. The SDK runtime enforces several extended invariants that are NOT in v2.2 published normative text.

**Implication:** SDK behavior may be **stricter than v2.2 corpus requires**. This is deliberate (SDK trajectory leads corpus publication). Adopters treating SDK behavior as normative should note SDK enforces a superset.

### 1.4 Published corpus subset vs full canonical corpus

The published corpus (3 documents in `corpus/` — Constitutional Core, Bill of Authorial Rights, Glossary) is the **Layer 0 subset** of the full canonical corpus. Additional canonical layers (Implementation Mapping, Threat Model, Consent & Capability Model, Constraints & Prohibitions, Decision Log Principles, Compliance Checklist, Governance & Change Control, plus the Assurance / Extended / Meta / Verification layers) exist in canonical internal development.

**Implication:** SDK crates implement enforcement for normative documents that are not yet publicly available. See § 2.2 for specific named cases. Adopters seeking complete normative reference should rely on the published Layer 0 documents plus the SDK source as a behavioral reference, until further layers are released.

---

## 2. Runtime coverage (Rust SDK)

### 2.1 Current implementation

SDK v1.3.2 provides a live governance daemon (`pai_governance_daemon` via axum HTTP server). The Rust workspace contains **28 members** (22 SDK primitive crates in `crates/pai_*` · 2 runtime binaries in `runtime/` · 2 protocol adapters in `adapters/` · 1 compliance test suite · 1 examples binary set). **18 of the 22 SDK crates are published to crates.io** as the `pai_*` v1.3.x family; the remaining 4 are workspace-internal helpers (`pai_harness` · `pai_interface` · `pai_influence` · `pai_examples`).

The runtime enforces the constitutional core invariants for in-process governance decisions. Test coverage and clippy status are tracked via the public CI pipeline (see [github.com/PAI-Kernel/pai-kernel/actions](https://github.com/PAI-Kernel/pai-kernel/actions)).

### 2.2 Areas requiring deploy-time validation

The following normative items are **defined in the PAI-CD corpus but NOT runtime-enforced** in SDK v1.3.2. Compliance requires deploy-time manual validation or external tooling:

- **TCB attestation** — corpus requires hardware attestation; SDK provides scaffolding (`pai_attestation`) but no hardware-attestation backend wired
- **Supply-chain provenance** — corpus requires cryptographic registry; SDK accepts declared hashes but does not verify upstream chain
- **Governance capture defense** — corpus requires role-bootstrapping independence; SDK supports the procedure but does not automatically enforce role-provider separation
- **Author vulnerability protection** — corpus specifies escalation paths (V1-V4); SDK provides scaffolding (`pai_vulnerability`); the V4 Lock-Out Resolution procedure remains in development
- **Graduated Response Framework** — Level 0-5 response ladder; SDK provides scaffolding for lower levels; levels 4-5 require operational controls beyond SDK scope

### 2.3 Docker image platform support

The `ghcr.io/pai-kernel/pai-kernel:v2.2.3` tag was built `linux/amd64` only (single-platform). Apple Silicon Macs (M1/M2/M3/M4) and `linux/arm64` Linux deployments require one of:

- **Recommended · multi-arch `:latest` tag**: `docker pull ghcr.io/pai-kernel/pai-kernel:latest` (includes both `linux/amd64` and `linux/arm64` native images · no emulation)
- **Pinned-version workaround · Rosetta emulation**: `docker pull --platform linux/amd64 ghcr.io/pai-kernel/pai-kernel:v2.2.3` (~5-10% slowdown on Apple Silicon · functional)

The `:v2.2.3` tag remains `linux/amd64`-only per release immutability discipline (no retroactive multi-arch republish). Future tag releases (v2.2.4 and beyond) will automatically produce multi-arch images on tag push.

**Implication:** adopters pinning to specific version v2.2.3 on Apple Silicon must use `--platform` flag. Adopters using `:latest` (no version pinning) receive multi-arch image transparently.

### 2.4 Specification-level vs runtime-level verification

The PAI-CD verification program produces **specification-level** verdicts via formal modeling (TLA+ state-space exploration), adversarial review, and narrative / case analysis.

**None of these methods is a runtime-SDK verifier.** Runtime SDK conformance is a separate workstream and is not delivered in this release.

**Implication:** PAI-CD compliance status under the current framework requires independent audit. The SDK alone is insufficient evidence of compliance. The independent certification framework and audit body are currently in development; until established, adopters may self-attest against the published Compliance Checklist.

---

## 3. Governance and process

### 3.1 Author vulnerability protection operationalization

The V4 Lock-Out Resolution procedure exists normatively but requires external Recovery Designee / V3 Auditor relationships not part of software scope. Adopters deploying PAI instances for vulnerable Authors must establish these relationships externally.

### 3.2 Multi-Principal Governance scope

Current Multi-Principal Governance covers multi-human classification (P1/P2/P3 principal categories). Multi-instance coordination (where multiple PAI instances coordinate on behalf of a single Author) is currently in development and not part of the v2.2.x release family.

**Implication:** v2.2.x governs **dyadic** deployments (one Author, one PAI instance). Multi-instance use cases should wait for future releases.

### 3.3 Zone Sovereignty and Dual Guarantee

Regulatory zone sovereignty and provider-usage disclosure mechanisms are currently in development, not part of the v2.2.x release family. Adopters operating across multi-jurisdictional deployment contexts should note this gap.

---

## 4. Documentation and traceability

### 4.1 Public corpus surface partial

Current `corpus.paikernel.org` hosts three canonical v2.2 documents (Constitutional Core, Bill of Authorial Rights, Glossary). Additional layers (assurance, extended, meta, verification documents) remain in canonical internal source.

**Implication:** the GitHub repository (`github.com/PAI-Kernel/pai-kernel` v2.2.3 tag) includes all published documents in scope. Browsable public rendering at `corpus.paikernel.org` covers the three-document surface only. A consolidated multi-language documentation portal is planned for v2.3 (~late May / early June 2026).

### 4.2 Operating Principles

The framework operating principles ratified for the v2.2.x release family govern the development process and public-surface rendering. Additional operating principles continue to develop iteratively; future releases publish updates.

### 4.3 Amendment procedure

The Constitutional Amendment procedure describes governance change control. Current practice in active development may evolve ahead of published procedure text. Adopters relying on the Amendment procedure for their own deployments should consult released documentation.

---

## 5. Licensing and attribution

- **Corpus** (Markdown documents): CC BY 4.0
- **SDK code** (Rust): MIT OR Apache-2.0 (dual license per Rust convention)
- **Paper**: SSRN standard posting terms (accessible under SSRN user agreement)

Full text: see `LICENSE` files in repository root.

### 5.1 Attribution format

When citing:

> Sergeev, M. A. (2026). PAI-CD: A Constitutional Framework for Authorial Sovereignty in Deployed AI Systems. *SSRN*. <https://doi.org/10.2139/ssrn.6512218>

When reusing corpus text under CC BY 4.0, attribute to the framework with a link to `github.com/PAI-Kernel/pai-kernel` or `corpus.paikernel.org`.

### 5.2 Organizational status

PAI-Kernel is currently operated by **Mikhail Sergeev as Independent Researcher / PAI-Kernel Initiative**. It is NOT:

- A registered foundation (no foundation exists)
- A fiscal-hosted project (not hosted by an external sponsor at this time)
- An adopted IEEE / ISO / W3C / IETF standard (the framework has not undergone formal adoption by a recognized standards body)

Contributions, funding structure, and organizational evolution are separate future decisions.

---

## 6. Security posture

### 6.1 Dual Guarantee framework

The Dual Guarantee (Author sovereignty + provider non-circumvention) is specified in the PAI-CD corpus. Provider-side operational controls (per-LLM-provider disclosure + submission checklist) are currently in development, not enforced in v2.2.3.

### 6.2 Threat model evolution

The Threat Model document enumerates threat classes covered by the v2.2.x release family. Additional threat classes are added iteratively as the framework matures; adopters should monitor future releases.

### 6.3 Security disclosure

Security vulnerabilities in SDK v1.3.2 or corpus normative text: disclose via GitHub private security advisory at [github.com/PAI-Kernel/pai-kernel/security/advisories](https://github.com/PAI-Kernel/pai-kernel/security/advisories) or direct contact per `SECURITY.md`.

---

## 7. Stability commitment (v2.2.3 specifically)

- **Corpus text** (3 public documents): **stable** for the duration of the v2.2.3 tag; no silent changes
- **Release artifacts** (paper PDF, CITATION.cff, LICENSE): **stable** for the duration of the v2.2.3 tag
- **SDK v1.3.2 crates on crates.io**: **immutable** once published; yanked only on critical security disclosure
- **Future releases** (further v2.2.x patches / future minor versions): new tag; v2.2.3 remains retrievable

---

## 8. Feedback and engagement

Per the invitation-only distribution policy, feedback is welcomed via:

- **GitHub Issues** at `github.com/PAI-Kernel/pai-kernel/issues` (public; any visitor)
- **Direct contact**: `contact@paikernel.org`
- **Security-sensitive**: GitHub security advisory (as above)

**What is valuable:**

- Objections, gaps, counterexamples
- Architectural critique
- Experience reports from attempted deployments
- Formal verification extensions

**What is not expected:**

- General-availability production deployments
- Public advocacy or promotion
- Reproductions in other frameworks (fine under CC BY 4.0 but please flag via Issue)

---

## 9. Changelog

| Version | Date | Changes |
|---|---|---|
| **v1.0** | **2026-04-23** | Initial publication alongside v2.2.1 release. Covers: snapshot nature, SDK vs corpus gap, specification-level verification scope, licensing, security, feedback channels. |
| **v2.0** | **2026-05-10** | Refresh for v2.2.3 release. Updated SDK references (v1.3.0 → v1.3.2). Corrected workspace/crate counts (was «18 crates · 286 tests» → 28 workspace members · 22 SDK primitive crates · 18 published to crates.io · test counts deferred to live CI surface). Added § 1.4 «Published corpus subset vs full canonical corpus» making the asymmetry between published documents and SDK enforcement scope explicit. Selective historical preservation of v2.2.1 references where they describe initial-release facts; current-scope statements aligned to v2.2.x release family or v2.2.3 specifically. |
| **v2.1** | **2026-05-10** | Added § 2.3 «Docker image platform support» documenting v2.2.3 image linux/amd64-only state · Apple Silicon adopter guidance (recommended `:latest` multi-arch · OR `--platform linux/amd64` Rosetta workaround for pinned v2.2.3). Future tag releases (v2.2.4+) produce multi-arch images automatically per release.yml workflow fix (pre-public `623a66c` · public `45039e5`). Renumbered subsequent §§ 2.3 → 2.4. |

---

*PAI-Kernel · Known Limitations · v2.2.3 release · 2026-05-10 documentation refresh*
