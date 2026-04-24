# Security Policy — PAI-Kernel

## Scope

This repository is a constitutional governance framework for personal AI systems. Security reports should focus on vulnerabilities that can enable governance compromise or adopter harm.

### In scope

- **SDK governance daemon** (`crates/`, `runtime/`, `adapters/`)
- **Witness chain integrity** — hash-chain breakage, replay, or forgery
- **Objective Registry bypass** — including the **semantic-substitution sub-class** documented in PF-2026-04-041 (six sub-vectors: synonymy, semantic displacement, composition, indirection via derived variables, language / locale variation, homoglyph / typographic substitution)
- **Corpus signed-artifact integrity** — `corpus.lock` manipulation, supply-chain injection
- **Consent / Conservative Mode bypass paths** — any route from untrusted input to capability activation
- **Portability / export bundle integrity** — tampering with export JSON or its signature
- **Decision log tampering** (LOG.TAMPER) — append-only invariant violation
- **Drift threshold manipulation** (DRIFT.OVERTHRESHOLD)
- **Upstream dependency vectors** — vulnerabilities in crates we depend on, with SDK-facing attack path

Breach-class shorthand codes (for advisory IDs): `GOV.BYPASS`, `OBJ.INJECTION`, `LOG.TAMPER`, `CONS.MODE.VIOLATION`, `DRIFT.OVERTHRESHOLD`.

### Out of scope

- User-level misconfiguration (please use GitHub Issues instead)
- Non-security bugs (please use GitHub Issues)
- Third-party LLM providers (Ollama, OpenAI, Anthropic, others) — report to respective projects
- Platform-level OS vulnerabilities
- Infrastructure-level DoS unless SDK-specific amplification exists
- Documented known-limitation items — see `KNOWN_LIMITATIONS.md` § 6 for scope-boundary items that are NOT security vulnerabilities (e.g., L1 Demo Mode design decision; semantic-substitution sub-class pending Phase Q-SDK defenses)

## Supported versions

| Version | Security patches | Status |
|---|---|---|
| v2.2.1 | ✅ Current | Early preview |
| v2.2.x (patches) | ✅ As issued | Patch line |
| Future major releases | ✅ Upon release | Active |
| Pre-v2.2.1 | ❌ Not supported | Superseded |
| `main` branch (untagged) | Best-effort | Development |

## Reporting a vulnerability

### Preferred channel — GitHub Security Advisory (private)

<https://github.com/PAI-Kernel/pai-kernel/security/advisories/new>

Benefits: confidential discussion thread, integrated CVE-request workflow, automatic notification to maintainers.

### Alternate channel — direct email

`security@paikernel.org` (or `contact@paikernel.org` with `[SECURITY]` in the subject line)

PGP key available on request for sensitive reports — email `contact@paikernel.org` to request.

### What to include

- Expected behavior vs observed behavior
- Breach classification (if applicable, from §In scope list above)
- Affected files, paths, versions, or crates
- Minimal reproduction steps
- Suggested fix or mitigation (if known)
- Your preferred credit / anonymity preference

Do NOT include exploit details in public issues, pull requests, or discussions.

## Response commitments

| Stage | Timeline |
|---|---|
| Initial acknowledgment | within 72 hours |
| Triage assessment | within 7 days |
| Remediation plan | within 14 days |
| Patch release | within 30 days (severity-dependent) |
| Public disclosure | coordinated with reporter |

During PAI-Kernel v2.2.1 early-preview phase, response may be expedited for critical findings affecting the invitation-only adopter cohort.

## Disclosure preferences

- **CVE:** requested where applicable via GitHub Security Advisory's built-in flow
- **Embargo:** respected per reporter request within reasonable timeline (typically ≤ 90 days)
- **Credit:** given in the published advisory unless reporter requests anonymity
- **Coordinated release:** preferred — simultaneous patch publication and advisory disclosure
- **Legal:** we will not pursue legal action against good-faith security researchers operating within this policy

## Current security controls

This repository employs:

- Pinned GitHub Actions (SHA-referenced, not tag-referenced) for supply-chain integrity
- Dependabot monitoring for Cargo, npm, and GitHub Actions dependencies
- `cargo audit` in CI for known upstream vulnerabilities
- CodeQL static analysis on Rust source
- OpenSSF Scorecard monitoring (scheduled weekly)
- Cryptographic integrity verification via SHA-256 manifests (`corpus.lock`)
- Append-only witness log with hash-chain verification (`/api/v1/log/verify`)
- Non-root container runtime (Docker image runs as user `pai`)
- SHA256 checksums published alongside every binary release artifact

## Supply chain

- All crates published to crates.io originate from verified repository commits
- `Cargo.lock` is committed to ensure reproducible builds
- Dependabot PRs monitored and reviewed before merge
- Release binaries built by GitHub-hosted runners, not self-hosted
- **SLSA Level 1 provenance attestation:** planned for v2.2.2
- **SBOM (Software Bill of Materials):** CycloneDX format, planned for v2.2.2

## Future hardening roadmap

### v2.2.2 (patch, post-v2.2.1 early preview)

- Unicode NFKC normalization + case/separator equivalence in denylist matcher (Defense 9 per PF-041, closes vector 1F + partial 1E)
- SLSA Level 1 provenance attestation on release artifacts
- OpenSSF Scorecard automation with public badge
- SBOM generation (CycloneDX)
- All GitHub Actions pinned to full-SHA references

### Next major release

- Embedding-similarity blocking for objective names (Defense 5 per PF-041, closes vectors 1A + 1B + full 1E)
- Provenance chain tracking for derived variables (Defense 6, closes 1C)
- Behavioral proxy detection via drift engine (Defense 7, closes 1D post-hoc)
- Self-Report Discipline pairing (Defense 8, closes 1D AI-side)
- Security findings registry (separate from process findings)
- Formal NIST SSDF mapping

### Future

- Provider Disposition Disclosure — cross-check AI self-report against provider training documentation (Defense 10)
- Apple Developer ID code signing for macOS binaries (pending legal-entity formation)
- Windows code signing certificate
- Full cross-provider security verification

## Known-limitation framing

Adopters should read `KNOWN_LIMITATIONS.md` § 6 before filing a security report — several documented items are **scope boundaries** not security vulnerabilities:

- **L1 Demo Mode** — the SDK does not mediate Ollama / LLM responses in v2.2.1; this is intentional design, not a bypass
- **Semantic-substitution sub-class** — six attack vectors documented in PF-2026-04-041, with defenses scheduled for v2.2.2 (Defense 9) and future releases (Defenses 5-8). Reports on these specific vectors are valuable as **feedback on the documented roadmap**, not new-vulnerability reports, unless they demonstrate a materially different exploitation path

Reports describing documented known-limitation items are still welcome — they help us validate adopter understanding and calibrate defense priorities.
