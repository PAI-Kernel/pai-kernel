# Known Limitations — PAI-Kernel v2.2.1

**Release:** PAI-CD v2.2.1 · `PAI-Kernel/pai-kernel@v2.2.1`
**Date:** 2026-04-23
**Status:** Early preview — invitation-only distribution
**Authority:** Adopter transparency commitment

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

The PAI-CD framework continues to develop beyond v2.2. Subsequent public releases (v2.2.x patch · future minor versions) will publish refined and extended content as the development trajectory progresses.

**Implication:** do not expect v2.2.1 published content to auto-update as the framework evolves. Future releases publish via explicit version increments.

### 1.3 SDK v1.3.0 vs corpus v2.2 scope gap

The Rust SDK is at **v1.3.0** — reflecting implementation work beyond the v2.2 corpus baseline. The SDK runtime enforces several extended invariants that are NOT in v2.2 normative text.

**Implication:** SDK behavior may be **stricter than v2.2 corpus requires**. This is deliberate (SDK trajectory leads corpus publication). Adopters treating SDK behavior as normative should note SDK enforces a superset.

---

## 2. Runtime coverage (Rust SDK)

### 2.1 Current implementation

SDK v1.3.0 provides a live governance daemon (`pai_governance_daemon` via axum HTTP server). The release publishes **18 crates** with **286 tests** passing on initial audit.

The runtime enforces the constitutional core invariants for in-process governance decisions.

### 2.2 Areas requiring deploy-time validation

The following normative items are **defined in the PAI-CD corpus but NOT runtime-enforced** in SDK v1.3.0. Compliance requires deploy-time manual validation or external tooling:

- **TCB attestation** — corpus requires hardware attestation; SDK provides scaffolding (`pai_attestation`) but no hardware-attestation backend wired
- **Supply-chain provenance** — corpus requires cryptographic registry; SDK accepts declared hashes but does not verify upstream chain
- **Governance capture defense** — corpus requires role-bootstrapping independence; SDK supports the procedure but does not automatically enforce role-provider separation
- **Author vulnerability protection** — corpus specifies escalation paths (V1-V4); SDK provides scaffolding (`pai_vulnerability`); the V4 Lock-Out Resolution procedure remains in development
- **Graduated Response Framework** — Level 0-5 response ladder; SDK provides scaffolding for lower levels; levels 4-5 require operational controls beyond SDK scope

### 2.3 Specification-level vs runtime-level verification

The PAI-CD verification program produces **specification-level** verdicts via formal modeling (TLA+ state-space exploration), adversarial review, and narrative / case analysis.

**None of these methods is a runtime-SDK verifier.** Runtime SDK conformance is a separate workstream and is not delivered in this release.

**Implication:** PAI-CD compliance status under the current framework requires independent audit. The SDK alone is insufficient evidence of compliance. The independent certification framework and audit body are currently in development; until established, adopters may self-attest against the published Compliance Checklist.

---

## 3. Governance and process

### 3.1 Author vulnerability protection operationalization

The V4 Lock-Out Resolution procedure exists normatively but requires external Recovery Designee / V3 Auditor relationships not part of software scope. Adopters deploying PAI instances for vulnerable Authors must establish these relationships externally.

### 3.2 Multi-Principal Governance scope

Current Multi-Principal Governance covers multi-human classification (P1/P2/P3 principal categories). Multi-instance coordination (where multiple PAI instances coordinate on behalf of a single Author) is currently in development and not part of v2.2.1.

**Implication:** v2.2.1 governs **dyadic** deployments (one Author, one PAI instance). Multi-instance use cases should wait for future releases.

### 3.3 Zone Sovereignty and Dual Guarantee

Regulatory zone sovereignty and provider-usage disclosure mechanisms are currently in development, not part of v2.2.1. Adopters operating across multi-jurisdictional deployment contexts should note this gap.

---

## 4. Documentation and traceability

### 4.1 Public corpus surface partial

Current `corpus.paikernel.org` hosts three canonical v2.2 documents (Constitutional Core, Bill of Authorial Rights, Glossary). Additional layers (assurance, extended, meta, verification documents) remain in canonical internal source.

**Implication:** the GitHub repository (`github.com/PAI-Kernel/pai-kernel` v2.2.1 tag) includes all published documents in scope. Browsable public rendering at `corpus.paikernel.org` covers the three-document surface only.

### 4.2 Operating Principles

The framework operating principles ratified for v2.2.1 govern the development process and public-surface rendering. Additional operating principles continue to develop iteratively; future releases publish updates.

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

The Dual Guarantee (Author sovereignty + provider non-circumvention) is specified in the PAI-CD corpus. Provider-side operational controls (per-LLM-provider disclosure + submission checklist) are currently in development, not enforced in v2.2.1.

### 6.2 Threat model evolution

The Threat Model document enumerates threat classes covered by v2.2.1. Additional threat classes are added iteratively as the framework matures; adopters should monitor future releases.

### 6.3 Security disclosure

Security vulnerabilities in SDK v1.3.0 or corpus normative text: disclose via GitHub private security advisory at [github.com/PAI-Kernel/pai-kernel/security/advisories](https://github.com/PAI-Kernel/pai-kernel/security/advisories) or direct contact per `SECURITY.md`.

---

## 7. Stability commitment (v2.2.1 specifically)

- **Corpus text** (3 public documents): **stable** for the duration of the v2.2.1 tag; no silent changes
- **Release artifacts** (paper PDF, CITATION.cff, LICENSE): **stable** for the duration of the v2.2.1 tag
- **SDK v1.3.0 crates on crates.io**: **immutable** once published; yanked only on critical security disclosure
- **Future releases** (v2.2.2 patch / future minor versions): new tag; v2.2.1 remains retrievable

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

---

*PAI-Kernel · Known Limitations · v2.2.1 release · 2026-04-23*
