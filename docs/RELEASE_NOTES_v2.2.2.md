# PAI-Kernel v2.2.2 — Release Notes

**Release date:** 2026-04-27
**Status:** Stabilization release · v2.2.x series
**Framework version:** PAI-CD v2.2.2 (corpus 10 canonical documents · invariants unchanged)
**SDK version:** v1.3.1 (parallel publication to crates.io)
**Repository:** [github.com/PAI-Kernel/pai-kernel](https://github.com/PAI-Kernel/pai-kernel)
**Paper:** [SSRN DOI 10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218)

---

## What this release is

v2.2.2 is the stabilization release for PAI-Kernel. The framework corpus
and invariants remain unchanged; this release ships:

- **T2/T3 authorization composition correctness fix** in `pai_governance_daemon` — see *Constitutional integrity fix* below
- **5 CVE patches** (rust-openssl family + rustls-webpki + time crate)
- **Rust toolchain bump** 1.86.0 → 1.88.0 (required by time CVE fix)
- **Defense-in-depth: Unicode NFKC normalization** for growth-signal denylist (homoglyph bypass mitigation)
- **Cross-crate resource cliff structural mitigation** (`scripts/check_cross_crate_includes.sh` — pre-publish gate)
- **Pre-compiled binary distribution** (Homebrew tap + `install.sh` fast path)
- **SBOM (CycloneDX 1.4)** · **OpenSSF Scorecard badge** · supply-chain transparency improvements
- **README v0.3 multi-audience entry structure** (Adopter / Researcher / Auditor / Contributor / Engineering paths)

### Scope

**In release:**
- T2/T3 compliance fix (full canonical 10/10 compliance suite passing)
- All security and tooling improvements above
- Constitutional Core unchanged (PAI-CD v2.2 corpus citationally stable)

**NOT in this release** (planned for **v2.2.3** ~late June / early July 2026):
- Multi-language localization of canonical 10 documents
- Bilingual mdBook documentation portal at `paikernel.org/corpus/{en,ru}/`
- Subdomain consolidation (`corpus.paikernel.org` continues to host the v2.2 Freeze Edition rendering through the v2.2.2 window; consolidates into the bilingual portal in v2.2.3)
- EN terminology revisions surfaced by translation (deferred to v2.3 Amendment cycle)

---

## Constitutional integrity fix

This release contains a constitutional integrity fix in
`pai_governance_daemon` covering the Tier ≥ 2 authorization composition
in `validate_authorization`.

### What was wrong (v1.3.0 baseline)

For Tier ≥ 2 governance actions the runtime evaluated authorization as
"active consent OR valid delegation", with the delegation branch
short-circuited to `true` whenever the actor was the Author. Two
canonical compliance tests (`T2_delegation_expired_rejected` and
`T3_tier2_action_requires_consent`) returned `Ok(())` instead of the
expected `Err(GovError::Unauthorized)`.

The composition violated:

- **Constitutional Document § Principle 1 — Authorship Supremacy** ("Delegation must be scoped, time-bound, and revocable")
- **Bill of Authorial Rights § Right 1 — Final Authority** (self-binding implied for Tier ≥ 2 capability use)
- **Consent and Capability Model § Principle 2** ("Be revocable" · "Be logged in Decision Log")
- **p0-3 Conservative Mode § BT-6 AUTHZ.FAIL** (canonical mapping `validate_and_apply() → Err(GovError::Unauthorized)`)

### What changed (v1.3.1 fix)

The Tier ≥ 2 path now evaluates active consent **AND** active
delegation/direct-authority as a conjoint requirement, with a narrowly
scoped Author bootstrap exception for the six built-in management
capabilities (`CAP.CAPABILITY.REGISTER`, `CAP.CONSENT.GRANT`,
`CAP.CONSENT.REVOKE`, `CAP.DELEGATION.GRANT`, `CAP.DELEGATION.REVOKE`,
`CAP.CONSERVATIVE.EXIT`) — these constitute the constitutional substrate
on which the consent and delegation machinery itself is built, and
require Author direct authority for bootstrap. All other Tier ≥ 2
capabilities (including `CAP.OBJECTIVE.RATIFY_ADD` and any user-defined
capability) bind the Author to active consent.

The fix also distinguishes lifecycle authorization failure (revocation
or expiry — constitutional features per the Consent Model "Be revocable"
clause) from breach detection (log tampering, bypass attempts,
injection, unregistered capability access, Tier 4 dual-confirm missing,
signature missing). Lifecycle authorization failure now returns
`Err(GovError::Unauthorized)` per the canonical BT-6 mapping; the
broader conservative-mode shift is reserved for actual breach detection.
This preserves the "every access" gate semantics from p2-7-8-9 § 2.5
without conflating normal lifecycle behavior with breach response.

### Adopter impact

If you depend on `pai_governance_daemon = "1.3.0"` directly or
transitively, run `cargo update -p pai_governance_daemon` after pulling
v2.2.2 to receive the v1.3.1 fix. The compliance suite distinguishes the
two states:

```text
$ cargo run -p pai_compliance --locked
# v1.3.0:  8/10 PASS  (T2/T3 fail)
# v1.3.1: 10/10 PASS  (T2/T3 pass · constitutional integrity restored)
```

The fix is backwards-compatible for all positive authorization paths
(authorization that should succeed, continues to succeed); only the
incorrect "should-have-rejected" paths are now rejected as canonically
required.

### Independent verification

The fix went through a four-stage compliance investigation procedure
with independent verification — including an independent governance
reviewer applying the canonical PAI-CD clauses cited above. The
reviewer found no violations of any constitutional invariant and
ratified the fix "PASS WITH NOTES" (notes captured for v2.3 corpus
refinement work, none blocking for v2.2.2). Full traceability records
(diagnostic, fix, acceptance, countersign) are maintained in the
project's internal governance record; adopters auditing the fix can
verify it directly via the compliance suite output and
`cargo run -p pai_compliance --locked`.

---

## What's New

### Security

- **CVE clearance** · time crate 0.3.41 → 0.3.47 (DoS via stack exhaustion)
- **CVE clearance** · rustls-webpki 0.103.12 → 0.103.13 (reachable panic in CRL parsing)
- **CVE clearance** · rust-openssl family (full list in release-artifact CVE log)
- **Defense-in-depth · Unicode NFKC normalization** in `pai_interface::validate_context()` — prevents homoglyph and full-width Unicode bypass of the growth-signal denylist
- **0 CVEs total** at release (cargo audit baseline)
- **CycloneDX 1.4 SBOM published** at `docs/sbom/sbom.json` (256+ components)
- **OpenSSF Scorecard badge** at `securityscorecards.dev/viewer/?uri=github.com/PAI-Kernel/pai-kernel`
- **NIST SSDF v1.1 informal first-pass mapping** maintained in project records (adopter due-diligence reference available on request)

### Engineering

- **Rust toolchain 1.88.0** (was 1.86.0) — MSRV bump required by time CVE fix
- **Cross-crate resource cliff structural mitigation** (`scripts/check_cross_crate_includes.sh`) — pre-publish gate catches workspace-boundary include paths that resolve in dev but break under cargo publish
- **Test-code embedding** — `pai_kernel` and `pai_policy` embed `policies/` so `cargo test --workspace` from a cloned tag passes
- **Forward defense scanner integrated in CI** — cross-crate scan blocks bad packaging at PR time
- **291/291 unit tests passing** in canonical workspace · pre-public mirror baseline 286/286 (Defense 9 unit tests are runtime-internal)
- **Markdown lint discipline** — clean baseline preserved (~270 files scanned)
- **Pinned GitHub Actions** — all action references SHA-pinned

### Documentation

- **README v0.3 multi-audience structure** — five entry paths (Adopter / Researcher / Auditor / Contributor / Engineering) with time estimates and reading lists
- **Three foundational corpus documents** continue at the root (Constitutional Document · Bill of Authorial Rights · Glossary)
- **Frozen v2.2 corpus snapshot** archived via DOI [10.5281/zenodo.19151900](https://doi.org/10.5281/zenodo.19151900) (citationally-stable reference)
- **Companion paper** preserved at `papers/` and on SSRN [DOI 10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218)
- **Sustained legacy surface** at [corpus.paikernel.org](https://corpus.paikernel.org) — v2.2 Freeze Edition rendering remains LIVE through the v2.2.x window (consolidates into the bilingual portal at v2.2.3)

### Distribution

- **Pre-compiled binaries** via Homebrew tap — `brew install PAI-Kernel/tap/pai-kernel`
- **install.sh fast path** preserved — `curl -fsSL https://paikernel.org/install.sh | sh`
- **18 crates re-published to crates.io** as v1.3.1 (constitutional integrity fix · CVE patches · MSRV 1.88)
- **GitHub Release immutable from publish** (signed tag · audit-trail integrity)

---

## Resolution of v2.2.1 Tag-Content Discrepancy

The v2.2.1 release surfaced a tag-content drift that v2.2.2 explicitly
resolves. Adopters cloning the v2.2.1 tag versus the v2.2.1
release-time state of `main` would have observed different repository
contents.

### What happened

The v2.2.1 tag points to commit `82ecb50e`. The `main` branch advanced
to `1bd73abe` between tag creation and the close of the v2.2.1
publication cycle, with four post-tag fixes applied directly to `main`:

- `32ce9cd` — internal path-dependency version qualifiers (cargo publish gate)
- `0598c53` — embedded `spec/` files in `pai_governance_daemon` (cargo publish isolation fix)
- `46bc07b` — embedded `policies/` files in `pai_api` (same pattern)
- `1bd73ab` — CITATION.cff SPDX expression form correction

### Why it happened

The v2.2.1 tag was signed and published per the release-engineering
procedure. Pre-publication validation passed at tag-creation time. The
four subsequent fixes addressed issues that surfaced during the cargo
publish chain (cross-crate isolation requirements at distribution time)
and CITATION.cff schema validation (CFF schema requires array form
rather than SPDX expression syntax for multi-license declarations).

The fixes shipped to `main` post-tag because the v2.2.1 tag was
protected (GitHub immutable tag policy) and re-tagging would have
invalidated the existing GPG signature chain. Choosing tag stability
preserved cryptographic audit-trail integrity at the cost of allowing
a minor `main` advance.

The 18 crates published to crates.io reflect the tag state
(commit `82ecb50e`) — these are the canonical adopter-facing artifacts
for v2.2.1.

### What v2.2.2 does

v2.2.2 ships a unified state across tag, `main`, and crates.io:

- The `v2.2.2` tag and `main` branch agree at the same commit at release moment
- The 18 crates re-published to crates.io as v1.3.1 reflect the tag state
- All cross-crate isolation fixes are baked in pre-tag
- All CITATION.cff schema corrections are applied pre-tag
- The pre-publish gate (`scripts/check_cross_crate_includes.sh`) catches any future occurrences of the same pattern before tag creation

### Adopter impact

If you cloned the v2.2.1 tag and your `cargo build` succeeded, you have
the canonical v2.2.1 state and your build is correct.

If your local clone followed `main` between v2.2.1 tag and v2.2.2
release moment, you have a slightly more recent state that does not
differ in runtime behavior — only in metadata and packaging artifacts
not exercised by typical adopter use.

For audit purposes, the canonical v2.2.1 reference is the tag (commit
`82ecb50e`); for forward compatibility, v2.2.2 supersedes both states
with a single unified release.

---

## Lessons Learned (Pre-Release Cycle)

The v2.2.1 → v2.2.2 cycle surfaced four operationally-significant
patterns that benefit from explicit acknowledgment alongside the
patches that close them.

### 1. Local context masks distribution boundaries

Two pre-release issues shared a common root: code that worked under
local workspace conditions failed when packaged for distribution.

- **Cargo publish isolation** — workspace member crates referencing
  workspace-root resources via `include_str!("../../../X")` resolved
  fine during `cargo build` (full workspace tree available) but failed
  during `cargo publish` (crate packaged in isolation · path resolution
  stops at crate boundary). Two production crates required mid-publish
  embedding fixes (`pai_governance_daemon` spec/ · `pai_api` policies/)
  before the chain could complete.

- **External tool requirement inversion** — documentation-platform
  bidirectional-sync setup required an empty target repository for
  first connection · the natural reading of the error message
  ("repository not empty") was inverted from intuition (which would
  suggest content was needed). Setup attempt added content first ·
  had to be undone.

Both patterns share the diagnostic: assumptions about distribution-time
behavior that hold only under development-time observation. Mitigation:
explicit pre-distribution checks for each new resource boundary
(`scripts/check_cross_crate_includes.sh` for the workspace boundary;
setup documentation pre-read discipline for external tools).

### 2. Voice consistency across audience boundaries requires forward defense

Internal collaboration vocabulary (perspective labels · governance
artifact names · tracked records terminology) accumulated naturally
across the development cycle. When release notes were generated, some
of this internal vocabulary appeared in published artifacts before
review caught it. Manual remediation closed the immediate exposure;
forward defense came in the form of an extended terminology screening
that applies across all public-bound artifacts going forward.

This release ships the first iteration where pre-publication scans run
against the full extended screening · with adversarial 2nd-pass review
applied to every public-bound artifact (multi-perspective check for
external adopter view · skeptical reviewer · maintainer continuity ·
50-year time horizon).

### 3. CI feedback loops are only as good as their first failing step

Markdown lint discipline accumulated debt across multiple cycles ·
unnoticed because CI reported the first error and stopped. When the
blocking error class (broken pinned action SHA) was finally resolved ·
552 pre-existing markdown errors surfaced at once. Resolution combined
configuration relaxation for legitimate style preferences · bulk
auto-fixes for genuine code-block-language gaps · explicit ignores for
vendored content · and selective file-specific exclusions where
structural changes required deeper review.

The same dynamic surfaced compliance test failures (the T2/T3
authorization composition issue) that had been masked by an earlier-
failing build step for an extended period. Layered CI checks need
ongoing maintenance — not just one-time setup. The four-stage
compliance investigation procedure that resolved T2/T3 in this release
is itself a forward defense against future test-masking patterns.

### 4. Coordinated release moments compress dependency surface

The v2.2.2 release engineering pattern bundles multiple
dependency-bearing operations into a single coordinated moment.
Distributing these across multiple smaller releases would have created
many transitional inconsistency windows; bundling them creates one
larger preparation surface but only a single transition.

The v2.2.3 release (~late June / early July 2026) will continue the
pattern with the multi-language documentation portal launch — full corpus localization,
mdBook deployment activation, subdomain consolidation, and the
remaining items deferred from v2.2.2 all land in a single coordinated
moment.

---

## Migration Guide

### From v2.2.1

**Cargo dependencies:** rerun `cargo build --locked` after pulling
v2.2.2. The `Cargo.lock` will reflect bumped time / rustls-webpki
versions and the v1.3.1 SDK update. To pull the constitutional
integrity fix specifically:

```sh
cargo update -p pai_governance_daemon
```

**MSRV bump:** v2.2.2 requires Rust 1.88.0 (was 1.75 in v2.2.1).
Adopters running older Rust must update their toolchain. Rust 1.88 has
been stable since Q1 2026 · widely available.

**SDK API:** no breaking changes. `pai_interface::validate_context()`
now applies NFKC normalization to context keys before denylist matching;
behavior is strictly more conservative (catches Unicode-bypass attempts
that previously slipped through). `pai_governance_daemon` Tier ≥ 2
authorization composition is corrected per the *Constitutional integrity
fix* section above; no source-level changes required from adopters
beyond the dependency update.

**Documentation surface:** the legacy [corpus.paikernel.org](https://corpus.paikernel.org)
v2.2 Freeze Edition rendering remains LIVE through the v2.2.x window.
The consolidated bilingual mdBook portal launches with v2.2.3
(~late June / early July 2026). For v2.2.2 the canonical 10 corpus
documents are accessible directly through the GitHub UI at
[github.com/PAI-Kernel/pai-kernel/tree/main/corpus](https://github.com/PAI-Kernel/pai-kernel/tree/main/corpus)
(raw `.md` content) and via the existing rendered surface for
Researcher/Auditor reading flow.

### From v2.2.0 or earlier

Read v2.2.1 release notes first · then this document. v2.2.0 → v2.2.1
required additional migration steps not covered here.

---

## Acknowledgments

PAI-Kernel v2.2.2 is the product of sustained solo development effort
across 2026 by the maintainer (Mikhail Sergeev, Independent Researcher /
PAI-Kernel Initiative). The framework's development followed an internal
governance discipline that systematically considered multiple
perspectives — product, technical, standards, research, audit, and
ecosystem — through documented decision-making processes.

The v2.2.2 release in particular received a four-stage compliance
investigation with independent verification on the constitutional
integrity fix; the discipline of distinguishing the analysis stage from
the implementation stage from the verification stage is itself a
contribution to the field of normative-framework engineering.

Specific acknowledgments to early adopters and reviewers are deferred
to the general-availability release. Early adopter contributions (once
feedback arrives) will be documented in a future CONTRIBUTORS file.

---

## Feedback

Welcomed via:

- **GitHub Issues** at [github.com/PAI-Kernel/pai-kernel/issues](https://github.com/PAI-Kernel/pai-kernel/issues) (public)
- **Direct contact:** [contact@paikernel.org](mailto:contact@paikernel.org)
- **Security-sensitive:** GitHub security advisory

Particularly valuable:

- Objections, counterexamples, architectural critique
- Formal verification extensions
- Experience reports from attempted deployments

---

## Next

- **v2.3** (planned · ~late May / early June 2026): multi-language localization of the canonical 10 documents (UN6 + adopter-priority languages) · TMD (Translation Memory Document) · TII (Terminology Issues Inventory) · multi-language mdBook documentation portal · subdomain consolidation (`corpus.paikernel.org` decommission · routing to consolidated portal) · `corpus-docs` legacy repo archive
- **v2.3** (planned · ~3 weeks after v2.2.3): EN terminology revisions surfaced by translation · standard Constitutional Amendment Procedure
- **General availability:** separate future ratification · gated on adopter feedback validating stability

---

*PAI-Kernel v2.2.2 · Release Notes · 2026-04-27*
*Tag: `v2.2.2` · Commit: filled at tag creation*
