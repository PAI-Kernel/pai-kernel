# PAI-Kernel

**The problem.** AI systems increasingly mediate decisions about humans —
about cognitive influence, about whose authority shapes whose thinking.
Current deployment models leave humans dependent on provider goodwill for
the protection of their cognitive sovereignty. There is no
provider-independent verification surface for what an AI is actually
optimizing toward.

**The framework.** PAI-CD (Personal Authorial Intelligence — Constitutional
Document) defines normative invariants — independently verifiable — that
deployed AI systems must satisfy to protect Author cognitive sovereignty
at the deployment layer. PAI-Kernel is the public reference implementation
in Rust (Apache 2.0 / MIT dual-licensed code · CC BY 4.0 documentation).

**Who should read further.** Researchers exploring AI-alignment normative
frameworks · engineers evaluating constitutional governance for deployed
AI · auditors verifying provider compliance · adopters integrating
PAI-CD-aligned governance into their AI systems · contributors to the
open canonical corpus.

```text
┌──────────────────────────────────────────────────────────┐
│                                                          │
│                           PAI                            │
│             Personal Authorial Intelligence              │
│                      (the concept)                       │
│                                                          │
│                            │                             │
│                            │ codified as                 │
│                            ▼                             │
│                                                          │
│                          PAI-CD                          │
│               PAI Constitutional Document                │
│                  (normative framework)                   │
│                                                          │
│                            │                             │
│                            │ implemented by              │
│                            ▼                             │
│                                                          │
│                      PAI-Kernel SDK                      │
│                 Reference implementation                 │
│              (Rust crates · dual-licensed)                │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19151900.svg)](https://doi.org/10.5281/zenodo.19151900)
[![DOI all versions](https://zenodo.org/badge/DOI/10.5281/zenodo.19151899.svg)](https://doi.org/10.5281/zenodo.19151899)
[![License (code)](https://img.shields.io/badge/code-MIT%20OR%20Apache--2.0-blue.svg)](./LICENSE-MIT)
[![License (docs)](https://img.shields.io/badge/docs-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/)
[![GitHub release](https://img.shields.io/github/v/release/PAI-Kernel/pai-kernel)](https://github.com/PAI-Kernel/pai-kernel/releases)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/PAI-Kernel/pai-kernel/badge)](https://securityscorecards.dev/viewer/?uri=github.com/PAI-Kernel/pai-kernel)
[![ORCID iD](https://info.orcid.org/wp-content/uploads/2019/11/orcid_16x16.png)](https://orcid.org/0009-0001-6443-855X)
[0009-0001-6443-855X](https://orcid.org/0009-0001-6443-855X)

## Personal Authorial Intelligence — Constitutional Framework

> A normative layer for human-centric AI governance.
> **Release v2.2.2** · 2026-04-27

### Choose your path

| If you are... | Start here | Time |
|---|---|---|
| **An adopter** evaluating PAI-CD for your AI system | [§ Adopter Path](#adopter-path) | ~30 min |
| **A researcher** interested in the constitutional framework | [§ Researcher Path](#researcher-path) | ~1 hour |
| **An auditor** verifying invariants and compliance | [§ Auditor Path](#auditor-path) | ~2 hours |
| **A contributor** wanting to propose changes | [§ Contributor Path](#contributor-path) | ~30 min |
| **An engineer** integrating PAI-Kernel runtime | [§ Engineering Path](#engineering-path) | ~1 hour |

### Adopter Path

**Goal:** evaluate fit · install · run a demo · understand scope.

1. Read [`docs/RELEASE_NOTES_v2.2.2.md`](./docs/RELEASE_NOTES_v2.2.2.md) — what's in this release.
2. Read [`docs/KNOWN_LIMITATIONS.md`](./docs/KNOWN_LIMITATIONS.md) — what's NOT in this release.
3. Follow [`docs/INSTALL.md`](./docs/INSTALL.md) — ~5–10 min via Homebrew binary OR `install.sh`.
4. Run demo: `pai_governance_daemon --version` · explore `/api/v1/health`.
5. Read [`corpus/PAI_Bill_of_Authorial_Rights.md`](./corpus/PAI_Bill_of_Authorial_Rights.md) — understand what's protected.

**Continue further:** [§ Engineering Path](#engineering-path) · [§ Researcher Path](#researcher-path) · [SSRN paper](https://doi.org/10.2139/ssrn.6512218).

### Researcher Path

**Goal:** understand the framework's normative architecture · invariants · threat model.

1. Read paper: *PAI-CD: A Constitutional Framework for Authorial Sovereignty in Deployed AI Systems* — [DOI 10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218) · ~30 min.
2. Read [`corpus/PAI_Constitutional_Document.md`](./corpus/PAI_Constitutional_Document.md) — ~15 min · six non-derogable invariants.
3. Read [`corpus/PAI_Bill_of_Authorial_Rights.md`](./corpus/PAI_Bill_of_Authorial_Rights.md) — ~20 min · enforceable rights.
4. Read [`corpus/Glossary.md`](./corpus/Glossary.md) — ~10 min · binding terminology.
5. Browse [corpus.paikernel.org](https://corpus.paikernel.org) — current rendered surface (legacy ReadMe.com hosting · sustained through v2.2.x window).

**Continue further:** [§ Auditor Path](#auditor-path) · cite the work · [contact maintainer](mailto:contact@paikernel.org) with research questions.

### Auditor Path

**Goal:** verify invariants hold · review compliance evidence · independent assessment.

1. Read [`corpus/PAI_Constitutional_Document.md`](./corpus/PAI_Constitutional_Document.md) — invariants enumerated.
2. Review [`docs/sbom/sbom.json`](./docs/sbom/sbom.json) — CycloneDX 1.4 supply chain inventory.
3. Review [OpenSSF Scorecard](https://securityscorecards.dev/viewer/?uri=github.com/PAI-Kernel/pai-kernel) — automated supply-chain scan.
4. Review CI runs: [github.com/PAI-Kernel/pai-kernel/actions](https://github.com/PAI-Kernel/pai-kernel/actions).
5. Verify GPG signature: `git tag --verify v2.2.2`.
6. Audit compliance test results: `cargo run -p pai_compliance --locked`.

**Continue further:** [contact maintainer](mailto:contact@paikernel.org) with audit findings · file issues at [github.com/PAI-Kernel/pai-kernel/issues](https://github.com/PAI-Kernel/pai-kernel/issues).

### Contributor Path

**Goal:** propose changes · understand contribution discipline · find where help is wanted.

1. Read [`CONTRIBUTING.md`](./CONTRIBUTING.md) — contribution model · amendment procedure.
2. Read [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md) — expected behavior.
3. Read [`SECURITY.md`](./SECURITY.md) — vulnerability disclosure procedure.
4. Browse open issues: [github.com/PAI-Kernel/pai-kernel/issues](https://github.com/PAI-Kernel/pai-kernel/issues).
5. Propose corpus changes via Amendment procedure (per Governance and Change Control).
6. Propose code changes via PR.

**Continue further:** [§ Engineering Path](#engineering-path) for code-level orientation.

### Engineering Path

**Goal:** integrate PAI-Kernel runtime · understand API surface · build adapter.

1. Read [`docs/INSTALL.md`](./docs/INSTALL.md) § HTTP API surface.
2. Browse published crates on [crates.io](https://crates.io/search?q=pai_governance_daemon).
3. Read [`runtime/governance_daemon/`](./runtime/governance_daemon/) — axum-based HTTP service.
4. Read [`crates/pai_api/`](./crates/pai_api/) — core API types.
5. Read [`crates/pai_witness/`](./crates/pai_witness/) — witness chain · audit log.
6. Build with `cargo build --workspace --locked`.
7. Run tests with `cargo test --workspace --locked`.

**Continue further:** [§ Auditor Path](#auditor-path) for verification approach · [§ Contributor Path](#contributor-path) if submitting changes.

-----

The frozen v2.2 corpus snapshot is archived via DOI:
<https://doi.org/10.5281/zenodo.19151900>

-----

## Document Status & Publication Model

**PAI-CD v2.2** is a constitutional framework consisting of
**10 normative documents**, organized as a layered system.

The **v2.2.2 release package** contains:

- **Three foundational corpus documents (Layer 0)** — constitutional invariants, authorial rights, binding terminology
- **Companion research paper** (SSRN DOI 10.2139/ssrn.6512218)
- **Rust SDK v1.3.1** — 22 library crates + governance daemon binary + 5 runnable examples (T2/T3 authorization composition correctness fix relative to v1.3.0)
- **Install guide** (`docs/INSTALL.md`) — cross-platform walkthrough
- **Release notes, known limitations, license, contribution policy, security policy, citation metadata**

The remaining corpus layers (interpretation rules, threat modeling beyond Layer 0, protocol constraints, compliance logic, audit procedures, governance mechanisms, implementation mapping) exist in internal canonical development and will be released progressively in subsequent publications.

This repository is **not a research paper**.

It is a **normative constitutional specification (Layer 0 infrastructure)**.

A companion **research paper** describing the problem space, threat model, invariant architecture, and system implications is published on SSRN:

> *"PAI-CD: A Constitutional Framework for Authorial Sovereignty in Deployed AI Systems"*
> Mikhail Anatolievich Sergeev · Independent Researcher; PAI-Kernel Initiative · 2026
> DOI: [10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218)

The paper PDF is also included in this repository under [`papers/`](./papers/).

-----

## What This Is

**PAI (Personal Authorial Intelligence)** is a normative framework
that ensures AI systems operate exclusively under the declared and
verifiable authority of the human author they represent.

PAI defines the constitutional conditions required to preserve
authorial sovereignty at the deployment layer of AI systems.

**PAI-Kernel** is the public normative layer of the
**PAI Constitutional Framework (PAI-CD)** — a formal specification
defining the constitutional principles, authorial rights, and
governance invariants for Personal Authorial Intelligence systems.

The PAI Constitutional Framework (PAI-CD) is a structured corpus
of normative documents establishing the governance model of
PAI systems.

This repository contains the **Public Edition** — the Layer 0
release of the PAI-CD corpus.

PAI-CD is not a product. It is not a startup. It is a
**Layer 0 normative infrastructure** — a constitutional substrate
on which compliant implementations, governance bodies, and
execution systems may be built.

-----

## Core Premise

Most AI governance discourse focuses on what AI systems *should do*.

PAI-CD focuses on something prior: **who holds final authority**,
and what structural guarantees protect that authority from erosion —
by providers, by optimization pressure, by infrastructure lock-in,
or by cumulative drift.

The framework defines six non-derogable invariants:

|Invariant                |What It Protects                                      |
|-------------------------|------------------------------------------------------|
|**Authorship Supremacy** |Final human authority over all consequential decisions|
|**Cognitive Sovereignty**|Freedom from covert persuasion and behavioral shaping |
|**Anti-Manipulation**    |Prohibition on undeclared optimization objectives     |
|**Provider Independence**|Portability and governance reproducibility            |
|**Reversibility**        |Reconstructability of all structural changes          |
|**Drift Immutability**   |Protection against cumulative invariant erosion       |

These invariants are **non-derogable** — they cannot be suspended
by emergency, majority vote, economic pressure, security update,
or provider policy.

PAI-CD operates at **Layer 0 — below models, providers, and application logic.**

-----

## Public Edition — Document Index

This repository publishes three foundational documents:

| Document                                                               | Description                         |
|------------------------------------------------------------------------|-------------------------------------|
| [`corpus/PAI_Constitutional_Document.md`](./corpus/PAI_Constitutional_Document.md)   | Invariants and interpretation rules |
| [`corpus/PAI_Bill_of_Authorial_Rights.md`](./corpus/PAI_Bill_of_Authorial_Rights.md) | Enforceable Author rights           |
| [`corpus/Glossary.md`](./corpus/Glossary.md)                                         | Binding terminology for PAI-CD v2.2 |

The full corpus (10 documents) includes implementation mapping,
threat modeling, compliance verification, and governance control
layers. The complete framework is maintained by the author and will be
extended as the project develops. A consolidated bilingual portal is planned for v2.2.3 (~late June / early July 2026).

-----

## Interpretation Principle

> Ambiguity resolves toward stronger invariant protection, minimal authority expansion, and maximum portability.

This principle applies to all documents in this repository and to any compliant implementation.

-----

## Status

|Item                    |Status                                          |
|------------------------|------------------------------------------------|
|Constitutional Framework|v2.2 corpus — Freeze Edition (March 2026)       |
|Release package         |v2.2.2 — Stabilization Release (2026-04-27)     |
|Distribution            |Invitation-only early adopter preview           |
|Domain                  |[paikernel.org](https://paikernel.org)          |
|Paper DOI               |[10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218) |
|Governance              |Currently maintained by the primary author      |

**Author:** Mikhail Sergeev · independent researcher · Russia.
**Background:** 12-year journey from a June 2014 manuscript к the present
constitutional corpus · ORCID [0009-0001-6443-855X](https://orcid.org/0009-0001-6443-855X).

> **Note on versioning:** The v2.2 corpus is a **frozen, citationally-stable snapshot** of the three foundational documents (March 2026). The v2.2.2 release package (2026-04-27) ships that corpus alongside SDK v1.3.1 (T2/T3 authorization composition fix relative to v1.3.0), the research paper, and adopter materials. Future releases may introduce additional normative content; v2.2.2 remains retrievable under its tag. For academic citation, see [`CITATION.cff`](./CITATION.cff).

-----

## License

This repository is **dual-licensed** between code and documentation:

- **Source code** (Rust crates · binaries · scripts · CI workflows):
  Licensed under **MIT OR Apache-2.0** (dual license · choose either at your discretion).
  See [`LICENSE-MIT`](./LICENSE-MIT) and [`LICENSE-APACHE`](./LICENSE-APACHE).

- **Documentation** (PAI-CD framework normative texts · Constitutional Document ·
  Bill of Authorial Rights · Glossary · this README's narrative sections):
  Licensed under **Creative Commons Attribution 4.0 International (CC BY 4.0)**.
  See [`LICENSE-CC-BY-4.0`](./LICENSE-CC-BY-4.0).

You are free to share, adapt, and use this material — including in commercial
products — provided you preserve copyright notices, give appropriate attribution
to PAI-Kernel Initiative, and indicate if changes were made.

For a license summary and FAQ, see [`LICENSE`](./LICENSE). A detailed
License FAQ is published progressively across v2.2.x releases.

-----

## Contributing

PAI-Kernel is in early formation. Community standards are published in this repository:

- [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md) — grounded in PAI-CD anti-manipulation principles
- [`CONTRIBUTING.md`](./CONTRIBUTING.md) — amendment procedure as contribution pathway
- [`SECURITY.md`](./SECURITY.md) — responsible disclosure for specification vulnerabilities

At this stage, the most valuable contributions are:

- Careful reading and substantive critique of the normative layer
- Identification of ambiguities that require clarification via Amendment
- Academic or institutional engagement

Please open an Issue using the provided templates to begin a public discussion.

-----

## Contact

For institutional inquiries, academic collaboration, or governance discussion:  
Open an Issue in this repository, reach out via [paikernel.org](https://paikernel.org), or email [contact@paikernel.org](mailto:contact@paikernel.org).

-----

*PAI-Kernel is a public normative layer of PAI-CD. It is not affiliated with any AI provider.*
