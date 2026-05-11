# Changelog

All notable changes to PAI-Kernel SDK are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) · this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) for the workspace SDK version (currently 1.3.2 · maps to release tag v2.2.3).

## Release semantics

- **Release tags** (`v2.2`, `v2.2.1`, `v2.2.2`, `v2.2.3`) are GPG-signed, immutable, and correspond to specific binary distributions on crates.io, GitHub Container Registry, and Homebrew tap.
- **`main` branch** can advance beyond the latest release tag with documentation, CI, and adopter-UX improvements that do not require a new binary version. These changes are reflected in the `main` branch only · binary tags are NOT republished.
- **Patch releases** (e.g., v2.2.3 → v2.2.4) are reserved for changes that affect the binary or runtime behavior.

---

## [Unreleased] · `main` HEAD post-v2.2.3.1

_(empty · accumulating для next release)_

---

## [v2.2.3.1] · 2026-05-11

Micro-patch release · Rust workspace v1.3.2 binary content unchanged · adopter-facing fix + signed-release coverage baseline.

### Fixed

- **`install.sh` · curl|sh overwrite prompt silent abort** · Critical adopter Day 0 upgrade path bug. When script run via `curl | sh`, stdin was the curl pipe (not the terminal), so `read -r reply` consumed EOF and silently aborted before any install action ran. Replaced single `read` with 3-mode resolution:
  1. `PAI_KERNEL_FORCE_INSTALL=1` env var for non-interactive override (CI · automation)
  2. `/dev/tty` openable for read · interactive prompt via controlling terminal (the primary fix)
  3. Neither · safe abort with explicit guidance
  Detection uses subshell open-test `(: < /dev/tty) 2>/dev/null` · authoritative over `[ -r /dev/tty ]` (which can return true for unopenable detached-session ttys). Empirically reproduced before and after on macOS · adopter end-to-end install now works for upgrade-over-existing case. Commits `bc2c58f` (pre-public) · `9dc4c53` (public).

### Added

