# PAI-Kernel

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
> **Release v2.2.1 — Early Preview** · April 2026

### Start here

- **New adopter?** Begin with [`docs/INSTALL.md`](./docs/INSTALL.md) — cross-platform install guide (~30–60 min).
- **Want to know what's inside and what's not?** See [`docs/KNOWN_LIMITATIONS.md`](./docs/KNOWN_LIMITATIONS.md).
- **Release overview:** [`docs/RELEASE_NOTES_v2.2.1.md`](./docs/RELEASE_NOTES_v2.2.1.md).
- **Research paper:** *"PAI-CD: A Constitutional Framework for Authorial Sovereignty in Deployed AI Systems"* · SSRN · [DOI 10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218).

-----

The frozen v2.2 corpus snapshot is archived via DOI:
<https://doi.org/10.5281/zenodo.19151900>

-----

## Document Status & Publication Model

**PAI-CD v2.2** is a constitutional framework consisting of
**10 normative documents**, organized as a layered system.

The **v2.2.1 release package** contains:

- **Three foundational corpus documents (Layer 0)** — constitutional invariants, authorial rights, binding terminology
- **Companion research paper** (SSRN DOI 10.2139/ssrn.6512218)
- **Rust SDK v1.3.0** — 22 library crates + governance daemon binary + 5 runnable examples
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
extended as the project develops.

-----

## Interpretation Principle

> Ambiguity resolves toward stronger invariant protection, minimal authority expansion, and maximum portability.

This principle applies to all documents in this repository and to any compliant implementation.

-----

## Status

|Item                    |Status                                          |
|------------------------|------------------------------------------------|
|Constitutional Framework|v2.2 corpus — Freeze Edition (March 2026)       |
|Release package         |v2.2.1 — Early Preview (April 2026)             |
|Distribution            |Invitation-only early adopter preview           |
|Domain                  |[paikernel.org](https://paikernel.org)          |
|Paper DOI               |[10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218) |
|Governance              |Currently maintained by the primary author      |

> **Note on versioning:** The v2.2 corpus is a **frozen, citationally-stable snapshot** of the three foundational documents (March 2026). The v2.2.1 release package (April 2026) publishes that corpus together with adopter materials, the research paper, and the Rust SDK v1.3.0. Future releases may introduce additional normative content; v2.2.1 remains retrievable under its tag. For academic citation, see [`CITATION.cff`](./CITATION.cff).

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
License FAQ is published in v2.2.2.

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
