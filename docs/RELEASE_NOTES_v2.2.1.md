# PAI-Kernel v2.2.1 — Release Notes

**Release date:** 2026-04-23
**Status:** Early preview · invitation-only distribution
**Framework version:** PAI-CD v2.2.1
**SDK version:** v1.3.0 (crates.io parallel publication)
**Repository:** [github.com/PAI-Kernel/pai-kernel](https://github.com/PAI-Kernel/pai-kernel)
**Paper:** [SSRN DOI 10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218)

---

## What this release is

**First public release of PAI-Kernel.** Previous history: internal development, SSRN paper publication (v2.2.1 — April 2026), public repository paused pending remediation.

The pause has been lifted. v2.2.1 is the authorized first release — **early preview**, not production-ready, not general-availability.

### Integration level · honest framing

PAI-Kernel v2.2.1 SDK provides the **governance substrate** — invariants, witness chain, consent gates, drift monitoring, export primitives. This release runs **side-by-side with Ollama** (or any LLM runtime) for local demonstration purposes. The AI-mediation wiring — routing model responses through governance gates, Conservative Mode blocking AI output mid-stream, witness entries auto-populated from chat turns — is scheduled for a future release.

See `INSTALL.md` § 8 "What you're seeing (Level 1 Demo Mode)" and `KNOWN_LIMITATIONS.md` for the full scope statement.

---

## Scope

### Framework corpus (3 canonical documents)

Published to repository root:

- **Constitutional Core** (`PAI_Constitutional_Document.md`) — 205 lines; non-derogable invariants, document hierarchy, interpretive rules
- **Bill of Authorial Rights** (`PAI_Bill_of_Authorial_Rights.md`) — 355 lines; 14 enforceable rights with non-derogation protection
- **Glossary** (`Glossary.md`) — 349 lines; binding terminology authority

Browsable HTML rendering: [corpus.paikernel.org](https://corpus.paikernel.org) (same content, different surface)

### SDK v1.3.0 (Rust workspace)

- **28 crates** implementing framework invariants
- **2 runtime binaries**: `pai_governance_daemon` (axum HTTP daemon) + CLI subcommands
- **262 tests** passing; 0 regressions
- **Phase 1 scope**: live daemon + witness chain + drift monitoring + consent gate + snapshot/export
- Publication to crates.io in parallel

### Paper

- Title (as published): *"PAI-CD: A Constitutional Framework for Authorial Sovereignty in Deployed AI Systems"*
- Author: Mikhail Anatolievich Sergeev (Independent Researcher; PAI-Kernel Initiative)
- Venue: SSRN · DOI 10.2139/ssrn.6512218
- PDF: `papers/PAI-CD_Authorial_Sovereignty_Deployed_AI_v2.2.1.pdf`

---

## What changed in v2.2.1

### From v2.2 Freeze Edition (2026-03-19)

v2.2.1 is **v2.2 + editorial patches + traceability metadata**. No new normative content.

#### R2 terminology patches

Applied to the 3 corpus documents:

- **Constitutional Core · Principle 6** — added inline clarifying note: *"'Standard' refers to the classification criterion, not standardization-body-level specification. PAI-CD is a normative framework, not a formally ratified standard."*
- **Bill of Rights · Right 5** — "standardized format" → "portable format" (semantic consistency with right title "RIGHT TO PORTABILITY")

Other R2 rulings (Ruling 3 "protocol between instances", Ruling 4 "normative standard") do not apply to v2.2 source text — those terms were introduced in later Amendment cycles (v2.3+ / v3.x).

#### Style normalization

Applied to all published Markdown:

- Human-readable H1 titles replace filename-style H1 (`# PAI_Constitutional_Document.md` → `# Constitutional Core`)
- Markdownlint configuration via `.markdownlint.json` (project style rules)
- Frontmatter added: title / slug / category / excerpt / metadata / `pai_cd.source.*` traceability block
- Footer traceability line added

#### Pre-release defect remediation

Prior to this release a public-repo defect inventory was maintained (ten items identified). Resolution before v2.2.1 push:

- **8 items remediated** — integrated into this release (terminology discipline fixes, copyright granularity, versioning notes, paper update, citation metadata)
- **2 items closed pre-release** (stale branch deleted; archival metadata title correction landed in prior cycle)
- **0 items carried forward** as known limitations

---

## What did NOT change

The v2.2 Freeze Edition surface at [corpus.paikernel.org](https://corpus.paikernel.org) (ReadMe-hosted `corpus-docs` sync repo) uses **pure v2.2 verbatim** content without R2 patches. That surface is a **pure archival snapshot**.

The v2.2.1 GitHub release surface **does apply** R2 patches. This matches the bifurcation model (public snapshot layer vs public release layer are distinct).

**Which to cite:** academic citation targets SSRN paper DOI (cites v2.2.1 reconstruction). Repository citations use `github.com/PAI-Kernel/pai-kernel@v2.2.1`.

---

## Known limitations

See `KNOWN_LIMITATIONS.md` for full text. Key items:

- v2.2 is a **citationally-stable freeze**; future corpus freezes may introduce additional normative content (not auto-reflected here)
- SDK v1.3.0 exceeds v2.2 corpus scope — the runtime implements additional invariants ahead of their publication in a future corpus freeze; adopters using SDK bind to this superset
- Several corpus-required items (TCB attestation, supply-chain provenance, multi-principal governance, author vulnerability protection V4 path) are **not runtime-enforced** in SDK v1.3.0 yet
- Multi-instance coordination (multiple PAI Authors cooperating) is proposal-stage — NOT in v2.2.1
- Regulatory zone governance + provider-disposition disclosure are scheduled for a later release — NOT in v2.2.1
- Formal verification methods produce **specification-level** verdicts, not runtime-SDK conformance

---

## Installation

### SDK (Rust)

```bash
cargo add pai-kernel
```

Or add to `Cargo.toml`:

```toml
[dependencies]
pai-kernel = "1.3"
```

Individual crates available (see workspace members). Running the daemon:

```bash
cargo install --path runtime/pai_kernel
pai_governance_daemon --config pai-kernel.toml
```

Default bind: `127.0.0.1:9100`. HTTP surface per `pai_api` crate.

### Framework corpus

- Clone repo: `git clone https://github.com/PAI-Kernel/pai-kernel.git`
- Checkout tag: `git checkout v2.2.1`
- Read corpus docs: repository root

Or browse online: [corpus.paikernel.org](https://corpus.paikernel.org)

---

## Distribution scope

**Invitation-only early adopter distribution.**

- Public GitHub release (discoverable but not announced)
- crates.io publication (Rust developers can find via search; not promoted)
- Direct personal invitations sent to initial adopter shortlist
- **No public announcement** on LinkedIn, X, HackerNews, Product Hunt until future DL authorizes it

Transition to general availability gated on:

- Next major corpus release publication, OR
- Early adopter feedback validating stability

Either transition requires new DL.

---

## Feedback

Welcomed via:

- **GitHub Issues** at `github.com/PAI-Kernel/pai-kernel/issues` (public)
- **Direct contact**: `contact@paikernel.org`
- **Security-sensitive**: GitHub security advisory

Valuable contributions:

- Objections, counterexamples, architectural critique
- Formal verification extensions
- Experience reports from attempted deployments

---

## Rollback posture

- **Pre-push rollback** — release authorization can be rescinded before first push (no external impact)
- **Post-push rollback** — repository can be set back to private; v2.2.1 tag deletion possible but creates "withdrawn release" signal (avoided unless critical)
- **Critical-failure archive path** — if post-release problems emerge, archive public repo and restart on new repo (high cost; reserved for critical defect)

---

## License

- **Corpus** (Markdown documents): **CC BY 4.0**
- **SDK code** (Rust): **MIT OR Apache-2.0** (dual license per Rust convention)
- **Paper**: SSRN standard posting terms

See `LICENSE` for full text.

---

## Acknowledgments

v2.2.1 is the product of sustained solo development effort across 2026 by the maintainer (Mikhail Sergeev as Independent Researcher / PAI-Kernel Initiative). The framework's development followed an internal governance discipline that systematically considered multiple perspectives — product, technical, standards, research, audit, and ecosystem — through documented decision-making processes.

Specific acknowledgments are deferred to the general-availability release; early adopter contributions (once feedback arrives) will be documented in a future CONTRIBUTORS file.

---

## Next

- **v2.2.2 (patch candidate):** reserved for security or citation-critical fixes only
- **Next major corpus release:** under active development; timeline subject to dedicated publication cycle
- **Public announcement:** separate future DL; gated on readiness signals above

---

*PAI-Kernel v2.2.1 · Release Notes · 2026-04-23*
*Tag: `v2.2.1` · Commit: [to be filled at tag creation]*