- **`.github/SECURITY-INSIGHTS.yml`** · OSS Security Insights v1.0.0 structured security metadata. 5 security-testing tools declared (CodeQL · Dependabot · OpenSSF Scorecard · cargo-fuzz · Docker hardening) · integration matrix per tool (ad-hoc · CI · before-release). 13 out-of-scope items mirror SECURITY.md § 2.2 verbatim. PGP key fingerprint inline. Distribution-points enumerate adopter Day 0 channels (GitHub Releases · crates.io · ghcr.io · paikernel.org/install.sh · homebrew-tap).
- **Fuzz harness expansion · 6 → 10 cargo-fuzz targets:**
  - `classify` · `pai_classify::RecActionClassifier::classify` · structured-byte fuzz (no `Deserialize` types) · 42-byte input layout · probes RAB-I1..RAB-I7 invariants
  - `attestation_manifest` · `pai_attestation::TcbManifest` `serde_json` deserialize · probes TCB-I1..TCB-I6 · hex-encoded hash byte/char-length confusion class (same as Session #21 `pai_witness` Hash256 finding)
  - `delegation_sequence` · `pai_delegation::DelegationStore` grant/validate/revoke op-sequence · structured-byte fuzz · probes DEL-I1..DEL-I6 · 1024 ops bounded
  - `drift_report` · `pai_drift::DriftReport` `serde_json` deserialize · probes DFT-I1..DFT-I5 · 7-dimension enum tag validation
- **Seed corpus expansion · 9 → 28 entries** across 10 targets · 17 JSON seeds parse-verified · 6 binary structured seeds layout-verified · libFuzzer mutates valid → invalid · representative seeds dramatically increase coverage hit rate per `feedback_cargo_fuzz_discipline.md`.
- **Sigstore signed-release coverage** (retroactive starting from this release). `release.yml` workflow signs each binary via `cosign sign-blob --yes` with GitHub Actions OIDC token (keyless · no key material managed in repo). `.sig` + `.cert` sidecars uploaded alongside binaries. Adopters verify via:
  ```
  cosign verify-blob \
    --certificate <name>.cert \
    --signature <name>.sig \
    --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
    --certificate-identity-regexp 'https://github.com/PAI-Kernel/pai-kernel/.github/workflows/release.yml@.*' \
    <name>
  ```
  Existing v2.2 · v2.2.2 · v2.2.3 releases cannot retroactively add signatures due to GitHub Release immutability one-way ratchet · new releases inherit signed coverage going forward.

### Changed (inherited from `[Unreleased]` since v2.2.3)

- `docs/INSTALL.md` · empirical correction post-adopter UX validation (16 fixes covering env var names · key generation patterns · TOML config structure · CLI binary names · API endpoints · Docker examples · Demo mode prerequisite · Verification prerequisite · Troubleshooting entries). See commit `90ad7ee` for sync from pre-public to public.
- `crates/pai_export/Cargo.toml` · added `[target.'cfg(target_arch = "wasm32")'.dependencies]` block enabling `getrandom = { version = "0.2", features = ["js"] }` for WebAssembly browser/JS runtime support. Native build behavior unchanged. See commit `4705050`.
- `Cargo.lock` · regenerated reflecting `pai_export` WASM target dep addition.
- INSTALL.md Further Reading · cleaned up broken links · `PAI_Constitutional_Document.md` link points to direct `corpus/` location instead of redirect placeholder.

### Added (inherited from `[Unreleased]` since v2.2.3)

- `examples/README.md` · adopter guidance for 5 reference example binaries (causal_graph · classify_output · compliance_identity · evasion_audit · vulnerability_check).
- `CHANGELOG.md` (this file) · standard convention release history.
- `docs/quickstart.md` · 5-minute getting-started path (parallel to comprehensive `docs/INSTALL.md`).
- `INSTALL.md` (root) · binary `init` command output reference compatibility.

### Fixed (inherited from `[Unreleased]` since v2.2.3)

- WASM target compilation for `pai_export`: `cargo build --target wasm32-unknown-unknown --release -p pai_export` now succeeds. Previously failed due to missing `getrandom` `js` feature flag for transitive dependency.

### Why this micro-patch?

Rust workspace version (`1.3.2`) unchanged · NO new crates.io publish · NO Zenodo DOI mint. This release exists to:
1. Ship `install.sh` fix to adopters using upgrade-over-existing path (critical · Day 0 upgrade flow was 100% broken before fix)
2. Establish Sigstore signed-release baseline for OpenSSF Scorecard `Signed-Releases` check (existing v2.2 · v2.2.2 · v2.2.3 releases predate Session #21 Sigstore workflow integration · cannot retroactively add signatures · going forward all new releases inherit signed coverage)
3. Bundle Session #22 security infrastructure additions (`SECURITY-INSIGHTS.yml` · expanded fuzz harness · seed corpus) into a signed canonical artifact

---

## [v2.2.3] · 2026-04-28

### Added

- Fail-closed environment variable defaults · daemon refuses to start without `PAI_AUTHOR_API_KEY` and `PAI_AUTHOR_SIGNING_KEY` configured.
- `--demo` flag for local testing · ephemeral keys · forces 127.0.0.1 bind · prints prominent stderr warnings.
- `pai_governance_daemon init` subcommand · creates default `pai-kernel.toml` config + `policies/` skeleton in current directory.

### Changed

- Author key initialization moved from compile-time defaults to environment variables (`PAI_AUTHOR_API_KEY` · `PAI_AUTHOR_SIGNING_KEY`).
- `install.sh` updated to bump to v2.2.3 release binary.
- `pai_api` `/api/v1/version` handler · dynamic `env!` macros instead of compile-time string.

### Security

- v2.2.3 patch addresses hardcoded demo keys in production runtime path · adopters must explicitly configure signing keys for any non-demo deployment.

---

## [v2.2.2] · 2026-04-27

### Added

- Phase 4 §3 finalization · ceremonial release artifacts complete.
- README v0.3 with hero diagram · OpenSSF Scorecard badge.
- Compliance binary CI gate · runs `pai_compliance --release` and blocks release on T1..T6 invariant failures.

### Changed

- SDK workspace version bumped to 1.3.1.
- MSRV (Minimum Supported Rust Version) bumped to 1.88 due to time crate CVE.
- `corpus.lock` · scrubbed internal-only labels from header.
- Authorization composition stage 3 fix.

### Fixed

- `cli_t01_version` test expects 1.3.1.
- README row 43 padding · diagram cells uniform 60-char width.

---

## [v2.2.1] · 2026-04 (early)

Initial post-v2.2 patch · documentation reorganization · `PAI_Constitutional_Document.md` moved to `corpus/` (root file is now a redirect placeholder for compatibility).

---

## [v2.2] · 2026-04 (initial)

First public release of PAI-Kernel SDK · 28-crate workspace · constitutional governance daemon · TLA+ formal model · Rego policy engine · SQLite witness backend.

---

## See also

- [`docs/INSTALL.md`](docs/INSTALL.md) — full installation guide
- [`docs/quickstart.md`](docs/quickstart.md) — 5-minute getting-started
- [`docs/RELEASE_NOTES_v2.2.3.md`](docs/RELEASE_NOTES_v2.2.3.md) — detailed v2.2.3 release notes
- [`docs/KNOWN_LIMITATIONS.md`](docs/KNOWN_LIMITATIONS.md) — known limitations and scope
- [GitHub Releases](https://github.com/PAI-Kernel/pai-kernel/releases) — binary distributions per release tag
