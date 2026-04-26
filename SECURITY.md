# Security Policy — PAI-Kernel

PAI-Kernel is a constitutional governance framework for personal AI systems, maintained by Mikhail Sergeev as Independent Researcher / PAI-Kernel Initiative. This document describes how to report security vulnerabilities, what is in scope, and what to expect after disclosure.

---

## 1. Reporting Channel

### 1.1 Preferred · GitHub Security Advisory (private)

<https://github.com/PAI-Kernel/pai-kernel/security/advisories/new>

This channel is preferred. Benefits:

- Confidential discussion thread between reporter and maintainer
- Integrated CVE-request workflow
- Automatic notification to maintainers
- Built-in coordinated-disclosure timeline tooling

### 1.2 Alternate · Direct email (PGP-encrypted)

`<contact@paikernel.org>` with `[SECURITY]` in the subject line.

For sensitive reports, encrypt with the project PGP key:

- **Fingerprint:** `95C4 B50E D565 44DC 033B 130D B84B 6C86 0ABA D0B1`
- **Public key:** Available via [keys.openpgp.org](https://keys.openpgp.org/search?q=Mikhail.Sergeev%40paikernel.org) (search by `Mikhail.Sergeev@paikernel.org`)
- **Direct URL:** <https://keys.openpgp.org/vks/v1/by-fingerprint/95C4B50ED56544DC033B130DB84B6C860ABAD0B1>

GPG retrieval:

```bash
gpg --keyserver hkps://keys.openpgp.org --search-keys Mikhail.Sergeev@paikernel.org
```

### 1.3 What to include

- Expected vs observed behavior
- Breach classification if applicable (see § 8 shorthand codes)
- Affected files, paths, versions, or crates
- Minimal reproduction steps (proof-of-concept required for triage)
- Suggested fix or mitigation if known
- Preferred credit / anonymity preference

Do **not** include exploit details in public issues, pull requests, or discussions.

---

## 2. Scope

### 2.1 In scope

- **SDK governance daemon** (`crates/`, `runtime/`, `adapters/`)
- **Witness chain integrity** — hash-chain breakage, replay, or forgery
- **Objective Registry bypass** — including the semantic-substitution sub-class (six sub-vectors: synonymy, semantic displacement, composition, indirection via derived variables, language / locale variation, homoglyph / typographic substitution)
- **Corpus signed-artifact integrity** — manifest manipulation, supply-chain injection
- **Consent / Conservative Mode bypass paths** — any route from untrusted input to capability activation
- **Portability / export bundle integrity** — tampering with export JSON or its signature
- **Decision log tampering** (`LOG.TAMPER`) — append-only invariant violation
- **Drift threshold manipulation** (`DRIFT.OVERTHRESHOLD`)
- **Upstream dependency vectors** — vulnerabilities in crates we depend on, with SDK-facing attack path

### 2.2 Out of scope

- **User-level misconfiguration** — please use GitHub Issues
- **Non-security bugs** — please use GitHub Issues
- **Third-party LLM providers** (Ollama, OpenAI, Anthropic, others) — report to respective projects
- **Platform-level OS vulnerabilities** — report to OS vendor
- **Infrastructure-level DoS** unless SDK-specific amplification exists
- **Internal governance processes** — decision records, process findings, and internal operating principles enforcement are not security-reportable
- **Internal communication channels** — internal coordination artifacts and protocols are not in scope
- **External services themselves** — vulnerabilities in GitHub, Zenodo, SSRN, or similar third-party infrastructure should be reported to those providers directly
- **Social engineering** against the maintainer or other PAI-Kernel-affiliated individuals — out of technical scope
- **Rate-limit bypass on external services** (e.g., GitHub API) — report to service operator
- **Typographic / orthographic errors in corpus** without enforcement consequences — please use GitHub Issues
- **Theoretical attacks** without proof-of-concept — please include reproduction steps
- **Vulnerabilities in commercial deployments by third parties** — handled separately per deployment contract; report to deploying organization
- **Documented known-limitation items** — see `docs/KNOWN_LIMITATIONS.md` § 6 for scope-boundary items that are NOT security vulnerabilities (e.g., L1 Demo Mode design decision)

---

## 3. Severity Scale

PAI-Kernel uses **CVSS 3.1 base scores** as the industry-standard starting point, with PAI-CD-specific modifiers reflecting the framework's normative invariants.

### 3.1 Base scoring

CVSS 3.1 calculator: <https://www.first.org/cvss/calculator/3.1>

| CVSS base score | Severity tier |
|---|---|
| 9.0 – 10.0 | Critical |
| 7.0 – 8.9 | High |
| 4.0 – 6.9 | Medium |
| 0.1 – 3.9 | Low |

### 3.2 PAI-CD severity modifiers

Severity tier is **escalated by one tier** when the breach affects framework invariants:

- **+1 tier** if breach affects Author Supremacy (any path that overrides Author authority over their own PAI instance)
- **+1 tier** if breach affects Reversibility (any path that prevents Author rollback, export, or revocation)
- **+1 tier** if breach is silent — i.e., not detected by routine witness-chain audit (`LOG.TAMPER` class)

Modifiers stack additively (e.g., a Medium-base CVSS finding affecting Author Supremacy AND silent becomes Critical).

The final severity shape is subject to architectural review and may be revised based on disclosed material.

---

## 4. SLA · Acknowledgment / Triage / Fix

| Stage | Target |
|---|---|
| Initial acknowledgment | within **72 hours** of report receipt |
| Triage assessment (severity + reproducibility confirmed) | within **7 calendar days** |
| Remediation plan communicated | within **14 calendar days** |
| Patch release · severity-dependent | Critical: **30 days** · High: **60 days** · Medium: **90 days** · Low: best-effort |
| Public disclosure | coordinated with reporter |

During the v2.2.1 early-preview phase, response timelines may be expedited for findings affecting the invitation-only adopter cohort.

---

## 5. Disclosure Timeline

### 5.1 Standard coordinated disclosure window

**90 days** from initial acknowledgment to public disclosure, extendable by mutual agreement between reporter and maintainer.

### 5.2 Accelerated disclosure

The maintainer may request accelerated disclosure (less than 90 days) when:

- Active exploitation observed in the wild
- Patch ready and tested significantly earlier than 90 days
- Reporter requests immediate publication

### 5.3 Extended disclosure

Disclosure may be extended beyond 90 days when:

- Patch development requires architectural changes
- Multiple downstream coordinated parties involved
- Reporter agrees to extension in writing

### 5.4 CVE assignment

CVE identifiers are requested via GitHub Security Advisory's built-in flow when applicable. CVE assignment is not contingent on severity tier.

### 5.5 Coordinated release

Preferred mode: simultaneous patch publication, advisory disclosure, and CVE publication.

---

## 6. Safe Harbor

PAI-Kernel adopts safe-harbor protections adapted from the disclose.io v0.5 framework (<https://disclose.io>).

> ⚠ **Legal review pending** — this safe-harbor language is provided in good faith but has not yet undergone qualified legal review. Reporters relying on these protections should consult counsel for jurisdiction-specific advice. The maintainer commits to revising this section after legal review without retroactive narrowing of protections offered to reports submitted in good faith under earlier versions of this policy.

### 6.1 Authorized testing

Security testing performed in accordance with this policy is considered authorized activity within the license terms governing PAI-Kernel artifacts (CC BY 4.0 for documentation · MIT OR Apache-2.0 for code).

### 6.2 Good-faith protection

Reporters acting in good faith under this policy:

- Will not be subject to legal action by PAI-Kernel for activity within declared scope
- Will not have their reports used as basis for civil or criminal proceedings against them
- May rely on this commitment for activity that complies with the boundaries declared in § 2 (Scope)

### 6.3 Boundary

Safe harbor does **not** apply to:

- Acts beyond the declared scope (§ 2.2 Out of scope)
- Activity that compromises systems, data, or persons not owned by the reporter
- Public disclosure prior to coordinated-disclosure window expiry
- Demands for compensation, threats, or coercive communication

### 6.4 Maintainer commitment

The maintainer will not pursue legal action against good-faith security researchers operating within this policy. This commitment is publicly stated and binding for reports filed under this version of the policy.

---

## 7. Credit Policy

### 7.1 Opt-in attribution

By default, valid security disclosures are credited in the published advisory:

- Reporter name (or pseudonym if requested)
- Optional affiliation
- Discovery date
- Coordinated-disclosure date

### 7.2 Anonymity

Reporters may request full anonymity. The maintainer will publish the advisory without reporter attribution while still acknowledging external discovery.

### 7.3 Hall of Fame

Once **3 or more** valid disclosures have been credited, the project will publish a Security Hall of Fame page listing contributors with their consent.

### 7.4 No bug bounty payments

PAI-Kernel does **not** offer monetary compensation for security disclosures at this time. This decision will be revisited if the project's funding structure changes. The Hall of Fame attribution and CVE credit are the primary forms of recognition.

---

## 8. Breach-Class Shorthand Codes

These codes are used in advisory IDs and triage notes:

| Code | Description |
|---|---|
| `GOV.BYPASS` | Any route that circumvents the constitutional governance daemon — capability activation, mode transition, or policy bypass without authorized confirmation |
| `OBJ.INJECTION` | Objective Registry compromise — injection or modification of declared objectives, including the semantic-substitution sub-class |
| `LOG.TAMPER` | Decision log integrity violation — append-only chain breakage, hash forgery, replay, or silent omission |
| `CONS.MODE.VIOLATION` | Conservative Mode bypass — capability use during Author-declared restricted operational state |
| `DRIFT.OVERTHRESHOLD` | Drift threshold manipulation — bypass or alteration of the framework's behavioral drift detection bounds |

---

## Supported versions

| Version | Security patches | Status |
|---|---|---|
| v2.2.1 | ✅ Current | Early preview (first public release) |
| v2.2.x (patches) | ✅ As issued | Patch line |
| Future releases | ✅ Upon release | Active |
| `main` branch (untagged) | best-effort | Development |

---

## Current security controls

This repository employs:

- Pinned GitHub Actions (SHA-referenced) for supply-chain integrity
- Dependabot monitoring for Cargo, npm, and GitHub Actions dependencies
- CodeQL static analysis on Rust source
- OpenSSF Scorecard monitoring (scheduled weekly)
- Append-only witness log with hash-chain verification (`pai_witness`)
- Non-root container runtime (Docker image runs as user `pai`)
- SHA256 checksums published alongside every binary release artifact

## Supply chain

- All crates published to crates.io originate from verified repository commits
- `Cargo.lock` is committed to ensure reproducible builds
- Dependabot PRs monitored and reviewed before merge
- Release binaries built by GitHub-hosted runners, not self-hosted

## Forward roadmap

Future releases will harden the framework along these axes:

- `cargo audit` integration in CI for known upstream vulnerabilities
- Cryptographic integrity verification via SHA-256 corpus manifest
- Unicode normalization in objective-name matching (in progress)
- SLSA Level 1 provenance attestation on release artifacts
- OpenSSF Scorecard automation with public badge
- SBOM generation (CycloneDX format)
- Embedding-similarity blocking for objective names
- Provenance chain tracking for derived variables
- Behavioral proxy detection via drift engine
- Provider self-report cross-validation
- Apple Developer ID code signing for macOS binaries (pending entity formation)
- Windows code signing certificate

The maintainer welcomes adopter feedback on roadmap priorities.

---

## Known-limitation framing

Adopters should read `docs/KNOWN_LIMITATIONS.md` § 6 before filing a security report — several documented items are scope boundaries, not security vulnerabilities:

- **L1 Demo Mode** — the SDK does not mediate Ollama / LLM responses in v2.2.1; this is intentional design, not a bypass
- **Semantic-substitution sub-class** — six attack vectors documented with defenses scheduled for upcoming releases. Reports on these specific vectors are valuable as feedback on the documented roadmap, not new-vulnerability reports, unless they demonstrate a materially different exploitation path

Reports describing documented known-limitation items are still welcome — they help validate adopter understanding and calibrate defense priorities.

---

*PAI-Kernel · Security Policy · v2.2.1 release*
