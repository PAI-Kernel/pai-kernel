# Known Limitations — PAI-Kernel v2.2.1

**Release:** PAI-CD v2.2.1 · `PAI-Kernel/pai-kernel@v2.2.1`
**Date:** 2026-04-23
**Status:** Early preview — invitation-only distribution
**Authority:** Release-gate Category B placeholder + adopter transparency commitment

---

## Read this first

v2.2.1 is the **first public release** of PAI-Kernel. It is an **early preview**, not a production-ready framework. This document describes what is intentionally incomplete, what is structurally known to be weak, and where work is actively in progress.

Framework users should read this alongside the [Constitutional Core](https://corpus.paikernel.org/docs/constitutional-core) and [Bill of Authorial Rights](https://corpus.paikernel.org/docs/bill-of-authorial-rights).

---

## 1. Release scope and snapshot nature

### 1.1 v2.2 is a citationally-stable freeze

The published framework is **PAI-CD v2.2 Freeze Edition** (March 2026 snapshot). This is the version cited by the SSRN paper (DOI 10.2139/ssrn.6512218).

**Implication:** external citations should reference v2.2.1. Adopters integrating against this release bind to v2.2 semantics.

### 1.2 Internal canonical evolves beyond v2.2

Under internal governance, PAI-CD continues to develop toward v3.1 Freeze Edition (Extended layers, additional Operating Principles, refined threat model). This internal development is deliberately **lagged from public surface** per bifurcation discipline (public snapshot layer ≠ canonical layer).

**Implication:** do not expect v2.2.1 published content to auto-update as canonical evolves. A future explicit rebase DL publishes v3.1 Freeze Edition.

### 1.3 SDK v1.3.0 vs corpus v2.2 scope gap

The Rust SDK is at **v1.3.0** — reflecting 3 sprint cycles of invariant implementation beyond the v2.2 corpus baseline. The SDK runtime enforces several v3.0-Extended and v3.1-trajectory invariants (MP-1 through MP-9) that are NOT in v2.2 normative text.

**Implication:** SDK behavior may be **stricter than v2.2 corpus requires**. This is deliberate (SDK trajectory leads corpus publication). Adopters treating SDK behavior as normative should note SDK enforces superset.

---

## 2. Runtime coverage (Rust SDK)

### 2.1 Phase 1 implementation

SDK v1.3.0 = Phase 1 live daemon (`pai_governance_daemon` via axum HTTP server). 28 crates, 262 tests. **PASS** on initial audit.

### 2.2 Not-yet-runtime-enforced from corpus

The following normative items are **defined in corpus but NOT runtime-enforced** in SDK v1.3.0. Compliance requires deploy-time manual validation or external tooling:

- **TCB attestation** (Doc 10) — corpus requires; SDK has scaffolding (`pai_attestation` crate) but no hardware-attestation backend wired
- **Supply-chain provenance** (Doc 11) — corpus requires cryptographic registry; SDK accepts declared hashes (not verifies upstream chain)
- **Governance capture defense** (Doc 15) — corpus requires role-bootstrapping independence; SDK supports the procedure but does not automatically enforce role-provider separation
- **Author vulnerability protection** (Doc 20) — corpus specifies V1-V4 escalation paths; SDK has `pai_vulnerability` crate (Sprint 4 scope) but V4 Lock-Out Resolution procedure is stub
- **Graduated Response Framework** (Doc 18) — Level 0-5 response ladder; SDK has Sprint 1 scaffolding; levels 4-5 require operational controls beyond SDK scope

### 2.3 Specification-level vs runtime-level verification

Package Q verification program (CAO-led) produces **specification-level** verdicts across three methods:

- **Method 1 · Formal (TLA+)** — invariants modeled at specification level; 464K states / 7 invariants verified
- **Method 2 · Adversarial review** — 9 BLOCKING objections open as of v2.2.1; Tier-2 responses staged
- **Method 3 · Narrative / Case** — Case N1 factual summary target 2026-04-26

**None of the three methods is a runtime-SDK verifier.** Runtime SDK conformance is a separate track (Phase Q-SDK; activated by Package Q verdict CONFIRMED or PARTIAL-alignment-value).

**Implication:** "PAI-compliant" status under current corpus (G-1 / H-2 certification nomenclature) requires independent audit; SDK alone is insufficient evidence of compliance.

---

## 3. Governance and process

### 3.1 Author vulnerability protection (Doc 20) operationalization

V4 Lock-Out Resolution procedure (corpus amendment S2-004 during v3.0-Extended freeze) exists normatively but requires external Recovery Designee / V3 Auditor relationships not part of software scope. Adopters deploying PAI instances for vulnerable Authors must establish these relationships externally.

### 3.2 Multi-Principal Governance (Doc 17) scope

Current Multi-Principal Governance covers multi-human (P1/P2/P3) classification. Multi-instance ATMAN coordination (Package P target v3.2 or v4.0) is **proposal-stage** — not in v2.2.1.

**Implication:** v2.2.1 governs **dyadic** deployments (one Author, one PAI instance). Multi-instance use cases should wait for Package P activation.

### 3.3 Zone Sovereignty and Dual Guarantee

Package S (regulatory zone sovereignty + provider-usage disclosure) is **v4.0 target** — proposal-stage, not in v2.2.1. Adopters operating across multi-jurisdictional deployment contexts should note this gap.

---

## 4. Documentation and traceability

### 4.1 Public corpus surface partial

Current `corpus.paikernel.org` hosts three canonical v2.2 documents (Constitutional Core, Bill of Authorial Rights, Glossary). Additional layers (Assurance, Extended, Meta, Verification — 22 canonical documents beyond the public three) remain in canonical internal source.

**Implication:** full corpus requires canonical source review (`github.com/PAI-Kernel/pai-kernel` v2.2.1 tag includes all published documents in scope). Browsable public rendering covers three-document surface only.

### 4.2 Operating Principles registry

OP-1 through OP-10 v2 are canonically ratified. OP-11 through OP-17 are at varying stages:

- OP-11 through OP-15: ratified (operational discipline)
- OP-16 Self-Report Discipline: **v1.0 in v3.1 Freeze Edition** — not in v2.2 normative text (but implicit compliance expectation from this release forward)
- OP-17 Markdown & Documentation Style: **draft v0.1** — governs v2.2.1 release production and public-surface rendering

### 4.3 Amendment procedure

Constitutional Amendment Manifest (Doc 22) describes the governance procedure. Current practice in active sessions may evolve ahead of Manifest text. Adopters relying on Amendment procedure for their own deployments should consult active Decision Log for current practice.

---

## 5. Licensing and attribution

- **Corpus** (Markdown documents): CC BY 4.0
- **SDK code** (Rust): MIT OR Apache-2.0 (dual license per Rust convention)
- **Paper**: SSRN standard posting terms (accessible under SSRN user agreement)

Full text: see `LICENSE` in repository root.

### 5.1 Attribution format

When citing:

> Sergeev, M. A. (2026). PAI-CD: A Constitutional Framework for Authorial Sovereignty in Deployed AI Systems. *SSRN*. https://doi.org/10.2139/ssrn.6512218

When reusing corpus text under CC BY 4.0, attribute to the framework with a link to `github.com/PAI-Kernel/pai-kernel` or `corpus.paikernel.org`.

### 5.2 Organizational status

PAI-Kernel is currently operated by **Mikhail Sergeev as Independent Researcher / PAI-Kernel Initiative**. It is NOT:

- A registered foundation (no foundation exists)
- A fiscal-hosted project (not hosted by OSC, Software Freedom Conservancy, or equivalent at this time)
- An adopted IEEE / ISO / W3C / IETF standard (per OP-10 v2 Category E discipline: "framework" not "standard" until formal adoption)

Contributions, funding structure, and organizational evolution are separate future decisions.

---

## 6. Security posture

### 6.1 Dual Guarantee framework

The Dual Guarantee (author sovereignty + provider non-circumvention) is specified in corpus. Provider-side operational controls (per-LLM-provider disclosure + submission checklist) are **Package S scope for v4.0** — not enforced in v2.2.1.

### 6.2 Known attack vectors

Threat Model (Doc 04) enumerates 11 threat classes — most recent (THREAT CLASS 11 · Self-Report Category Error) is **v3.1 scope** — not in v2.2 normative text but adopters should be aware per OP-16 discipline.

### 6.3 Security disclosure

Security vulnerabilities in SDK v1.3.0 or corpus normative text: disclose via GitHub private security advisory at [github.com/PAI-Kernel/pai-kernel/security/advisories](https://github.com/PAI-Kernel/pai-kernel/security/advisories) or direct contact per `SECURITY.md`.

---

## 7. Stability commitment (v2.2.1 specifically)

- **Corpus text** (3 public documents): **stable** for duration of v2.2.1 tag; no silent changes
- **Release artifacts** (paper PDF, CITATION.cff, LICENSE): **stable** for duration of v2.2.1 tag
- **SDK v1.3.0 crates on crates.io**: **immutable** once published; yanked only on critical security disclosure
- **Future releases** (v2.2.2 patch / v3.1 minor): new tag; v2.2.1 remains retrievable

---

## 8. Feedback and engagement

Per invitation-only distribution policy: feedback welcomed via:

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
| **v1.0** | **2026-04-23** | Initial publication alongside v2.2.1 release. Covers: snapshot nature, SDK vs corpus gap, Package Q specification-level scope, Operating Principles registry status, licensing, security, feedback channels. |

---

*PAI-Kernel · Known Limitations · v2.2.1 release · 2026-04-23*
*Source: generated per release-gate Category B remediation protocol*
